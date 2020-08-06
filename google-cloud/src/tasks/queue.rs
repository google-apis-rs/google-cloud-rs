use crate::tasks::{api, TaskConfig, ROUTING_METADATA_KEY};
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

    /// Create a new task in this queue
    /// Requires the following roles on service account:
    /// - roles/cloudtasks.viewer
    /// - roles/cloudtasks.enqueuer
    pub async fn new_task(&mut self, config: TaskConfig) -> Result<Task, Error> {
        let request = api::CreateTaskRequest {
            parent: self.name.clone(),
            task: Some(config.into()),
            response_view: 0,
        };
        let mut request = self.client.construct_request(request).await?;
        request.metadata_mut().insert(
            ROUTING_METADATA_KEY,
            format!("parent={}", self.name.clone()).parse().unwrap(),
        );
        let response = self.client.service.create_task(request).await?;
        let task = response.into_inner();
        Ok((self.client.clone(), task).into())
    }
}
