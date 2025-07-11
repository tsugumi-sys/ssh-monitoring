use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::{connect_ssh_session, run_command};
use super::ssh_limits::SSH_LIMITER;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum GpuInfo {
    Loading,
    Success {
        name: String,
        memory_total_mb: u32,
        memory_used_mb: u32,
        utilization_percent: u8,
        temperature_c: u8,
    },
    Failure(String),
}

impl GpuInfo {
    pub fn success(
        name: String,
        memory_total_mb: u32,
        memory_used_mb: u32,
        utilization_percent: u8,
        temperature_c: u8,
    ) -> Self {
        GpuInfo::Success {
            name,
            memory_total_mb,
            memory_used_mb,
            utilization_percent,
            temperature_c,
        }
    }

    pub fn failure(msg: impl Into<String>) -> Self {
        GpuInfo::Failure(msg.into())
    }
}

pub type SharedGpuInfo = Arc<Mutex<HashMap<String, GpuInfo>>>;

pub fn fetch_gpu_info(info: &SshHostInfo) -> GpuInfo {
    let (_permit, _guard) = SSH_LIMITER.acquire(&info.id);

    let session = match connect_ssh_session(info) {
        Ok(s) => s,
        Err(e) => return GpuInfo::failure(e),
    };

    let uname_cmd = "uname -s";
    let platform = match run_command(&session, uname_cmd) {
        Ok(out) => out.trim().to_string(),
        Err(e) => return GpuInfo::failure(format!("Failed to detect platform: {}", e)),
    };

    match platform.as_str() {
        "Linux" => {
            let nvidia_cmd = concat!(
                "nvidia-smi --query-gpu=name,memory.total,memory.used,utilization.gpu,temperature.gpu ",
                "--format=csv,noheader,nounits"
            );

            if let Ok(out) = run_command(&session, nvidia_cmd) {
                if let Some(line) = out.lines().next() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() == 5 {
                        let name = parts[0].to_string();
                        let memory_total_mb = parts[1].parse().unwrap_or(0);
                        let memory_used_mb = parts[2].parse().unwrap_or(0);
                        let utilization_percent = parts[3].parse().unwrap_or(0);
                        let temperature_c = parts[4].parse().unwrap_or(0);

                        return GpuInfo::success(
                            name,
                            memory_total_mb,
                            memory_used_mb,
                            utilization_percent,
                            temperature_c,
                        );
                    }
                }
            }

            // nvidia-smi not found or failed to parse; treat as unavailable
            GpuInfo::failure("nvidia-smi not available")
        }

        "Darwin" => {
            let sp_cmd = r#"system_profiler SPDisplaysDataType | grep -E 'Chipset Model|VRAM'"#;
            let output = match run_command(&session, sp_cmd) {
                Ok(out) => out.trim().to_string(),
                Err(e) => return GpuInfo::failure(e),
            };

            if output.is_empty() {
                return GpuInfo::failure("No GPU info found with system_profiler");
            }

            let mut name = None;
            let mut memory_total_mb = 0;
            for line in output.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("Chipset Model:") {
                    name = Some(rest.trim().to_string());
                } else if line.contains("VRAM") {
                    if let Some(value) = line.split(':').nth(1) {
                        let value = value.trim();
                        let mut parts = value.split_whitespace();
                        if let Some(num_str) = parts.next() {
                            if let Ok(mut num) = num_str.replace(',', "").parse::<u32>() {
                                if let Some(unit) = parts.next() {
                                    if unit.eq_ignore_ascii_case("GB") {
                                        num *= 1024;
                                    }
                                }
                                memory_total_mb = num;
                            }
                        }
                    }
                }
            }

            let name = name.unwrap_or_else(|| "Unknown".to_string());

            GpuInfo::success(name, memory_total_mb, 0, 0, 0)
        }

        other => GpuInfo::failure(format!("Unsupported platform: {}", other)),
    }
}
