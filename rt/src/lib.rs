use tokio::{fs::File, io::AsyncReadExt, task::JoinSet};
use tracing::{debug, error, info, instrument};
use utils::core_types::CoreResult;

#[instrument]
pub async fn test_errors() -> CoreResult<()> {
    debug!("Opening file.");

    let mut found_file = File::open("non-existent-file").await?;
    let mut buffer = vec![];

    debug!("Reading file contents.");

    let byte_count = found_file.read_to_end(&mut buffer).await?;

    info!("Read {byte_count} bytes.");

    Ok(())
}

#[instrument]
pub async fn test_tasks(task_count: usize) -> CoreResult<()> {
    let mut task_tracker: JoinSet<String> = JoinSet::new();

    for i in 0..task_count {
        task_tracker.spawn(async move {
            debug!("Entered task {i}");

            format!("Hello from task {i}")
        });
    }

    while let Some(task_result) = task_tracker.join_next().await {
        match task_result {
            Ok(msg) => info!("Received msg \"{}\"", msg),
            Err(e) => error!("Failed to process task: {}", e),
        }
    }

    Ok(())
}
