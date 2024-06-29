use anyhow::anyhow;
use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{self, Error},
    oidc,
};

use super::server::AppState;

#[derive(Deserialize)]
pub struct InitParams {
    redirect_target: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SignedCookie {
    value: String,
    tag: Vec<u8>,
}

pub async fn init(
    State(state): State<AppState>,
    jar: CookieJar,
    params: Query<InitParams>,
) -> Result<(CookieJar, Redirect), Error> {
    let (auth_url, oidc_state) =
        oidc::begin_auth(&state.oidc_provider, params.redirect_target.clone())
            .map_err(|err| Error::Unauthenticated(err.into()))?;

    let cookie_value =
        serde_json::to_string(&oidc_state).map_err(|err| Error::Other(err.into()))?;

    let tag = ring::hmac::sign(&state.key, cookie_value.as_bytes());
    let signed_cookie = serde_json::to_string(&SignedCookie {
        value: cookie_value,
        tag: tag.as_ref().to_vec(),
    })
    .map_err(|err| Error::Other(err.into()))?;

    let jar = jar.add(
        Cookie::build(("s", signed_cookie))
            .path("/")
            .http_only(true)
            .secure(true)
            // must be lax so that the cookie is attached upon redirect from the authorization server
            .same_site(SameSite::Lax)
            .max_age(cookie::time::Duration::seconds(180))
            .build(),
    );

    Ok((jar, Redirect::temporary(auth_url.as_ref())))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    state: String,
    code: String,
}

pub async fn callback(
    jar: CookieJar,
    State(state): State<AppState>,
    params: Query<CallbackParams>,
) -> Result<(CookieJar, Redirect), Error> {
    let signed_cookie = serde_json::from_str::<SignedCookie>(
        jar.get("s")
            .ok_or(Error::Unauthenticated(anyhow!(
                "Missing OIDC state cookie."
            )))?
            .value(),
    )
    .map_err(|err| Error::Other(err.into()))?;

    ring::hmac::verify(
        &state.key,
        signed_cookie.value.as_bytes(),
        signed_cookie.tag.as_ref(),
    )
    .map_err(|_| Error::Unauthenticated(anyhow!("Cookie signature validation failed.")))?;

    let oidc_state = serde_json::from_str::<oidc::AuthState>(&signed_cookie.value)
        .map_err(|err| Error::Other(err.into()))?;

    // remove state cookie now that it has been used
    let jar = jar.remove(Cookie::from("s"));

    // exchange with oidc provider
    let (authenticated, redirect_target) = oidc::complete_auth(
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
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .max_age(cookie::time::Duration::seconds(
                core::session::SESSION_EXPIRES_IN,
            )),
    );

    Ok((
        jar,
        Redirect::temporary(redirect_target.as_ref().map_or("/", |s| s.as_str())),
    ))
}
