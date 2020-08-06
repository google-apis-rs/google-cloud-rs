use std::collections::HashMap;
use crate::tasks::{AuthorizationHeader, api};

/// All supported HTTP methods for Cloud Tasks
#[derive(Clone, Copy, Debug)]
pub enum HttpMethod {
    /// HTTP method unspecified
    Unspecified,
    /// HTTP POST
    Post,
    /// HTTP GET
    Get,
    /// HTTP HEAD
    Head,
    /// HTTP PUT
    Put,
    /// HTTP DELETE
    Delete,
    /// HTTP PATCH
    Patch,
    /// HTTP OPTIONS
    Options,
}

impl From<api::HttpMethod> for HttpMethod{
    fn from(item: api::HttpMethod) -> Self {
        match item{
            api::HttpMethod::Unspecified => HttpMethod::Unspecified,
            api::HttpMethod::Post => HttpMethod::Post,
            api::HttpMethod::Get => HttpMethod::Get,
            api::HttpMethod::Head => HttpMethod::Head,
            api::HttpMethod::Put => HttpMethod::Put,
            api::HttpMethod::Delete => HttpMethod::Delete,
            api::HttpMethod::Patch => HttpMethod::Patch,
            api::HttpMethod::Options => HttpMethod::Options,
        }
    }
}

impl From<HttpMethod> for api::HttpMethod{
    fn from(item: HttpMethod) -> Self {
        match item{
            HttpMethod::Unspecified => api::HttpMethod::Unspecified,
            HttpMethod::Post => api::HttpMethod::Post,
            HttpMethod::Get => api::HttpMethod::Get,
            HttpMethod::Head => api::HttpMethod::Head,
            HttpMethod::Put => api::HttpMethod::Put,
            HttpMethod::Delete => api::HttpMethod::Delete,
            HttpMethod::Patch => api::HttpMethod::Patch,
            HttpMethod::Options => api::HttpMethod::Options,
        }
    }
}

/// Configuration to create custom AppEngine target for AppEngine HTTP request
#[derive(Clone, Debug)]
pub struct AppEngineRoutingConfig {
    /// App service.
    ///
    /// By default, the task is sent to the service which is the default
    /// service when the task is attempted.
    pub service: Option<String>,
    /// App version.
    ///
    /// By default, the task is sent to the version which is the default
    /// version when the task is attempted.
    pub version: Option<String>,
}

impl From<AppEngineRoutingConfig> for api::AppEngineRouting{
    fn from(item: AppEngineRoutingConfig) -> Self {
        Self{
            service: item.service.unwrap_or("".to_string()),
            version: item.version.unwrap_or("".to_string()),
            instance: "".to_string(),
            host: "".to_string()
        }
    }
}

/// Target configuration for AppEngine HTTP request
#[derive(Clone, Debug)]
pub struct AppEngineRouting {
    pub(crate) service: String,
    pub(crate) version: String,
    pub(crate) instance: String,
    pub(crate) host: String,
}

impl From<api::AppEngineRouting> for AppEngineRouting{
    fn from(item: api::AppEngineRouting) -> Self {
        Self{
            service: item.service,
            version: item.version,
            instance: item.instance,
            host: item.host
        }
    }
}

impl AppEngineRouting {
    /// Target service
    pub fn service(&self) -> &str {
        self.service.as_str()
    }
    /// Target app version.
    pub fn version(&self) -> &str {
        self.version.as_str()
    }
    /// Target app instance.
    pub fn instance(&self) -> &str {
        self.instance.as_str()
    }
    /// The host that the task is sent to.
    pub fn host(&self) -> &str {
        self.host.as_str()
    }
}

