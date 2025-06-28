pub mod ssh_hosts;
pub mod ssh_status;

pub use ssh_hosts::load_ssh_configs;
pub use ssh_status::{SshHostState, SshStatus, update_ssh_status};

pub fn load_ssh_host_states() -> Vec<SshHostState> {
    match load_ssh_configs() {
        Ok(hosts) => hosts
            .into_iter()
            .map(|info| SshHostState {
                info,
                status: SshStatus::Loading,
            })
            .collect(),
        Err(err) => {
            eprintln!("Failed to load SSH config: {err}");
            vec![]
        }
    }
}
