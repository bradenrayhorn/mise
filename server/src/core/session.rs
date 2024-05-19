use anyhow::anyhow;
use base64::Engine;
use ring::rand::SecureRandom;

use crate::{
    core,
    domain::{Session, SessionKey, User},
    oidc,
    session_store::{self, SessionStore},
};

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("crypto random error")]
    CryptoRandom,

    #[error("time out of bounds")]
    TimeOutOfBounds,
}

impl From<Error> for core::Error {
    fn from(value: Error) -> Self {
        core::Error::Other(value.into())
    }
}

// maximum number of seconds a session may last without refresh
pub const SESSION_EXPIRES_IN: i64 = 60 * 60 * 24;

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
) -> Result<(String, SessionKey), core::Error> {
    let session: Session = store.get(key.clone()).await.map_err(|err| match err {
        session_store::Error::NotFound => {
            core::Error::Unauthenticated(anyhow!("session not found"))
        }
        _ => err.into(),
    })?;

    if session.revalidate_at < chrono::Utc::now() {
        // TODO - need to acquire lock on session key before attempting to refresh
        //      - add a new feature to cache, locks, separate from normal cache value.
        //      - prevents two requests from the same session trying to use the same refresh token.
        //
        let authenticated = match oidc::refresh_auth(oidc, session.refresh_token.clone()).await {
            Ok(authenticated) => Ok(authenticated),
            // TODO - can delete session from cache if refresh fails
            Err(err) => Err(core::Error::Unauthenticated(err.into())),
        }?;

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

        // TODO - instead of removing old session right away, give grace period by setting the
        // expiration to 15 seconds in future.
        store.delete(key).await?;
        store.set(new_session).await?;

        Ok((session.user_id, SessionKey(new_key)))
    } else if session.expires_at < chrono::Utc::now() {
        return Err(core::Error::Unauthenticated(anyhow!("session expired")));
    } else {
        Ok((session.user_id, SessionKey(session.key)))
    }
}

fn new_session_id() -> Result<String, Error> {
    let mut bytes: [u8; 32] = [0; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| Error::CryptoRandom)?;

    Ok(base64::engine::general_purpose::URL_SAFE.encode(bytes))
}
