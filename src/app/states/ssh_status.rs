use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::connect_ssh_session;
use super::ssh_limits::SSH_LIMITER;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum SshStatus {
    Connected,
    Failed(String),
    Loading,
}

pub type SharedSshStatuses = Arc<Mutex<HashMap<String, SshStatus>>>;

pub fn verify_connection(info: &SshHostInfo) -> SshStatus {
    let (_permit, _guard) = SSH_LIMITER.acquire(&info.id);

    match connect_ssh_session(info) {
        Ok(_) => SshStatus::Connected,
        Err(e) => SshStatus::Failed(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        task,
        time::{Duration, timeout},
    };

    #[tokio::test]
    async fn test_connection_to_public_ssh_should_fail() {
        let info = SshHostInfo {
            id: "test".into(),
            name: "rebex_test".into(),
            ip: "test.rebex.net".into(), // Public test server
            port: 22,
            user: "demo".into(),               // Valid user
            identity_file: "/dev/null".into(), // Invalid key path
        };

        let result = timeout(
            Duration::from_secs(10),
            task::spawn_blocking(move || verify_connection(&info)),
        )
        .await;

        match result {
            Ok(Ok(status)) => {
                println!("Test result: {:?}", status);
                assert!(
                    matches!(status, SshStatus::Failed(_)),
                    "Expected failure, got: {:?}",
                    status
                );
            }
            Ok(Err(e)) => panic!("Thread join error: {e}"),
            Err(_) => panic!("Timeout"),
        }
    }
}
