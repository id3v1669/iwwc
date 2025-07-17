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
}

// main func that calls daemon::launch func if daemon flag is set
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

    crate::data::icons::get_system_icons_paths();
    crate::gui::app::start().expect("REASON");

    Ok(())
}
