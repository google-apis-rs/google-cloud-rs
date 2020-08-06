use crate::tasks::{api, TaskConfig};
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
    pub async fn new_task(&mut self, config: TaskConfig) -> Result<Task, Error>{
        let request = api::CreateTaskRequest{
            parent: self.name.clone(),
            task: Some(config.into()),
            response_view: 0
        };
        let request = self.client.construct_request(request).await?;
        let response = self.client.service.create_task(request).await?;
        let task = response.into_inner();
        Ok((self.client.clone(), task).into())
    }
}
