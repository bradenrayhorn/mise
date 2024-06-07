use mise::oidc;
use rand::{distributions::Alphanumeric, Rng};
use std::{net::TcpListener, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use reqwest::StatusCode;

use crate::http::{requests, responses};

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
    cache_path: String,
    client: reqwest::Client,
    base_url: String,
    session_id: Option<String>,
}

impl Drop for Harness {
    fn drop(&mut self) {
        // TODO - shutdown db pools AND server
        let _ = std::fs::remove_file(&self.db_path);
        let _ = std::fs::remove_file(&self.cache_path);
    }
}

impl Harness {
    async fn new() -> Result<Self> {
        let oidc_server = OidcServer::new().await?;

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let http_port = listener.local_addr()?.port();
        std::mem::drop(listener);

        let random_prefix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let db_path = format!("/tmp/{}-mise.db", random_prefix);
        let cache_path = format!("/tmp/{}-mise-cache.db", random_prefix);

        let config = mise::config::Config {
            http_port,
            oidc: mise::config::Oidc {
                issuer_url: format!("http://localhost:{}", oidc_server.port),
                client_id: "dev-client".to_string(),
                client_secret: "secure-secret".to_string(),
                origin: format!("http://localhost:{http_port}"),
            },
            sqlite: mise::config::Sqlite {
                db_path: db_path.clone(),
            },
        };

        let oidc = oidc::Provider::new((&config).try_into().unwrap())
            .await
            .unwrap();

        let sv_db_path = db_path.clone();
        let sv_cache_path = cache_path.clone();
        tokio::task::spawn(async move {
            let (_, connections) = mise::sqlite::datastore_handler(&sv_db_path).unwrap();
            let session_store = mise::sqlite::session_store(&sv_cache_path).unwrap();

            let server = mise::http::Server::new(
                config,
                mise::datastore::Pool::new(connections),
                mise::session_store::SessionStore::new(session_store),
                oidc,
            );
            if let Err(err) = server.start().await {
                println!("Failed to start http server: {:?}", err);
            }
        });

        let client = reqwest::ClientBuilder::new().cookie_store(false).build()?;

        Ok(Harness {
            _oidc_server: oidc_server,
            http_port,
            db_path,
            cache_path,
            client,
            base_url: format!("http://localhost:{http_port}"),
            session_id: None,
        })
    }

    pub async fn authenticate(&mut self, username: &str) -> Result<()> {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let auth_client = reqwest::ClientBuilder::new().cookie_provider(jar).build()?;

        let r = auth_client
            .get(format!("http://localhost:{}/auth/init", self.http_port))
            .send()
            .await?;

        let login_url = format!("{}&username={username}", r.url().as_str());

        let r2 = auth_client.get(&login_url).send().await?;
        let id_cookie = cookie::Cookie::parse(
            r2.headers()
                .get_all("set-cookie")
                .into_iter()
                .find(|header| header.to_str().unwrap().contains("id="))
                .map(|v| v.to_str().unwrap())
                .unwrap_or_else(|| ""),
        )?;

        self.session_id = Some(id_cookie.value().to_string());

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
    pub async fn create_tag(&self, name: &str) -> Result<i64> {
        let response = self
            .post(&format!("/api/v1/tags"))
            .json(&requests::CreateTag { name: name.into() })
            .send()
            .await?;

        assert_eq!(StatusCode::OK, response.status());
        Ok(response.json::<responses::CreateTag>().await?.data)
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
