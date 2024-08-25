use std::{env, fs};

use anyhow::anyhow;

#[derive(Clone)]
pub enum ImageBackend {
    S3(ImageBackendS3),
    File(ImageBackendFile),
}

mod internal {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Config {
        pub http_port: Option<u16>,
        pub origin: String,
        pub insecure_cookies: Option<bool>,

        pub oidc: Oidc,
        pub sqlite: Sqlite,
        pub images: Images,
    }

    #[derive(Deserialize)]
    pub struct Images {
        pub s3: Option<S3>,
        pub file: Option<File>,

        pub backend: ImageBackend,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ImageBackend {
        S3,
        File,
    }

    #[derive(Deserialize)]
    pub struct Oidc {
        pub issuer_url: String,
        pub client_id: String,
        pub client_secret: String,
    }

    #[derive(Deserialize)]
    pub struct Sqlite {
        pub db_path: String,
        pub session_db_path: String,
    }

    #[derive(Deserialize)]
    pub struct S3 {
        pub host: String,
        pub bucket: String,
        pub region: String,
        pub secret_access_key: String,
        pub secret_access_key_id: String,
    }

    #[derive(Deserialize)]
    pub struct File {
        pub directory: String,
    }
}

#[derive(Clone)]
pub struct Config {
    pub http_port: u16,
    pub origin: String,
    pub insecure_cookies: bool,
    pub static_build_path: String,

    pub oidc: Oidc,
    pub sqlite: Sqlite,
    pub image_backend: ImageBackend,
}

#[derive(Clone)]
pub struct Oidc {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct ImageBackendS3 {
    pub host: String,
    pub bucket: String,
    pub region: String,
    pub secret_access_key: String,
    pub secret_access_key_id: String,
}

#[derive(Clone)]
pub struct ImageBackendFile {
    pub directory: String,
}

#[derive(Clone)]
pub struct Sqlite {
    pub db_path: String,
    pub session_db_path: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Parse(#[from] toml::de::Error),

    #[error("malformed config")]
    Malformed(#[from] anyhow::Error),
}

pub fn from_filesystem() -> Result<Config, Error> {
    let config_path = env::var("MISE_CONFIG")
        .ok()
        .unwrap_or("mise.toml".to_owned());

    let raw_config = fs::read_to_string(config_path)?;

    let parsed: internal::Config = toml::from_str(&raw_config)?;

    Ok(Config {
        http_port: parsed.http_port.unwrap_or(3000),
        origin: parsed.origin,
        insecure_cookies: parsed.insecure_cookies.unwrap_or(false),
        static_build_path: env::var("MISE_STATIC_BUILD")
            .ok()
            .unwrap_or("../ui/build".to_owned()),
        oidc: Oidc {
            issuer_url: parsed.oidc.issuer_url,
            client_id: parsed.oidc.client_id,
            client_secret: parsed.oidc.client_secret,
        },
        sqlite: Sqlite {
            db_path: parsed.sqlite.db_path,
            session_db_path: parsed.sqlite.session_db_path,
        },
        image_backend: match parsed.images.backend {
            internal::ImageBackend::S3 => {
                let config = parsed.images.s3.ok_or(Error::Malformed(anyhow!(
                    "Image backend is s3, but missing s3 config."
                )))?;

                ImageBackend::S3(ImageBackendS3 {
                    host: config.host,
                    bucket: config.bucket,
                    region: config.region,
                    secret_access_key: config.secret_access_key,
                    secret_access_key_id: config.secret_access_key_id,
                })
            }
            internal::ImageBackend::File => {
                let config = parsed.images.file.ok_or(Error::Malformed(anyhow!(
                    "Image backend is file, but missing file config."
                )))?;

                ImageBackend::File(ImageBackendFile {
                    directory: config.directory,
                })
            }
        },
    })
}
