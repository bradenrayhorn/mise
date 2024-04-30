use mise::{config::Config, http::Server};

#[tokio::main]
async fn main() {
    let config = Config { port: 3000 };
    let s = Server::new(config);

    if let Err(err) = s.start().await {
        println!("Failed to start http server: {:?}", err)
    }
}
