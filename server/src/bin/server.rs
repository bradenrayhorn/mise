use mise::{config::Config, datastore, http::Server, sqlite};

#[tokio::main]
async fn main() {
    let config = Config { port: 3000 };
    let (_worker_pool, senders) = match sqlite::worker_pool() {
        Ok(pool) => pool,
        Err(err) => {
            println!("error with pool: {:?}", err);
            return;
        }
    };
    let pool = datastore::Pool::new(senders);

    let s = Server::new(config, pool);

    if let Err(err) = s.start().await {
        println!("Failed to start http server: {:?}", err)
    }

    // TODO - graceful shutdown of pool and http server.
    // can sort of test the sqlite graceful shutdown by seeing if the wal and shm files are deleted
}
