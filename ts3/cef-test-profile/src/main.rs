//! cef-test profile server — standalone headless CEF process.
//!
//! Loads a URL and renders it off-screen, logging frame output.
//! No XPC, no multi-browser — just CEF rendering in isolation.
//!
//! Usage:
//!   cef-test-profile --url https://google.com [--width 800] [--height 600] [--scale 2.0]

use clap::Parser;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

static FRAME_COUNTER: AtomicU64 = AtomicU64::new(0);
static START_TIME: OnceLock<Instant> = OnceLock::new();
static QUIT_FLAG: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
mod cfrunloop {
    use std::ffi::c_void;

    type CFStringRef = *const c_void;
    type CFTimeInterval = f64;

    extern "C" {
        static kCFRunLoopDefaultMode: CFStringRef;
        fn CFRunLoopRunInMode(
            mode: CFStringRef,
            seconds: CFTimeInterval,
            return_after_source_handled: u8,
        ) -> i32;
    }

    /// Run the main thread's CFRunLoop for up to `seconds`, returning after
    /// one source is handled or the timeout expires.
    pub fn run_for(seconds: f64) -> i32 {
        unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, 1) }
    }
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    url: String,

    /// Logical width for CEF view_rect
    #[arg(long, default_value = "800")]
    width: u32,

    /// Logical height for CEF view_rect
    #[arg(long, default_value = "600")]
    height: u32,

    /// Device scale factor (e.g. 2.0 for Retina)
    #[arg(long, default_value = "2.0")]
    scale: f32,
}

fn main() {
    let args = Args::parse();
    println!(
        "cef-test-profile: url='{}', size={}x{}, scale={}",
        args.url, args.width, args.height, args.scale
    );

    #[cfg(target_os = "macos")]
    run(args);

    #[cfg(not(target_os = "macos"))]
    {
        let _ = args;
        eprintln!("cef-test-profile: CEF not supported on this platform");
        std::process::exit(1);
    }
}

struct ProfileState {
    url: String,
    width: std::sync::atomic::AtomicU32,
    height: std::sync::atomic::AtomicU32,
    scale: f32,
}

#[cfg(target_os = "macos")]
fn run(args: Args) {
    use cef::library_loader::LibraryLoader;

    let exe = std::env::current_exe().expect("Failed to get executable path");
    println!("cef-test-profile: exe={:?}", exe);

    // Load CEF framework (false = main process, not a helper)
    let _loader = LibraryLoader::new(&exe, false);
    if !_loader.load() {
        eprintln!("cef-test-profile: Failed to load CEF framework");
        std::process::exit(1);
    }
    println!("cef-test-profile: CEF framework loaded");

    // Required before creating App objects
    let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);

    // Subprocess check (early return for helper processes)
    let cef_args = cef::args::Args::new();
    let exit_code = cef::execute_process(
        Some(cef_args.as_main_args()),
        None::<&mut cef::App>,
        std::ptr::null_mut(),
    );
    if exit_code >= 0 {
        std::process::exit(exit_code);
    }
    println!("cef-test-profile: Main process (ret={})", exit_code);

    // Profile state
    let state = Arc::new(ProfileState {
        url: args.url.clone(),
        width: std::sync::atomic::AtomicU32::new(args.width),
        height: std::sync::atomic::AtomicU32::new(args.height),
        scale: args.scale,
    });

    // Cache path
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let cache_path = std::path::PathBuf::from(home).join(".config/cef-test/default");
    std::fs::create_dir_all(&cache_path).ok();
    println!("cef-test-profile: cache={:?}", cache_path);

    // CEF settings
    let settings = cef::Settings {
        windowless_rendering_enabled: 1,
        no_sandbox: 1,
        log_severity: cef::LogSeverity::VERBOSE,
        log_file: cef::CefString::from("/tmp/cef-test-debug.log"),
        root_cache_path: cef::CefString::from(cache_path.to_str().unwrap()),
        persist_session_cookies: 1,
        ..Default::default()
    };

    let mut app = cef_handlers::create_app(Arc::clone(&state));

    let init_result = cef::initialize(
        Some(cef_args.as_main_args()),
        Some(&settings),
        Some(&mut app),
        std::ptr::null_mut(),
    );
    if init_result != 1 {
        eprintln!(
            "cef-test-profile: CEF initialize failed (returned {})",
            init_result
        );
        std::process::exit(1);
    }
    println!("cef-test-profile: CEF initialized");

    // Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("cef-test-profile: Ctrl+C, shutting down...");
        QUIT_FLAG.store(true, Ordering::Relaxed);
    })
    .expect("Failed to set Ctrl+C handler");

    // Message loop (matching ts3's pattern exactly)
    println!("cef-test-profile: Running message loop...");
    let mut loop_count: u64 = 0;
    let mut max_mlw_us: u128 = 0;
    let mut max_cfl_us: u128 = 0;
    let mut mlw_spike_count: u64 = 0;

    while !QUIT_FLAG.load(Ordering::Relaxed) {
        let t0 = Instant::now();

        cef::do_message_loop_work();
        let t1 = Instant::now();

        cfrunloop::run_for(0.001);
        let t2 = Instant::now();

        let mlw_us = (t1 - t0).as_micros();
        let cfl_us = (t2 - t1).as_micros();

        if mlw_us > max_mlw_us {
            max_mlw_us = mlw_us;
        }
        if cfl_us > max_cfl_us {
            max_cfl_us = cfl_us;
        }
        if mlw_us > 1000 {
            mlw_spike_count += 1;
        }

        loop_count += 1;
        if loop_count % 1000 == 0 {
            println!(
                "[LOOP-TIMING] iter={} max_mlw={}us max_cfl={}us mlw_spikes={}",
                loop_count, max_mlw_us, max_cfl_us, mlw_spike_count
            );
        }
    }

    println!(
        "[LOOP-TIMING] FINAL iter={} max_mlw={}us max_cfl={}us mlw_spikes={}",
        loop_count, max_mlw_us, max_cfl_us, mlw_spike_count
    );

    // Shutdown
    println!("cef-test-profile: Shutting down...");
    cef::shutdown();
    println!("cef-test-profile: Done");
}

