use chrono::{NaiveDateTime, Duration};
use tonic::Status;
use crate::tasks::api;

const NANOS_PER_SEC: i64 = 1_000_000_000;

pub fn timestamp_to_prost(timestamp: NaiveDateTime) -> prost_types::Timestamp{
    prost_types::Timestamp{ seconds: timestamp.timestamp(), nanos: timestamp.timestamp_subsec_nanos() as i32 }
}

pub fn prost_to_timestamp(timestamp: prost_types::Timestamp) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(timestamp.seconds, timestamp.nanos as u32)
}

pub fn duration_to_prost(duration: Duration) -> prost_types::Duration{
    let seconds = duration.num_seconds();
    let duration_rem = duration - chrono::Duration::seconds(seconds);
    let nanos = duration_rem.num_nanoseconds().unwrap_or(0) as i32;
    prost_types::Duration{ seconds, nanos }
}

pub fn prost_to_duration(duration: prost_types::Duration) -> Duration {
    Duration::nanoseconds(duration.seconds * NANOS_PER_SEC + (duration.nanos as i64))
}

/// For now this drops details as not sure how to convert
pub fn convert_status(prost_status: api::google::rpc::Status) -> Status {
    Status::new(prost_status.code.into(), prost_status.message)
}
