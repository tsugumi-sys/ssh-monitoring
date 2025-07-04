use super::{queue::TaskQueue, task::BackgroundTask};
use crate::app::states::{MemoryInfo, SharedMemoryInfo, SharedSshHosts, fetch_memory_info};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::timeout};

pub struct MemoryInfoTask {
    pub ssh_hosts: SharedSshHosts,
    pub memory_info: SharedMemoryInfo,
}

#[async_trait]
impl BackgroundTask for MemoryInfoTask {
    fn name(&self) -> &'static str {
        "memory_info_checker"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    async fn run(&self, queue: Arc<TaskQueue>) {
        let hosts_info = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.values().cloned().collect::<Vec<_>>()
        };

        for info in hosts_info {
            let memory_info = Arc::clone(&self.memory_info);
            let host_id = info.id.clone();
            queue
                .enqueue(host_id.clone(), async move {
                    {
                        let mut statuses = memory_info.lock().await;
                        statuses.insert(host_id.clone(), MemoryInfo::Loading);
                    }

                    let result = timeout(
                        Duration::from_secs(10),
                        task::spawn_blocking(move || fetch_memory_info(&info)),
                    )
                    .await;

                    let mem_result = match result {
                        Ok(Ok(info)) => info,
                        Ok(Err(e)) => MemoryInfo::failure(format!("Thread error: {e}")),
                        Err(_) => MemoryInfo::failure("Timed out"),
                    };

                    {
                        let mut statuses = memory_info.lock().await;
                        statuses.insert(host_id, mem_result);
                    }
                })
                .await;
        }
    }
}
