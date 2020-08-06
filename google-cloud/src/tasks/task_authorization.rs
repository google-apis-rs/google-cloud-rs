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

