pub fn run_action(action: &str) {
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(action)
        .spawn()
    {
        Ok(_child) => {}
        Err(e) => log::warn!("failed to run action `{action}`: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_true_succeeds() {
        run_action("true");
    }

    #[test]
    fn run_garbage_does_not_panic() {
        run_action("this_command_definitely_does_not_exist_12345");
    }
}
