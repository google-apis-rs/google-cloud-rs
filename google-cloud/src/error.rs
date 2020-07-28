use std::env;
use std::io;

use thiserror::Error;

/// The main error-handling type.
#[derive(Debug, Error)]
pub enum Error {
    /// An unexpected status code was received.
    #[error("unexpected status from GCP: {0}")]
    Status(#[from] tonic::Status),
    /// An error with the gRPC transport channel.
    #[error("transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    /// An IO error.
    #[error("IO error: {0}")]
    IO(#[from] io::Error),
    /// A JSON (de)serialization error.
    #[error("JSON error: {0}")]
    JSON(#[from] json::Error),
    /// An environment-related error (missing variable).
    #[error("environment error: {0}")]
    Env(#[from] env::VarError),
    /// Reqwest error (HTTP errors).
    #[cfg(feature = "storage")]
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// conversion error (`try_from(..)` or `try_into(..)` errors).
    #[error("conversion error: {0}")]
    Convert(#[from] ConvertError),
    /// authentication-related error.
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),
}

/// The error type for value conversions.
#[derive(Debug, Error)]
pub enum ConvertError {
    /// An expected property was missing.
    #[error("expected property `{0}` was missing")]
    MissingProperty(String),
    /// A value, expected to be an entity, turned out to not be one.
    #[error("expected property type `{expected}`, got `{got}`")]
    UnexpectedPropertyType {
        /// The name of the expected type.
        expected: String,
        /// The name of the actual encountered type.
        got: String,
    },
}

/// The error type for value conversions.
#[derive(Debug, Error)]
pub enum AuthError {
    /// A JWT-related error.
    #[error("JWT error: {0}")]
    JWT(#[from] jwt::errors::Error),
    /// A JSON (de)serialization error.
    #[error("JSON error: {0}")]
    JSON(#[from] json::Error),
    /// HTTP errors
    #[error("Hyper error: {0}")]
    Http(#[from] http::Error),
    /// Hyper errors
    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),
}
