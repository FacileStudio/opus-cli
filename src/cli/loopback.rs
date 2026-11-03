use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;

use crate::cli::loopback_page::respond;

const WAIT: Duration = Duration::from_secs(180);

pub struct Loopback {
    listener: TcpListener,
}

pub fn random_state() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let mut bytes = Vec::with_capacity(16);
    for round in 0..2u8 {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u128(nanos);
        hasher.write_u8(round);
        bytes.extend_from_slice(&hasher.finish().to_le_bytes());
    }
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub fn open_browser(url: &str) -> bool {
    let mut command = if cfg!(target_os = "macos") {
        Command::new("open")
    } else if cfg!(target_os = "windows") {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", ""]);
        command
    } else {
        Command::new("xdg-open")
    };
    command
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

impl Loopback {
    pub fn listen() -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("cannot open a loopback port to receive the login: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        Ok(Self { listener })
    }

    pub fn port(&self) -> u16 {
        self.listener.local_addr().map(|a| a.port()).unwrap_or(0)
    }

    pub fn wait_for_code(&self, state: &str) -> Result<String, String> {
        let deadline = Instant::now() + WAIT;
        while Instant::now() < deadline {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Some(code) = answer(stream, state) {
                        return Ok(code);
                    }
                }
                Err(_) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
        Err("timed out waiting for the browser, run 'opus login' again".to_string())
    }
}

fn query_param(query: &str, name: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_string())
}

fn answer(mut stream: TcpStream, state: &str) -> Option<String> {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buffer = [0u8; 4096];
    let read = stream.read(&mut buffer).ok()?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let target = request.lines().next()?.split_whitespace().nth(1)?.to_string();
    let (path, query) = target.split_once('?').unwrap_or((target.as_str(), ""));
    let waiting = "The login started in your terminal is still waiting.";
    if path != "/" {
        respond(&mut stream, "404 Not Found", "Not found", "", true);
        return None;
    }
    if query_param(query, "state").as_deref() != Some(state) {
        respond(&mut stream, "400 Bad Request", "That is not this login", waiting, true);
        return None;
    }
    match query_param(query, "code").filter(|code| !code.is_empty()) {
        Some(code) => {
            let done = "You can close this tab and go back to your terminal.";
            respond(&mut stream, "200 OK", "Signed in", done, false);
            Some(code)
        }
        None => {
            respond(&mut stream, "400 Bad Request", "No login code", waiting, true);
            None
        }
    }
}
