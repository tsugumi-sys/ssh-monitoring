use eyre::Result;
use ssh2_config::{Host, ParseRule, SshConfig};
use std::{fs::File, io::BufReader};

pub const PLACEHOLDER_IP: &str = "-";
pub const PLACEHOLDER_USER: &str = "-";
pub const PLACEHOLDER_PORT: u16 = 22;

/// Core data model: parsed from SSH config
#[derive(Debug, Clone)]
pub struct SshHostInfo {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub user: String,
}

impl SshHostInfo {
    // pub fn is_placeholder_ip(&self) -> bool {
    //     self.ip == PLACEHOLDER_IP
    // }

    // pub fn is_placeholder_user(&self) -> bool {
    //     self.user == PLACEHOLDER_USER
    // }

    // pub fn is_placeholder_port(&self) -> bool {
    //     self.port == PLACEHOLDER_PORT
    // }
}

/// Load and parse SSH config (~/.ssh/config) into a list of host entries
pub fn load_ssh_configs() -> Result<Vec<SshHostInfo>> {
    let path = dirs::home_dir()
        .ok_or_else(|| eyre::eyre!("Could not resolve home dir"))?
        .join(".ssh/config");

    let mut reader = BufReader::new(File::open(path)?);
    let config = SshConfig::default().parse(&mut reader, ParseRule::STRICT)?;

    let hosts = config
        .get_hosts()
        .iter()
        .filter_map(|host: &Host| {
            let name = host.pattern.first()?.to_string();
            let ip = host.params.host_name.clone();
            let user = host.params.user.clone();

            if ip.is_none() && user.is_none() {
                return None; // Skip useless entries
            }
            Some(SshHostInfo {
                name,
                ip: ip.unwrap_or_else(|| PLACEHOLDER_IP.into()),
                port: host.params.port.unwrap_or(PLACEHOLDER_PORT),
                user: user.unwrap_or_else(|| PLACEHOLDER_USER.into()),
            })
        })
        .collect();

    Ok(hosts)
}
