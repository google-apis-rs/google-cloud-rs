use std::fmt;

use json::json;
use serde::{Deserialize, Serialize};

pub(crate) const TLS_CERTS: &[u8] = include_bytes!("../../roots.pem");

const AUTH_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

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
    expiry: chrono::DateTime<chrono::offset::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TokenManager {
    creds: ApplicationCredentials,
    scopes: String,
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
            scopes: scopes.join(" "),
            current_token: None,
        }
    }

    pub(crate) fn token(&mut self) -> String {
        let hour = chrono::Duration::minutes(45);
        let current_time = chrono::Utc::now();
        match self.current_token {
            Some(ref token) if token.expiry >= current_time => token.value.to_string(),
            _ => {
                let expiry = current_time + hour;
                let header = json!({
                    "alg": "RS256",
                    "typ": "JWT",
                });
                let payload = json!({
                    "iss": self.creds.client_email.as_str(),
                    "scope": self.scopes.as_str(),
                    "aud": AUTH_ENDPOINT,
                    "exp": expiry.timestamp(),
                    "iat": current_time.timestamp(),
                });
                let token = jwt::encode(
                    header,
                    &self.creds.private_key.as_str(),
                    &payload,
                    jwt::Algorithm::RS256,
                );
                let body = format!(
                    r#"grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={}"#,
                    token.unwrap(),
                );
                let request = isahc::http::Request::post(AUTH_ENDPOINT)
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(body)
                    .unwrap();
                let response = isahc::send(request)
                    .unwrap()
                    .body_mut()
                    .json::<AuthResponse>()
                    .unwrap();
                let value = TokenValue::Bearer(response.access_token);
                let token = value.to_string();
                self.current_token = Some(Token { expiry, value });
                token
            }
        }
    }
}
