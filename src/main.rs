pub mod config;
pub mod daemon;
pub mod iconlookup;
pub mod ipc;
pub mod notification;
pub mod render;
pub mod tray;

use crate::ipc::{Command, IpcClient, IpcError, Response};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arg {
    /// Enable debug logging
    #[arg(short = 'd', long = "debug", global = true)]
    debug: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start the daemon
    Daemon {
        /// Use a custom config file
        #[arg(short = 'c', long = "config")]
        config: Option<PathBuf>,
    },
    /// Update a variable: iwwc update <name> <value>
    Update { name: String, value: String },
    /// Read a variable: iwwc get <name>
    Get { name: String },
    /// Open a window: iwwc open <window>
    Open { window: String },
    /// Close a window: iwwc close <window>
    Close { window: String },
    /// Toggle a window: iwwc toggle <window>
    Toggle { window: String },
    /// Reload the daemon's config: iwwc reload
    Reload,
}

pub fn main() {
    let cli = Arg::parse();
    init_logger(cli.debug);
    match cli.cmd {
        Cmd::Daemon { config } => run_daemon(config),
        Cmd::Update { name, value } => client_dispatch(Command::Update { name, value }),
        Cmd::Get { name } => client_dispatch(Command::Get { name }),
        Cmd::Open { window } => client_dispatch(Command::Open { window }),
        Cmd::Close { window } => client_dispatch(Command::Close { window }),
        Cmd::Toggle { window } => client_dispatch(Command::Toggle { window }),
        Cmd::Reload => client_dispatch(Command::Reload),
    }
}

fn init_logger(debug: bool) {
    //let level = if debug { "debug" } else { "warn" };
    let level = if debug {
        "debug, sctk=off, iced_layershell=off, naga=off, zbus=off, iced_wgpu=off, cosmic_text=off, wgpu_core=off, wgpu_hal=off"
    } else {
        "info, sctk=off, iced_layershell=off, naga=off, zbus=off, iced_wgpu=off, cosmic_text=off, wgpu_core=off, wgpu_hal=off"
    };
    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", level)).init();
    log::debug!("Logger initialized");
}

fn client_dispatch(command: Command) {
    let to_stdout = matches!(command, Command::Get { .. });
    let result = tokio::runtime::Runtime::new()
        .expect("create tokio runtime")
        .block_on(IpcClient::send(&command));
    match result {
        Ok(Response::Ok) => {}
        Ok(Response::Note(msg)) => {
            if to_stdout {
                println!("{msg}");
            } else {
                eprintln!("{msg}");
            }
        }
        Ok(Response::Error(msg)) => {
            eprintln!("{msg}");
            std::process::exit(1);
        }
        Err(IpcError::NotRunning) => {
            eprintln!("error: daemon is not running");
            std::process::exit(1);
        }
        Err(IpcError::Io(e)) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn run_daemon(config: Option<PathBuf>) {
    use crate::config::store::Store;
    use crate::config::{self, LoadError};

    let path = match config {
        Some(custom) => custom,
        None => match config::discover_path() {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("error: {}", msg);
                std::process::exit(1);
            }
        },
    };

    let store = match config::load_from_path(&path) {
        Ok(ok) => {
            for w in &ok.warnings {
                eprintln!("{}", w);
            }
            match Store::new(ok.config) {
                Ok(store) => {
                    for w in store.warnings() {
                        eprintln!("{}", w);
                    }
                    store
                }
                Err(errs) => {
                    for e in &errs {
                        eprintln!("{}", e);
                    }
                    std::process::exit(1);
                }
            }
        }
        Err(LoadError::Semantic(msgs)) => {
            for m in &msgs {
                eprintln!("{}", m);
            }
            std::process::exit(1);
        }
        Err(LoadError::Io(e, p)) => {
            eprintln!("error: cannot read {}: {}", p.display(), e);
            std::process::exit(1);
        }
        Err(LoadError::PathDiscovery(msg)) => {
            eprintln!("error: {}", msg);
            std::process::exit(1);
        }
        Err(LoadError::Syntax(e)) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let surface_errs = store.validate_surfaces();
    if !surface_errs.is_empty() {
        for e in &surface_errs {
            eprintln!("{}", e);
        }
        std::process::exit(1);
    }

    let socket = crate::ipc::socket_path();
    match std::os::unix::net::UnixStream::connect(&socket) {
        Ok(_) => {
            eprintln!("error: daemon is already running");
            std::process::exit(1);
        }
        Err(_) => {
            if socket.exists() {
                let _ = std::fs::remove_file(&socket);
            }
        }
    }

    let config_dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    crate::notification::icons::ensure_default_svg(&config_dir);

    if let Err(e) = crate::daemon::run(store, path) {
        eprintln!("daemon error: {e}");
        std::process::exit(1);
    }
}
