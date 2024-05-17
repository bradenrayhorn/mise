use std::sync::{mpsc, Arc, Mutex};

use anyhow::anyhow;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::domain::User;

// Error

#[derive(Error, Debug)]
pub enum Error {
    #[error("record not found")]
    NotFound,

    #[error("not in a transaction")]
    NotTransaction,

    #[error("no available connections")]
    NoConnections,

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
pub struct Pool {
    // TODO - More efficient locking and data structure could be used, but this is fine for now.
    connections: Arc<Mutex<Vec<mpsc::Sender<DatastoreMessage>>>>,
}

#[derive(Clone)]
struct PoolConnection {
    sender: mpsc::Sender<DatastoreMessage>,
    connections: Arc<Mutex<Vec<mpsc::Sender<DatastoreMessage>>>>,
}

impl Drop for PoolConnection {
    fn drop(&mut self) {
        // release thread back to pool when connection is done
        let mut connections = self.connections.lock().unwrap();
        connections.push(self.sender.clone())
    }
}

impl Pool {
    pub fn new(connections: Vec<mpsc::Sender<DatastoreMessage>>) -> Self {
        Pool {
            connections: Arc::new(Mutex::new(connections)),
        }
    }

    fn conn(&self) -> Result<PoolConnection, Error> {
        // TODO - should check if the connection is healthy before use.
        // - benchmark the change before and after to make sure performance is still okay.
        let mut connections = self.connections.lock().unwrap();
        match connections.pop() {
            Some(conn) => Ok(PoolConnection {
                sender: conn,
                connections: self.connections.clone(),
            }),
            None => Err(Error::NoConnections),
        }
    }

    pub async fn get_user(&self, id: String) -> Result<User, Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = DatastoreMessage::GetUser { id, respond_to: tx };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn upsert_user_by_oauth_id(&self, user: User) -> Result<(), Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = DatastoreMessage::UpsertUserByOauthId {
            user,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }
}

pub enum DatastoreMessage {
    Health {
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    GetUser {
        id: String,
        respond_to: oneshot::Sender<Result<User, Error>>,
    },
    UpsertUserByOauthId {
        user: User,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
