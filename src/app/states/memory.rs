use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::{connect_ssh_session, run_command};
use super::ssh_limits::SSH_LIMITER;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum MemoryInfo {
    Loading,
    Success {
        total: String,
        used: String,
        usage_percent: String,
    },
    Failure(String),
}

pub type SharedMemoryInfo = Arc<Mutex<HashMap<String, MemoryInfo>>>;

impl MemoryInfo {
    pub fn success(total: String, used: String, usage_percent: String) -> Self {
        MemoryInfo::Success {
            total,
            used,
            usage_percent,
        }
    }

    pub fn failure(msg: impl Into<String>) -> Self {
        MemoryInfo::Failure(msg.into())
    }
}

pub fn fetch_memory_info(info: &SshHostInfo) -> MemoryInfo {
    let (_permit, _guard) = SSH_LIMITER.acquire(&info.id);

    let session = match connect_ssh_session(info) {
        Ok(s) => s,
        Err(e) => return MemoryInfo::failure(e),
    };

    let uname_cmd = "uname -s";
    let platform = match run_command(&session, uname_cmd) {
        Ok(out) => out.trim().to_string(),
        Err(e) => return MemoryInfo::failure(format!("Failed to detect platform: {}", e)),
    };

    match platform.as_str() {
        "Linux" => {
            let mem_cmd = "free -m | awk '/Mem:/ {print $2, $3}'";
            let output = match run_command(&session, mem_cmd) {
                Ok(out) => out,
                Err(e) => return MemoryInfo::failure(e),
            };
            let parts: Vec<&str> = output.split_whitespace().collect();
            if parts.len() < 2 {
                return MemoryInfo::failure(format!("Unexpected free output: {}", output));
            }
            let total_mb = parts[0].parse::<u64>().unwrap_or(0);
            let used_mb = parts[1].parse::<u64>().unwrap_or(0);
            let percent = if total_mb > 0 {
                (used_mb as f32 / total_mb as f32) * 100.0
            } else {
                0.0
            };
            MemoryInfo::success(
                format!("{} MB", total_mb),
                format!("{} MB", used_mb),
                format!("{:.1}%", percent),
            )
        }
        "Darwin" => {
            let total_cmd = "sysctl -n hw.memsize";
            let vm_cmd = "vm_stat";

            // ────────────────────────────
            // 💾 Total memory (bytes)
            // ────────────────────────────
            let total_str = match run_command(&session, total_cmd) {
                Ok(out) => out.trim().to_string(),
                Err(e) => return MemoryInfo::failure(e),
            };
            let total_bytes = total_str.parse::<u64>().unwrap_or(0);
            let total_mb = total_bytes / 1024 / 1024;

            // ────────────────────────────
            // 📊 Parse vm_stat output
            // ────────────────────────────
            let vm_output = match run_command(&session, vm_cmd) {
                Ok(out) => out,
                Err(e) => return MemoryInfo::failure(e),
            };

            let mut page_size = 4096u64;
            let mut pages_active = 0u64;
            let mut pages_speculative = 0u64;
            let mut pages_compressed = 0u64;
            let mut pages_wired = 0u64;
            let mut file_cache_pages = 0u64;

            use std::collections::HashMap;

            let mut counters: HashMap<&str, &mut u64> = HashMap::from([
                ("Pages active", &mut pages_active),
                ("Pages speculative", &mut pages_speculative),
                ("Pages occupied by compressor", &mut pages_compressed),
                ("Pages wired down", &mut pages_wired),
                ("File-backed pages", &mut file_cache_pages),
            ]);

            for line in vm_output.lines() {
                if line.contains("page size of") {
                    if let Some(num) = line.split("page size of").nth(1) {
                        if let Some(v) = num.split_whitespace().next() {
                            page_size = v.parse::<u64>().unwrap_or(4096);
                        }
                    }
                } else if let Some((key, value)) = line.split_once(':') {
                    if let Some(counter) = counters.get_mut(key.trim()) {
                        let count = value
                            .trim()
                            .trim_end_matches('.')
                            .replace(".", "")
                            .parse::<u64>()
                            .unwrap_or(0);
                        **counter = count;
                    }
                }
            }

            // ────────────────────────────
            // 🧮 Calculate used memory
            // ────────────────────────────
            let used_pages = pages_active
                + pages_speculative
                + pages_wired
                + pages_compressed
                + file_cache_pages;
            let used_mb = (used_pages * page_size) / 1024 / 1024;

            let percent = if total_mb > 0 {
                (used_mb as f32 / total_mb as f32) * 100.0
            } else {
                0.0
            };

            MemoryInfo::success(
                format!("{} MB", total_mb),
                format!("{} MB", used_mb),
                format!("{:.1}%", percent),
            )
        }
        other => MemoryInfo::failure(format!("Unsupported platform: {}", other)),
    }
}
