use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub oauth_id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct RegisteringUser {
    pub potential_id: String,
    pub oauth_id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct SessionKey(pub String);

impl Deref for SessionKey {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Session {
    pub key: String,
    pub user_id: String,
    pub refresh_token: String,
    pub revalidate_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub enum SessionStatus {
    MustRevalidate,
    Expired,
    Ok,
}

impl Session {
    pub fn status(&self) -> SessionStatus {
        let now = chrono::Utc::now();

        if self.expires_at <= now {
            return SessionStatus::Expired;
        } else if self.revalidate_at <= now {
            return SessionStatus::MustRevalidate;
        } else {
            return SessionStatus::Ok;
        }
    }
}
