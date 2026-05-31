use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

// ABI ownership model:
// - Config/app/surface handles returned by Roastty are heap-owned by Roastty and
//   released only by their matching free function.
// - Runtime callback userdata, surface config pointers, strings, env arrays,
//   platform pointers, and app pointers stored on surfaces are borrowed from the
//   caller; this skeleton records scalar values but never frees borrowed data.
// - GhosttyString values are freed only by ghostty_string_free and only when
//   they were returned by Roastty string-returning functions.
pub type GhosttyApp = *mut c_void;
pub type GhosttyConfig = *mut c_void;
pub type GhosttySurface = *mut c_void;

const GHOSTTY_SUCCESS: c_int = 0;
const GHOSTTY_BUILD_MODE_DEBUG: c_int = 0;

#[repr(C)]
pub struct GhosttyInfo {
    build_mode: c_int,
    version: *const c_char,
    version_len: usize,
}

#[repr(C)]
pub struct GhosttyDiagnostic {
    message: *const c_char,
}

#[repr(C)]
pub struct GhosttyString {
    ptr: *const c_char,
    len: usize,
    sentinel: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyEnvVar {
    key: *const c_char,
    value: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPlatformMacos {
    nsview: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyPlatformIos {
    uiview: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union GhosttyPlatform {
    macos: GhosttyPlatformMacos,
    ios: GhosttyPlatformIos,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttySurfaceConfig {
    platform_tag: c_int,
    platform: GhosttyPlatform,
    userdata: *mut c_void,
    scale_factor: f64,
    font_size: f32,
    working_directory: *const c_char,
    command: *const c_char,
    env_vars: *mut GhosttyEnvVar,
    env_var_count: usize,
    initial_input: *const c_char,
    wait_after_command: bool,
    context: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GhosttySurfaceSize {
    columns: u16,
    rows: u16,
    width_px: u32,
    height_px: u32,
    cell_width_px: u32,
    cell_height_px: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyClipboardContent {
    mime: *const c_char,
    data: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyTarget {
    tag: c_int,
    surface: GhosttySurface,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyAction {
    tag: c_int,
    storage: [usize; 8],
}

type WakeupCallback = Option<unsafe extern "C" fn(*mut c_void)>;
type ActionCallback =
    Option<unsafe extern "C" fn(GhosttyApp, GhosttyTarget, GhosttyAction) -> bool>;
type ReadClipboardCallback = Option<unsafe extern "C" fn(*mut c_void, c_int, *mut c_void) -> bool>;
type ConfirmReadClipboardCallback =
    Option<unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void, c_int)>;
type WriteClipboardCallback =
    Option<unsafe extern "C" fn(*mut c_void, c_int, *const GhosttyClipboardContent, usize, bool)>;
type CloseSurfaceCallback = Option<unsafe extern "C" fn(*mut c_void, bool)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GhosttyRuntimeConfig {
    userdata: *mut c_void,
    supports_selection_clipboard: bool,
    wakeup_cb: WakeupCallback,
    action_cb: ActionCallback,
    read_clipboard_cb: ReadClipboardCallback,
    confirm_read_clipboard_cb: ConfirmReadClipboardCallback,
    write_clipboard_cb: WriteClipboardCallback,
    close_surface_cb: CloseSurfaceCallback,
}

struct Config {
    finalized: bool,
}

struct App {
    runtime: GhosttyRuntimeConfig,
    focused: bool,
    color_scheme: c_int,
}

struct Surface {
    app: GhosttyApp,
    userdata: *mut c_void,
    scale_factor_x: f64,
    scale_factor_y: f64,
    focused: bool,
    occluded: bool,
    size: GhosttySurfaceSize,
    color_scheme: c_int,
}

static VERSION: &[u8] = b"0.1.0-roastty\0";
static EMPTY_DIAGNOSTIC: &[u8] = b"\0";

fn config_from_handle<'a>(handle: GhosttyConfig) -> Option<&'a mut Config> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *(handle.cast::<Config>()) })
    }
}

fn app_from_handle<'a>(handle: GhosttyApp) -> Option<&'a mut App> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *(handle.cast::<App>()) })
    }
}

fn surface_from_handle<'a>(handle: GhosttySurface) -> Option<&'a mut Surface> {
    if handle.is_null() {
        None
    } else {
        Some(unsafe { &mut *(handle.cast::<Surface>()) })
    }
}

fn empty_string() -> GhosttyString {
    GhosttyString {
        ptr: ptr::null(),
        len: 0,
        sentinel: false,
    }
}

fn allocated_string(bytes: &[u8]) -> GhosttyString {
    let owned = bytes.to_vec().into_boxed_slice();
    let len = owned.len();
    let ptr = Box::into_raw(owned).cast::<u8>();
    GhosttyString {
        ptr: ptr.cast::<c_char>(),
        len,
        sentinel: false,
    }
}

fn allocated_c_string(value: &str) -> GhosttyString {
    let c_string = CString::new(value).expect("static strings must not contain interior nuls");
    let len = c_string.as_bytes().len();
    let ptr = c_string.into_raw();
    GhosttyString {
        ptr,
        len,
        sentinel: true,
    }
}

