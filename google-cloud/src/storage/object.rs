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
    // pub async fn reader(&mut self, object: Object) -> Result<(), Error> {
    //     Ok(())
    // }

    // /// Get an object stored in the bucket.
    // pub async fn writer(&mut self) -> Result<Object, Error> {
    //     Ok(())
    // }

    /// Delete the object.
    pub async fn delete(self) -> Result<(), Error> {
        let mut client = self.client;
        let inner = client.client;
        let uri = format!("{}/b/{}/o/{}", Client::ENDPOINT, self.bucket, self.name);

        let request = inner
            .delete(uri.as_str())
            .header("authorization", client.token_manager.token())
            .send();
        let response = request.await?;
        response.error_for_status()?;

        Ok(())
    }
}
