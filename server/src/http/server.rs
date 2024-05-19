use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{FromRef, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Extension, Router,
};
use axum_extra::extract::{cookie::Key, CookieJar};
use cookie::Cookie;

use crate::{
    cache,
    config::Config,
    core::{self, Error},
    datastore::Pool,
    domain::SessionKey,
    http, oidc,
};

pub struct Server {
    config: Config,
    datasource: Pool,
    cache: cache::Cache,
}

#[derive(Clone)]
pub struct AppState {
    pub datasource: Pool,
    pub cache: cache::Cache,
    pub key: Key,
    pub oidc_provider: Arc<oidc::Provider>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl Server {
    pub fn new(config: Config, datasource: Pool, cache: cache::Cache) -> Self {
        Server {
            config,
            datasource,
            cache,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        println!("Starting http server on port {:?}", self.config.http_port);

        let oidc_provider = oidc::Provider::new((&self.config).try_into()?).await?;

        let state = AppState {
            key: Key::generate(),
            cache: self.cache.clone(),
            datasource: self.datasource.clone(),
            oidc_provider: Arc::new(oidc_provider),
        };

        let router: Router = Router::new()
            .route("/health-check", axum::routing::get(|| async { "ok" }))
            //
            // OIDC routes
            .route("/auth/init", axum::routing::get(http::auth::init))
            .route("/auth/complete", axum::routing::get(http::auth::callback))
            //
            // Authenticated routes
            .nest(
                "/",
                Router::new()
                    .route("/auth/me", axum::routing::get(get_me))
                    .layer(middleware::from_fn_with_state(state.clone(), auth)),
            )
            //
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.http_port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn get_me(Extension(user): Extension<AuthenticatedUser>) -> Result<String, Error> {
    Ok(format!("you are: {}", user.id))
}

#[derive(Clone)]
pub struct AuthenticatedUser {
    id: String,
}

async fn auth(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<(CookieJar, Response), Error> {
    let cookie = jar
        .get("id")
        .ok_or(Error::Unauthenticated(anyhow!("missing session cookie")))?
        .value();

    let (user_id, new_session_key) = core::session::get(
        &state.cache,
        &state.oidc_provider,
        SessionKey(cookie.to_string()),
    )
    .await?;

    let jar = if cookie != new_session_key.to_string() {
        jar.add(
            Cookie::build(("id", new_session_key.to_string()))
                .http_only(true)
                .secure(true)
                .same_site(cookie::SameSite::Strict)
                .max_age(cookie::time::Duration::seconds(
                    core::session::SESSION_EXPIRES_IN,
                )),
        )
    } else {
        jar
    };

    let user = AuthenticatedUser { id: user_id };
    req.extensions_mut().insert(user);

    Ok((jar, next.run(req).await))
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            // TODO - log all errors, not just unknown, if debug logging is on
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            Error::Unauthenticated(err) => {
                println!("error: {:?}", err);
                (StatusCode::UNAUTHORIZED, "Unauthenticated.").into_response()
            }
            Error::Other(err) => {
                println!("error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
