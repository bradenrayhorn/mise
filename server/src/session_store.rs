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

const PRUNE_EXPIRED_SESSIONS_DELAY: u64 = 60 * 60;
const PRUNE_EXPIRED_REFRESH_LOCKS_DELAY: u64 = 60 * 60;

#[derive(Clone)]
pub struct SessionStore {
    sender: mpsc::Sender<Message>,
}

pub struct RefreshLockStatus {
    pub was_locked: bool,
}

impl SessionStore {
    #[must_use]
    pub fn new(
        sender: mpsc::Sender<Message>,
        background_job_result_sender: mpsc::Sender<BackgroundResultMessage>,
    ) -> Self {
        let prune_sessions_sender = sender.clone();
        let prune_refresh_locks_sender = sender.clone();
        let prune_sessions_background_sender = background_job_result_sender.clone();
        let prune_refresh_locks_background_sender = background_job_result_sender;

        // prune expired sessions
        tokio::spawn(async move {
            loop {
                let (tx, rx) = oneshot::channel();
                let msg = Message::PruneExpired { respond_to: tx };

                let _ = prune_sessions_sender.send(msg).await;
                let result = rx
                    .await
                    .context("SessionStore::PruneExpired")
                    .map_err(Error::Other)
                    .and_then(|x| x);
                let _ = prune_sessions_background_sender
                    .send(BackgroundResultMessage::PruneExpiredSessions { result })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(PRUNE_EXPIRED_SESSIONS_DELAY))
                    .await;
            }
        });

        // prune expired refresh locks
        tokio::spawn(async move {
            loop {
                let (tx, rx) = oneshot::channel();
                let msg = Message::PruneExpiredRefresh { respond_to: tx };

                let _ = prune_refresh_locks_sender.send(msg).await;
                let result = rx
                    .await
                    .context("SessionStore::PruneRefreshLocks")
                    .map_err(Error::Other)
                    .and_then(|x| x);
                let _ = prune_refresh_locks_background_sender
                    .send(BackgroundResultMessage::PruneExpiredRefreshLocks { result })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(
                    PRUNE_EXPIRED_REFRESH_LOCKS_DELAY,
                ))
                .await;
            }
        });

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

#[derive(Debug)]
pub enum BackgroundResultMessage {
    PruneExpiredSessions { result: Result<(), Error> },
    PruneExpiredRefreshLocks { result: Result<(), Error> },
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
    PruneExpired {
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
    PruneExpiredRefresh {
        respond_to: oneshot::Sender<Result<(), Error>>,
    },
}
