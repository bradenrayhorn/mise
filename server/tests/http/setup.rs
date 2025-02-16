use base64::Engine;
use mise::{file, image_processing::ImageProcessor, imagestore::ImageStore, oidc, search::Backend};
use rand::Rng;
use std::{net::TcpListener, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use reqwest::StatusCode;

use crate::http::{requests, responses};

const JPEG: &str = "/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";

pub struct OidcServer {
    process: std::process::Child,
    pub port: u16,
}

impl Drop for OidcServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

impl OidcServer {
    async fn new() -> Result<Self> {
        // find open port
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        std::mem::drop(listener);

        // start process
        let mut process = std::process::Command::new("fake-oidc")
            .env("FAKE_OIDC_HTTP_PORT", format!("{port}"))
            .stdout(std::process::Stdio::null())
            .spawn()?;

        // wait for it to be ready
        if let Err(err) = wait_for(format!("http://localhost:{port}/ready")).await {
            let _ = process.kill();
            return Err(err);
        };

        Ok(Self { process, port })
    }
}

async fn wait_for(url: String) -> Result<()> {
    let increment = 2;
    let mut total = 0;

    let client = reqwest::ClientBuilder::new().build()?;

    loop {
        if total > 1000 {
            return Err(anyhow!("timed out waiting for {url}"));
        }

        let result = client.get(&url).send().await;
        if let Ok(r) = &result {
            if r.status() == StatusCode::OK {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(increment)).await;
        total += increment;
    }
}

pub struct Harness {
    _oidc_server: OidcServer,
    http_port: u16,
    db_path: String,
    session_db_path: String,
    images_path: String,
    index_path: String,
    client: reqwest::Client,
    base_url: String,
    session_id: Option<String>,
}

impl Drop for Harness {
    fn drop(&mut self) {
        // TODO - shutdown db pools AND server
        let _ = std::fs::remove_file(&self.db_path);
        let _ = std::fs::remove_file(&self.session_db_path);
        let _ = std::fs::remove_dir_all(&self.images_path);
        let _ = std::fs::remove_dir_all(&self.index_path);
    }
}

impl Harness {
    async fn new() -> Result<Self> {
        let oidc_server = OidcServer::new().await?;

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let http_port = listener.local_addr()?.port();
        std::mem::drop(listener);

        let random_prefix: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let images_path = format!("/tmp/{}-mise-images", random_prefix);
        let index_path = format!("/tmp/{}-mise-indexes", random_prefix);
        let db_path = format!("/tmp/{}-mise.db", random_prefix);
        let session_db_path = format!("/tmp/{}-mise-sessions.db", random_prefix);

        std::fs::create_dir(&images_path)?;

        let config = mise::config::Config {
            http_port,
            origin: format!("http://localhost:{http_port}"),
            insecure_cookies: false,
            static_build_path: "../ui/build".to_string(),
            search_index_directory: index_path.clone(),
            oidc: mise::config::Oidc {
                issuer_url: format!("http://[::]:{}", oidc_server.port),
                client_id: "dev-client".to_string(),
                client_secret: "secure-secret".to_string(),
            },
            sqlite: mise::config::Sqlite {
                db_path: db_path.clone(),
                session_db_path: session_db_path.clone(),
            },
            image_backend: mise::config::ImageBackend::File(mise::config::ImageBackendFile {
                directory: images_path.clone(),
            }),
        };

        let oidc = oidc::Provider::new((&config).try_into().unwrap())
            .await
            .unwrap();

        let sv_db_path = db_path.clone();
        let sv_cache_path = session_db_path.clone();
        let sv_images_path = images_path.clone();
        let sv_index_path = index_path.clone();
        tokio::task::spawn(async move {
            let (_, connections) = mise::sqlite::datastore_handler(
                &sv_db_path,
                &mise::sqlite::DatastoreConfig {
                    recipe_page_size: 2,
                    recipe_dump_page_size: 10,
                },
            )
            .expect("could not make datastore");
            let session_store =
                mise::sqlite::session_store(&sv_cache_path).expect("could not make session store");

            let (background_result_sender, mut receiver) = tokio::sync::mpsc::channel(8);
            tokio::task::spawn(async move {
                let _ = receiver.recv().await;
            });

            let datastore = mise::datastore::Pool::new(connections);
            let sb = Backend::new(&sv_index_path, datastore.clone()).unwrap();

            let server = mise::http::Server::new(
                config,
                datastore,
                mise::session_store::SessionStore::new(session_store, background_result_sender),
                oidc,
                ImageStore::new(Box::from(
                    file::ImageBackend::new(&sv_images_path)
                        .await
                        .expect("could not make image backend"),
                )),
                ImageProcessor::new()
                    .await
                    .expect("could not init image processor"),
                sb,
            );
            if let Err(err) = server.start().await {
                println!("Failed to start http server: {:?}", err);
            }
        });

        wait_for(format!("http://localhost:{http_port}/health-check")).await?;

        let client = reqwest::ClientBuilder::new().cookie_store(false).build()?;

        Ok(Harness {
            _oidc_server: oidc_server,
            http_port,
            db_path,
            session_db_path,
            images_path,
            index_path,
            client,
            base_url: format!("http://localhost:{http_port}"),
            session_id: None,
        })
    }

    pub async fn authenticate(&mut self, username: &str) -> Result<()> {
        let jar = reqwest_cookie_store::CookieStoreMutex::new(
            reqwest_cookie_store::CookieStore::new(None),
        );
        let jar = Arc::new(jar);
        let auth_client = reqwest::ClientBuilder::new()
            .cookie_provider(jar.clone())
            .build()?;

        let r = auth_client
            .get(format!("http://localhost:{}/auth/init", self.http_port))
            .send()
            .await?;

        let login_url = format!("{}&username={username}", r.url().as_str());

        auth_client.get(&login_url).send().await?;

        let session_id = {
            let store = jar.lock().unwrap();
            store
                .get("localhost", "/", "id")
                .unwrap()
                .value()
                .to_string()
        };

        self.session_id = Some(session_id);

        Ok(())
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let builder = self.client.get(format!("{}{path}", self.base_url));

        self.add_auth(builder)
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        let builder = self.client.post(format!("{}{path}", self.base_url));

        self.add_auth(builder)
    }

    pub fn put(&self, path: &str) -> reqwest::RequestBuilder {
        let builder = self.client.put(format!("{}{path}", self.base_url));

        self.add_auth(builder)
    }

    fn add_auth(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(session_id) = &self.session_id {
            builder.header("cookie", format!("id={session_id}"))
        } else {
            builder
        }
    }
}

impl Harness {
    pub async fn create_tag(&self, name: &str) -> Result<String> {
        let response = self
            .post(&format!("/api/v1/tags"))
            .json(&requests::CreateTag { name: name.into() })
            .send()
            .await?;

        assert_eq!(StatusCode::OK, response.status());
        Ok(response.json::<responses::CreateTag>().await?.data)
    }

    pub async fn create_image(&self) -> Result<String> {
        let base64_engine = base64::engine::general_purpose::STANDARD;

        let body = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(base64_engine.decode(JPEG.as_bytes())?),
        );

        let response = self.post("/api/v1/images").multipart(body).send().await?;
        assert_eq!(StatusCode::OK, response.status());

        let id = response.json::<responses::CreateImage>().await?.data;
        Ok(id)
    }
}

pub async fn harness() -> Result<Harness> {
    Harness::new().await
}

pub async fn with_auth() -> Result<Harness> {
    let mut harness = Harness::new().await?;
    harness.authenticate("user").await?;
    Ok(harness)
}
