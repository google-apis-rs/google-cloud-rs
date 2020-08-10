use crate::tasks::{api, TaskConfig, ROUTING_METADATA_KEY, View};
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
    pub async fn create_task(&mut self, config: TaskConfig) -> Result<Task, Error> {
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

    /// Get task from this queue by ID (name)
    /// Only the `id` part of the task name should be supplied
    pub async fn get_task(&mut self, task_id: &str, view: Option<View>) -> Result<Task, Error> {
        let name = format!("{}/tasks/{}", self.name.clone(), task_id);
        let view : api::task::View = view.unwrap_or_default().into();
        let request = api::GetTaskRequest{ name: name.clone(), response_view: view as i32 };
        let mut request = self.client.construct_request(request).await?;
        request.metadata_mut().insert(
            ROUTING_METADATA_KEY,
            format!("name={}", name).parse().unwrap(),
        );
        let response = self.client.service.get_task(request).await?;
        let task = response.into_inner();
        Ok((self.client.clone(), task).into())
    }
}
