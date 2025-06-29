use super::task::BackgroundTask;
use crate::app::states::{SharedSshHosts, SharedSshStatuses, SshStatus, verify_connection};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{task, time::timeout};

pub struct SshStatusTask {
    pub ssh_hosts: SharedSshHosts,
    pub ssh_statuses: SharedSshStatuses,
}

#[async_trait]
impl BackgroundTask for SshStatusTask {
    fn name(&self) -> &'static str {
        "ssh_status_checker"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(120)
    }

    async fn run(&self) {
        let infos = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.values().cloned().collect::<Vec<_>>() // Vec<SshHostInfo>
        };

        for info in infos {
            let id = info.id.clone();
            let statuses = Arc::clone(&self.ssh_statuses);

            tokio::spawn(async move {
                // Mark as loading
                {
                    let mut st = statuses.lock().await;
                    st.insert(id.clone(), SshStatus::Loading);
                }

                // Perform the check with timeout
                let result = timeout(
                    Duration::from_secs(10),
                    task::spawn_blocking(move || verify_connection(&info)),
                )
                .await;

                let status = match result {
                    Ok(Ok(status)) => status,
                    Ok(Err(e)) => SshStatus::Failed(format!("Thread error: {}", e)),
                    Err(_) => SshStatus::Failed("Timed out".into()),
                };

                // Update status map
                {
                    let mut st = statuses.lock().await;
                    st.insert(id, status);
                }
            });
        }
    }
}
