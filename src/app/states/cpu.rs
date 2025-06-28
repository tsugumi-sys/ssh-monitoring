use super::ssh_hosts::SshHostInfo;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub core_count: usize,
    pub usage_percent: f32,
}

pub fn fetch_cpu_info(info: &SshHostInfo) -> Result<CpuInfo, String> {
    let addr = format!("{}:{}", info.ip, info.port);
    let tcp = TcpStream::connect(&addr).map_err(|e| format!("TCP error: {}", e))?;

    let mut session = Session::new().map_err(|e| format!("Session error: {}", e))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|e| format!("Handshake error: {}", e))?;

    let identity_path = PathBuf::from(&info.identity_file);
    if !identity_path.exists() {
        return Err(format!(
            "Identity file not found: {}",
            identity_path.display()
        ));
    }

    let mut agent = session.agent().map_err(|e| format!("Agent error: {}", e))?;
    agent
        .connect()
        .map_err(|e| format!("Agent connect error: {}", e))?;
    agent
        .list_identities()
        .map_err(|e| format!("Agent list error: {}", e))?;

    let mut authenticated = false;
    for identity in agent.identities().unwrap_or_default() {
        if agent.userauth(&info.user, &identity).is_ok() && session.authenticated() {
            authenticated = true;
            break;
        }
    }

    if !authenticated {
        return Err("SSH authentication failed".into());
    }

    // Detect platform
    let os_name = run_command(&session, "uname").unwrap_or_default();
    let is_mac = os_name.trim() == "Darwin";

    let cpu_core_cmd = if is_mac { "sysctl -n hw.ncpu" } else { "nproc" };
    let cpu_usage_cmd = "ps -A -o %cpu | awk '{s+=$1} END {print s}'";

    let core_str = run_command(&session, cpu_core_cmd)?;
    let usage_str = run_command(&session, cpu_usage_cmd)?;

    let core_count = core_str
        .trim()
        .parse::<usize>()
        .map_err(|e| format!("Parse error: {e}"))?;
    let usage_percent = usage_str
        .trim()
        .parse::<f32>()
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(CpuInfo {
        core_count,
        usage_percent,
    })
}

fn run_command(session: &Session, command: &str) -> Result<String, String> {
    let mut channel = session
        .channel_session()
        .map_err(|e| format!("Channel error: {}", e))?;
    channel
        .exec(command)
        .map_err(|e| format!("Exec error: {}", e))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| format!("Read error: {}", e))?;
    channel
        .wait_close()
        .map_err(|e| format!("Wait close error: {}", e))?;

    Ok(output)
}

#[tokio::test]
async fn test_fetch_cpu_info_should_fail() {
    use tokio::time::Duration;
    let info = SshHostInfo {
        name: "invalid_host".into(),
        ip: "test.rebex.net".into(),
        port: 22,
        user: "demo".into(),
        identity_file: "/dev/null".into(),
    };

    let result = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::task::spawn_blocking(move || fetch_cpu_info(&info)),
    )
    .await;

    match result {
        Ok(Ok(Ok(cpu_info))) => println!("CPU Info: {:?}", cpu_info),
        Ok(Ok(Err(e))) => println!("Expected failure: {}", e),
        Ok(Err(e)) => panic!("Join error: {}", e),
        Err(_) => panic!("Timeout"),
    }
}
