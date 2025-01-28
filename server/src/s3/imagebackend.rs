use std::time::Duration;

use anyhow::{anyhow, Context};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::StatusCode;

use crate::{config, imagestore::Error};

#[derive(Clone)]
pub struct ImageBackend {
    config: Config,
    client: reqwest::Client,
}

impl ImageBackend {
    pub fn new(config: Config) -> Result<Self, Error> {
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .context("Could not build HTTP client.")?;

        Ok(ImageBackend { config, client })
    }
}

#[derive(Clone)]
pub struct Config {
    host: String,
    bucket: String,
    region: String,
    secret_access_key: String,
    secret_access_key_id: String,
}

impl TryFrom<&config::ImageBackendS3> for Config {
    type Error = Error;

    fn try_from(value: &config::ImageBackendS3) -> Result<Self, Self::Error> {
        if value.host.ends_with('/') {
            return Err(Error::Config(anyhow!("Host cannot end with slash.")));
        }

        if value.host.contains("http://") || value.host.contains("https://") {
            return Err(Error::Config(anyhow!("Host should not contain protocol.")));
        }

        Ok(Config {
            host: value.host.clone(),
            bucket: value.bucket.clone(),
            region: value.region.clone(),
            secret_access_key: value.secret_access_key.clone(),
            secret_access_key_id: value.secret_access_key_id.clone(),
        })
    }
}

#[async_trait]
impl crate::imagestore::ImageBackend for ImageBackend {
    async fn get(&self, path: &str) -> Result<Vec<u8>, Error> {
        let date = chrono::Utc::now();

        let uri = format!("/{}/{path}", self.config.bucket);
        let content_hash = sha256::digest("");

        let canonical_request =
            self.create_canonical_request("GET", &uri, "", &content_hash, &date);
        let canonical_hash = sha256::digest(&canonical_request);

        let string_to_sign = self.create_string_to_sign(&date, &canonical_hash);
        let signature = self.create_signature(&date, &string_to_sign);
        let authorization = self.create_authorization_header(&date, &signature);

        let result = self
            .client
            .get(format!("https://{host}{uri}", host = &self.config.host))
            .header("Authorization", authorization)
            .header("x-amz-date", date.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-content-sha256", content_hash)
            .send()
            .await
            .context("Could not fetch from s3.")?;

        let result = result
            .error_for_status()
            .map_err(|err| match err.status() {
                Some(StatusCode::NOT_FOUND) => Error::NotFound(err.into()),
                _ => Error::Other(err.into()),
            })?;

        let bytes = result
            .bytes()
            .await
            .context("Reading bytes from s3 response.")?;
        Ok(bytes.to_vec())
    }

    async fn upload(&self, path: &str, file: Vec<u8>) -> Result<(), Error> {
        let date = chrono::Utc::now();

        let uri = format!("/{}/{path}", self.config.bucket);
        let content_hash = sha256::digest(&file);

        let canonical_request =
            self.create_canonical_request("PUT", &uri, "", &content_hash, &date);
        let canonical_hash = sha256::digest(&canonical_request);

        let string_to_sign = self.create_string_to_sign(&date, &canonical_hash);
        let signature = self.create_signature(&date, &string_to_sign);
        let authorization = self.create_authorization_header(&date, &signature);

        let result = self
            .client
            .put(format!("https://{host}{uri}", host = &self.config.host))
            .header("Authorization", authorization)
            .header("x-amz-date", date.format("%Y%m%dT%H%M%SZ").to_string())
            .header("x-amz-content-sha256", content_hash)
            .body(file)
            .send()
            .await
            .context("Could not fetch from s3.")?;

        result
            .error_for_status()
            .context("Could not upload to s3.")
            .map_err(Error::Other)?;

        Ok(())
    }
}

impl ImageBackend {
    fn create_canonical_request(
        &self,
        method: &str,
        uri: &str,
        query_string: &str,
        content_sha_256: &str,
        date: &chrono::DateTime<Utc>,
    ) -> String {
        format!(
            "\
                {method}\n\
                {encoded_uri}\n\
                {query_string}\n\
                host:{host}\n\
                x-amz-content-sha256:{content_sha_256}\n\
                x-amz-date:{iso8601}\n\
                \n\
                host;x-amz-content-sha256;x-amz-date\n\
                {content_sha_256}",
            encoded_uri = encode_uri(uri.as_bytes()),
            host = &self.config.host,
            iso8601 = date.format("%Y%m%dT%H%M%SZ")
        )
    }

    fn create_string_to_sign(&self, date: &chrono::DateTime<Utc>, request_hash: &str) -> String {
        format!(
            "\
                AWS4-HMAC-SHA256\n\
                {iso8601}\n\
                {iso8601_date}/{region}/s3/aws4_request\n\
                {request_hash}",
            iso8601 = date.format("%Y%m%dT%H%M%SZ"),
            iso8601_date = date.format("%Y%m%d"),
            region = &self.config.region,
        )
    }

    fn create_signature(&self, date: &chrono::DateTime<Utc>, string_to_sign: &str) -> String {
        let date_key = ring::hmac::sign(
            &ring::hmac::Key::new(
                ring::hmac::HMAC_SHA256,
                format!("AWS4{key}", key = self.config.secret_access_key).as_bytes(),
            ),
            date.format("%Y%m%d").to_string().as_bytes(),
        );

        let date_region_key = ring::hmac::sign(
            &ring::hmac::Key::new(ring::hmac::HMAC_SHA256, date_key.as_ref()),
            self.config.region.as_bytes(),
        );

        let date_region_service_key = ring::hmac::sign(
            &ring::hmac::Key::new(ring::hmac::HMAC_SHA256, date_region_key.as_ref()),
            "s3".as_bytes(),
        );

        let signing_key = ring::hmac::sign(
            &ring::hmac::Key::new(ring::hmac::HMAC_SHA256, date_region_service_key.as_ref()),
            "aws4_request".as_bytes(),
        );

        let signature = ring::hmac::sign(
            &ring::hmac::Key::new(ring::hmac::HMAC_SHA256, signing_key.as_ref()),
            string_to_sign.as_bytes(),
        );

        hex::encode(signature.as_ref())
    }

    fn create_authorization_header(&self, date: &chrono::DateTime<Utc>, signature: &str) -> String {
        format!(
            "AWS4-HMAC-SHA256 \
                Credential={secret_key_id}/{date}/{region}/s3/aws4_request,\
                SignedHeaders=host;x-amz-content-sha256;x-amz-date,\
                Signature={signature}",
            secret_key_id = self.config.secret_access_key_id,
            date = date.format("%Y%m%d"),
            region = self.config.region,
        )
    }
}

fn encode_uri(bytes: &[u8]) -> String {
    let mut encoded_uri = String::new();
    for byte in bytes {
        if matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/')
        {
            encoded_uri.push(*byte as char);
        } else {
            encoded_uri.push_str(&format!("%{}", hex::encode_upper([*byte])));
        }
    }
    encoded_uri
}
