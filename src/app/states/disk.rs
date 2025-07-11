use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::{connect_ssh_session, run_command};
use super::ssh_limits::SSH_LIMITER;
use std::collections::HashMap;
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
    let (_permit, _guard) = SSH_LIMITER.acquire(&info.id);

    let session = match connect_ssh_session(info) {
        Ok(s) => s,
        Err(e) => return DiskInfo::failure(e),
    };

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
