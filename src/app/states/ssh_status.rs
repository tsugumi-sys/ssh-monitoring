use super::ssh_hosts::SshHostInfo;
use ssh2::Session;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{Duration, timeout};

#[derive(Debug, Clone)]
pub enum SshStatus {
    Connected,
    Failed(String),
    Loading,
}

#[derive(Debug, Clone)]
pub struct SshHostState {
    pub info: SshHostInfo,
    pub status: SshStatus,
}

pub fn update_ssh_status(ssh_hosts: Arc<Mutex<Vec<SshHostState>>>) {
    tokio::spawn(async move {
        loop {
            let cloned_hosts = {
                let hosts = ssh_hosts.lock().await;
                hosts.iter().map(|h| h.info.clone()).collect::<Vec<_>>()
            };

            for (index, info) in cloned_hosts.into_iter().enumerate() {
                let ssh_hosts = Arc::clone(&ssh_hosts);

                tokio::spawn(async move {
                    // Step 1: Set this host to Loading
                    {
                        let mut hosts = ssh_hosts.lock().await;
                        if let Some(host) = hosts.get_mut(index) {
                            host.status = SshStatus::Loading;
                        }
                    }

                    // Step 2: Run connection test
                    let status = test_ssh_connection_with_timeout(info).await;

                    // Step 3: Update this host’s status
                    {
                        let mut hosts = ssh_hosts.lock().await;
                        if let Some(host) = hosts.get_mut(index) {
                            host.status = status;
                        }
                    }
                });
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

pub async fn test_ssh_connection_with_timeout(info: SshHostInfo) -> SshStatus {
    match timeout(
        Duration::from_secs(10),
        task::spawn_blocking(move || test_ssh_connection(&info)),
    )
    .await
    {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => SshStatus::Failed(format!("Thread error: {}", e)),
        Err(_) => SshStatus::Failed("Timed out".into()),
    }
}

pub fn test_ssh_connection(info: &SshHostInfo) -> SshStatus {
    let addr = format!("{}:{}", info.ip, info.port);

    let tcp = match TcpStream::connect(&addr) {
        Ok(t) => t,
        Err(e) => return SshStatus::Failed(format!("TCP error: {}", e)),
    };

    let mut session = match Session::new() {
        Ok(s) => s,
        Err(e) => return SshStatus::Failed(format!("Session error: {}", e)),
    };

    session.set_tcp_stream(tcp);
    if let Err(e) = session.handshake() {
        return SshStatus::Failed(format!("Handshake error: {}", e));
    }

    let identity_path = PathBuf::from(&info.identity_file);

    if !identity_path.exists() {
        return SshStatus::Failed(format!(
            "Identity file not found: {}",
            identity_path.display()
        ));
    }

    let mut agent = match session.agent() {
        Ok(a) => a,
        Err(e) => return SshStatus::Failed(format!("Agent error: {}", e)),
    };
    if let Err(e) = agent.connect() {
        return SshStatus::Failed(format!("Agent connect error: {}", e));
    }
    if let Err(e) = agent.list_identities() {
        return SshStatus::Failed(format!("Agent list error: {}", e));
    }
    for identity in agent.identities().unwrap_or_default() {
        if agent.userauth(&info.user, &identity).is_ok() && session.authenticated() {
            return SshStatus::Connected;
        }
    }
    SshStatus::Failed("Agent auth failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_connection_to_host() {
        let info = SshHostInfo {
            name: "minipc".into(),
            ip: "sshminipc.tsugumisys.com".into(), // or your actual hostname
            port: 22,
            user: "tsugumisys".into(),
            identity_file: "~/.ssh/id_rsa".into(),
        };

        let result = test_ssh_connection_with_timeout(info).await;
        println!("Test result: {:?}", result);

        // Optionally:
        // assert!(matches!(result, SshStatus::Connected | SshStatus::Failed(_)));
    }
}
