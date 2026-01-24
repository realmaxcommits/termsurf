use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// Protocol Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    id: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    id: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl Response {
    fn ok(id: &str, data: Option<serde_json::Value>) -> Self {
        Self {
            id: id.to_string(),
            status: "ok".to_string(),
            data,
            error: None,
        }
    }

    fn error(id: &str, error: &str) -> Self {
        Self {
            id: id.to_string(),
            status: "error".to_string(),
            data: None,
            error: Some(error.to_string()),
        }
    }
}

// ============================================================================
// Profile Mode
// ============================================================================

#[derive(Debug, Clone)]
enum ProfileMode {
    Named(String),
    Incognito(String), // UUID for unique socket path
}

impl ProfileMode {
    fn cache_path(&self) -> Option<PathBuf> {
        match self {
            ProfileMode::Named(name) => {
                let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                Some(PathBuf::from(format!(
                    "{}/.config/termsurf/cef/{}",
                    home, name
                )))
            }
            ProfileMode::Incognito(_) => None,
        }
    }

    fn socket_path(&self) -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let sockets_dir = PathBuf::from(format!("{}/.config/termsurf/sockets", home));

        // Ensure sockets directory exists
        let _ = fs::create_dir_all(&sockets_dir);

        match self {
            ProfileMode::Named(name) => sockets_dir.join(format!("{}.sock", name)),
            ProfileMode::Incognito(uuid) => sockets_dir.join(format!("incognito-{}.sock", uuid)),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            ProfileMode::Named(name) => name.as_str(),
            ProfileMode::Incognito(_) => "incognito",
        }
    }
}

// ============================================================================
// Argument Parsing
// ============================================================================

fn validate_profile_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Profile name cannot be empty".into());
    }

    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() {
        return Err("Profile name must start with a lowercase letter".into());
    }

    for c in name.chars() {
        if !c.is_ascii_lowercase() && !c.is_ascii_digit() {
            return Err(format!(
                "Profile name must be lowercase alphanumeric, found '{}'",
                c
            ));
        }
    }

    Ok(())
}

fn parse_args() -> Result<(bool, ProfileMode, Option<String>), String> {
    let args: Vec<String> = env::args().collect();

    let is_subprocess = args.iter().any(|a| a == "--browser-subprocess");
    let has_incognito = args.iter().any(|a| a == "--incognito");

    // Find --profile value
    let profile_value = args
        .iter()
        .position(|a| a == "--profile")
        .and_then(|i| args.get(i + 1).cloned());

    // Find --incognito-id value (for subprocess)
    let incognito_id = args
        .iter()
        .position(|a| a == "--incognito-id")
        .and_then(|i| args.get(i + 1).cloned());

    // Find URL (first arg that doesn't start with --)
    let url = args
        .iter()
        .skip(1) // Skip program name
        .find(|a| !a.starts_with("--") && !a.is_empty())
        .cloned();

    // Check for mutual exclusivity
    if has_incognito && profile_value.is_some() {
        return Err("Cannot specify both --incognito and --profile".into());
    }

    let profile_mode = if has_incognito || incognito_id.is_some() {
        // Use provided ID or generate new one
        let uuid = incognito_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        ProfileMode::Incognito(uuid)
    } else if let Some(name) = profile_value {
        validate_profile_name(&name)?;
        ProfileMode::Named(name)
    } else {
        ProfileMode::Named("default".to_string())
    };

    Ok((is_subprocess, profile_mode, url))
}

// ============================================================================
// CEF Loading
// ============================================================================

