use tokio::time::Duration;

#[async_trait::async_trait]
pub trait BackgroundTask: Send + Sync {
    fn name(&self) -> &'static str;

    /// The interval at which this task should repeat
    fn interval(&self) -> Duration;

    /// The actual logic to run
    async fn run(&self);
}
