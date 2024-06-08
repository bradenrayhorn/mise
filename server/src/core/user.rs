use crate::{
    core::Error,
    datastore::{self, Pool},
    domain::{self, RegisteringUser, SessionKey},
    oidc,
    session_store::SessionStore,
};

use super::session;

pub async fn get(datasource: &Pool, id: &str) -> Result<String, Error> {
    let user = datasource
        .get_user(id.to_owned())
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => Error::NotFound(format!("user {id} does not exist")),
            _ => Error::Other(err.into()),
        })?;

    Ok(user.name)
}

pub async fn on_authenticated(
    datasource: &Pool,
    session_store: &SessionStore,
    authenticated: &oidc::Authenticated,
) -> Result<SessionKey, Error> {
    let registering = RegisteringUser {
        potential_id: domain::user::Id::new().into(),
        oauth_id: format!("custom|{}", authenticated.subject),
        name: authenticated.name.to_string(),
    };

    let user = datasource
        .upsert_user_by_oauth_id(registering)
        .await
        .map_err(|err| Error::Other(err.into()))?;

    let session_key = session::begin(session_store, &user, authenticated).await?;

    Ok(session_key)
}
