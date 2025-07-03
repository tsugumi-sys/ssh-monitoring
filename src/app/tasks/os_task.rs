use super::task::BackgroundTask;
use crate::app::states::{OsInfo, SharedOsInfo, SharedSshHosts, fetch_os_info};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::timeout};

pub struct OsInfoTask {
    pub ssh_hosts: SharedSshHosts,
    pub os_info: SharedOsInfo,
}

#[async_trait]
impl BackgroundTask for OsInfoTask {
    fn name(&self) -> &'static str {
        "os_info_checker"
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
            let os_info = Arc::clone(&self.os_info);
            let host_id = info.id.clone();

            tokio::spawn(async move {
                let _permit = crate::app::tasks::SSH_SEMAPHORE.acquire().await.unwrap();
                {
                    let mut statuses = os_info.lock().await;
                    statuses.insert(host_id.clone(), OsInfo::Loading);
                }

                let result = timeout(
                    Duration::from_secs(10),
                    task::spawn_blocking(move || fetch_os_info(&info)),
                )
                .await;

                let os_result = match result {
                    Ok(Ok(info)) => info,
                    Ok(Err(e)) => OsInfo::failure(format!("Thread error: {e}")),
                    Err(_) => OsInfo::failure("Timed out"),
                };

                {
                    let mut statuses = os_info.lock().await;
                    statuses.insert(host_id, os_result);
                }
            });
        }
    }
}
