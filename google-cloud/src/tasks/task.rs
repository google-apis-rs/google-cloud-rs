use crate::tasks::Client;
use crate::tasks::{
    api, convert_status, duration_to_prost, prost_to_duration, prost_to_timestamp,
    timestamp_to_prost, AppEngineHttpRequestConfig, HttpRequestConfig, PayloadType,
    PayloadTypeConfig,
};
use chrono::{Duration, NaiveDateTime};
use tonic::Status;

/// Type of task view - basic or full
#[derive(Clone, Copy, Debug)]
pub enum View {
    /// Unspecified. Defaults to BASIC.
    Unspecified,
    /// The basic view omits fields which can be large or can contain
    /// sensitive data.
    ///
    /// This view does not include the
    /// body in AppEngineHttpRequest.
    /// Bodies are desirable to return only when needed, because they
    /// can be large and because of the sensitivity of the data that you
    /// choose to store in it.
    Basic,
    /// All information is returned.
    ///
    /// Authorization for Full requires `cloudtasks.tasks.fullView` permission on the resource.
    Full,
}

impl From<View> for api::task::View {
    fn from(item: View) -> Self {
        match item {
            View::Unspecified => api::task::View::Unspecified,
            View::Basic => api::task::View::Basic,
            View::Full => api::task::View::Full,
        }
    }
}

impl From<api::task::View> for View {
    fn from(item: api::task::View) -> Self {
        match item {
            api::task::View::Unspecified => View::Unspecified,
            api::task::View::Basic => View::Basic,
            api::task::View::Full => View::Full,
        }
    }
}

/// Configuration for creating a new task
#[derive(Debug)]
pub struct TaskConfig {
    /// Optional ID of the task. This can be used for task deduplication, although since
    /// this will require looking up existing tasks, latency for creating task with an ID is much
    /// higher than for task without an ID.
    /// An ID is only freed about an hour after the task with the same ID was completed,
    /// so even if there are no task with the same ID in the queue at the moment of call, if
    /// a task with the same ID existed up to an hour earlier, the call will fail
    id: Option<String>,
    schedule_time: Option<NaiveDateTime>,
    /// The deadline for requests sent to the worker. If the worker does not
    /// respond by this deadline then the request is cancelled and the attempt
    /// is marked as a `DEADLINE_EXCEEDED` failure. Cloud Tasks will retry the
    /// task according to the RetryConfig
    dispatch_deadline: Option<Duration>,
    payload_type: PayloadTypeConfig,
}

impl From<TaskConfig> for api::Task {
    fn from(item: TaskConfig) -> Self {
        Self {
            name: item.id.unwrap_or("".to_string()),
            schedule_time: item.schedule_time.map(timestamp_to_prost),
            create_time: None,
            dispatch_deadline: item.dispatch_deadline.map(duration_to_prost),
            dispatch_count: 0,
            response_count: 0,
            first_attempt: None,
            last_attempt: None,
            view: 0,
            payload_type: Some(item.payload_type.into()),
        }
    }
}

impl TaskConfig {
    /// Create new AppEngine HTTP task
    pub fn new_appengine_http_task(task: AppEngineHttpRequestConfig) -> Self {
        Self {
            id: None,
            schedule_time: None,
            dispatch_deadline: None,
            payload_type: PayloadTypeConfig::AppEngineHttpRequest(task),
        }
    }
    /// Create new HTTP task
    pub fn new_http_task(task: HttpRequestConfig) -> Self {
        Self {
            id: None,
            schedule_time: None,
            dispatch_deadline: None,
            payload_type: PayloadTypeConfig::HttpRequest(task),
        }
    }
    /// Set Task ID
    /// Parent is the name of the queue the task should go into
    /// ID is the ID of the task
    pub fn id(mut self, parent: &str, id: &str) -> Self {
        self.id.replace(format!("{}/{}", parent, id));
        self
    }
    /// Schedule Task for a specific time
    pub fn schedule_time(mut self, time: NaiveDateTime) -> Self {
        self.schedule_time.replace(time);
        self
    }
    /// Set Dispatch deadline
    pub fn dispatch_deadline(mut self, deadline: Duration) -> Self {
        self.dispatch_deadline.replace(deadline);
        self
    }
}

