use std::{fs::File, io::BufReader};

use eyre::Result;
use ssh2_config::{Host, ParseRule, SshConfig};

pub fn load_ssh2_config_hosts() -> Result<Vec<(String, Option<String>, Option<u16>, Option<String>)>>
{
    let path = dirs::home_dir()
        .ok_or_else(|| eyre::eyre!("Could not resolve home dir"))?
        .join(".ssh/config");

    let mut reader = BufReader::new(File::open(path)?);
    let config = SshConfig::default().parse(&mut reader, ParseRule::STRICT)?;

    let hosts = config
        .get_hosts()
        .iter()
        .filter_map(|host: &Host| {
            let name = host.pattern.get(0)?.to_string();
            let ip = host.params.host_name.clone();
            let port = host.params.port;
            let user = host.params.user.clone();
            Some((name, ip, port, user))
        })
        .collect();

    Ok(hosts)
}
