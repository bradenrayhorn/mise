use std::time::Duration;

use anyhow::Context;
use chrono::Utc;
use openidconnect::{
    core::{
        CoreAuthenticationFlow, CoreClient, CoreGenderClaim, CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm, CoreProviderMetadata,
    },
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, EmptyAdditionalClaims,
    EmptyExtraTokenFields, IdTokenFields, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, StandardTokenResponse, TokenResponse,
    TokenType,
};
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("CSRF state mismatch.")]
    CsrfMismatch,

    #[error("Missing ID Token.")]
    MissingIdToken,

    #[error("Missing Refresh Token.")]
    MissingRefreshToken,

    #[error("Missing Expires In.")]
    MissingExpiresIn,

    #[error("Could not compute expires at from: {0:?}.")]
    InvalidExpiresIn(Duration),

    #[error("Invalid OIDC configuration.")]
    InvalidConfiguration(#[from] openidconnect::ConfigurationError),

    #[error("Claims verification failed.")]
    InvalidClaims(#[from] openidconnect::ClaimsVerificationError),

    #[error("Origin must not end with slash.")]
    OriginMustNotEndWithSlash,

    #[error("{msg}")]
    InvalidUrl {
        msg: String,
        #[source]
        source: openidconnect::url::ParseError,
    },

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct Config {
    issuer_url: String,
    client_id: String,
    client_secret: String,
    origin: String,
}

impl TryFrom<&config::Config> for Config {
    type Error = Error;

    fn try_from(value: &config::Config) -> Result<Self, Self::Error> {
        if value.origin.ends_with('/') {
            return Err(Error::OriginMustNotEndWithSlash);
        }

        Ok(Config {
            issuer_url: value.oidc.issuer_url.clone(),
            client_id: value.oidc.client_id.clone(),
            client_secret: value.oidc.client_secret.clone(),
            origin: value.origin.clone(),
        })
    }
}

#[derive(Clone)]
pub struct Provider {
    http_client: reqwest::Client,
    openid_client: CoreClient<
        openidconnect::EndpointSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointNotSet,
        openidconnect::EndpointMaybeSet,
        openidconnect::EndpointMaybeSet,
    >,
}

impl Provider {
    pub async fn new(config: Config) -> Result<Self, Error> {
        let client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .context("Could not build HTTP client.")?;

        let issuer = IssuerUrl::new(config.issuer_url).map_err(|err| Error::InvalidUrl {
            msg: "Invalid Issuer URL".to_string(),
            source: err,
        })?;

        let provider_metadata = CoreProviderMetadata::discover_async(issuer, &client)
            .await
            .context("Could not find OIDC metadata.")?;

        let openid_client = CoreClient::from_provider_metadata(
            provider_metadata,
            ClientId::new(config.client_id),
            Some(ClientSecret::new(config.client_secret)),
        )
        .set_redirect_uri(
            RedirectUrl::new(format!("{}/auth/complete", config.origin)).map_err(|err| {
                Error::InvalidUrl {
                    msg: "Invalid Redirect URL".to_string(),
                    source: err,
                }
            })?,
        );

        Ok(Provider {
            http_client: client,
            openid_client,
        })
    }
}

// authentication flow

#[derive(Serialize, Deserialize)]
pub struct AuthState {
    csrf_token: CsrfToken,
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
}

pub struct CallbackParams<'a> {
    pub state: &'a str,
    pub code: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct Authenticated {
    pub subject: String,
    pub name: String,
    pub refresh_token: String,
    pub expires_at: chrono::DateTime<Utc>,
}

pub fn begin_auth(provider: &Provider) -> (openidconnect::url::Url, AuthState) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token, nonce) = provider
        .openid_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(pkce_challenge)
        .add_scope(Scope::new("access".into()))
        .add_scope(Scope::new("offline_access".into()))
        .add_scope(Scope::new("profile".into()))
        .url();

    (
        auth_url,
        AuthState {
            csrf_token,
            nonce,
            pkce_verifier,
        },
    )
}

pub async fn complete_auth<'a>(
    provider: &Provider,
    state: AuthState,
    params: CallbackParams<'a>,
) -> Result<Authenticated, Error> {
    if sha256::digest(state.csrf_token.secret()) != sha256::digest(params.state) {
        return Err(Error::CsrfMismatch);
    }

    let token_response = provider
        .openid_client
        .exchange_code(AuthorizationCode::new(params.code.to_string()))?
        .set_pkce_verifier(state.pkce_verifier)
        .request_async(&provider.http_client)
        .await
        .context("Code exchange failure.")?;

    to_authenticated(provider, Some(&state.nonce), &token_response)
}

pub async fn refresh_auth(
    provider: &Provider,
    refresh_token: String,
) -> Result<Authenticated, Error> {
    let token_response = provider
        .openid_client
        .exchange_refresh_token(&RefreshToken::new(refresh_token))?
        .request_async(&provider.http_client)
        .await
        .context("Refresh token exchange failure.")?;

    to_authenticated(provider, None, &token_response)
}

fn to_authenticated<TT: TokenType>(
    provider: &Provider,
    nonce: Option<&Nonce>,
    token_response: &StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        TT,
    >,
) -> Result<Authenticated, Error> {
    let id_token = token_response.id_token().ok_or(Error::MissingIdToken)?;
    let id_token_verifier = provider.openid_client.id_token_verifier();

    let claims = match nonce {
        Some(nonce) => id_token.claims(&id_token_verifier, nonce)?,
        None => id_token.claims(&id_token_verifier, ignore_nonce_verification)?,
    };

    let refresh_token = token_response
        .refresh_token()
        .ok_or(Error::MissingRefreshToken)?;

    let expires_in = token_response.expires_in().ok_or(Error::MissingExpiresIn)?;

    let expires_at = chrono::Utc::now()
        .checked_add_signed(chrono::TimeDelta::from_std(expires_in).context("Invalid duration.")?)
        .ok_or(Error::InvalidExpiresIn(expires_in))?;

    let subject = claims.subject();

    let default_name = subject.to_string();
    let name = match claims.name() {
        Some(localized_name) => match localized_name.iter().next() {
            Some((_, name)) => name.as_str().to_string(),
            None => default_name,
        },
        None => default_name,
    };

    Ok(Authenticated {
        subject: subject.to_string(),
        name,
        refresh_token: refresh_token.secret().to_owned(),
        expires_at,
    })
}

#[allow(clippy::unnecessary_wraps)]
fn ignore_nonce_verification(_nonce: Option<&Nonce>) -> Result<(), String> {
    Ok(())
}