#[cfg(target_os = "macos")]
fn load_cef(profile: &ProfileMode) -> Result<(), String> {
    use cef::args::Args;
    use cef::library_loader::LibraryLoader;
    use cef::{api_hash, execute_process, initialize, sys, CefString, Settings};

    let exe = env::current_exe().map_err(|e| format!("current_exe: {e}"))?;

    let loader = LibraryLoader::new(&exe, false);
    if !loader.load() {
        return Err("Failed to load CEF framework".into());
    }

    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();

    let ret = execute_process(
        Some(args.as_main_args()),
        None::<&mut cef::App>,
        std::ptr::null_mut(),
    );
    if ret >= 0 {
        std::process::exit(ret);
    }

    let cache_path_str = match profile.cache_path() {
        Some(path) => {
            let _ = fs::create_dir_all(&path);
            path.to_string_lossy().to_string()
        }
        None => String::new(),
    };

    let helper_path = exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("Frameworks")
        .join("WezTerm Helper.app")
        .join("Contents")
        .join("MacOS")
        .join("WezTerm Helper");
    let helper_path_str = helper_path.to_string_lossy().to_string();

    let settings = Settings {
        windowless_rendering_enabled: 1,
        external_message_pump: 1,
        no_sandbox: 1,
        root_cache_path: CefString::from(cache_path_str.as_str()),
        browser_subprocess_path: CefString::from(helper_path_str.as_str()),
        ..Default::default()
    };

    if initialize(
        Some(args.as_main_args()),
        Some(&settings),
        None::<&mut cef::App>,
        std::ptr::null_mut(),
    ) != 1
    {
        return Err("CEF initialize failed".into());
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn load_cef(_profile: &ProfileMode) -> Result<(), String> {
    Err("CEF loading not yet implemented for this platform".into())
}

// ============================================================================
// Browser State (shared across connections)
// ============================================================================

struct BrowserState {
    browser_count: AtomicUsize,
    next_browser_id: AtomicU64,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            browser_count: AtomicUsize::new(0),
            next_browser_id: AtomicU64::new(1),
        }
    }

    fn open_browser(&self, url: &str) -> u64 {
        let id = self.next_browser_id.fetch_add(1, Ordering::SeqCst);
        self.browser_count.fetch_add(1, Ordering::SeqCst);
        println!("[Subprocess] Opened browser {} with url: {}", id, url);
        id
    }

    fn close_browser(&self, id: u64) -> bool {
        let prev_count = self.browser_count.fetch_sub(1, Ordering::SeqCst);
        println!(
            "[Subprocess] Closed browser {} (remaining: {})",
            id,
            prev_count - 1
        );
        prev_count == 1 // Returns true if this was the last browser
    }

    fn count(&self) -> usize {
        self.browser_count.load(Ordering::SeqCst)
    }
}

// ============================================================================
// Socket Server (Browser Subprocess)
// ============================================================================

fn handle_request(request: &Request, state: &Arc<BrowserState>) -> (Response, bool) {
    let should_exit = false;

    match request.action.as_str() {
        "ping" => (
            Response::ok(&request.id, Some(serde_json::json!({"pong": true}))),
            should_exit,
        ),

        "open_browser" => {
            let url = request
                .data
                .as_ref()
                .and_then(|d| d.get("url"))
                .and_then(|u| u.as_str())
                .unwrap_or("about:blank");

            let browser_id = state.open_browser(url);

            (
                Response::ok(
                    &request.id,
                    Some(serde_json::json!({"browser_id": browser_id})),
                ),
                should_exit,
            )
        }

        "close_browser" => {
            let browser_id = request
                .data
                .as_ref()
                .and_then(|d| d.get("browser_id"))
                .and_then(|id| id.as_u64())
                .unwrap_or(0);

            let was_last = state.close_browser(browser_id);

            (Response::ok(&request.id, None), was_last)
        }

        "get_status" => (
            Response::ok(
                &request.id,
                Some(serde_json::json!({
                    "browser_count": state.count(),
                    "pid": std::process::id()
                })),
            ),
            should_exit,
        ),

        _ => (
            Response::error(&request.id, &format!("Unknown action: {}", request.action)),
            should_exit,
        ),
    }
}

fn handle_connection(mut stream: UnixStream, state: Arc<BrowserState>) -> bool {
    let peer_id = Uuid::new_v4().to_string()[..8].to_string();
    println!("[Subprocess] Client {} connected", peer_id);

    let reader = BufReader::new(stream.try_clone().expect("Failed to clone stream"));
    let mut should_exit = false;

    for line in reader.lines() {
        match line {
            Ok(line) if line.is_empty() => continue,
            Ok(line) => {
                let (response, exit_after) = match serde_json::from_str::<Request>(&line) {
                    Ok(request) => {
                        println!("[Subprocess] {} -> {:?}", peer_id, request);
                        handle_request(&request, &state)
                    }
                    Err(e) => (
                        Response::error("unknown", &format!("Invalid JSON: {}", e)),
                        false,
                    ),
                };

                let response_json = serde_json::to_string(&response).unwrap();
                println!("[Subprocess] {} <- {}", peer_id, response_json);

                if let Err(e) = writeln!(stream, "{}", response_json) {
                    eprintln!("[Subprocess] Failed to write response: {}", e);
                    break;
                }
                let _ = stream.flush();

                if exit_after {
                    should_exit = true;
                    break;
                }
            }
            Err(e) => {
                eprintln!("[Subprocess] Error reading from {}: {}", peer_id, e);
                break;
            }
        }
    }

    println!("[Subprocess] Client {} disconnected", peer_id);
    should_exit
}

