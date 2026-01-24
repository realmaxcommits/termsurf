use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(target_os = "macos")]
fn load_cef() -> Result<(), String> {
    use cef::args::Args;
    use cef::library_loader::LibraryLoader;
    use cef::{api_hash, execute_process, initialize, sys, CefString, Settings};

    let exe = env::current_exe().map_err(|e| format!("current_exe: {e}"))?;

    // Load CEF framework
    let loader = LibraryLoader::new(&exe, false);
    if !loader.load() {
        return Err("Failed to load CEF framework".into());
    }

    // Configure CEF API version
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();

    // Check if we're a subprocess (renderer, GPU, etc.)
    let ret = execute_process(Some(args.as_main_args()), None::<&mut cef::App>, std::ptr::null_mut());
    if ret >= 0 {
        // We're a CEF subprocess, exit with the return code
        std::process::exit(ret);
    }

    // Create CEF cache directory
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let cef_cache = PathBuf::from(format!("{}/.config/termsurf/cef", home));
    let _ = std::fs::create_dir_all(&cef_cache);
    let cache_path_str = cef_cache.to_string_lossy().to_string();

    // Compute path to helper binary
    // exe is: .../wezterm-gui.app/Contents/MacOS/web
    // helper is: .../wezterm-gui.app/Contents/Frameworks/WezTerm Helper.app/Contents/MacOS/WezTerm Helper
    let helper_path = exe
        .parent()
        .unwrap() // MacOS
        .parent()
        .unwrap() // Contents
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
fn load_cef() -> Result<(), String> {
    Err("CEF loading not yet implemented for this platform".into())
}

fn run_browser_subprocess() {
    match load_cef() {
        Ok(()) => {
            println!("loaded CEF");
            // Shutdown CEF
            #[cfg(target_os = "macos")]
            cef::shutdown();
        }
        Err(e) => {
            eprintln!("Failed to load CEF: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_coordinator() {
    let exe = env::current_exe().expect("Failed to get current executable path");

    println!("Coordinator: spawning browser subprocess...");

    let child = Command::new(&exe)
        .arg("--browser-subprocess")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn browser subprocess");

    let output = child.wait_with_output().expect("Failed to wait for subprocess");

    println!("Subprocess stdout: {}", String::from_utf8_lossy(&output.stdout));
    if !output.stderr.is_empty() {
        eprintln!("Subprocess stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
    println!("Subprocess exited with: {}", output.status);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--browser-subprocess") {
        run_browser_subprocess();
    } else {
        run_coordinator();
    }
}
