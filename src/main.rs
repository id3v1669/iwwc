use clap::Parser;

mod data;
mod notification;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set a custom config file
    #[arg(short = 'c', value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    /// Launch daemon
    #[arg(short = 'D')]
    daemon: bool,

    /// Nvidia moment flag
    #[arg(short = 'n')]
    nvidia: bool,

    /// Enable Debug Mode
    #[arg(short, long)]
    debug: bool,
}

// main func that calls daemon::launch func if daemon flag is set
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    std::env::set_var("RUST_LOG", "warn");
    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
    }
    if args.nvidia {
        let mut nvidia_sucks = data::shared_data::NVIDIA_SUCKS.lock().unwrap();
        *nvidia_sucks = true;
        std::env::set_var("WGPU_BACKEND", "gl");
    }
    env_logger::init();
    log::debug!("Logger initialized");

    if args.daemon {
        crate::data::icons::get_system_icons_paths();
        crate::notification::app::gen_ui().expect("REASON");
    }

    Ok(())
}
