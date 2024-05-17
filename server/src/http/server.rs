use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{FromRef, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};
use axum_extra::extract::cookie::Key;

use crate::{
    config::Config,
    core::{self, Error},
    datastore::Pool,
    http, oidc,
};

pub struct Server {
    config: Config,
    datasource: Pool,
}

#[derive(Clone)]
pub struct AppState {
    pub datasource: Pool,
    pub key: Key,
    pub oidc_provider: Arc<oidc::Provider>,
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
            .route("/health-check", axum::routing::get(|| async { "ok" }))
            .route("/get-a-user", axum::routing::get(get_user))
            .route("/auth/init", axum::routing::get(http::auth::init))
            .route("/auth/complete", axum::routing::get(http::auth::callback))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.http_port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn get_user(State(state): State<AppState>) -> Result<String, Error> {
    core::user::get(&state.datasource, "123").await
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
