use crate::storage::api::object::ObjectResource;
use crate::storage::{Client, Error, Object};

/// Represents a Cloud Storage bucket.
#[derive(Clone)]
pub struct Bucket {
    pub(crate) client: Client,
    pub(crate) name: String,
}

impl Bucket {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Bucket {
        Bucket {
            client,
            name: name.into(),
        }
    }

    /// Get the bucket's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Insert a new object into the bucket.
    pub async fn create_object(
        &mut self,
        name: &str,
        data: impl Into<Vec<u8>>,
    ) -> Result<Object, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!("{}/b/{}/o", Client::ENDPOINT, self.name);

        let request = inner
            .post(uri.as_str())
            .query(&[("uploadType", "media"), ("name", name)])
            .header("authorization", client.token_manager.token())
            .header("content-type", "application/octet-stream")
            .body(data.into())
            .send();
        let response = request.await?;
        let string = response.error_for_status()?.text().await?;
        let resource = json::from_str::<ObjectResource>(dbg!(string).as_str())?;

        Ok(Object::new(
            client.clone(),
            self.name.clone(),
            resource.name,
        ))
    }

    /// Get an object stored in the bucket.
    pub async fn object(&mut self, name: &str) -> Result<Object, Error> {
        let client = &mut self.client;
        let inner = &client.client;
        let uri = format!("{}/b/{}/o/{}", Client::ENDPOINT, self.name, name);

        let request = inner
            .get(uri.as_str())
            .header("authorization", client.token_manager.token())
            .send();
        let response = request.await?;
        let string = response.error_for_status()?.text().await?;
        let resource = json::from_str::<ObjectResource>(dbg!(string).as_str())?;

        Ok(Object::new(
            client.clone(),
            self.name.clone(),
            resource.name,
        ))
    }

    /// Delete the bucket.
    pub async fn delete(self) -> Result<(), Error> {
        let mut client = self.client;
        let inner = client.client;
        let uri = format!("{}/b/{}", Client::ENDPOINT, self.name);

        let request = inner
            .delete(uri.as_str())
            .header("authorization", client.token_manager.token())
            .send();
        let response = request.await?;
        response.error_for_status()?;

        Ok(())
    }
}
