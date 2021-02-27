use std::fmt;

use chrono::offset::Utc;
use chrono::DateTime;
use hyper::client::{Client, HttpConnector};
use hyper_rustls::HttpsConnector;
use json::json;
use serde::{Deserialize, Serialize};

use crate::error::AuthError;

#[allow(unused)]
pub(crate) const TLS_CERTS: &[u8] = include_bytes!("../../roots.pem");

const AUTH_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

/// Represents application credentials for accessing Google Cloud Platform services.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationCredentials {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub project_id: String,
    pub private_key_id: String,
    pub private_key: String,
    pub client_email: String,
    pub client_id: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: String,
    pub client_x509_cert_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenValue {
    Bearer(String),
}

impl fmt::Display for TokenValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenValue::Bearer(token) => write!(f, "Bearer {}", token.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    value: TokenValue,
    expiry: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub(crate) struct TokenManager {
    client: Client<HttpsConnector<HttpConnector>>,
    scopes: String,
    creds: ApplicationCredentials,
    current_token: Option<Token>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AuthResponse {
    access_token: String,
}

impl TokenManager {
    pub(crate) fn new(creds: ApplicationCredentials, scopes: &[&str]) -> TokenManager {
        TokenManager {
            creds,
            client: Client::builder().build::<_, hyper::Body>(HttpsConnector::with_native_roots()),
            scopes: scopes.join(" "),
            current_token: None,
        }
    }

    pub(crate) async fn token(&mut self) -> Result<String, AuthError> {
        let hour = chrono::Duration::minutes(45);
        let current_time = chrono::Utc::now();
        match self.current_token {
            Some(ref token) if token.expiry >= current_time => Ok(token.value.to_string()),
            _ => {
                let expiry = current_time + hour;
                let claims = json!({
                    "iss": self.creds.client_email.as_str(),
                    "scope": self.scopes.as_str(),
                    "aud": AUTH_ENDPOINT,
                    "exp": expiry.timestamp(),
                    "iat": current_time.timestamp(),
                });
                let token = jwt::encode(
                    &jwt::Header::new(jwt::Algorithm::RS256),
                    &claims,
                    &jwt::EncodingKey::from_rsa_pem(&self.creds.private_key.as_bytes())?,
                )?;
                let form = format!(
                    "grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer&assertion={}",
                    token.as_str()
                );

                let req = hyper::Request::builder()
                    .method("POST")
                    .uri(AUTH_ENDPOINT)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(hyper::Body::from(form))?;

                let data = hyper::body::to_bytes(self.client.request(req).await?.into_body())
                    .await?
                    .to_vec();

                let ar: AuthResponse = json::from_slice(&data)?;

                let value = TokenValue::Bearer(ar.access_token);
                let token = value.to_string();
                self.current_token = Some(Token { expiry, value });

                Ok(token)
            }
        }
    }
}
