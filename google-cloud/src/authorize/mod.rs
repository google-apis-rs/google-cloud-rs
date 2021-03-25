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
    use_metadata_server: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct AuthResponse {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GCPTokenMetadata {
    access_token: String,
    token_type: String,
    expires_in: i64, // seconds to expiration
}

async fn get_metadata() -> Result<GCPTokenMetadata, hyper::Error> {
    // See
    // https://cloud.google.com/kubernetes-engine/docs/how-to/workload-identity#using_from_your_code
    let auth_endpoint = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

    let client = hyper::Client::default();
    let req = hyper::Request::builder()
        .method("GET")
        .uri(auth_endpoint)
        .header("Metadata-Flavor", "Google")
        .body(hyper::Body::empty())
        .expect("request builder");

    let res = client.request(req).await?;
    let data = hyper::body::to_bytes(res).await?;

    let gcp_meta: GCPTokenMetadata = json::from_slice(&data).expect("parse json");
    Ok(gcp_meta)
}

impl TokenManager {
    pub(crate) fn new(creds: ApplicationCredentials, scopes: &[&str]) -> TokenManager {
        TokenManager {
            creds,
            client: Client::builder().build::<_, hyper::Body>(HttpsConnector::new()),
            scopes: scopes.join(" "),
            current_token: None,
            use_metadata_server: true,
        }
    }

    pub(crate) async fn from_metadata_server() -> TokenManager {
        let token_metadata = get_metadata().await.unwrap();
        // println!("{:?}", token_metadata);

        // Hack: ApplicationCredentials are required by the type system
        // But given the behavior of the `token` method,
        // we can bypass it using a `current_token`.
        let fake_creds = ApplicationCredentials {
            cred_type: "".to_string(),
            project_id: "".to_string(),
            private_key_id: "".to_string(),
            private_key: "".to_string(),
            client_email: "".to_string(),
            client_id: "".to_string(),
            auth_uri: "".to_string(),
            token_uri: "".to_string(),
            auth_provider_x509_cert_url: "".to_string(),
            client_x509_cert_url: "".to_string(),
        };

        let lifetime = chrono::Duration::seconds(token_metadata.expires_in - 1);
        let current_time = chrono::Utc::now();

        TokenManager {
            creds: fake_creds,
            client: Client::builder().build::<_, hyper::Body>(HttpsConnector::new()),
            scopes: "".to_string(),
            use_metadata_server: true,
            current_token: Some(Token {
                expiry: current_time + lifetime,
                value: TokenValue::Bearer(token_metadata.access_token),
            }),
        }
    }

    pub(crate) async fn token(&mut self) -> Result<String, AuthError> {
        let hour = chrono::Duration::minutes(45);
        let current_time = chrono::Utc::now();
        match self.current_token {
            Some(ref token) if token.expiry >= current_time => Ok(token.value.to_string()),
            Some(ref token) if token.expiry >= current_time && self.use_metadata_server => {
                //
                // TODO
                // logic is a little convoluted but makes a clean diff
                // need to test
                //
                let token_metadata = get_metadata().await.unwrap();
                println!("\n\nNEW\n\n{:?}\n\n", token_metadata);
                let lifetime = chrono::Duration::seconds(token_metadata.expires_in - 1);
                let token_value = TokenValue::Bearer(token_metadata.access_token);
                let token_contents = token_value.to_string();
                let token = Token {
                    expiry: current_time + lifetime,
                    value: token_value,
                };

                self.current_token = Some(token);
                Ok(token_contents)
            }
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
