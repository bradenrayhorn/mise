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
    config::Config,
    core::{self, Error},
    datastore::Pool,
    domain::{self, SessionKey},
    http, oidc,
    session_store::SessionStore,
};

pub struct Server {
    config: Config,
    datasource: Pool,
    session_store: SessionStore,
    oidc_provider: Arc<oidc::Provider>,
}

#[derive(Clone)]
pub struct AppState {
    pub datasource: Pool,
    pub session_store: SessionStore,
    pub key: Key,
    pub oidc_provider: Arc<oidc::Provider>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

impl Server {
    #[must_use]
    pub fn new(
        config: Config,
        datasource: Pool,
        session_store: SessionStore,
        oidc_provider: oidc::Provider,
    ) -> Self {
        Server {
            config,
            datasource,
            session_store,
            oidc_provider: Arc::new(oidc_provider),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        println!("Starting http server on port {:?}", self.config.http_port);

        let state = AppState {
            key: Key::generate(),
            session_store: self.session_store.clone(),
            datasource: self.datasource.clone(),
            oidc_provider: self.oidc_provider.clone(),
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
                "/api/v1",
                Router::new()
                    .route("/auth/me", axum::routing::get(get_me))
                    .route("/recipes", axum::routing::get(http::recipe::list))
                    .route("/recipes", axum::routing::post(http::recipe::create))
                    .route("/recipes/:id", axum::routing::get(http::recipe::get))
                    .route("/recipes/:id", axum::routing::put(http::recipe::update))
                    .route("/tags", axum::routing::post(http::tag::create))
                    .route("/tags", axum::routing::get(http::tag::get_all))
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

impl From<AuthenticatedUser> for domain::user::Authenticated {
    fn from(val: AuthenticatedUser) -> Self {
        Self { id: val.id }
    }
}

async fn auth(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<(CookieJar, Response), Error> {
    let session_key = jar
        .get("id")
        .ok_or(Error::Unauthenticated(anyhow!("missing session cookie")))?
        .value();

    let session = match core::session::get(
        &state.session_store,
        &state.oidc_provider,
        SessionKey(session_key.to_string()),
    )
    .await
    {
        Ok(session) => session,
        Err(err) => {
            let jar = jar.remove(Cookie::from("id"));
            return Ok((jar, err.into_response()));
        }
    };

    // Update the cookie if the session key changed.
    let jar = if session_key == session.key.to_string() {
        jar
    } else {
        jar.add(
            Cookie::build(("id", session.key.to_string()))
                .http_only(true)
                .secure(true)
                .path("/")
                .same_site(cookie::SameSite::Strict)
                .max_age(cookie::time::Duration::seconds(
                    core::session::SESSION_EXPIRES_IN,
                )),
        )
    };

    let user = AuthenticatedUser {
        id: session.user_id,
    };
    req.extensions_mut().insert(user);

    Ok((jar, next.run(req).await))
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            // TODO - log all errors, not just unknown, if debug logging is on
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, msg).into_response(),
            Error::Unauthenticated(err) => {
                println!("error: {err:?}");
                (StatusCode::UNAUTHORIZED, "Unauthenticated.").into_response()
            }
            Error::DomainValidation(err) => {
                println!("error: {err:?}");
                (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()).into_response()
            }
            Error::Other(err) => {
                println!("error: {err:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error").into_response()
            }
        }
    }
}
