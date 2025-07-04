use super::{queue::TaskQueue, task::BackgroundTask};
use crate::app::states::{CpuInfo, SharedCpuInfo, SharedSshHosts, fetch_cpu_info};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::timeout};

pub struct CpuInfoTask {
    pub ssh_hosts: SharedSshHosts,
    pub cpu_info: SharedCpuInfo,
}

#[async_trait]
impl BackgroundTask for CpuInfoTask {
    fn name(&self) -> &'static str {
        "cpu_info_checker"
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
            let cpu_info = Arc::clone(&self.cpu_info);
            let host_id = info.id.clone();
            queue
                .enqueue(host_id.clone(), async move {
                    {
                        let mut statuses = cpu_info.lock().await;
                        statuses.insert(host_id.clone(), CpuInfo::Loading);
                    }

                    let result = timeout(
                        Duration::from_secs(10),
                        task::spawn_blocking(move || fetch_cpu_info(&info)),
                    )
                    .await;

                    let cpu_result = match result {
                        Ok(Ok(info)) => info,
                        Ok(Err(e)) => CpuInfo::failure(format!("Thread error: {e}")),
                        Err(_) => CpuInfo::failure("Timed out"),
                    };

                    {
                        let mut statuses = cpu_info.lock().await;
                        statuses.insert(host_id, cpu_result);
                    }
                })
                .await;
        }
    }
}
