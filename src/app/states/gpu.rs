use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::{connect_ssh_session, run_command};
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
    Fallback(String), // For basic text-only fallback
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

    pub fn fallback(info: impl Into<String>) -> Self {
        GpuInfo::Fallback(info.into())
    }

    pub fn failure(msg: impl Into<String>) -> Self {
        GpuInfo::Failure(msg.into())
    }
}

pub type SharedGpuInfo = Arc<Mutex<HashMap<String, GpuInfo>>>;

pub fn fetch_gpu_info(info: &SshHostInfo) -> GpuInfo {
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

            // Fallback to basic GPU info
            let lspci_cmd = r#"lspci | grep -i 'vga\|3d'"#;
            let output = match run_command(&session, lspci_cmd) {
                Ok(out) => out.trim().to_string(),
                Err(e) => return GpuInfo::failure(e),
            };

            if output.is_empty() {
                return GpuInfo::failure("No GPU info found with lspci");
            }

            GpuInfo::fallback(output)
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

            let lines: Vec<_> = output.lines().map(|line| line.trim().to_string()).collect();

            GpuInfo::fallback(lines.join("; "))
        }

        other => GpuInfo::failure(format!("Unsupported platform: {}", other)),
    }
}
