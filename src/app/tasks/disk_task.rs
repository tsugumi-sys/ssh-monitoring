use super::task::BackgroundTask;
use crate::app::states::{DiskInfo, SharedDiskInfo, SharedSshHosts, fetch_disk_info};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::timeout};

pub struct DiskInfoTask {
    pub ssh_hosts: SharedSshHosts,
    pub disk_info: SharedDiskInfo,
}

#[async_trait]
impl BackgroundTask for DiskInfoTask {
    fn name(&self) -> &'static str {
        "disk_info_checker"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(60 * 60)
    }

    async fn run(&self) {
        let hosts_info = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.values().cloned().collect::<Vec<_>>()
        };

        for info in hosts_info {
            let disk_info = Arc::clone(&self.disk_info);
            let host_id = info.id.clone();

            tokio::spawn(async move {
                {
                    let mut statuses = disk_info.lock().await;
                    statuses.insert(host_id.clone(), DiskInfo::Loading);
                }

                let result = timeout(
                    Duration::from_secs(10),
                    task::spawn_blocking(move || fetch_disk_info(&info)),
                )
                .await;

                let disk_result = match result {
                    Ok(Ok(info)) => info,
                    Ok(Err(e)) => DiskInfo::failure(format!("Thread error: {e}")),
                    Err(_) => DiskInfo::failure("Timed out"),
                };

                {
                    let mut statuses = disk_info.lock().await;
                    statuses.insert(host_id, disk_result);
                }
            });
        }
    }
}
