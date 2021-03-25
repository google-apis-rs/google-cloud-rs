// use crate::tasks;
// use chrono::{Utc, Duration};
//
// macro_rules! assert_ok {
//     ($expr:expr) => {
//         match $expr {
//             Ok(value) => value,
//             Err(err) => {
//                 panic!("asserted result is an error: {}", err);
//             }
//         }
//     };
// }
//
// macro_rules! assert_some {
//     ($expr:expr) => {
//         match $expr {
//             Some(value) => value,
//             None => {
//                 panic!("asserted option is an none");
//             }
//         }
//     };
// }
//
// async fn setup_client() -> Result<tasks::Client, tasks::Error> {
//     let creds = super::load_creds();
//     let location = env!("GCP_QUEUE_LOCATION_ID");
//     tasks::Client::from_credentials(env!("GCP_TEST_PROJECT"), location, creds).await
// }
//
// #[tokio::test]
// async fn test_task_create_read_delete() {
//     //? Setup test client.
//     let mut client = assert_ok!(setup_client().await);
//
//     //? Get test queue
//     let queue = client.queue(env!("GCP_TEST_TOPIC")).await;
//     let mut queue = assert_ok!(queue);
//
//     //? Create test task
//     let request_config = tasks::HttpRequestConfig::new("https://example.com")
//         .http_method(tasks::HttpMethod::Get)
//         .header(
//             "X-Test-Header-One",
//             "Header value one",
//         )
//         .header("X-Test-Header-Two", "Header Value Two")
//         .body("some payload");
//
//     let task_config = tasks::TaskConfig::new_http_task(request_config)
//         .schedule_time(Utc::now().naive_utc() + Duration::minutes(10));
//     let task = assert_ok!(queue.create_task(task_config).await);
//
//     //? Check can retrieve created task
//     let stored_task = queue.get_task(task.id(), None).await;
//     let mut stored_task = assert_ok!(stored_task);
//
//     //? Check can delete the task
//     let delete_result = stored_task.delete_task().await;
//     assert_ok!(delete_result);
// }
