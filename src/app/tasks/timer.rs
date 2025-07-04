use super::queue::TaskQueue;
use super::task::BackgroundTask;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub struct TaskTimer {
    tasks: Vec<Box<dyn BackgroundTask>>, 
    queue: Arc<TaskQueue>,
}

impl TaskTimer {
    pub fn new(queue: Arc<TaskQueue>) -> Self {
        Self { tasks: vec![], queue }
    }

    pub fn register<T: BackgroundTask + 'static>(&mut self, task: T) {
        self.tasks.push(Box::new(task));
    }

    pub fn start(self) {
        for task in self.tasks {
            let interval = task.interval();
            let queue = Arc::clone(&self.queue);
            let name = task.name();
            tokio::spawn(async move {
                loop {
                    tracing::debug!("Running task: {}", name);
                    task.run(Arc::clone(&queue)).await;
                    sleep(interval).await;
                }
            });
        }
    }
}
