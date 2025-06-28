use super::task::BackgroundTask;
use tokio::time::sleep;

pub struct TaskExecutor {
    tasks: Vec<Box<dyn BackgroundTask>>,
}

impl TaskExecutor {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }

    pub fn register<T: BackgroundTask + 'static>(&mut self, task: T) {
        self.tasks.push(Box::new(task));
    }

    pub fn start(self) {
        for task in self.tasks {
            let interval = task.interval();
            let name = task.name();
            tokio::spawn(async move {
                loop {
                    tracing::debug!("Running task: {}", name);
                    task.run().await;
                    sleep(interval).await;
                }
            });
        }
    }
}
