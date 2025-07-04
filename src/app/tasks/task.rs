use std::sync::Arc;
use tokio::time::Duration;
use super::queue::TaskQueue;

#[async_trait::async_trait]
pub trait BackgroundTask: Send + Sync {
    fn name(&self) -> &'static str;

    /// The interval at which this task should repeat
    fn interval(&self) -> Duration;

    /// The actual logic to run. The queue will execute the work for each host.
    async fn run(&self, queue: Arc<TaskQueue>);
}