#[no_mangle]
pub extern "C" fn ghostty_init(_argc: usize, _argv: *mut *mut c_char) -> c_int {
    GHOSTTY_SUCCESS
}

#[no_mangle]
pub extern "C" fn ghostty_info() -> GhosttyInfo {
    GhosttyInfo {
        build_mode: GHOSTTY_BUILD_MODE_DEBUG,
        version: VERSION.as_ptr().cast::<c_char>(),
        version_len: VERSION.len() - 1,
    }
}

#[no_mangle]
pub extern "C" fn ghostty_string_free(value: GhosttyString) {
    if value.ptr.is_null() || value.len == 0 {
        return;
    }

    unsafe {
        if value.sentinel {
            drop(CString::from_raw(value.ptr.cast_mut()));
        } else {
            let slice = ptr::slice_from_raw_parts_mut(value.ptr.cast::<u8>().cast_mut(), value.len);
            drop(Box::from_raw(slice));
        }
    }
}

#[no_mangle]
pub extern "C" fn ghostty_config_new() -> GhosttyConfig {
    Box::into_raw(Box::new(Config { finalized: false })).cast()
}

#[no_mangle]
pub extern "C" fn ghostty_config_free(config: GhosttyConfig) {
    if !config.is_null() {
        unsafe {
            drop(Box::from_raw(config.cast::<Config>()));
        }
    }
}

#[no_mangle]
pub extern "C" fn ghostty_config_clone(config: GhosttyConfig) -> GhosttyConfig {
    let finalized = config_from_handle(config)
        .map(|config| config.finalized)
        .unwrap_or(false);
    Box::into_raw(Box::new(Config { finalized })).cast()
}

#[no_mangle]
pub extern "C" fn ghostty_config_load_cli_args(_config: GhosttyConfig) {}

#[no_mangle]
pub extern "C" fn ghostty_config_load_file(_config: GhosttyConfig, _path: *const c_char) {}

#[no_mangle]
pub extern "C" fn ghostty_config_load_default_files(_config: GhosttyConfig) {}

#[no_mangle]
pub extern "C" fn ghostty_config_load_recursive_files(_config: GhosttyConfig) {}

