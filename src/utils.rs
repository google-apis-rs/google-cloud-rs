use json::json;

use crate::authorize::ApplicationCredentials;

/// Legacy function, allowing to generate short-lived tokens without using OAuth.
#[allow(dead_code)]
pub(crate) fn generate_token(
    endpoint: &str,
    creds: &ApplicationCredentials,
) -> Result<String, jwt::Error> {
    let current_time = chrono::Utc::now();
    let max_age = chrono::Duration::seconds(3600);
    let header = json!({
        "alg": "RS256",
        "typ": "JWT",
        "kid": creds.private_key_id.as_str(),
    });
    let payload = json!({
        "iss": creds.client_email.as_str(),
        "sub": creds.client_email.as_str(),
        "aud": endpoint,
        "iat": current_time.timestamp(),
        "exp": (current_time + max_age).timestamp(),
    });
    let token = jwt::encode(header, &creds.private_key, &payload, jwt::Algorithm::RS256)?;
    Ok(token)
}
