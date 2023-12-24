use tokio::task::JoinSet;
use tracing::{debug, info, Level};
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
        task_tracker.spawn(async move {
            debug!("Entered task {i}");

            format!("Hello from task {i}")
        });
    }

    while let Some(msg) = task_tracker.join_next().await {
        info!("Received msg \"{}\"", msg.unwrap());
    }
}
