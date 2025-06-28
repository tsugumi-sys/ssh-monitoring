use super::ssh_hosts::SshHostInfo;
use ssh2::Session;
use std::net::TcpStream;
use std::path::PathBuf;

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

pub fn verify_connection(info: &SshHostInfo) -> SshStatus {
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
    use tokio::{
        task,
        time::{Duration, timeout},
    };

    #[tokio::test]
    async fn test_connection_to_public_ssh_should_fail() {
        let info = SshHostInfo {
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
