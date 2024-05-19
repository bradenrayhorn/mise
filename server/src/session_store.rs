use anyhow::Context;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use crate::domain::{Session, SessionKey};

// Error

#[derive(Error, Debug)]
pub enum Error {
    #[error("no session found")]
    NotFound,

    #[error("serialization error")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<tokio::sync::oneshot::error::RecvError> for Error {
    fn from(value: tokio::sync::oneshot::error::RecvError) -> Self {
        Error::Other(value.into())
    }
}

#[derive(Clone)]
pub struct SessionStore {
    sender: mpsc::Sender<Message>,
}

impl SessionStore {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        SessionStore { sender }
    }

    pub async fn get(&self, key: SessionKey) -> Result<Session, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Get {
            key,
            respond_to: tx,
        };

        let _ = self.sender.send(msg).await;
        rx.await.context("Session::get")?
    }

    pub async fn set(&self, session: Session) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Set {
            session,
            respond_to: tx,
        };

        let _ = self.sender.send(msg).await;
        rx.await.context("Session::set")?
    }

    pub async fn delete(&self, key: SessionKey) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Delete {
            key,
            respond_to: tx,
        };

        let _ = self.sender.send(msg).await;
        rx.await.context("Session::delete")?
    }
}

pub enum Message {
    Get {
        key: SessionKey,
        respond_to: oneshot::Sender<Result<Session, Error>>,
    },
    Delete {
        key: SessionKey,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
    Set {
        session: Session,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
