use base64::Engine;
use ring::rand::SecureRandom;

use crate::{
    core,
    domain::{Session, SessionKey, SessionStatus, User},
    oidc,
    session_store::{self, SessionStore},
};

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("crypto random error")]
    CryptoRandom,

    #[error("session is expired")]
    Expired,

    #[error("time out of bounds")]
    TimeOutOfBounds,
}

impl From<Error> for core::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Expired => core::Error::Unauthenticated(value.into()),
            _ => core::Error::Other(value.into()),
        }
    }
}

// maximum number of seconds a session may last before refresh
pub const SESSION_EXPIRES_IN: i64 = 60 * 60 * 24 * 90;

pub struct Active {
    pub key: SessionKey,
    pub user_id: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub async fn begin(
    store: &SessionStore,
    user: &User,
    authenticated: &oidc::Authenticated,
) -> Result<SessionKey, core::Error> {
    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::TimeDelta::seconds(SESSION_EXPIRES_IN))
        .ok_or(Error::TimeOutOfBounds)?;

    let session = Session {
        key: new_session_id()?,
        user_id: user.id.clone(),
        refresh_token: authenticated.refresh_token.clone(),
        revalidate_at: authenticated.expires_at,
        expires_at,
    };

    let session_key = session.key.clone();

    // TODO - also encrypt session value before inserting
    store.set(session).await?;

    Ok(SessionKey(session_key))
}

pub async fn get(
    store: &SessionStore,
    oidc: &oidc::Provider,
    key: SessionKey,
) -> Result<Active, core::Error> {
    let session = find_session(store, key.clone()).await?;

    match session.status() {
        SessionStatus::Ok => Ok(Active {
            key: SessionKey(session.key),
            user_id: session.user_id,
            expires_at: session.expires_at,
        }),
        SessionStatus::Expired => Err(Error::Expired.into()),
        SessionStatus::MustRevalidate => {
            // A refresh token can only be used once, therefore only one request to refresh must
            // be attempted at a time otherwise multiple requests could try to use
            // the same refresh token.
            store.lock_refresh(key.clone()).await?;

            let result = refresh_session(store, oidc, key.clone()).await;

            store.unlock_refresh(key).await?;

            result
        }
    }
}

async fn find_session(store: &SessionStore, key: SessionKey) -> Result<Session, core::Error> {
    store.get(key).await.map_err(|err| match err {
        session_store::Error::NotFound(err) => {
            core::Error::Unauthenticated(err.context("session not found"))
        }
        _ => err.into(),
    })
}

async fn refresh_session(
    store: &SessionStore,
    oidc: &oidc::Provider,
    key: SessionKey,
) -> Result<Active, core::Error> {
    let session = find_session(store, key.clone()).await?;

    match session.status() {
        SessionStatus::Ok => Ok(Active {
            key: SessionKey(session.key),
            user_id: session.user_id,
            expires_at: session.expires_at,
        }),
        SessionStatus::Expired => Err(Error::Expired.into()),
        SessionStatus::MustRevalidate => {
            let authenticated = match oidc::refresh_auth(oidc, session.refresh_token.clone()).await
            {
                Ok(authenticated) => Ok(authenticated),
                Err(err) => {
                    store.delete(key.clone()).await?;
                    Err(core::Error::Unauthenticated(err.into()))
                }
            }?;

            // build a new session
            let expires_at = chrono::Utc::now()
                .checked_add_signed(chrono::TimeDelta::seconds(SESSION_EXPIRES_IN))
                .ok_or(Error::TimeOutOfBounds)?;

            let new_key = new_session_id()?;
            let new_session = Session {
                key: new_key.clone(),
                user_id: session.user_id.clone(),
                refresh_token: authenticated.refresh_token,
                revalidate_at: authenticated.expires_at,
                expires_at,
            };

            // the old session should be kept alive for a short grace period to allow any requests
            // still using the old session key to process successfully.
            let original_session_grace = chrono::Utc::now()
                .checked_add_signed(chrono::TimeDelta::seconds(15))
                .ok_or(Error::TimeOutOfBounds)?;

            let original_session = Session {
                key: key.to_string(),
                user_id: session.user_id.clone(),
                refresh_token: session.refresh_token,
                revalidate_at: original_session_grace,
                expires_at: original_session_grace,
            };

            store.set(original_session).await?;
            store.set(new_session).await?;

            Ok(Active {
                key: SessionKey(new_key),
                user_id: session.user_id.clone(),
                expires_at,
            })
        }
    }
}

fn new_session_id() -> Result<String, Error> {
    let mut bytes: [u8; 32] = [0; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| Error::CryptoRandom)?;

    Ok(base64::engine::general_purpose::URL_SAFE.encode(bytes))
}
