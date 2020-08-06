use chrono::{NaiveDateTime, Duration};

pub fn convert_timestamp(timestamp: NaiveDateTime) -> prost_types::Timestamp{
    prost_types::Timestamp{ seconds: timestamp.timestamp(), nanos: timestamp.timestamp_subsec_nanos() as i32 }
}

pub fn convert_duration(duration: Duration) -> prost_types::Duration{
    let seconds = duration.num_seconds();
    let duration_rem = duration - chrono::Duration::seconds(seconds);
    let nanos = duration_rem.num_nanoseconds().unwrap_or(0) as i32;
    prost_types::Duration{ seconds, nanos }
}
