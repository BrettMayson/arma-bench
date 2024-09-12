use std::{
    io::{Read, Write},
    sync::Once,
};

use arma_bench::{Client, ServerConfig};

static SERVER: Once = Once::new();
static PORT: u16 = 31846;

fn start_server() {
    SERVER.call_once(|| {
        std::thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .expect("Failed to create runtime")
                .block_on(async {
                    arma_bench_server::server(format!("localhost:{PORT}")).await;
                });
        });
    });
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[test]
fn client() {
    start_server();
    Client::connect_with_port("localhost", PORT, &ServerConfig::default())
        .expect("Failed to connect");
}

#[test]
fn bad_header() {
    start_server();
    let mut stream =
        std::net::TcpStream::connect(format!("localhost:{PORT}")).expect("Failed to connect");
    stream
        .write_all(b"SENDINGBADHEADER")
        .expect("Failed to send bad header");
    let mut buf = [0; 16];
    stream
        .read_exact(&mut buf)
        .expect("Failed to read header ID");
    assert_eq!(&buf, arma_bench::HEADER_ID);
    // server should have disconnected
    let mut buf = [0; 1];
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(100)))
        .expect("Failed to set read timeout");
    let res = stream.read_exact(&mut buf);
    assert!(res.is_err());
}
