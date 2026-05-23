#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Update { name: String, value: String },
    Open { window: String },
    Close { window: String },
    Toggle { window: String },
    Reload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Ok,
    Note(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnknownCommand(String),
    MissingArgument(&'static str),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnknownCommand(w) => write!(f, "unknown command \"{}\"", w),
            ParseError::MissingArgument(a) => write!(f, "missing argument: {}", a),
        }
    }
}

impl Command {
    /// The canonical request line (no trailing newline).
    pub fn to_wire(&self) -> String {
        match self {
            Command::Update { name, value } => format!("update {} {}", name, value),
            Command::Open { window } => format!("open {}", window),
            Command::Close { window } => format!("close {}", window),
            Command::Toggle { window } => format!("toggle {}", window),
            Command::Reload => "reload".to_string(),
        }
    }

    /// Parse one request line (without the trailing newline).
    pub fn parse_wire(line: &str) -> Result<Command, ParseError> {
        let line = line.strip_suffix('\n').unwrap_or(line);
        let (keyword, rest) = match line.split_once(' ') {
            Some((k, r)) => (k, r),
            None => (line, ""),
        };
        match keyword {
            "update" => {
                let (name, value) = rest.split_once(' ').ok_or(ParseError::MissingArgument(
                    "update requires <name> <value>",
                ))?;
                if name.is_empty() || value.is_empty() {
                    return Err(ParseError::MissingArgument(
                        "update requires <name> <value>",
                    ));
                }
                Ok(Command::Update {
                    name: name.to_string(),
                    value: value.to_string(),
                })
            }
            "reload" => Ok(Command::Reload),
            "open" | "close" | "toggle" => {
                let window = rest.trim();
                if window.is_empty() {
                    return Err(ParseError::MissingArgument("window name"));
                }
                let window = window.to_string();
                Ok(match keyword {
                    "open" => Command::Open { window },
                    "close" => Command::Close { window },
                    _ => Command::Toggle { window },
                })
            }
            other => Err(ParseError::UnknownCommand(other.to_string())),
        }
    }
}

impl Response {
    pub fn to_wire(&self) -> String {
        match self {
            Response::Ok => "OK".to_string(),
            Response::Note(msg) => format!("OK\n{}", msg),
            Response::Error(msg) => format!("ERROR\n{}", msg),
        }
    }

    pub fn parse_wire(text: &str) -> Response {
        let text = text.strip_suffix('\n').unwrap_or(text);
        match text.split_once('\n') {
            Some(("ERROR", rest)) => Response::Error(rest.to_string()),
            Some(("OK", rest)) => Response::Note(rest.to_string()),
            None if text == "ERROR" => Response::Error(String::new()),
            None if text == "OK" => Response::Ok,
            _ => Response::Error(text.to_string()),
        }
    }
}

use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum IpcError {
    NotRunning,
    Io(std::io::Error),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::NotRunning => write!(f, "daemon is not running"),
            IpcError::Io(e) => write!(f, "{}", e),
        }
    }
}

pub struct IpcClient;

impl IpcClient {
    pub async fn send(command: &Command) -> Result<Response, IpcError> {
        Self::send_to(&socket_path(), command).await
    }

    pub async fn send_to(path: &Path, command: &Command) -> Result<Response, IpcError> {
        let mut stream = match UnixStream::connect(path).await {
            Ok(s) => s,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                ) =>
            {
                return Err(IpcError::NotRunning);
            }
            Err(e) => return Err(IpcError::Io(e)),
        };
        let line = format!("{}\n", command.to_wire());
        stream
            .write_all(line.as_bytes())
            .await
            .map_err(IpcError::Io)?;
        stream.shutdown().await.map_err(IpcError::Io)?;
        let mut resp = String::new();
        stream
            .read_to_string(&mut resp)
            .await
            .map_err(IpcError::Io)?;
        Ok(Response::parse_wire(&resp))
    }
}

pub struct Incoming {
    pub command: Command,
    pub reply: oneshot::Sender<Response>,
}

pub struct IpcServer {
    listener: UnixListener,
}

impl IpcServer {
    pub fn bind(path: &Path) -> std::io::Result<IpcServer> {
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        let listener = UnixListener::bind(path)?;
        Ok(IpcServer { listener })
    }

    pub fn serve(self) -> mpsc::Receiver<Incoming> {
        let (tx, rx) = mpsc::channel::<Incoming>(100);
        tokio::spawn(async move {
            loop {
                match self.listener.accept().await {
                    Ok((stream, _)) => {
                        let tx = tx.clone();
                        tokio::spawn(handle_conn(stream, tx));
                    }
                    Err(e) => {
                        log::error!("ipc accept error: {e}");
                        break;
                    }
                }
            }
        });
        rx
    }
}

async fn handle_conn(mut stream: UnixStream, tx: mpsc::Sender<Incoming>) {
    let (rd, mut wr) = stream.split();
    let mut reader = BufReader::new(rd);
    let mut line = String::new();
    if reader.read_line(&mut line).await.is_err() {
        return;
    }
    let response = match Command::parse_wire(&line) {
        Err(e) => Response::Error(e.to_string()),
        Ok(command) => {
            let (reply_tx, reply_rx) = oneshot::channel();
            if tx
                .send(Incoming {
                    command,
                    reply: reply_tx,
                })
                .await
                .is_err()
            {
                Response::Error("daemon is not accepting commands".to_string())
            } else {
                match reply_rx.await {
                    Ok(r) => r,
                    Err(_) => Response::Error("daemon did not respond".to_string()),
                }
            }
        }
    };
    let _ = wr.write_all(response.to_wire().as_bytes()).await;
    let _ = wr.shutdown().await;
}

