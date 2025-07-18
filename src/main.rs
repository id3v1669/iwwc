use clap::Parser;

mod data;
mod gui;
mod handler;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set a custom config file
    #[arg(short = 'c', long = "config", value_name = "Path")]
    config: Option<std::path::PathBuf>,

    /// Enable Debug Mode
    #[arg(short = 'd', long = "debug")]
    debug: bool,

    #[arg(value_name = "COMMAND")]
    command: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    unsafe {
        if args.debug {
            std::env::set_var("RUST_LOG", "debug");
        }
    }

    env_logger::init();
    log::debug!("Logger initialized");

    if let Some(command) = args.command {
        match command.as_str() {
            "test" => crate::handler::ipc::IpcServer::send_ipc_command("test").await?,
            "daemon" => {
                if crate::handler::ipc::IpcServer::is_active().await {
                    log::error!("Daemon is already running.");
                    std::process::exit(1);
                }
                crate::data::icons::get_system_icons_paths();
                crate::gui::app::start().expect("REASON");
            }
            _ => {
                log::error!("Unknown command: {command}");
                std::process::exit(1);
            }
        }
        return Ok(());
    } else {
        println!("help to be printed for stdinput");
    }

    Ok(())
}