fn run_socket_server(socket_path: PathBuf, state: Arc<BrowserState>, shutdown_flag: Arc<std::sync::atomic::AtomicBool>) {
    // Remove stale socket if it exists
    if socket_path.exists() {
        if let Err(e) = fs::remove_file(&socket_path) {
            eprintln!(
                "[Subprocess] Failed to remove stale socket: {} (continuing anyway)",
                e
            );
        }
    }

    let listener = match UnixListener::bind(&socket_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[Subprocess] Failed to bind socket: {}", e);
            return;
        }
    };

    // Set non-blocking so we can check shutdown flag
    listener.set_nonblocking(true).expect("Failed to set non-blocking");

    println!(
        "[Subprocess] Socket server listening at {:?}",
        socket_path
    );

    let mut handles = Vec::new();

    // Accept connections in a loop
    loop {
        if shutdown_flag.load(Ordering::SeqCst) {
            println!("[Subprocess] Shutdown flag set, stopping accept loop");
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                // Set stream to blocking mode (listener is non-blocking but we want blocking reads)
                stream.set_nonblocking(false).expect("Failed to set stream to blocking");

                let state_clone = Arc::clone(&state);
                let shutdown_clone = Arc::clone(&shutdown_flag);

                let handle = thread::spawn(move || {
                    let should_exit = handle_connection(stream, state_clone);
                    if should_exit {
                        shutdown_clone.store(true, Ordering::SeqCst);
                    }
                });
                handles.push(handle);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No connection ready, sleep briefly and check shutdown flag
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                eprintln!("[Subprocess] Failed to accept connection: {}", e);
            }
        }
    }

    // Wait for all connection handlers to finish
    println!("[Subprocess] Waiting for {} connections to close...", handles.len());
    for handle in handles {
        let _ = handle.join();
    }

    // Cleanup socket
    let _ = fs::remove_file(&socket_path);
    println!("[Subprocess] Socket cleaned up");
}

fn run_browser_subprocess(profile: ProfileMode) {
    let socket_path = profile.socket_path();

    println!(
        "[Subprocess] Starting with profile={}",
        profile.display_name()
    );

    match load_cef(&profile) {
        Ok(()) => {
            println!(
                "[Subprocess] CEF initialized with profile={}",
                profile.display_name()
            );
        }
        Err(e) => {
            eprintln!("[Subprocess] Failed to load CEF: {}", e);
            std::process::exit(1);
        }
    }

    // Create shared browser state and shutdown flag
    let state = Arc::new(BrowserState::new());
    let shutdown_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Run socket server (this blocks until shutdown)
    run_socket_server(socket_path, state, shutdown_flag);

    #[cfg(target_os = "macos")]
    cef::shutdown();

    println!("[Subprocess] Exiting");
}

// ============================================================================
// Socket Client (Coordinator)
// ============================================================================

fn try_connect(socket_path: &PathBuf) -> Option<UnixStream> {
    if !socket_path.exists() {
        return None;
    }

    match UnixStream::connect(socket_path) {
        Ok(stream) => Some(stream),
        Err(_) => {
            // Socket exists but can't connect - stale socket
            let _ = fs::remove_file(socket_path);
            None
        }
    }
}

