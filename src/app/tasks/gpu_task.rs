use super::task::BackgroundTask;
use crate::app::states::{GpuInfo, SharedGpuInfo, SharedSshHosts, fetch_gpu_info};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::timeout};

pub struct GpuInfoTask {
    pub ssh_hosts: SharedSshHosts,
    pub gpu_info: SharedGpuInfo,
}

#[async_trait]
impl BackgroundTask for GpuInfoTask {
    fn name(&self) -> &'static str {
        "gpu_info_checker"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(60) // Slower interval than CPU
    }

    async fn run(&self) {
        let hosts_info = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.values().cloned().collect::<Vec<_>>()
        };

        for info in hosts_info {
            let gpu_info = Arc::clone(&self.gpu_info);
            let host_id = info.id.clone();

            tokio::spawn(async move {
                let _permit = crate::app::tasks::SSH_SEMAPHORE.acquire().await.unwrap();
                {
                    let mut statuses = gpu_info.lock().await;
                    statuses.insert(host_id.clone(), GpuInfo::Loading);
                }

                let result = timeout(
                    Duration::from_secs(10),
                    task::spawn_blocking(move || fetch_gpu_info(&info)),
                )
                .await;

                let gpu_result = match result {
                    Ok(Ok(info)) => info,
                    Ok(Err(e)) => GpuInfo::failure(format!("Thread error: {e}")),
                    Err(_) => GpuInfo::failure("Timed out"),
                };

                {
                    let mut statuses = gpu_info.lock().await;
                    statuses.insert(host_id, gpu_result);
                }
            });
        }
    }
}
