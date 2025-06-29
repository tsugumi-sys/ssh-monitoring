use super::ssh_hosts::SshHostInfo;
use super::ssh_utils::{connect_ssh_session, run_command};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum OsInfo {
    Loading,
    Success {
        name: String,
        version: String,
        timezone: String,
    },
    Failure(String),
}

impl OsInfo {
    pub fn success(name: String, version: String, timezone: String) -> Self {
        OsInfo::Success {
            name,
            version,
            timezone,
        }
    }

    pub fn failure(msg: impl Into<String>) -> Self {
        OsInfo::Failure(msg.into())
    }
}

pub type SharedOsInfo = Arc<Mutex<HashMap<String, OsInfo>>>;

pub fn fetch_os_info(info: &SshHostInfo) -> OsInfo {
    let session = match connect_ssh_session(info) {
        Ok(s) => s,
        Err(e) => return OsInfo::failure(e),
    };

    // First, try to detect platform with uname
    let uname_cmd = "uname -s";
    let platform = match run_command(&session, uname_cmd) {
        Ok(out) => out.trim().to_string(),
        Err(e) => return OsInfo::failure(format!("Failed to detect platform: {}", e)),
    };

    match platform.as_str() {
        "Linux" => {
            let os_cmd =
                r#"awk -F= '/^NAME=|^VERSION_ID=/{gsub(/"/, "", $2); print $2}' /etc/os-release"#;
            let output = match run_command(&session, os_cmd) {
                Ok(out) => out,
                Err(e) => return OsInfo::failure(e),
            };

            let mut lines = output.lines();
            let name = lines.next().unwrap_or("").trim();
            let version = lines.next().unwrap_or("").trim();

            if name.is_empty() || version.is_empty() {
                return OsInfo::failure(format!("Unexpected Linux os-release output: {}", output));
            }

            let tz_cmd = "cat /etc/timezone || timedatectl | grep 'Time zone' | awk '{print $3}'";
            let timezone = match run_command(&session, tz_cmd) {
                Ok(out) => out.lines().next().unwrap_or("Unknown").trim().to_string(),
                Err(_) => "Unknown".to_string(),
            };

            OsInfo::success(name.to_string(), version.to_string(), timezone)
        }

        "Darwin" => {
            let os_cmd = r#"sw_vers -productName && sw_vers -productVersion"#;
            let output = match run_command(&session, os_cmd) {
                Ok(out) => out,
                Err(e) => return OsInfo::failure(e),
            };

            let mut lines = output.lines();
            let name = lines.next().unwrap_or("").trim();
            let version = lines.next().unwrap_or("").trim();

            if name.is_empty() || version.is_empty() {
                return OsInfo::failure(format!("Unexpected macOS sw_vers output: {}", output));
            }

            let tz_cmd = "readlink /etc/localtime";
            let timezone = match run_command(&session, tz_cmd) {
                Ok(out) => {
                    // Example: "/var/db/timezone/zoneinfo/Asia/Tokyo"
                    out.trim()
                        .split("zoneinfo/")
                        .nth(1)
                        .unwrap_or("Unknown")
                        .to_string()
                }
                Err(_) => "Unknown".to_string(),
            };

            OsInfo::success(name.to_string(), version.to_string(), timezone)
        }

        other => OsInfo::failure(format!("Unsupported platform: {}", other)),
    }
}
