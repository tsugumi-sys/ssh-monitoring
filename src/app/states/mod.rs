pub mod cpu;
pub mod ssh_hosts;
pub mod ssh_status;

pub use cpu::{CpuInfo, fetch_cpu_info};
pub use ssh_hosts::load_ssh_configs;
pub use ssh_status::{SharedSshHosts, SshHostState, SshStatus, verify_connection};
use std::collections::HashMap;

pub fn load_ssh_host_states() -> HashMap<String, SshHostState> {
    match load_ssh_configs() {
        Ok(hosts) => hosts
            .into_iter()
            .map(|info| {
                let id = info.id.clone();
                let state = SshHostState {
                    info,
                    status: SshStatus::Loading,
                };
                (id, state)
            })
            .collect(),
        Err(err) => {
            eprintln!("Failed to load SSH config: {err}");
            HashMap::new()
        }
    }
}