/// Describes Cloud Task delivery attempts
#[derive(Clone, Debug)]
pub struct Attempt {
    pub(crate) schedule_time: Option<NaiveDateTime>,
    pub(crate) dispatch_time: Option<NaiveDateTime>,
    pub(crate) response_time: Option<NaiveDateTime>,
    pub(crate) response_status: Option<Status>,
}

impl From<api::Attempt> for Attempt {
    fn from(item: api::Attempt) -> Self {
        Self {
            schedule_time: item.schedule_time.map(prost_to_timestamp),
            dispatch_time: item.dispatch_time.map(prost_to_timestamp),
            response_time: item.response_time.map(prost_to_timestamp),
            response_status: item.response_status.map(convert_status),
        }
    }
}

impl Attempt {
    ///The time that this attempt was scheduled.
    pub fn schedule_time(&self) -> Option<NaiveDateTime> {
        self.schedule_time
    }
    ///The time that this attempt was scheduled.
    pub fn dispatch_time(&self) -> Option<NaiveDateTime> {
        self.dispatch_time
    }
    ///The time that this attempt was scheduled.
    pub fn response_time(&self) -> Option<NaiveDateTime> {
        self.response_time
    }
    ///The time that this attempt was scheduled.
    pub fn response_status(&self) -> Option<&Status> {
        self.response_status.as_ref()
    }
}

/// Represents a task
#[derive(Clone)]
pub struct Task {
    pub(crate) client: Client,
    pub(crate) name: String,
    pub(crate) schedule_time: Option<NaiveDateTime>,
    pub(crate) create_time: Option<NaiveDateTime>,
    pub(crate) dispatch_deadline: Option<Duration>,
    pub(crate) dispatch_count: i32,
    pub(crate) response_count: i32,
    pub(crate) first_attempt: Option<Attempt>,
    pub(crate) last_attempt: Option<Attempt>,
    pub(crate) view: View,
    pub(crate) payload_type: Option<PayloadType>,
}

impl From<(Client, api::Task)> for Task {
    fn from(item: (Client, api::Task)) -> Self {
        let (client, task) = item;
        let view = task.view();
        Self {
            client,
            name: task.name,
            schedule_time: task.schedule_time.map(prost_to_timestamp),
            create_time: task.create_time.map(prost_to_timestamp),
            dispatch_deadline: task.dispatch_deadline.map(prost_to_duration),
            dispatch_count: task.dispatch_count,
            response_count: task.response_count,
            first_attempt: task.first_attempt.map(Attempt::from),
            last_attempt: task.last_attempt.map(Attempt::from),
            view: view.into(),
            payload_type: task.payload_type.map(PayloadType::from),
        }
    }
}

impl Task {
    /// The task's full name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    /// The time when the task is scheduled to be attempted.
    pub fn schedule_time(&self) -> Option<NaiveDateTime> {
        self.schedule_time
    }
    /// The time that the task was created.
    pub fn create_time(&self) -> Option<NaiveDateTime> {
        self.create_time
    }
    /// The deadline for requests sent to the worker.
    pub fn dispatch_deadline(&self) -> Option<Duration> {
        self.dispatch_deadline
    }
    /// The number of attempts dispatched.
    pub fn dispatch_count(&self) -> i32 {
        self.dispatch_count
    }
    /// The number of attempts which have received a response.
    pub fn response_count(&self) -> i32 {
        self.response_count
    }
    /// The status of the task's first attempt.
    pub fn first_attempt(&self) -> Option<&Attempt> {
        self.first_attempt.as_ref()
    }
    /// The status of the task's last attempt.
    pub fn last_attempt(&self) -> Option<&Attempt> {
        self.last_attempt.as_ref()
    }
    /// The view specifies which subset of the Task has been returned.
    pub fn view(&self) -> View {
        self.view
    }
    /// The message to send to the worker. May be absent in Basic view
    pub fn payload_type(&self) -> Option<&PayloadType> {
        self.payload_type.as_ref()
    }
}