/// Configuration to create new AppEngine HTTP request
#[derive(Clone, Debug)]
pub struct AppEngineHttpRequestConfig {
    /// The HTTP method to use for the request. The default is POST.
    http_method: HttpMethod,
    /// Task-level setting for App Engine routing.
    app_engine_routing: Option<AppEngineRoutingConfig>,
    /// The relative URI.
    ///
    /// The relative URI must begin with "/" and must be a valid HTTP relative URI.
    /// It can contain a path and query string arguments.
    /// If the relative URI is empty, then the root path "/" will be used.
    /// No spaces are allowed, and the maximum length allowed is 2083 characters.
    relative_uri: String,
    /// HTTP request headers.
    ///
    /// This map contains the header field names and values.
    /// Headers can be set when the
    /// Repeated headers are not supported but a header value can contain commas.
    ///
    /// Cloud Tasks sets some headers to default values:
    ///
    /// * `User-Agent`: By default, this header is
    ///   `"AppEngine-Google; (+http://code.google.com/appengine)"`.
    ///   This header can be modified, but Cloud Tasks will append
    ///   `"AppEngine-Google; (+http://code.google.com/appengine)"` to the
    ///   modified `User-Agent`.
    ///
    /// If the task has a body, Cloud Tasks sets the following headers:
    ///
    /// * `Content-Type`: By default, the `Content-Type` header is set to
    ///   `"application/octet-stream"`. The default can be overridden by explicitly
    ///   setting `Content-Type` to a particular media type when the
    ///   task is created.
    ///   For example, `Content-Type` can be set to `"application/json"`.
    /// * `Content-Length`: This is computed by Cloud Tasks. This value is
    ///   output only.   It cannot be changed.
    ///
    /// The headers below cannot be set or overridden:
    ///
    /// * `Host`
    /// * `X-Google-*`
    /// * `X-AppEngine-*`
    ///
    /// In addition, Cloud Tasks sets some headers when the task is dispatched,
    /// such as headers containing information about the task; see
    /// [request
    /// headers](https://cloud.google.com/appengine/docs/python/taskqueue/push/creating-handlers#reading_request_headers).
    /// These headers are set only when the task is dispatched, so they are not
    /// visible when the task is returned in a Cloud Tasks response.
    ///
    /// Although there is no specific limit for the maximum number of headers or
    /// the size, there is a limit on the maximum size of the Task.
    headers: HashMap<String, String>,
    /// HTTP request body.
    ///
    /// A request body is allowed only if the HTTP method is POST or PUT. It is
    /// an error to set a body on a task with an incompatible HttpMethod.
    body: Vec<u8>,
}

impl From<AppEngineHttpRequestConfig> for api::AppEngineHttpRequest{
    fn from(item: AppEngineHttpRequestConfig) -> Self {
        let mut request = Self{
            http_method: 0,
            app_engine_routing: item.app_engine_routing.map(|routing| routing.into()),
            relative_uri: item.relative_uri,
            headers: item.headers,
            body: item.body
        };
        request.set_http_method(item.http_method.into());
        request
    }
}

impl AppEngineHttpRequestConfig{
    /// Create new App Engine HTTP request
    pub fn new() -> Self {
        Self {
            http_method: HttpMethod::Post,
            app_engine_routing: None,
            relative_uri: "".to_string(),
            headers: Default::default(),
            body: vec![]
        }
    }
    /// Set http method
    pub fn http_method(mut self, method: HttpMethod) -> Self {
        self.http_method = method;
        self
    }
    /// Configure task-level routing
    pub fn app_engine_routing(mut self, routing: AppEngineRoutingConfig) -> Self {
        self.app_engine_routing.replace(routing);
        self
    }
    /// Set uri for the request. Default is `/`
    pub fn relative_uri(mut self, uri: &str) -> Self {
        self.relative_uri = uri.to_string();
        self
    }
    /// Add header. Repeated headers are not supported but a header value can contain commas.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    /// Set request body. Should only be set for POST, PUT and PATCH requests
    pub fn body<T: Into<Vec<u8>>>(mut self, data: T) -> Self {
        self.body = data.into();
        self
    }
}

/// Represents HTTP rtequest that targets AppEngine App
#[derive(Clone, Debug)]
pub struct AppEngineHttpRequest {
    http_method: HttpMethod,
    app_engine_routing: Option<AppEngineRouting>,
    relative_uri: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl From<api::AppEngineHttpRequest> for AppEngineHttpRequest{
    fn from(item: api::AppEngineHttpRequest) -> Self {
        Self{
            http_method: item.http_method().into(),
            app_engine_routing: item.app_engine_routing.map(AppEngineRouting::from),
            relative_uri: item.relative_uri,
            headers: item.headers,
            body: item.body
        }
    }
}

impl AppEngineHttpRequest {
    /// The HTTP method of this request.
    pub fn http_method(&self) -> HttpMethod {
        self.http_method
    }
    /// Task-level setting for App Engine routing.
    pub fn app_engine_routing(&self) -> Option<&AppEngineRouting> {
        self.app_engine_routing.as_ref()
    }
    /// The relative URI.
    pub fn relative_uri(&self) -> &str {
        self.relative_uri.as_str()
    }
    /// HTTP request headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    /// HTTP request body.
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }
}