fn wait_for_socket(socket_path: &PathBuf, timeout: Duration) -> Result<UnixStream, String> {
    let start = std::time::Instant::now();

    loop {
        if let Some(stream) = try_connect(socket_path) {
            return Ok(stream);
        }

        if start.elapsed() > timeout {
            return Err(format!("Timeout waiting for socket at {:?}", socket_path));
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn send_request(stream: &mut UnixStream, action: &str, data: Option<serde_json::Value>) -> Result<Response, String> {
    let request = Request {
        id: Uuid::new_v4().to_string(),
        action: action.to_string(),
        data,
    };

    let request_json = serde_json::to_string(&request).unwrap();
    writeln!(stream, "{}", request_json).map_err(|e| format!("Failed to write: {}", e))?;
    stream.flush().map_err(|e| format!("Failed to flush: {}", e))?;

    let mut reader = BufReader::new(stream.try_clone().map_err(|e| format!("Clone failed: {}", e))?);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).map_err(|e| format!("Failed to read: {}", e))?;

    serde_json::from_str(&response_line).map_err(|e| format!("Invalid response JSON: {}", e))
}

fn spawn_subprocess(profile: &ProfileMode) {
    let exe = env::current_exe().expect("Failed to get current executable path");

    let mut cmd = Command::new(&exe);
    cmd.arg("--browser-subprocess");

    match profile {
        ProfileMode::Named(name) => {
            cmd.arg("--profile").arg(name);
        }
        ProfileMode::Incognito(uuid) => {
            cmd.arg("--incognito").arg("--incognito-id").arg(uuid);
        }
    }

    // Spawn in background - don't wait for it
    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("Failed to spawn browser subprocess");
}

fn run_coordinator(profile: ProfileMode, url: Option<String>) {
    let socket_path = profile.socket_path();
    let url = url.unwrap_or_else(|| "about:blank".to_string());

    // Try to connect to existing subprocess
    let mut stream = if let Some(stream) = try_connect(&socket_path) {
        println!(
            "Connected to existing subprocess for profile={}",
            profile.display_name()
        );
        stream
    } else {
        println!(
            "Spawning new subprocess for profile={}...",
            profile.display_name()
        );
        spawn_subprocess(&profile);

        match wait_for_socket(&socket_path, Duration::from_secs(10)) {
            Ok(stream) => {
                println!("Connected to subprocess");
                stream
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    // Open a browser
    println!("Opening browser: {}", url);
    let response = send_request(
        &mut stream,
        "open_browser",
        Some(serde_json::json!({"url": url})),
    );

    let browser_id = match response {
        Ok(resp) if resp.status == "ok" => {
            let id = resp.data.as_ref()
                .and_then(|d| d.get("browser_id"))
                .and_then(|id| id.as_u64())
                .unwrap_or(0);
            println!("Browser opened with id={}", id);
            id
        }
        Ok(resp) => {
            eprintln!("Failed to open browser: {}", resp.error.unwrap_or_default());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to open browser: {}", e);
            std::process::exit(1);
        }
    };

    // Get subprocess status
    if let Ok(resp) = send_request(&mut stream, "get_status", None) {
        if let Some(data) = resp.data {
            println!(
                "Subprocess status: pid={}, browsers={}",
                data.get("pid").and_then(|p| p.as_u64()).unwrap_or(0),
                data.get("browser_count").and_then(|c| c.as_u64()).unwrap_or(0)
            );
        }
    }

    // Wait for user to press Enter (simulating browser being open)
    println!("\nPress Enter to close browser...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);

    // Close the browser
    println!("Closing browser {}...", browser_id);
    match send_request(
        &mut stream,
        "close_browser",
        Some(serde_json::json!({"browser_id": browser_id})),
    ) {
        Ok(resp) if resp.status == "ok" => {
            println!("Browser closed");
        }
        Ok(resp) => {
            eprintln!("Failed to close browser: {}", resp.error.unwrap_or_default());
        }
        Err(e) => {
            eprintln!("Failed to close browser: {}", e);
        }
    }

    println!("Coordinator exiting");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    match parse_args() {
        Ok((is_subprocess, profile, url)) => {
            if is_subprocess {
                run_browser_subprocess(profile);
            } else {
                run_coordinator(profile, url);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!();
            eprintln!("Usage: web [URL] [--profile <name>] [--incognito]");
            eprintln!();
            eprintln!("Options:");
            eprintln!("  --profile <name>  Use named profile (default: 'default')");
            eprintln!("  --incognito       Use incognito mode (no persistent storage)");
            eprintln!();
            eprintln!("Profile names must be lowercase alphanumeric and start with a letter.");
            std::process::exit(1);
        }
    }
}
