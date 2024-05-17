use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    PrivateCookieJar,
};
use serde::Deserialize;

use crate::{
    core::{self, Error},
    oidc,
};

use super::server::AppState;

#[derive(Deserialize)]
pub struct AuthCompleteParams {
    state: String,
    code: String,
}

pub async fn init(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, Redirect), Error> {
    let (auth_url, oidc_state) = oidc::begin_auth(&state.oidc_provider);

    let jar = jar.add(
        Cookie::build((
            "s",
            serde_json::to_string(&oidc_state).map_err(|err| Error::Other(err.into()))?,
        ))
        .http_only(true)
        .secure(true)
        // must be lax so that the cookie is attached upon redirect from the authorization server
        .same_site(SameSite::Lax)
        .max_age(cookie::time::Duration::seconds(180))
        .build(),
    );

    Ok((jar, Redirect::temporary(&auth_url.to_string())))
}

pub async fn callback(
    jar: PrivateCookieJar,
    State(state): State<AppState>,
    params: Query<AuthCompleteParams>,
) -> Result<String, Error> {
    let oidc_state = serde_json::from_str::<oidc::AuthState>(
        jar.get("s")
            .ok_or(Error::Unauthenticated(anyhow!(
                "Missing OIDC state cookie."
            )))?
            .value(),
    )
    .map_err(|err| Error::Other(err.into()))?;

    let authenticated = oidc::complete_auth(
        &state.oidc_provider,
        oidc_state,
        oidc::CallbackParams {
            state: &params.state,
            code: &params.code,
        },
    )
    .await
    .map_err(|err| Error::Unauthenticated(err.into()))?;

    // persist user
    core::user::upsert(
        &state.datasource,
        "custom",
        &authenticated.subject,
        &authenticated.name,
    )
    .await?;

    Ok(format!("authenticated = {}", authenticated.subject))
}
