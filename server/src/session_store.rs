use std::time::Duration;

use anyhow::Context;
use rand::Rng;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};

use crate::domain::{Session, SessionKey};

// Error

#[derive(Error, Debug)]
pub enum Error {
    #[error("no session found")]
    NotFound(#[source] anyhow::Error),

    #[error("could acquire refresh lock")]
    RefreshLockTimeout,

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

pub struct RefreshLockStatus {
    pub was_locked: bool,
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

    pub async fn lock_refresh(&self, key: SessionKey) -> Result<(), Error> {
        let mut retries = 0;
        while retries < 9 {
            let (tx, rx) = oneshot::channel();
            let msg = Message::LockRefresh {
                key: key.clone(),
                max_lock: Duration::from_secs(45),
                respond_to: tx,
            };

            let _ = self.sender.send(msg).await;
            let has_lock = rx.await.context("Session::lock_refresh")??;

            if has_lock {
                return Ok(());
            }

            let delay_ms = (2_u64.pow(retries) * 200) + (rand::thread_rng().gen_range(0..50));
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            retries += 1;
        }

        Err(Error::RefreshLockTimeout)
    }

    pub async fn unlock_refresh(&self, key: SessionKey) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::UnlockRefresh {
            key,
            respond_to: tx,
        };

        let _ = self.sender.send(msg).await;
        rx.await.context("Session::unlock_refresh")?
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
    LockRefresh {
        key: SessionKey,
        max_lock: Duration,
        respond_to: oneshot::Sender<Result<bool, Error>>,
    },
    UnlockRefresh {
        key: SessionKey,
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
