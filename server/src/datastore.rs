use std::sync::{mpsc, Arc, Mutex};

use anyhow::anyhow;
use thiserror::Error;
use tokio::sync::oneshot;

use crate::domain::{RecipeDocument, RegisteringUser, User};

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
    connections: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
}

#[derive(Clone)]
struct PoolConnection {
    sender: mpsc::Sender<Message>,
    connections: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
}

impl Drop for PoolConnection {
    fn drop(&mut self) {
        // release thread back to pool when connection is done
        let mut connections = self.connections.lock().unwrap();
        connections.push(self.sender.clone());
    }
}

impl Pool {
    #[must_use]
    pub fn new(connections: Vec<mpsc::Sender<Message>>) -> Self {
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
        let msg = Message::GetUser { id, respond_to: tx };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn upsert_user_by_oauth_id(&self, user: RegisteringUser) -> Result<User, Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::UpsertUserByOauthId {
            user,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    // recipe

    pub async fn get_recipe(&self, id: String) -> Result<RecipeDocument, Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::GetRecipe { id, respond_to: tx };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn create_recipe(&self, recipe: RecipeDocument) -> Result<(), Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::CreateRecipe {
            recipe,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }

    pub async fn update_recipe(&self, recipe: RecipeDocument) -> Result<(), Error> {
        let conn = self.conn()?;
        let (tx, rx) = oneshot::channel();
        let msg = Message::UpdateRecipe {
            recipe,
            respond_to: tx,
        };

        let _ = conn.sender.send(msg);
        rx.await?
    }
}

pub enum Message {
    Health {
        respond_to: oneshot::Sender<Result<(), Error>>,
    },

    // user
    GetUser {
        id: String,
        respond_to: oneshot::Sender<Result<User, Error>>,
    },
    UpsertUserByOauthId {
        user: RegisteringUser,
        respond_to: oneshot::Sender<Result<User, Error>>,
    },

    // recipe
    GetRecipe {
        id: String,
        respond_to: oneshot::Sender<Result<RecipeDocument, Error>>,
    },
    CreateRecipe {
        recipe: RecipeDocument,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    UpdateRecipe {
        recipe: RecipeDocument,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
