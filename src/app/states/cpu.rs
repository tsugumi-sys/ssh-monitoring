use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::{connect_ssh_session, run_command};
use super::ssh_limits::SSH_LIMITER;
use std::collections::HashMap;
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
    let (_permit, _guard) = SSH_LIMITER.acquire(&info.id);

    let session = match connect_ssh_session(info) {
        Ok(s) => s,
        Err(e) => return CpuInfo::failure(e),
    };

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
