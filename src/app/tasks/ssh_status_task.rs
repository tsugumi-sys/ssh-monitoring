use super::task::BackgroundTask;
use crate::app::states::{SshHostState, SshStatus, verify_connection};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex, task, time::timeout};

pub struct SshStatusTask {
    pub ssh_hosts: Arc<Mutex<HashMap<String, SshHostState>>>,
}

#[async_trait]
impl BackgroundTask for SshStatusTask {
    fn name(&self) -> &'static str {
        "ssh_status_checker"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    async fn run(&self) {
        let hosts_info: Vec<_> = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.values().map(|h| h.info.clone()).collect()
        };

        for info in hosts_info {
            let id = info.id.clone();
            let ssh_hosts = Arc::clone(&self.ssh_hosts);

            tokio::spawn(async move {
                // Mark as loading
                {
                    let mut hosts = ssh_hosts.lock().await;
                    if let Some(host) = hosts.get_mut(&id) {
                        host.status = SshStatus::Loading;
                    }
                }

                // Run the connection check with timeout
                let status = match timeout(
                    Duration::from_secs(10),
                    task::spawn_blocking(move || verify_connection(&info)),
                )
                .await
                {
                    Ok(Ok(result)) => result,
                    Ok(Err(e)) => SshStatus::Failed(format!("Thread error: {}", e)),
                    Err(_) => SshStatus::Failed("Timed out".into()),
                };

                // Update the host status
                {
                    let mut hosts = ssh_hosts.lock().await;
                    if let Some(host) = hosts.get_mut(&id) {
                        host.status = status;
                    }
                }
            });
        }
    }
}
