use std::collections::HashMap;
use crate::tasks::AuthorizationHeader;

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

#[derive(Clone, Debug)]
pub struct AppEngineRouting {
    pub(crate) service: String,
    pub(crate) version: String,
    pub(crate) instance: String,
    pub(crate) host: String,
}

impl AppEngineRouting {
    pub fn service(&self) -> &str {
        self.service.as_str()
    }
    pub fn version(&self) -> &str {
        self.version.as_str()
    }
    pub fn instance(&self) -> &str {
        self.instance.as_str()
    }
    pub fn host(&self) -> &str {
        self.host.as_str()
    }
}

#[derive(Clone, Debug)]
pub struct AppEngineHttpRequestConfig {
    /// The HTTP method to use for the request.
    pub http_method: HttpMethod,
    /// Task-level setting for App Engine routing.
    pub app_engine_routing: AppEngineRoutingConfig,
    /// The relative URI.
    ///
    /// The relative URI must begin with "/" and must be a valid HTTP relative URI.
    /// It can contain a path and query string arguments.
    /// If the relative URI is empty, then the root path "/" will be used.
    /// No spaces are allowed, and the maximum length allowed is 2083 characters.
    pub relative_uri: String,
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
    pub headers: HashMap<String, String>,
    /// HTTP request body.
    ///
    /// A request body is allowed only if the HTTP method is POST or PUT. It is
    /// an error to set a body on a task with an incompatible HttpMethod.
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct AppEngineHttpRequest {
    pub http_method: HttpMethod,
    pub app_engine_routing: Option<AppEngineRouting>,
    pub relative_uri: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl AppEngineHttpRequest {
    /// The HTTP method of this request.
    pub fn http_method(&self) -> HttpMethod {
        self.http_method
    }
    /// Task-level setting for App Engine routing.
    pub fn app_engine_routing(&self) -> Option<&AppEngineRouting> {
        self.app_engine_routing.as_deref()
    }
    /// The relative URI.
    pub fn relative_uri(&self) -> &str {
        self.relative_uri.as_str()
    }
    /// HTTP request headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        self.headers.as_ref()
    }
    /// HTTP request body.
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }
}

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
    pub url: String,
    /// The HTTP method to use for the request
    pub http_method: HttpMethod,
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
    pub headers: HashMap<String, String>,
    /// HTTP request body.
    ///
    /// A request body is allowed only if the HTTP method is POST, PUT, or PATCH. It is an
    /// error to set body on a task with an incompatible HttpMethod.
    pub body: Vec<u8>,
    /// The mode for generating an `Authorization` header for HTTP requests.
    ///
    /// If specified, all `Authorization` headers in the `HttpRequest.headers`
    /// field will be overridden.
    pub authorization_header: Option<AuthorizationHeader>,
}

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub(crate) url: String,
    pub(crate) http_method: HttpMethod,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
    pub(crate) authorization_header: Option<AuthorizationHeader>,
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
        self.headers.as_ref()
    }
    /// HTTP request body.
    pub fn body(&self) -> &[u8] {
        self.body.as_slice()
    }
    /// Google-generated authorization headers.
    pub fn authorization_header(&self) -> Option<&AuthorizationHeader> {
        self.authorization_header.as_deref()
    }
}

#[derive(Clone, Debug)]
pub enum PayloadTypeConfig {
    AppEngineHttpRequest(AppEngineHttpRequestConfig),
    HttpRequest(HttpRequestConfig),
}

#[derive(Clone, Debug)]
pub enum PayloadType {
    AppEngineHttpRequest(AppEngineHttpRequest),
    HttpRequest(HttpRequest),
}
