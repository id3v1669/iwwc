use std::io::Write;
use zbus::{connection, interface, zvariant::Value};

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
    env_logger::init();
    log::debug!("Logger initialized");

    if gtk::init().is_err() {
        log::warn!("Failed to initialize GTK, icons would not be used.");
        let mut gtk_active = crate::data::shared_data::GTK_ACTIVE.lock().unwrap();
        *gtk_active = false;
    }

    if args.daemon {
        tokio::spawn(async move {
            crate::notification::app::gen_ui()
                .await
                .unwrap();
        });
    }

    println!("After daemon launch");
    std::future::pending::<()>().await;
    Ok(())
}
