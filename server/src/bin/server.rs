use mise::{config, datastore, http::Server, session_store::SessionStore, sqlite};

#[tokio::main]
async fn main() {
    let config = match config::from_filesystem() {
        Ok(config) => config,
        Err(err) => {
            println!("error with config: {:?}", err);
            return;
        }
    };

    let (_worker_pool, senders) = match sqlite::datastore_handler("mise.db") {
        Ok(pool) => pool,
        Err(err) => {
            println!("error with pool: {:?}", err);
            return;
        }
    };

    let session_store_sender = match sqlite::session_store("mise_sessions.db") {
        Ok(sender) => sender,
        Err(err) => {
            println!("error with sqlite session store: {:?}", err);
            return;
        }
    };

    let pool = datastore::Pool::new(senders);
    let cache = SessionStore::new(session_store_sender);

    let s = Server::new(config, pool, cache);

    if let Err(err) = s.start().await {
        println!("Failed to start http server: {:?}", err)
    }

    // TODO - graceful shutdown of pool and http server.
    // can sort of test the sqlite graceful shutdown by seeing if the wal and shm files are deleted
}
