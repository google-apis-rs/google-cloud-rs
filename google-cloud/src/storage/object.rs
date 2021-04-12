use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;

use crate::storage::{Client, Error};

/// Represents a Cloud Storage bucket.
#[derive(Clone)]
pub struct Object {
    pub(crate) client: Client,
    pub(crate) name: String,
    pub(crate) bucket: String,
}

impl Object {
    pub(crate) fn new(
        client: Client,
        bucket: impl Into<String>,
        name: impl Into<String>,
    ) -> Object {
        Object {
            client,
            name: name.into(),
            bucket: bucket.into(),
        }
    }

    /// Get the object's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the object's bucket name.
    pub fn bucket(&self) -> &str {
        self.bucket.as_str()
    }

    // /// Insert a new object into the bucket.
    // pub async fn writer(&mut self, object: Object) -> Result<(), Error> {
    //     Ok(())
    // }

    /// Get an object stored in the bucket.
    pub async fn reader(&mut self) -> Result<impl tokio::io::AsyncRead, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!(
            "{}/b/{}/o/{}",
            Client::ENDPOINT,
            utf8_percent_encode(&self.bucket, NON_ALPHANUMERIC),
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
        );

        let token = client.token_manager.lock().await.token().await?;
        let request = inner
            .get(uri.as_str())
            .query(&[("alt", "media")])
            .header("authorization", token)
            .send();
        let response = request.await?;
        let stream = response.error_for_status()?.bytes_stream();

        // Convert the stream into an futures::io::AsyncRead.
        // We must first convert the reqwest::Error into an futures::io::Error.
        let stream = stream.map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e)).into_async_read();

        // Convert the futures::io::AsyncRead into a tokio::io::AsyncRead.
        Ok(stream.compat())
    }

    /// Get the entire contents of the object.
    pub async fn get(&mut self) -> Result<Vec<u8>, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!(
            "{}/b/{}/o/{}",
            Client::ENDPOINT,
            utf8_percent_encode(&self.bucket, NON_ALPHANUMERIC),
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
        );

        let token = client.token_manager.lock().await.token().await?;
        let request = inner
            .get(uri.as_str())
            .query(&[("alt", "media")])
            .header("authorization", token)
            .send();
        let response = request.await?;
        let bytes = response.error_for_status()?.bytes().await?.to_vec();

        Ok(bytes)
    }

    /// Delete the object.
    pub async fn delete(self) -> Result<(), Error> {
        let client = self.client;
        let inner = client.client;
        let uri = format!(
            "{}/b/{}/o/{}",
            Client::ENDPOINT,
            utf8_percent_encode(&self.bucket, NON_ALPHANUMERIC),
            utf8_percent_encode(&self.name, NON_ALPHANUMERIC),
        );

        let token = client.token_manager.lock().await.token().await?;
        let request = inner
            .delete(uri.as_str())
            .header("authorization", token)
            .send();
        let response = request.await?;
        response.error_for_status()?;

        Ok(())
    }
}
