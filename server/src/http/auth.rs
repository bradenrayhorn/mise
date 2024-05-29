use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::Deserialize;

use crate::{
    core::{self, Error},
    oidc,
};

use super::server::AppState;

#[derive(Deserialize)]
pub struct CallbackParams {
    state: String,
    code: String,
}

pub async fn init(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), Error> {
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

    Ok((jar, Redirect::temporary(auth_url.as_ref())))
}

pub async fn callback(
    jar: CookieJar,
    State(state): State<AppState>,
    params: Query<CallbackParams>,
) -> Result<(CookieJar, String), Error> {
    let oidc_state = serde_json::from_str::<oidc::AuthState>(
        jar.get("s")
            .ok_or(Error::Unauthenticated(anyhow!(
                "Missing OIDC state cookie."
            )))?
            .value(),
    )
    .map_err(|err| Error::Other(err.into()))?;

    // remove state cookie now that it has been used
    let jar = jar.remove(Cookie::from("s"));

    // exchange with oidc provider
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

    // persist user and create session
    let session_key =
        core::user::on_authenticated(&state.datasource, &state.session_store, &authenticated)
            .await?;

    let jar = jar.add(
        Cookie::build(("id", session_key.to_string()))
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .max_age(cookie::time::Duration::seconds(
                core::session::SESSION_EXPIRES_IN,
            )),
    );

    Ok((jar, format!("authenticated = {}", authenticated.subject)))
}
