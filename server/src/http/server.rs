use std::net::SocketAddr;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Router,
};

use crate::{
    config::Config,
    datastore::{self, Pool},
};

pub struct Server {
    config: Config,
    datasource: Pool,
}

#[derive(Clone)]
struct AppState {
    datasource: Pool,
}

impl Server {
    pub fn new(config: Config, datasource: Pool) -> Self {
        Server { config, datasource }
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        println!("Starting http server on port {:?}", self.config.port);

        let state = AppState {
            datasource: self.datasource.clone(),
        };

        let router: Router = Router::new()
            .route("/health-check", axum::routing::get(health))
            .route("/get-a-user", axum::routing::get(get_user))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn health() -> &'static str {
    "ok"
}

async fn get_user(State(state): State<AppState>) -> Result<String, datastore::Error> {
    get_user_logic(state.datasource, "123").await
}

async fn get_user_logic(datasource: Pool, id: &str) -> Result<String, datastore::Error> {
    let user = datasource.get_user(id.to_owned()).await?;
    Ok(user.name)
}

impl IntoResponse for datastore::Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
