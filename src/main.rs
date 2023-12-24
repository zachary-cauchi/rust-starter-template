use tokio::task::JoinSet;
use tracing::{debug, error, info, span, Instrument, Level};
use utils::logging::LogSubscriberBuilder;

#[tokio::main]
async fn main() {
    LogSubscriberBuilder::new()
        .with_max_level(Level::DEBUG)
        .build_global();

    info!("Hello World");

    const TOTAL_TASKS: usize = 14;

    let mut task_tracker = JoinSet::new();

    for i in 0..TOTAL_TASKS {
        task_tracker.spawn(
            async move {
                debug!("Entered task {i}");

                format!("Hello from task {i}")
            }
            .instrument(span!(Level::DEBUG, "Hello-Tasks")),
        );
    }

    while let Some(task_result) = task_tracker.join_next().await {
        match task_result {
            Ok(msg) => info!("Received msg \"{}\"", msg),
            Err(e) => error!("Failed to process task: {}", e),
        }
    }
}
