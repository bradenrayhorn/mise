use anyhow::Context;
use mise::{
    config, datastore, file,
    http::Server,
    image_processing::ImageProcessor,
    imagestore::{self, ImageBackend},
    oidc, s3,
    search::Backend,
    session_store::{self, SessionStore},
    sqlite,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let config = match config::from_filesystem() {
        Ok(config) => config,
        Err(err) => {
            println!("error with config: {:?}", err);
            return;
        }
    };

    let (_worker_pool, senders) = match sqlite::datastore_handler(
        &config.sqlite.db_path,
        &sqlite::DatastoreConfig {
            recipe_page_size: 20,
            recipe_dump_page_size: 250,
        },
    ) {
        Ok(pool) => pool,
        Err(err) => {
            println!("error with pool: {:?}", err);
            return;
        }
    };

    let session_store_sender = match sqlite::session_store(&config.sqlite.session_db_path) {
        Ok(sender) => sender,
        Err(err) => {
            println!("error with sqlite session store: {:?}", err);
            return;
        }
    };

    let (background_result_sender, mut receiver) = mpsc::channel(8);
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            match msg {
                session_store::BackgroundResultMessage::PruneExpiredSessions { result } => {
                    if let Err(err) = result {
                        println!("Failed to prune sessions: {}.", err);
                    }
                }
                session_store::BackgroundResultMessage::PruneExpiredRefreshLocks { result } => {
                    if let Err(err) = result {
                        println!("Failed to prune refresh locks: {}.", err);
                    }
                }
            }
        }
    });

    let pool = datastore::Pool::new(senders);
    let cache = SessionStore::new(session_store_sender, background_result_sender);

    let oidc_provider = oidc::Provider::new((&config).try_into().unwrap())
        .await
        .unwrap();

    let image_backend: Box<dyn ImageBackend + Send + Sync> = match &config.image_backend {
        config::ImageBackend::S3(config) => {
            let backend = s3::imagebackend::ImageBackend::new(config.try_into().unwrap()).unwrap();

            Box::from(backend)
        }
        config::ImageBackend::File(config) => {
            let backend = file::ImageBackend::new(&config.directory).await.unwrap();
            Box::from(backend)
        }
    };

    let image_processor = ImageProcessor::new()
        .await
        .context("Initialize image processor.")
        .unwrap();

    println!("indexing recipes...");
    let sb = Backend::new(&config.search_index_directory, pool.clone()).unwrap();
    sb.index_recipes().await.unwrap();
    println!("recipe index complete.");

    let s = Server::new(
        config,
        pool,
        cache,
        oidc_provider,
        imagestore::ImageStore::new(image_backend),
        image_processor,
        sb,
    );

    if let Err(err) = s.start().await {
        println!("Failed to start http server: {:?}", err)
    }

    // TODO - graceful shutdown of pool and http server.
    // can sort of test the sqlite graceful shutdown by seeing if the wal and shm files are deleted
}
