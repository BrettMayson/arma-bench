use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    CompareRequest, CompareResult, ExecuteResult, Message, Request, Response, ServerConfig,
    DEFAULT_PORT, HEADER_ID,
};

pub struct Client {
    stream: Mutex<TcpStream>,
}

impl Client {
    /// Connect to the server with a specific port.
    ///
    /// # Errors
    /// Returns a string error if the connection fails.
    ///
    /// # Panics
    /// Panics on TCP stream errors.
    pub fn connect_with_port(host: &str, port: u16, config: &ServerConfig) -> Result<Self, String> {
        let mut stream = TcpStream::connect(format!("{host}:{port}")).expect("Failed to connect");
        // Send the header ID to the server.
        stream
            .write_all(HEADER_ID)
            .expect("Failed to send header ID");

        // Expect the server to echo the header ID back to us.
        let mut buf = [0; 16];
        stream
            .read_exact(&mut buf)
            .expect("Failed to read header ID");
        if buf != *HEADER_ID {
            return Err("Invalid header ID".to_string());
        }

        // Send the server config to the server.
        config
            .write(&mut stream)
            .expect("Failed to send server config");

        // Expect a 1 for wait, disconnect if not.
        let mut buf = [0; 1];
        stream.read_exact(&mut buf).expect("Failed to read ACK");
        if buf[0] != 1 {
            return Err("Invalid ACK".to_string());
        }

        Ok(Self {
            stream: Mutex::new(stream),
        })
    }

    /// Connect to the server with the default port.
    ///
    /// # Errors
    /// Returns a string error if the connection fails.
    ///
    /// # Panics
    /// Panics on TCP stream errors.
    pub fn connect(host: &str, config: &ServerConfig) -> Result<Self, String> {
        Self::connect_with_port(host, DEFAULT_PORT, config)
    }

    /// Execute a script on the server, returning the benchmark and result.
    ///
    /// # Errors
    /// Returns a string error if the request fails.
    ///
    /// # Panics
    /// Panics on TCP stream errors.
    pub fn execute(&self, content: &str) -> Result<ExecuteResult, String> {
        let result = {
            let mut stream = self.stream.lock().expect("Failed to lock stream");
            Request::Execute(content.to_string())
                .write(&mut *stream)
                .expect("Failed to send request");
            Response::from_reader(&mut *stream).expect("Failed to read response")
        };
        match result {
            Response::Execute(Ok(res)) => Ok(res),
            Response::Execute(Err(err)) | Response::Error(err) => Err(err),
            _ => Err("Invalid response".to_string()),
        }
    }

    /// Compare multiple scripts on the server, returning the benchmarks and results.
    ///
    /// # Errors
    /// Returns a string error if the request fails.
    ///
    /// # Panics
    /// Panics on TCP stream errors.
    pub fn compare(&self, requests: Vec<CompareRequest>) -> Result<Vec<CompareResult>, String> {
        let result = {
            let mut stream = self.stream.lock().expect("Failed to lock stream");
            Request::Compare(requests)
                .write(&mut *stream)
                .expect("Failed to write request");
            Response::from_reader(&mut *stream).expect("Failed to read response")
        };
        match result {
            Response::Compare(Ok(result)) => Ok(result),
            Response::Compare(Err(err)) | Response::Error(err) => Err(err),
            _ => Err("Invalid response".to_string()),
        }
    }
}
