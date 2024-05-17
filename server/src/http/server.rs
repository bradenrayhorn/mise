use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{FromRef, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Router,
};
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar,
};
use serde::Deserialize;

use crate::{
    config::Config,
    core::Error,
    datastore::{self, Pool},
    oidc,
};

pub struct Server {
    config: Config,
    datasource: Pool,
}

#[derive(Clone)]
struct AppState {
    datasource: Pool,
    key: Key,
    oidc_provider: Arc<oidc::Provider>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl Server {
    pub fn new(config: Config, datasource: Pool) -> Self {
        Server { config, datasource }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        println!("Starting http server on port {:?}", self.config.http_port);

        let oidc_provider = oidc::Provider::new((&self.config).try_into()?).await?;

        let state = AppState {
            key: Key::generate(),
            datasource: self.datasource.clone(),
            oidc_provider: Arc::new(oidc_provider),
        };

        let router: Router = Router::new()
            .route("/health-check", axum::routing::get(health))
            .route("/get-a-user", axum::routing::get(get_user))
            .route("/auth/init", axum::routing::get(init_auth))
            .route("/auth/complete", axum::routing::get(auth_complete))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.http_port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn get_user(State(state): State<AppState>) -> Result<String, Error> {
    get_user_logic(state.datasource, "123").await
}

async fn init_auth(
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

#[derive(Deserialize)]
struct AuthCompleteParams {
    state: String,
    code: String,
}

async fn auth_complete(
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

    Ok(format!("authenticated = {}", authenticated.subject))
}

async fn get_user_logic(datasource: Pool, id: &str) -> Result<String, Error> {
    let user = datasource
        .get_user(id.to_owned())
        .await
        .map_err(|err| match err {
            datastore::Error::NotFound => Error::NotFound(format!("user {} does not exist", id)),
            _ => Error::Other(err.into()),
        })?;

    Ok(user.name)
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            // TODO - log all errors, not just unknown, if debug logging is on
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            Error::Unauthenticated(_) => {
                (StatusCode::UNAUTHORIZED, "Unauthenticated.").into_response()
            }
            Error::Other(err) => {
                println!("error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
