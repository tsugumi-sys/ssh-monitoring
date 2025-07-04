use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Semaphore};

pub type BoxFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct TaskQueue {
    senders: Mutex<HashMap<String, mpsc::Sender<BoxFuture>>>,
    semaphore: Arc<Semaphore>,
}

impl TaskQueue {
    pub fn new(max_concurrent_hosts: usize) -> Arc<Self> {
        Arc::new(Self {
            senders: Mutex::new(HashMap::new()),
            semaphore: Arc::new(Semaphore::new(max_concurrent_hosts)),
        })
    }

    pub async fn enqueue<F>(&self, host_id: String, fut: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut map = self.senders.lock().await;
        if let Some(tx) = map.get(&host_id) {
            let _ = tx.send(Box::pin(fut)).await;
            return;
        }

        let (tx, mut rx) = mpsc::channel::<BoxFuture>(100);
        tx.send(Box::pin(fut)).await.ok();
        map.insert(host_id.clone(), tx);
        let semaphore = Arc::clone(&self.semaphore);
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let permit = semaphore.acquire().await;
                let _permit = permit.expect("semaphore closed");
                task.await;
                // permit dropped here
            }
        });
    }
}
