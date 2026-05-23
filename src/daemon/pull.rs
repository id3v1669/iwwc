pub async fn run(command: String, default: String) -> String {
    match tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .await
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echo_returns_trimmed_stdout() {
        assert_eq!(run("echo hi".into(), "d".into()).await, "hi");
    }

    #[tokio::test]
    async fn failure_returns_default() {
        assert_eq!(run("exit 3".into(), "d".into()).await, "d");
        assert_eq!(
            run("this_cmd_does_not_exist_zzz 2>/dev/null".into(), "d".into()).await,
            "d"
        );
    }
}
