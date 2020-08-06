use crate::tasks::api;
use crate::tasks::{Client, Error, Task};

/// Represents a Queue
#[derive(Clone)]
pub struct Queue {
    pub(crate) client: Client,
    pub(crate) name: String,
}

impl Queue {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Queue {
        Queue {
            client,
            name: name.into(),
        }
    }

    /// Returns the unique identifier within its project
    pub fn id(&self) -> &str {
        self.name.rsplit('/').next().unwrap()
    }

    /// Delete the queue.
    pub async fn delete(mut self) -> Result<(), Error> {
        let request = api::DeleteQueueRequest {
            name: self.name.clone(),
        };
        let request = self.client.construct_request(request).await?;
        self.client.service.delete_queue(request).await?;

        Ok(())
    }

    /// Create a new task in this queue
    pub async fn new_task(&mut self) -> Result<Task, Error>{
        let request = api::CreateTaskRequest{
            parent: self.name.clone(),
            task: None,
            response_view: 0
        };
        todo!()
    }
}
