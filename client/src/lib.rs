mod client;

use std::io::{Read, Write};

use arma_rs::Value;
use rmp_serde::Deserializer;
use serde::{Deserialize, Serialize};

pub use client::Client;

/// Sent between the client and server at the start of a connection.
pub static HEADER_ID: &[u8; 16] = b"ARMABENCH-VER010";
pub static DEFAULT_PORT: u16 = 7562;

pub trait Message: Deserialize<'static> + Serialize + Sync {
    /// Read a message from a reader.
    ///
    /// # Errors
    /// Returns a string error if the message could not be read.
    ///
    /// # Panics
    /// Panics on I/O errors.
    fn from_reader<R: Read>(reader: &'_ mut R) -> Result<Self, String>
    where
        Self: Sized,
    {
        let mut len_buf = [0; 8];
        reader.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
        let len = usize::try_from(u64::from_le_bytes(len_buf)).map_err(|e| e.to_string())?;
        let mut payload = vec![0; len];
        reader.read_exact(&mut payload).map_err(|e| e.to_string())?;
        Deserialize::deserialize(&mut Deserializer::new(payload.as_slice()))
            .map_err(|e| e.to_string())
    }

    /// Write a message to a writer.
    ///
    /// # Errors
    /// Returns a string error if the message could not be written.
    ///
    /// # Panics
    /// Panics on I/O errors.
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), String> {
        let payload = rmp_serde::to_vec(self).map_err(|e| e.to_string())?;
        let mut len_buf = [0; 8];
        len_buf.copy_from_slice(&(payload.len() as u64).to_le_bytes());
        writer.write_all(&len_buf).map_err(|e| e.to_string())?;
        writer.write_all(&payload).map_err(|e| e.to_string())
    }

    #[cfg(feature = "tokio")]
    fn from_async_reader<R: tokio::io::AsyncRead + Unpin + Send>(
        reader: &'_ mut R,
    ) -> impl std::future::Future<Output = Result<Self, String>> + Send
    where
        Self: Sized,
    {
        async move {
            use tokio::io::AsyncReadExt;
            let mut len_buf = [0; 8];
            reader
                .read_exact(&mut len_buf)
                .await
                .map_err(|e| e.to_string())?;
            let len = usize::try_from(u64::from_le_bytes(len_buf)).map_err(|e| e.to_string())?;
            let mut payload = vec![0; len];
            reader
                .read_exact(&mut payload)
                .await
                .map_err(|e| e.to_string())?;
            Deserialize::deserialize(&mut Deserializer::new(payload.as_slice()))
                .map_err(|e| e.to_string())
        }
    }

    #[cfg(feature = "tokio")]
    fn write_async<W: tokio::io::AsyncWrite + Unpin + Send>(
        &self,
        writer: &mut W,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            use tokio::io::AsyncWriteExt;
            let payload = rmp_serde::to_vec(self).map_err(|e| e.to_string())?;
            let mut len_buf = [0; 8];
            len_buf.copy_from_slice(&(payload.len() as u64).to_le_bytes());
            writer
                .write_all(&len_buf)
                .await
                .map_err(|e| e.to_string())?;
            writer
                .write_all(&payload)
                .await
                .map_err(|e| e.to_string())?;
            writer.flush().await.map_err(|e| e.to_string())
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub binary: String,
    pub branch: String,
    pub branch_password: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            binary: "arma3server_x64".to_string(),
            branch: "public".to_string(),
            branch_password: String::new(),
        }
    }
}

impl Message for ServerConfig {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Request {
    Execute(String),
    Compare(Vec<CompareRequest>),
}

impl Message for Request {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompareRequest {
    pub id: u16,
    pub sqfc: bool,
    pub content: Vec<u8>,
}

impl Message for CompareRequest {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompareResult {
    pub id: u16,
    pub time: f64,
    pub iter: u32,
    pub ret: Value,
}

impl Message for CompareResult {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecuteResult {
    pub time: f64,
    pub iter: u32,
    pub ret: Value,
}

impl Message for ExecuteResult {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Response {
    Error(String),
    Execute(Result<ExecuteResult, String>),
    Compare(Result<Vec<CompareResult>, String>),
}

impl Message for Response {}
