use super::task::BackgroundTask;
use crate::app::states::{SshHostState, SshStatus, verify_connection};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Mutex, task, time::timeout};

pub struct SshStatusTask {
    pub ssh_hosts: Arc<Mutex<Vec<SshHostState>>>,
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
        let hosts_info = {
            let hosts = self.ssh_hosts.lock().await;
            hosts.iter().map(|h| h.info.clone()).collect::<Vec<_>>()
        };

        for (index, info) in hosts_info.into_iter().enumerate() {
            let ssh_hosts = Arc::clone(&self.ssh_hosts);

            tokio::spawn(async move {
                // Mark as loading
                {
                    let mut hosts = ssh_hosts.lock().await;
                    if let Some(host) = hosts.get_mut(index) {
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
                    if let Some(host) = hosts.get_mut(index) {
                        host.status = status;
                    }
                }
            });
        }
    }
}
