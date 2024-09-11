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
    pub fn connect_with_port(host: &str, port: u16, config: ServerConfig) -> Result<Self, String> {
        let mut stream = TcpStream::connect(format!("{}:{}", host, port)).unwrap();

        // Send the header ID to the server.
        stream.write_all(HEADER_ID).unwrap();

        // Expect the server to echo the header ID back to us.
        let mut buf = [0; 16];
        stream.read_exact(&mut buf).unwrap();
        if buf != *HEADER_ID {
            return Err("Invalid header ID".to_string());
        }

        // Send the server config to the server.
        config.write(&mut stream).unwrap();

        // Expect a 1 for wait, disconnect if not.
        let mut buf = [0; 1];
        stream.read_exact(&mut buf).unwrap();
        if buf[0] != 1 {
            return Err("Invalid ACK".to_string());
        }

        Ok(Self {
            stream: Mutex::new(stream),
        })
    }

    pub fn connect(host: &str, config: ServerConfig) -> Result<Self, String> {
        Self::connect_with_port(host, DEFAULT_PORT, config)
    }

    pub fn execute(&self, content: &str) -> Result<ExecuteResult, String> {
        let mut stream = self.stream.lock().unwrap();
        Request::Execute(content.to_string())
            .write(&mut *stream)
            .unwrap();

        let result = Response::from_reader(&mut *stream).unwrap();
        match result {
            Response::Execute(Ok(res)) => Ok(res),
            Response::Execute(Err(err)) => Err(err),
            Response::Error(err) => Err(err),
            _ => Err("Invalid response".to_string()),
        }
    }

    pub fn compare(&self, requests: Vec<CompareRequest>) -> Result<Vec<CompareResult>, String> {
        let mut stream = self.stream.lock().unwrap();
        Request::Compare(requests).write(&mut *stream).unwrap();

        let result = Response::from_reader(&mut *stream).unwrap();
        match result {
            Response::Compare(Ok(result)) => Ok(result),
            Response::Compare(Err(err)) => Err(err),
            Response::Error(err) => Err(err),
            _ => Err("Invalid response".to_string()),
        }
    }
}