/// Configuration to create HTTP request
#[derive(Clone, Debug)]
pub struct HttpRequestConfig {
    /// Required. The full url path that the request will be sent to.
    ///
    /// This string must begin with either "http://" or "https://". Some examples
    /// are: `http://acme.com` and `https://acme.com/sales:8080`. Cloud Tasks will
    /// encode some characters for safety and compatibility. The maximum allowed
    /// URL length is 2083 characters after encoding.
    ///
    /// The `Location` header response from a redirect response [`300` - `399`]
    /// may be followed. The redirect is not counted as a separate attempt.
    url: String,
    /// The HTTP method to use for the request. The default is POST.
    http_method: HttpMethod,
    /// HTTP request headers.
    ///
    /// This map contains the header field names and values.
    /// Headers can be set when the task is created.
    ///
    /// These headers represent a subset of the headers that will accompany the
    /// task's HTTP request. Some HTTP request headers will be ignored or replaced.
    ///
    /// A partial list of headers that will be ignored or replaced is:
    ///
    /// * Host: This will be computed by Cloud Tasks and derived from HttpRequest.url.
    /// * Content-Length: This will be computed by Cloud Tasks.
    /// * User-Agent: This will be set to `"Google-Cloud-Tasks"`.
    /// * X-Google-*: Google use only.
    /// * X-AppEngine-*: Google use only.
    ///
    /// `Content-Type` won't be set by Cloud Tasks. You can explicitly set
    /// `Content-Type` to a media type when the task is created.
    ///  For example, `Content-Type` can be set to `"application/octet-stream"` or
    ///  `"application/json"`.
    ///
    /// Headers which can have multiple values (according to RFC2616) can be
    /// specified using comma-separated values.
    ///
    /// The size of the headers must be less than 80KB.
    headers: HashMap<String, String>,
    /// HTTP request body.
    ///
    /// A request body is allowed only if the HTTP method is POST, PUT, or PATCH. It is an
    /// error to set body on a task with an incompatible HttpMethod.
    body: Vec<u8>,
    /// The mode for generating an `Authorization` header for HTTP requests.
    ///
    /// If specified, all `Authorization` headers in the `HttpRequest.headers`
    /// field will be overridden.
    authorization_header: Option<AuthorizationHeader>,
}

impl From<HttpRequestConfig> for api::HttpRequest{
    fn from(item: HttpRequestConfig) -> Self {
        let mut request = Self{
            url: item.url,
            http_method: 0,
            headers: item.headers,
            body: item.body,
            authorization_header: item.authorization_header.map(|header| header.into())
        };
        request.set_http_method(item.http_method.into());
        request
    }
}

impl HttpRequestConfig{
    /// Create new HttpRequest. URI must be specified at a minimum
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            http_method: HttpMethod::Post,
            headers: Default::default(),
            body: vec![],
            authorization_header: None
        }
    }
    /// Set http method
    pub fn http_method(mut self, method: HttpMethod) -> Self {
        self.http_method = method;
        self
    }
    /// Add header. Repeated headers are not supported but a header value can contain commas.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    /// Set request body. Should only be set for POST, PUT and PATCH requests
    pub fn body<T: Into<Vec<u8>>>(mut self, data: T) -> Self {
        self.body = data.into();
        self
    }
    /// Allows setting data for Google-generated authorization header
    pub fn authorization_header(mut self, authorization: AuthorizationHeader) -> Self {
        self.authorization_header.replace(authorization);
        self
    }
}

/// Represents HTTP Request
#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub(crate) url: String,
    pub(crate) http_method: HttpMethod,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
    pub(crate) authorization_header: Option<AuthorizationHeader>,
}

impl From<api::HttpRequest> for HttpRequest{
    fn from(item: api::HttpRequest) -> Self {
        let method = item.http_method();
        Self{
            url: item.url,
            http_method: method.into(),
            headers: item.headers,
            body: item.body,
            authorization_header: item.authorization_header.map(AuthorizationHeader::from)
        }
    }
}

impl HttpRequest {
    /// The full url path that the request will be sent to.
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    /// The HTTP method of this request.
    pub fn http_method(&self) -> HttpMethod {
        self.http_method
    }
    /// HTTP request headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
    /// HTTP request body.
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }
    /// Google-generated authorization headers.
    pub fn authorization_header(&self) -> Option<&AuthorizationHeader> {
        self.authorization_header.as_ref()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PayloadTypeConfig {
    AppEngineHttpRequest(AppEngineHttpRequestConfig),
    HttpRequest(HttpRequestConfig),
}

impl From<PayloadTypeConfig> for api::task::PayloadType{
    fn from(item: PayloadTypeConfig) -> Self {
        match item {
            PayloadTypeConfig::HttpRequest(request) => api::task::PayloadType::HttpRequest(request.into()),
            PayloadTypeConfig::AppEngineHttpRequest(request) => api::task::PayloadType::AppEngineHttpRequest(request.into()),
        }
    }
}

/// Types of CLoud Task payloads
#[derive(Clone, Debug)]
pub enum PayloadType {
    /// HTTP request that targets AppEngine App
    AppEngineHttpRequest(AppEngineHttpRequest),
    /// HTTP request that targets any public URI
    HttpRequest(HttpRequest),
}

impl From<api::task::PayloadType> for PayloadType{
    fn from(item: api::task::PayloadType) -> Self {
        match item{
            api::task::PayloadType::AppEngineHttpRequest(request) => PayloadType::AppEngineHttpRequest(request.into()),
            api::task::PayloadType::HttpRequest(request) => PayloadType::HttpRequest(request.into()),
        }
    }
}
