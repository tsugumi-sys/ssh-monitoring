use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::run_command;
use ssh2::Session;
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum CpuInfo {
    Loading,
    Success {
        core_count: usize,
        usage_percent: f32,
    },
    Failure(String),
}

pub type SharedCpuInfo = Arc<Mutex<HashMap<String, CpuInfo>>>;

impl CpuInfo {
    pub fn success(core_count: usize, usage_percent: f32) -> Self {
        CpuInfo::Success {
            core_count,
            usage_percent,
        }
    }

    pub fn failure(msg: impl Into<String>) -> Self {
        CpuInfo::Failure(msg.into())
    }
}

pub fn fetch_cpu_info(info: &SshHostInfo) -> CpuInfo {
    let addr = format!("{}:{}", info.ip, info.port);
    let tcp = match TcpStream::connect(&addr) {
        Ok(t) => t,
        Err(e) => return CpuInfo::failure(format!("TCP error: {}", e)),
    };

    let mut session = match Session::new() {
        Ok(s) => s,
        Err(e) => return CpuInfo::failure(format!("Session error: {}", e)),
    };

    session.set_tcp_stream(tcp);
    if let Err(e) = session.handshake() {
        return CpuInfo::failure(format!("Handshake error: {}", e));
    }

    let identity_path = PathBuf::from(&info.identity_file);
    if !identity_path.exists() {
        return CpuInfo::failure(format!(
            "Identity file not found: {}",
            identity_path.display()
        ));
    }

    let mut agent = match session.agent() {
        Ok(a) => a,
        Err(e) => return CpuInfo::failure(format!("Agent error: {}", e)),
    };

    if let Err(e) = agent.connect() {
        return CpuInfo::failure(format!("Agent connect error: {}", e));
    }

    if let Err(e) = agent.list_identities() {
        return CpuInfo::failure(format!("Agent list error: {}", e));
    }

    let mut authenticated = false;
    for identity in agent.identities().unwrap_or_default() {
        if agent.userauth(&info.user, &identity).is_ok() && session.authenticated() {
            authenticated = true;
            break;
        }
    }

    if !authenticated {
        return CpuInfo::failure("SSH authentication failed");
    }

    let os_name = run_command(&session, "uname").unwrap_or_default();
    let is_mac = os_name.trim() == "Darwin";
    let cpu_core_cmd = if is_mac { "sysctl -n hw.ncpu" } else { "nproc" };
    let cpu_usage_cmd = "ps -A -o %cpu | awk '{s+=$1} END {print s}'";

    let core_str = match run_command(&session, cpu_core_cmd) {
        Ok(s) => s,
        Err(e) => return CpuInfo::failure(e),
    };
    let usage_str = match run_command(&session, cpu_usage_cmd) {
        Ok(s) => s,
        Err(e) => return CpuInfo::failure(e),
    };

    let core_count = match core_str.trim().parse::<usize>() {
        Ok(n) => n,
        Err(e) => return CpuInfo::failure(format!("Parse core count error: {e}")),
    };
    let usage_percent = match usage_str.trim().parse::<f32>() {
        Ok(n) => n,
        Err(e) => return CpuInfo::failure(format!("Parse usage percent error: {e}")),
    };

    CpuInfo::success(core_count, usage_percent)
}
