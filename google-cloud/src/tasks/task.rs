use std::collections::HashMap;

use crate::tasks::{api, PayloadTypeConfig, PayloadType};
use crate::tasks::{Client, Error};
use chrono::{NaiveDateTime, Duration};
use tonic::Status;

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

/// Configuration for creating a new task
#[derive(Debug)]
pub struct TaskConfig {
    /// Optional ID of the task. This can be used for task deduplication, although since
    /// this will require looking up existing tasks, latency for creating task with an ID is much
    /// higher than for task without an ID.
    /// An ID is only freed about an hour after the task with the same ID was completed,
    /// so even if there are no task with the same ID in the queue at the moment of call, if
    /// a task with the same ID existed up to an hour earlier, the call will fail
    pub id: Option<String>,
    pub schedule_time: Option<NaiveDateTime>,
    /// The deadline for requests sent to the worker. If the worker does not
    /// respond by this deadline then the request is cancelled and the attempt
    /// is marked as a `DEADLINE_EXCEEDED` failure. Cloud Tasks will retry the
    /// task according to the RetryConfig
    pub dispatch_deadline: Option<NaiveDateTime>,
    pub payload_type: PayloadTypeConfig,
}

#[derive(Clone, Debug)]
pub struct Attempt {
    pub schedule_time: Option<NaiveDateTime>,
    pub dispatch_time: Option<NaiveDateTime>,
    pub response_time: Option<NaiveDateTime>,
    pub response_status: Option<Status>,
}

impl Attempt{
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
        self.response_status.as_deref()
    }
}

/// Represents a task
#[derive(Clone, Debug)]
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
        self.first_attempt.as_deref()
    }
    /// The status of the task's last attempt.
    pub fn last_attempt(&self) -> Option<&Attempt> {
        self.last_attempt.as_deref()
    }
    /// The view specifies which subset of the Task has been returned.
    pub fn view(&self) -> View {
        self.view
    }
    /// The message to send to the worker. May be absent in Basic view
    pub fn payload_type(&self) -> Option<&PayloadType> {
        self.payload_type.as_deref()
    }
}
