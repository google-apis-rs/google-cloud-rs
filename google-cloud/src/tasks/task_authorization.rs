use crate::tasks::api;

/// Configuration for Google-generated Authorization header
#[derive(Clone, Debug)]
pub enum AuthorizationHeader {
    /// If specified, an OAuth token
    /// will be generated and attached as an `Authorization` header in the HTTP
    /// request.
    ///
    /// This type of authorization should generally only be used when calling
    /// Google APIs hosted on *.googleapis.com.
    OauthToken(OAuthToken),
    /// If specified, an OIDC token will be generated and attached
    /// as an `Authorization` header in the HTTP request.
    ///
    /// This type of authorization can be used for many scenarios, including
    /// calling Cloud Run, or endpoints where you intend to validate the token
    /// yourself.
    OidcToken(OidcToken),
}

impl From<AuthorizationHeader> for api::http_request::AuthorizationHeader {
    fn from(item: AuthorizationHeader) -> Self {
        match item {
            AuthorizationHeader::OauthToken(token_config) => {
                api::http_request::AuthorizationHeader::OauthToken(token_config.into())
            }
            AuthorizationHeader::OidcToken(token_config) => {
                api::http_request::AuthorizationHeader::OidcToken(token_config.into())
            }
        }
    }
}

impl From<api::http_request::AuthorizationHeader> for AuthorizationHeader {
    fn from(item: api::http_request::AuthorizationHeader) -> Self {
        match item {
            api::http_request::AuthorizationHeader::OauthToken(token_config) => {
                AuthorizationHeader::OauthToken(token_config.into())
            }
            api::http_request::AuthorizationHeader::OidcToken(token_config) => {
                AuthorizationHeader::OidcToken(token_config.into())
            }
        }
    }
}

/// Config for google-generated Oauth token
/// https://cloud.google.com/tasks/docs/creating-http-target-tasks#sa
#[derive(Clone, Debug)]
pub struct OAuthToken {
    /// Service account email to be used for generating OAuth token.
    /// The service account must be within the same project as the queue. The
    /// caller must have iam.serviceAccounts.actAs permission for the service
    /// account.
    pub service_account_email: String,
    /// OAuth scope to be used for generating OAuth access token.
    /// If not specified, "https://www.googleapis.com/auth/cloud-platform"
    /// will be used.
    pub scope: String,
}

impl From<OAuthToken> for api::OAuthToken {
    fn from(item: OAuthToken) -> Self {
        Self {
            service_account_email: item.service_account_email,
            scope: item.scope,
        }
    }
}

impl From<api::OAuthToken> for OAuthToken {
    fn from(item: api::OAuthToken) -> Self {
        Self {
            service_account_email: item.service_account_email,
            scope: item.scope,
        }
    }
}

/// Config for google-generated Oidc token
/// https://cloud.google.com/tasks/docs/creating-http-target-tasks#token
#[derive(Clone, Debug)]
pub struct OidcToken {
    /// Service account email to be used for generating OIDC token.
    /// The service account must be within the same project as the queue. The
    /// caller must have iam.serviceAccounts.actAs permission for the service
    /// account.
    pub service_account_email: String,
    /// Audience to be used when generating OIDC token. If not specified, the URI
    /// specified in target will be used.
    pub audience: String,
}

impl From<OidcToken> for api::OidcToken {
    fn from(item: OidcToken) -> Self {
        Self {
            service_account_email: item.service_account_email,
            audience: item.audience,
        }
    }
}

impl From<api::OidcToken> for OidcToken {
    fn from(item: api::OidcToken) -> Self {
        Self {
            service_account_email: item.service_account_email,
            audience: item.audience,
        }
    }
}