// ============================================================================
// CEF Handlers
// ============================================================================

#[cfg(target_os = "macos")]
mod cef_handlers {
    use super::ProfileState;
    use cef::rc::Rc;
    use cef::{
        wrap_app, wrap_browser_process_handler, wrap_client, wrap_context_menu_handler,
        wrap_render_handler, AcceleratedPaintInfo, App, Browser, BrowserProcessHandler,
        BrowserSettings, CefString, Client, ContextMenuHandler, ContextMenuParams, Frame,
        ImplApp, ImplBrowser, ImplBrowserHost, ImplBrowserProcessHandler, ImplClient,
        ImplCommandLine, ImplContextMenuHandler, ImplMenuModel, ImplRenderHandler, MenuModel,
        PaintElementType, Rect, RenderHandler, ScreenInfo, WindowInfo, WrapApp,
        WrapBrowserProcessHandler, WrapClient, WrapContextMenuHandler, WrapRenderHandler,
    };
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    // ====== Render Handler ======

    #[derive(Clone)]
    struct RenderHandlerInner {
        state: Arc<ProfileState>,
    }

    wrap_render_handler! {
        pub struct TestRenderHandler {
            inner: RenderHandlerInner,
        }

        impl RenderHandler {
            fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
                if let Some(rect) = rect {
                    rect.width = self.inner.state.width.load(Ordering::Relaxed) as i32;
                    rect.height = self.inner.state.height.load(Ordering::Relaxed) as i32;
                    println!("[VIEW_RECT] {}x{}", rect.width, rect.height);
                }
            }

            fn screen_info(
                &self,
                _browser: Option<&mut Browser>,
                screen_info: Option<&mut ScreenInfo>,
            ) -> ::std::os::raw::c_int {
                if let Some(info) = screen_info {
                    info.device_scale_factor = self.inner.state.scale;
                    return 1;
                }
                0
            }

            fn on_accelerated_paint(
                &self,
                _browser: Option<&mut Browser>,
                type_: PaintElementType,
                _dirty_rects: Option<&[Rect]>,
                info: Option<&AcceleratedPaintInfo>,
            ) {
                let Some(info) = info else { return };

                // Only handle PET_VIEW (skip popups)
                if type_ != PaintElementType::default() {
                    return;
                }

                let handle = info.shared_texture_io_surface as *mut std::ffi::c_void;
                if handle.is_null() {
                    return;
                }

                let frame_id =
                    crate::FRAME_COUNTER.fetch_add(1, crate::Ordering::Relaxed);
                let start = *crate::START_TIME.get_or_init(std::time::Instant::now);
                let t_ms = start.elapsed().as_millis() as i64;
                let w = info.extra.coded_size.width;
                let h = info.extra.coded_size.height;

                println!(
                    "[FRAME-TX] frame={} w={} h={} time={}ms",
                    frame_id, w, h, t_ms
                );
            }
        }
    }

    // ====== Context Menu Handler ======

    #[derive(Clone)]
    struct ContextMenuInner;

    wrap_context_menu_handler! {
        pub struct TestContextMenuHandler {
            inner: ContextMenuInner,
        }

        impl ContextMenuHandler {
            fn on_before_context_menu(
                &self,
                _browser: Option<&mut Browser>,
                _frame: Option<&mut Frame>,
                _params: Option<&mut ContextMenuParams>,
                model: Option<&mut MenuModel>,
            ) {
                if let Some(model) = model {
                    model.clear();
                }
            }
        }
    }

    // ====== Client ======

    wrap_client! {
        pub struct TestClient {
            render_handler: RenderHandler,
            context_menu_handler: ContextMenuHandler,
        }

        impl Client {
            fn render_handler(&self) -> Option<RenderHandler> {
                Some(self.render_handler.clone())
            }

            fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
                Some(self.context_menu_handler.clone())
            }
        }
    }

    // ====== Browser Process Handler ======

    wrap_browser_process_handler! {
        pub struct TestBPH {
            state: Arc<ProfileState>,
        }

        impl BrowserProcessHandler {
            fn on_context_initialized(&self) {
                println!("cef-test-profile: CEF context initialized, creating browser...");

                let render_inner = RenderHandlerInner {
                    state: Arc::clone(&self.state),
                };
                let render_handler = TestRenderHandler::new(render_inner);
                let context_menu_handler = TestContextMenuHandler::new(ContextMenuInner);
                let mut client = TestClient::new(render_handler, context_menu_handler);

                let window_info = WindowInfo {
                    windowless_rendering_enabled: 1,
                    shared_texture_enabled: 1,
                    ..Default::default()
                };

                let browser_settings = BrowserSettings {
                    windowless_frame_rate: 60,
                    background_color: 0xFFFFFFFF,
                    ..Default::default()
                };

                let url: CefString = self.state.url.as_str().into();

                let browser = cef::browser_host_create_browser_sync(
                    Some(&window_info),
                    Some(&mut client),
                    Some(&url),
                    Some(&browser_settings),
                    None,
                    None,
                );

                match browser {
                    Some(b) => {
                        let id = b.identifier();
                        println!(
                            "cef-test-profile: Browser {} created for '{}'",
                            id, self.state.url
                        );
                        // Set initial focus (toggle for proper initialization)
                        if let Some(host) = b.host() {
                            host.set_focus(0);
                            host.set_focus(1);
                        }
                    }
                    None => eprintln!("cef-test-profile: Failed to create browser"),
                }
            }
        }
    }

    // ====== App ======

    wrap_app! {
        pub struct TestApp {
            handler: BrowserProcessHandler,
        }

        impl App {
            fn on_before_command_line_processing(
                &self,
                _process_type: Option<&cef::CefStringUtf16>,
                command_line: Option<&mut cef::CommandLine>,
            ) {
                if let Some(command_line) = command_line {
                    command_line.append_switch(Some(&"no-startup-window".into()));
                    command_line.append_switch(Some(&"enable-logging".into()));
                    command_line.append_switch_with_value(
                        Some(&"v".into()),
                        Some(&"1".into()),
                    );
                }
            }

            fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
                Some(self.handler.clone())
            }
        }
    }

    pub fn create_app(state: Arc<ProfileState>) -> App {
        let handler = TestBPH::new(state);
        TestApp::new(handler)
    }
}
