use std::process::Stdio;
use tokio::process::Command;

pub async fn run_system_command(command: &str) {
    if let Err(e) = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        log::error!("Failed to execute {command}");
        log::error!("Error: {e}");
    }
}
