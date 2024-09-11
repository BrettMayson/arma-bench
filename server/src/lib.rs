use arma_bench::{Message, Request, Response, ServerConfig, HEADER_ID};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info, trace};

mod arma;
mod build;
mod server;

#[derive(Debug)]
pub struct InternalRequest {
    config: ServerConfig,
    request: Request,
}

#[derive(Debug)]
pub struct RequestHandle {
    callback: tokio::sync::oneshot::Sender<Response>,
    request: InternalRequest,
}

pub async fn server(addr: String) {
    info!("Starting on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();

    let (request_sender, mut request_receiver) = tokio::sync::mpsc::channel(16);

    tokio::spawn(async move {
        loop {
            match request_receiver.recv().await {
                Some(request) => {
                    handle(request).await;
                }
                None => {
                    error!("Failed to receive request");
                }
            }
        }
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let request_sender = request_sender.clone();
        tokio::spawn(async move {
            process(socket, request_sender).await;
        });
    }
}

async fn handle(request: RequestHandle) {
    debug!("req: {:?}", request);
    let RequestHandle { callback, request } = request;
    let InternalRequest { config, request } = request;
    let built = build::build(&request);
    let mut child = match arma::start(&config, &built).await {
        Ok((_profile, child)) => child,
        Err(e) => {
            error!("Failed to start server: {}", e);
            let _ = callback.send(Response::Error(e));
            return;
        }
    };
    let _ = child.wait().await;
    match request {
        Request::Execute(_) => {
            match std::panic::catch_unwind(move || {
                let content = std::fs::read_to_string(built.path.join("execute.txt")).unwrap();
                serde_json::from_str(&content).unwrap()
            }) {
                Ok(result) => {
                    let _ = callback.send(Response::Execute(Ok(result)));
                }
                Err(e) => {
                    error!("Failed to read execute.txt: {:?}", e);
                    let _ = callback.send(Response::Error(format!("panic: {:?}", e)));
                }
            }
        }
        Request::Compare(_) => {
            match std::panic::catch_unwind(move || {
                let content = std::fs::read_to_string(built.path.join("compare.txt")).unwrap();
                serde_json::from_str(&content).unwrap()
            }) {
                Ok(results) => {
                    let _ = callback.send(Response::Compare(Ok(results)));
                }
                Err(e) => {
                    error!("Failed to read compare.txt: {:?}", e);
                    let _ = callback.send(Response::Error(format!("panic: {:?}", e)));
                }
            }
        }
    }
}

async fn process(mut socket: TcpStream, queue: tokio::sync::mpsc::Sender<RequestHandle>) {
    let addr = socket.peer_addr().unwrap();
    trace!("[{}] Connection received", addr);
    let (read, write) = socket.split();
    let mut read = BufReader::new(read);
    let mut write = BufWriter::new(write);
    // Write the header ID to the client.
    write.write_all(HEADER_ID).await.unwrap();
    write.flush().await.unwrap();
    // Expect the client to echo the header ID back to us.
    let mut buf = [0; 16];
    read.read_exact(&mut buf).await.unwrap();
    if buf != *HEADER_ID {
        error!("[{}] Invalid header ID", addr);
        return;
    }
    // The client has successfully connected.
    info!("[{}] Connected", addr);

    let server_config = ServerConfig::from_async_reader(&mut read).await.unwrap();
    debug!("[{}] Received server config: {:?}", addr, server_config);

    // Send wait packet to client
    write.write_all(&[1]).await.unwrap();
    write.flush().await.unwrap();

    loop {
        let request = arma_bench::Request::from_async_reader(&mut read)
            .await
            .unwrap();
        debug!("[{}] Received request: {:?}", addr, request);
        let (tx, rx) = tokio::sync::oneshot::channel();
        queue
            .send(RequestHandle {
                callback: tx,
                request: InternalRequest {
                    config: server_config.clone(),
                    request,
                },
            })
            .await
            .unwrap();
        let response = rx.await.unwrap();
        debug!("[{}] Sending response: {:?}", addr, response);
        response.write_async(&mut write).await.unwrap();
    }
}
