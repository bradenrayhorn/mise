use std::{env, fs};

mod internal {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Config {
        pub http_port: Option<u16>,

        pub oidc: Oidc,
        pub sqlite: Sqlite,
    }

    #[derive(Deserialize)]
    pub struct Oidc {
        pub issuer_url: String,
        pub client_id: String,
        pub client_secret: String,
        pub origin: String,
    }

    #[derive(Deserialize)]
    pub struct Sqlite {
        pub db_path: String,
    }
}

pub struct Config {
    pub http_port: u16,

    pub oidc: Oidc,
    pub sqlite: Sqlite,
}

pub struct Oidc {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub origin: String,
}

pub struct Sqlite {
    pub db_path: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Parse(#[from] toml::de::Error),
}

pub fn from_filesystem() -> Result<Config, Error> {
    let config_path = env::var("MISE_CONFIG")
        .ok()
        .unwrap_or("mise.toml".to_owned());

    let raw_config = fs::read_to_string(config_path)?;

    let parsed: internal::Config = toml::from_str(&raw_config)?;

    Ok(Config {
        http_port: parsed.http_port.unwrap_or(3000),
        oidc: Oidc {
            issuer_url: parsed.oidc.issuer_url,
            client_id: parsed.oidc.client_id,
            client_secret: parsed.oidc.client_secret,
            origin: parsed.oidc.origin,
        },
        sqlite: Sqlite {
            db_path: parsed.sqlite.db_path,
        },
    })
}
