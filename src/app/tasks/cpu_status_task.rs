use super::task::BackgroundTask;
use crate::app::states::{CpuInfo, SshHostState, fetch_cpu_info};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex, task, time::timeout};

#[derive(Debug, Clone)]
pub enum CpuInfoStatus {
    Loading,
    Fetched(CpuInfo),
    Failed(String),
}

pub struct CpuInfoTask {
    pub ssh_hosts: Arc<Mutex<Vec<SshHostState>>>,
    pub cpu_statuses: Arc<Mutex<Vec<CpuInfoStatus>>>,
}

#[async_trait]
impl BackgroundTask for CpuInfoTask {
    fn name(&self) -> &'static str {
        "cpu_info_checker"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    async fn run(&self) {
        let hosts_info = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.iter().map(|h| h.info.clone()).collect::<Vec<_>>()
        };

        for (index, info) in hosts_info.into_iter().enumerate() {
            let cpu_statuses = Arc::clone(&self.cpu_statuses);

            tokio::spawn(async move {
                // Set loading
                {
                    let mut statuses = cpu_statuses.lock().await;
                    if index >= statuses.len() {
                        statuses.resize(index + 1, CpuInfoStatus::Loading);
                    }
                    statuses[index] = CpuInfoStatus::Loading;
                }

                // Run fetch_cpu_info with timeout
                let status = match timeout(
                    Duration::from_secs(10),
                    task::spawn_blocking(move || fetch_cpu_info(&info)),
                )
                .await
                {
                    Ok(Ok(Ok(info))) => CpuInfoStatus::Fetched(CpuInfo {
                        core_count: info.core_count,
                        usage_percent: info.usage_percent,
                    }),
                    Ok(Ok(Err(e))) => CpuInfoStatus::Failed(format!("Fetch error: {e}")),
                    Ok(Err(e)) => CpuInfoStatus::Failed(format!("Thread error: {e}")),
                    Err(_) => CpuInfoStatus::Failed("Timed out".into()),
                };

                // Update result
                {
                    let mut statuses = cpu_statuses.lock().await;
                    if index >= statuses.len() {
                        statuses.resize(index + 1, CpuInfoStatus::Loading);
                    }
                    statuses[index] = status;
                }
            });
        }
    }
}
