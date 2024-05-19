use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use chrono::Utc;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

// Error

#[derive(Error, Debug)]
pub enum Error {
    #[error("no cache item found matching key")]
    NoMatchingValue,

    #[error("no available connections")]
    NoConnections,

    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(value: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::Unknown(anyhow!(value))
    }
}

// Pool

#[derive(Clone)]
pub struct Cache {
    // TODO - More efficient locking and data structure could be used, but this is fine for now.
    connections: Arc<Mutex<Vec<mpsc::Sender<CacheMessage>>>>,
}

#[derive(Clone)]
struct CacheConnection {
    sender: mpsc::Sender<CacheMessage>,
    connections: Arc<Mutex<Vec<mpsc::Sender<CacheMessage>>>>,
}

impl Drop for CacheConnection {
    fn drop(&mut self) {
        // release thread back to pool when connection is done
        let mut connections = self.connections.lock().unwrap();
        connections.push(self.sender.clone())
    }
}

impl Cache {
    pub fn new(connections: Vec<mpsc::Sender<CacheMessage>>) -> Self {
        Cache {
            connections: Arc::new(Mutex::new(connections)),
        }
    }

    fn conn(&self) -> Result<CacheConnection, Error> {
        // TODO - handle:
        //   connection healthcheck before use
        //   waiting for new connections if none available
        let mut connections = self.connections.lock().unwrap();
        match connections.pop() {
            Some(conn) => Ok(CacheConnection {
                sender: conn,
                connections: self.connections.clone(),
            }),
            None => Err(Error::NoConnections),
        }
    }

    pub async fn get<T>(&self, key: String) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = CacheMessage::Get {
            key,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg).await;
        let result = rx.await.context("send CacheMessage::Get")??;
        Ok(serde_json::from_str::<T>(&result)?)
    }

    pub async fn set(
        &self,
        key: String,
        value: impl Serialize,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = CacheMessage::Set {
            key,
            value: serde_json::to_string(&value)?,
            expires_at,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg).await;
        rx.await.context("send CacheMessage::Set")?
    }

    pub async fn remove(&self, key: String) -> Result<(), Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = CacheMessage::Remove {
            key,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg).await;
        rx.await.context("send CacheMessage::Remove")?
    }
}

pub enum CacheMessage {
    Health {
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    Get {
        key: String,
        respond_to: oneshot::Sender<Result<String, Error>>,
    },
    Remove {
        key: String,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    Set {
        key: String,
        value: String,
        expires_at: chrono::DateTime<Utc>,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
