use super::ssh_hosts::SshHostInfo;
use eyre::Result;
use ssh2::Session;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum SshStatus {
    Connected,
    Failed,
    Loading,
}

#[derive(Debug, Clone)]
pub struct SshHostState {
    pub info: SshHostInfo,
    pub status: SshStatus,
}

/// Test SSH connection using ssh-agent
pub fn test_ssh_connection(info: &SshHostInfo) -> Result<SshStatus> {
    let addr = format!("{}:{}", info.ip, info.port);
    let tcp = TcpStream::connect(&addr)
        .map_err(|e| eyre::eyre!("Failed to connect to {}: {}", addr, e))?;
    tcp.set_read_timeout(Some(Duration::from_secs(5)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;

    // Use ssh-agent for authentication
    let mut agent = session.agent()?;
    agent.connect()?;
    agent.list_identities()?;

    for identity in agent.identities()? {
        if agent.userauth(info.user.as_str(), &identity).is_ok() && session.authenticated() {
            return Ok(SshStatus::Connected);
        }
    }

    Ok(SshStatus::Failed)
}
