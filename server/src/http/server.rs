use std::net::SocketAddr;

use axum::Router;

use crate::config::Config;

pub struct Server {
    config: Config,
}

impl Server {
    pub fn new(config: Config) -> Self {
        return Server { config };
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        println!("Starting http server on port {:?}", self.config.port);

        let router: Router = Router::new().route("/health-check", axum::routing::get(health));

        let addr = SocketAddr::from(([127, 0, 0, 1], self.config.port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, router).await?;

        Ok(())
    }
}

async fn health() -> &'static str {
    "ok"
}
