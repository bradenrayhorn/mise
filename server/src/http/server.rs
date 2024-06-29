use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{DefaultBodyLimit, FromRef, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    Extension, Router,
};
use axum_extra::extract::{cookie::Key, CookieJar};
use cookie::Cookie;

use crate::{
    config::Config,
    core::{self, Error},
    datastore::Pool,
    domain::{self, SessionKey},
    http,
    imagestore::ImageStore,
    oidc,
    session_store::SessionStore,
};

const MAX_IMAGE_BODY_SIZE: usize = 1024 * 1024 * 10;

pub struct Server {
    config: Config,
    datasource: Pool,
    session_store: SessionStore,
    oidc_provider: Arc<oidc::Provider>,
    image_store: Arc<ImageStore>,
}

#[derive(Clone)]
pub struct AppState {
    pub datasource: Pool,
    pub session_store: SessionStore,
    pub key: Key,
    pub oidc_provider: Arc<oidc::Provider>,
    pub image_store: Arc<ImageStore>,
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
        image_store: ImageStore,
    ) -> Self {
        Server {
            config,
            datasource,
            session_store,
            oidc_provider: Arc::new(oidc_provider),
            image_store: Arc::new(image_store),
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        println!("Starting http server on port {:?}", self.config.http_port);

        let state = AppState {
            key: Key::generate(),
            session_store: self.session_store.clone(),
            datasource: self.datasource.clone(),
            oidc_provider: self.oidc_provider.clone(),
            image_store: self.image_store.clone(),
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
                    //
                    // Nested /images router with large max body size
                    .nest(
                        "/images",
                        Router::new()
                            .route("/", axum::routing::post(http::image::upload))
                            .route("/:id", axum::routing::get(http::image::get))
                            .layer(DefaultBodyLimit::max(MAX_IMAGE_BODY_SIZE)),
                    )
                    .layer(middleware::from_fn_with_state(
                        state.clone(),
                        auth_middleware,
                    )),
            )
            .route("/", axum::routing::get(handle_base_redirect))
            .fallback_service(
                tower_http::services::ServeDir::new("../ui/build").not_found_service(
                    tower_http::services::ServeFile::new("../ui/build/index.html"),
                ),
            )
            //
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.http_port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn handle_base_redirect(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Redirect) {
    let previous_jar = jar.clone();

    match check_if_authenticated(&state.session_store, &state.oidc_provider, jar).await {
        Ok((jar, _)) => (jar, Redirect::temporary("/recipes")),
        Err(_) => (previous_jar, Redirect::temporary("/login")),
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

async fn auth_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<(CookieJar, Response), Error> {
    let previous_jar = jar.clone();

    let (jar, user) =
        match check_if_authenticated(&state.session_store, &state.oidc_provider, jar).await {
            Ok(r) => r,
            Err(err) => {
                let jar = previous_jar.remove(Cookie::from("id"));
                return Ok((jar, err.into_response()));
            }
        };

    req.extensions_mut().insert(user);

    Ok((jar, next.run(req).await))
}

async fn check_if_authenticated(
    session_store: &SessionStore,
    oidc_provider: &oidc::Provider,
    jar: CookieJar,
) -> Result<(CookieJar, AuthenticatedUser), Error> {
    let session_key = jar
        .get("id")
        .ok_or(Error::Unauthenticated(anyhow!("missing session cookie")))?
        .value();

    // try to fetch the session
    let session = core::session::get(
        session_store,
        oidc_provider,
        SessionKey(session_key.to_string()),
    )
    .await?;

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

    Ok((jar, user))
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
            Error::Invalid(err) => {
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