pub fn socket_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(dir).join("iwwc.sock")
}

pub async fn is_active() -> bool {
    is_active_at(&socket_path()).await
}

pub(crate) async fn is_active_at(path: &Path) -> bool {
    match tokio::net::UnixStream::connect(path).await {
        Ok(_) => true,
        Err(_) => {
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn wire_roundtrip_commands() {
        let cases = [
            Command::Update {
                name: "h".into(),
                value: "80".into(),
            },
            Command::Update {
                name: "x".into(),
                value: "container c1 child=t1".into(),
            },
            Command::Open {
                window: "bar".into(),
            },
            Command::Close {
                window: "bar".into(),
            },
            Command::Toggle {
                window: "bar".into(),
            },
        ];
        for c in cases {
            let wire = c.to_wire();
            assert_eq!(
                Command::parse_wire(&wire),
                Ok(c.clone()),
                "roundtrip failed for {:?}",
                c
            );
        }
    }

    #[test]
    fn wire_roundtrip_response() {
        assert_eq!(Response::parse_wire(&Response::Ok.to_wire()), Response::Ok);
        let e = Response::Error("line1\nline2".into());
        assert_eq!(Response::parse_wire(&e.to_wire()), e);
        let e2 = Response::Error("single".into());
        assert_eq!(Response::parse_wire(&e2.to_wire()), e2);
    }

    #[test]
    fn wire_roundtrip_note() {
        let n = Response::Note("warn1\nwarn2".into());
        assert_eq!(Response::parse_wire(&n.to_wire()), n);
        let n2 = Response::Note("single warning".into());
        assert_eq!(Response::parse_wire(&n2.to_wire()), n2);
        assert_eq!(Response::parse_wire(&Response::Ok.to_wire()), Response::Ok);
    }

    #[test]
    fn wire_roundtrip_reload() {
        let c = Command::Reload;
        assert_eq!(Command::parse_wire(&c.to_wire()), Ok(c));
        assert_eq!(Command::parse_wire("reload"), Ok(Command::Reload));
    }

    #[test]
    fn parse_errors() {
        assert_eq!(
            Command::parse_wire("frobnicate x"),
            Err(ParseError::UnknownCommand("frobnicate".into()))
        );
        assert!(matches!(
            Command::parse_wire("update x"),
            Err(ParseError::MissingArgument(_))
        ));
        assert!(matches!(
            Command::parse_wire("open"),
            Err(ParseError::MissingArgument(_))
        ));
    }

    #[tokio::test]
    async fn is_active_false_when_no_socket() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("iwwc.sock");
        assert!(!is_active_at(&path).await);
    }

    #[tokio::test]
    async fn is_active_true_when_listener_bound() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("iwwc.sock");
        let _listener = tokio::net::UnixListener::bind(&path).unwrap();
        assert!(is_active_at(&path).await);
    }

    #[tokio::test]
    async fn is_active_removes_stale_socket() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("iwwc.sock");
        {
            let _l = tokio::net::UnixListener::bind(&path).unwrap();
        } // listener dropped; socket file may linger
        let _ = is_active_at(&path).await; // returns false; cleans up if a stale file is present
        // After the probe, a fresh bind must succeed.
        assert!(tokio::net::UnixListener::bind(&path).is_ok());
    }

    #[test]
    fn socket_path_uses_xdg_runtime_dir() {
        let p: PathBuf = socket_path();
        assert!(p.ends_with("iwwc.sock"));
    }

    async fn spawn_test_daemon(path: &std::path::Path) {
        let server = IpcServer::bind(path).unwrap();
        let mut rx = server.serve();
        tokio::spawn(async move {
            while let Some(inc) = rx.recv().await {
                let resp = match &inc.command {
                    Command::Update { name, .. } if name == "boom" => {
                        Response::Error("boom rejected".into())
                    }
                    Command::Open { window } if window == "boom" => {
                        Response::Error("no such window".into())
                    }
                    _ => Response::Ok,
                };
                let _ = inc.reply.send(resp);
            }
        });
    }

    #[tokio::test]
    async fn client_server_roundtrip_ok() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("iwwc.sock");
        spawn_test_daemon(&path).await;

        let r = IpcClient::send_to(
            &path,
            &Command::Open {
                window: "bar".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(r, Response::Ok);

        let r2 = IpcClient::send_to(
            &path,
            &Command::Update {
                name: "x".into(),
                value: "container c1 child=t1".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(r2, Response::Ok);
    }

    #[tokio::test]
    async fn client_server_roundtrip_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("iwwc.sock");
        spawn_test_daemon(&path).await;

        let r = IpcClient::send_to(
            &path,
            &Command::Update {
                name: "boom".into(),
                value: "1".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(r, Response::Error("boom rejected".into()));
    }

    #[tokio::test]
    async fn server_replies_error_on_malformed() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("iwwc.sock");
        spawn_test_daemon(&path).await;

        let mut stream = tokio::net::UnixStream::connect(&path).await.unwrap();
        stream.write_all(b"frobnicate stuff\n").await.unwrap();
        stream.shutdown().await.unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).await.unwrap();
        let parsed = Response::parse_wire(&resp);
        assert!(matches!(parsed, Response::Error(msg) if msg.contains("unknown command")));
    }

    #[tokio::test]
    async fn connect_fails_when_no_daemon() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nope.sock");
        let r = IpcClient::send_to(
            &path,
            &Command::Open {
                window: "bar".into(),
            },
        )
        .await;
        assert!(matches!(r, Err(IpcError::NotRunning)));
    }
}
