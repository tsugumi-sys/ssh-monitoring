use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::run_command;
use ssh2::Session;
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum DiskInfo {
    Loading,
    Success {
        total: String,
        used: String,
        avail: String,
        usage_percent: String,
    },
    Failure(String),
}

pub type SharedDiskInfo = Arc<Mutex<HashMap<String, DiskInfo>>>;

impl DiskInfo {
    pub fn success(total: String, used: String, avail: String, usage_percent: String) -> Self {
        DiskInfo::Success {
            total,
            used,
            avail,
            usage_percent,
        }
    }

    pub fn failure(msg: impl Into<String>) -> Self {
        DiskInfo::Failure(msg.into())
    }
}

pub fn fetch_disk_info(info: &SshHostInfo) -> DiskInfo {
    let addr = format!("{}:{}", info.ip, info.port);
    let tcp = match TcpStream::connect(&addr) {
        Ok(t) => t,
        Err(e) => return DiskInfo::failure(format!("TCP error: {}", e)),
    };

    let mut session = match Session::new() {
        Ok(s) => s,
        Err(e) => return DiskInfo::failure(format!("Session error: {}", e)),
    };

    session.set_tcp_stream(tcp);
    if let Err(e) = session.handshake() {
        return DiskInfo::failure(format!("Handshake error: {}", e));
    }

    let identity_path = PathBuf::from(&info.identity_file);
    if !identity_path.exists() {
        return DiskInfo::failure(format!(
            "Identity file not found: {}",
            identity_path.display()
        ));
    }

    let mut agent = match session.agent() {
        Ok(a) => a,
        Err(e) => return DiskInfo::failure(format!("Agent error: {}", e)),
    };

    if let Err(e) = agent.connect() {
        return DiskInfo::failure(format!("Agent connect error: {}", e));
    }

    if let Err(e) = agent.list_identities() {
        return DiskInfo::failure(format!("Agent list error: {}", e));
    }

    let mut authenticated = false;
    for identity in agent.identities().unwrap_or_default() {
        if agent.userauth(&info.user, &identity).is_ok() && session.authenticated() {
            authenticated = true;
            break;
        }
    }

    if !authenticated {
        return DiskInfo::failure("SSH authentication failed");
    }

    let disk_cmd = "df -h / | awk 'NR==2 {print $2, $3, $4, $5}'";
    let output = match run_command(&session, disk_cmd) {
        Ok(out) => out,
        Err(e) => return DiskInfo::failure(e),
    };

    let parts: Vec<&str> = output.split_whitespace().collect();
    if parts.len() < 4 {
        return DiskInfo::failure(format!("Unexpected df output: {}", output));
    }

    DiskInfo::success(
        parts[0].to_string(), // Total
        parts[1].to_string(), // Used
        parts[2].to_string(), // Available
        parts[3].to_string(), // Usage %
    )
}
