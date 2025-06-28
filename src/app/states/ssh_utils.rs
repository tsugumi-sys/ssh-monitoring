use ssh2::Session;
use std::io::Read;

pub fn run_command(session: &Session, command: &str) -> Result<String, String> {
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