#[no_mangle]
pub extern "C" fn ghostty_config_finalize(config: GhosttyConfig) {
    if let Some(config) = config_from_handle(config) {
        config.finalized = true;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_config_diagnostics_count(_config: GhosttyConfig) -> u32 {
    0
}

#[no_mangle]
pub extern "C" fn ghostty_config_get_diagnostic(
    _config: GhosttyConfig,
    _index: u32,
) -> GhosttyDiagnostic {
    GhosttyDiagnostic {
        message: EMPTY_DIAGNOSTIC.as_ptr().cast::<c_char>(),
    }
}

#[no_mangle]
pub extern "C" fn ghostty_config_open_path() -> GhosttyString {
    allocated_string(b"roastty-config")
}

#[no_mangle]
pub extern "C" fn ghostty_app_new(
    runtime: *const GhosttyRuntimeConfig,
    _config: GhosttyConfig,
) -> GhosttyApp {
    let runtime = if runtime.is_null() {
        GhosttyRuntimeConfig {
            userdata: ptr::null_mut(),
            supports_selection_clipboard: false,
            wakeup_cb: None,
            action_cb: None,
            read_clipboard_cb: None,
            confirm_read_clipboard_cb: None,
            write_clipboard_cb: None,
            close_surface_cb: None,
        }
    } else {
        unsafe { *runtime }
    };

    Box::into_raw(Box::new(App {
        runtime,
        focused: false,
        color_scheme: 0,
    }))
    .cast()
}

#[no_mangle]
pub extern "C" fn ghostty_app_free(app: GhosttyApp) {
    if !app.is_null() {
        unsafe {
            drop(Box::from_raw(app.cast::<App>()));
        }
    }
}

#[no_mangle]
pub extern "C" fn ghostty_app_tick(_app: GhosttyApp) {}

#[no_mangle]
pub extern "C" fn ghostty_app_userdata(app: GhosttyApp) -> *mut c_void {
    app_from_handle(app)
        .map(|app| app.runtime.userdata)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn ghostty_app_set_focus(app: GhosttyApp, focused: bool) {
    if let Some(app) = app_from_handle(app) {
        app.focused = focused;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_app_update_config(_app: GhosttyApp, _config: GhosttyConfig) {}

#[no_mangle]
pub extern "C" fn ghostty_app_needs_confirm_quit(_app: GhosttyApp) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn ghostty_app_has_global_keybinds(_app: GhosttyApp) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn ghostty_app_set_color_scheme(app: GhosttyApp, color_scheme: c_int) {
    if let Some(app) = app_from_handle(app) {
        app.color_scheme = color_scheme;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_config_new() -> GhosttySurfaceConfig {
    GhosttySurfaceConfig {
        platform_tag: 0,
        platform: GhosttyPlatform {
            macos: GhosttyPlatformMacos {
                nsview: ptr::null_mut(),
            },
        },
        userdata: ptr::null_mut(),
        scale_factor: 1.0,
        font_size: 0.0,
        working_directory: ptr::null(),
        command: ptr::null(),
        env_vars: ptr::null_mut(),
        env_var_count: 0,
        initial_input: ptr::null(),
        wait_after_command: false,
        context: 0,
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_new(
    app: GhosttyApp,
    config: *const GhosttySurfaceConfig,
) -> GhosttySurface {
    if app.is_null() {
        return ptr::null_mut();
    }

    let config = if config.is_null() {
        ghostty_surface_config_new()
    } else {
        unsafe { *config }
    };

    Box::into_raw(Box::new(Surface {
        app,
        userdata: config.userdata,
        scale_factor_x: config.scale_factor,
        scale_factor_y: config.scale_factor,
        focused: false,
        occluded: false,
        size: GhosttySurfaceSize {
            columns: 0,
            rows: 0,
            width_px: 0,
            height_px: 0,
            cell_width_px: 0,
            cell_height_px: 0,
        },
        color_scheme: 0,
    }))
    .cast()
}

#[no_mangle]
pub extern "C" fn ghostty_surface_free(surface: GhosttySurface) {
    if !surface.is_null() {
        unsafe {
            drop(Box::from_raw(surface.cast::<Surface>()));
        }
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_userdata(surface: GhosttySurface) -> *mut c_void {
    surface_from_handle(surface)
        .map(|surface| surface.userdata)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn ghostty_surface_app(surface: GhosttySurface) -> GhosttyApp {
    surface_from_handle(surface)
        .map(|surface| surface.app)
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn ghostty_surface_update_config(_surface: GhosttySurface, _config: GhosttyConfig) {}

#[no_mangle]
pub extern "C" fn ghostty_surface_needs_confirm_quit(_surface: GhosttySurface) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn ghostty_surface_process_exited(_surface: GhosttySurface) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn ghostty_surface_set_content_scale(surface: GhosttySurface, x: f64, y: f64) {
    if let Some(surface) = surface_from_handle(surface) {
        surface.scale_factor_x = x;
        surface.scale_factor_y = y;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_set_focus(surface: GhosttySurface, focused: bool) {
    if let Some(surface) = surface_from_handle(surface) {
        surface.focused = focused;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_set_occlusion(surface: GhosttySurface, occluded: bool) {
    if let Some(surface) = surface_from_handle(surface) {
        surface.occluded = occluded;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_set_size(surface: GhosttySurface, width: u32, height: u32) {
    if let Some(surface) = surface_from_handle(surface) {
        surface.size.width_px = width;
        surface.size.height_px = height;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_size(surface: GhosttySurface) -> GhosttySurfaceSize {
    surface_from_handle(surface)
        .map(|surface| surface.size)
        .unwrap_or(GhosttySurfaceSize {
            columns: 0,
            rows: 0,
            width_px: 0,
            height_px: 0,
            cell_width_px: 0,
            cell_height_px: 0,
        })
}

#[no_mangle]
pub extern "C" fn ghostty_surface_foreground_pid(_surface: GhosttySurface) -> u64 {
    0
}

#[no_mangle]
pub extern "C" fn ghostty_surface_tty_name(surface: GhosttySurface) -> GhosttyString {
    if surface.is_null() {
        empty_string()
    } else {
        allocated_c_string("roastty-skeleton-tty")
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_set_color_scheme(surface: GhosttySurface, color_scheme: c_int) {
    if let Some(surface) = surface_from_handle(surface) {
        surface.color_scheme = color_scheme;
    }
}

#[no_mangle]
pub extern "C" fn ghostty_surface_request_close(_surface: GhosttySurface) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_shape_matches_ghostty() {
        let value = empty_string();
        assert!(value.ptr.is_null());
        assert_eq!(value.len, 0);
        assert!(!value.sentinel);
        ghostty_string_free(value);
    }

    #[test]
    fn allocated_non_sentinel_string_can_be_freed() {
        let value = ghostty_config_open_path();
        assert!(!value.ptr.is_null());
        assert_eq!(value.len, "roastty-config".len());
        assert!(!value.sentinel);
        ghostty_string_free(value);
    }

    #[test]
    fn allocated_sentinel_string_can_be_freed() {
        let config = ghostty_config_new();
        let runtime = GhosttyRuntimeConfig {
            userdata: ptr::null_mut(),
            supports_selection_clipboard: false,
            wakeup_cb: None,
            action_cb: None,
            read_clipboard_cb: None,
            confirm_read_clipboard_cb: None,
            write_clipboard_cb: None,
            close_surface_cb: None,
        };
        let app = ghostty_app_new(&runtime, config);
        let surface_config = ghostty_surface_config_new();
        let surface = ghostty_surface_new(app, &surface_config);

        let value = ghostty_surface_tty_name(surface);
        assert!(!value.ptr.is_null());
        assert_eq!(value.len, "roastty-skeleton-tty".len());
        assert!(value.sentinel);
        ghostty_string_free(value);

        ghostty_surface_free(surface);
        ghostty_app_free(app);
        ghostty_config_free(config);
    }
}
