//! Custom (shadertoy-style) shader support.
//!
//! The `CustomShaderUniforms` value type custom shaders read, plus the
//! Shadertoy-style shader loader. This is a faithful Rust port of upstream
//! `renderer/shadertoy.zig`: configured files are wrapped with
//! `shadertoy_prefix.glsl`, compiled as GLSL fragments to SPIR-V, then
//! cross-compiled to the renderer target.
#![allow(dead_code)]

use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::config::ConfigFilePath;
use crate::renderer::cursor::Style;
use crate::renderer::shader::CellTextVertex;
use crate::terminal::color::{Palette, Rgb};

const MAX_SHADER_BYTES: u64 = 4 * 1024 * 1024;
const SHADERTOY_PREFIX_GLSL: &str =
    include_str!("../../../vendor/ghostty/src/renderer/shaders/shadertoy_prefix.glsl");

/// The output language the custom-shader loader cross-compiles to (upstream
/// `shadertoy.Target`): `Glsl` for OpenGL, `Msl` for Metal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Glsl,
    Msl,
}

#[derive(Debug)]
pub(crate) enum ShaderLoadError {
    Io {
        path: PathBuf,
        error: io::Error,
    },
    TooLarge {
        path: PathBuf,
        size: u64,
        limit: u64,
    },
    GlslCompile {
        path: PathBuf,
        error: String,
    },
    CrossCompile {
        path: PathBuf,
        error: String,
    },
    UnsupportedTarget(Target),
}

impl fmt::Display for ShaderLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, error } => {
                write!(f, "{}: {error}", path.display())
            }
            Self::TooLarge { path, size, limit } => {
                write!(
                    f,
                    "{}: shader is {size} bytes, above {limit} byte limit",
                    path.display()
                )
            }
            Self::GlslCompile { path, error } => {
                write!(f, "{}: GLSL compile failed: {error}", path.display())
            }
            Self::CrossCompile { path, error } => {
                write!(
                    f,
                    "{}: SPIR-V cross-compile failed: {error}",
                    path.display()
                )
            }
            Self::UnsupportedTarget(target) => {
                write!(f, "unsupported custom shader target: {target:?}")
            }
        }
    }
}

impl std::error::Error for ShaderLoadError {}

pub(crate) fn load_from_files(
    paths: &[ConfigFilePath],
    target: Target,
) -> Result<Vec<String>, ShaderLoadError> {
    let mut sources = Vec::new();
    for config_path in paths {
        match load_from_file(Path::new(config_path.path()), target) {
            Ok(source) => sources.push(source),
            Err(ShaderLoadError::Io { error, .. })
                if config_path.optional() && error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(sources)
}

pub(crate) fn load_from_file(path: &Path, target: Target) -> Result<String, ShaderLoadError> {
    let file = File::open(path).map_err(|error| ShaderLoadError::Io {
        path: path.to_path_buf(),
        error,
    })?;
    let mut bytes = Vec::new();
    file.take(MAX_SHADER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| ShaderLoadError::Io {
            path: path.to_path_buf(),
            error,
        })?;
    if bytes.len() as u64 > MAX_SHADER_BYTES {
        return Err(ShaderLoadError::TooLarge {
            path: path.to_path_buf(),
            size: bytes.len() as u64,
            limit: MAX_SHADER_BYTES,
        });
    }
    let source = String::from_utf8(bytes).map_err(|error| ShaderLoadError::Io {
        path: path.to_path_buf(),
        error: io::Error::new(io::ErrorKind::InvalidData, error),
    })?;
    let glsl = glsl_from_shader(&source);
    let spirv = spirv_from_glsl(&glsl, path)?;
    match target {
        Target::Msl => msl_from_spv(&spirv, path),
        Target::Glsl => Err(ShaderLoadError::UnsupportedTarget(target)),
    }
}

pub(crate) fn glsl_from_shader(source: &str) -> String {
    let mut glsl = String::with_capacity(SHADERTOY_PREFIX_GLSL.len() + 1 + source.len());
    glsl.push_str(SHADERTOY_PREFIX_GLSL);
    if !glsl.ends_with('\n') {
        glsl.push('\n');
    }
    glsl.push_str(source);
    glsl
}

fn spirv_from_glsl(source: &str, path: &Path) -> Result<Vec<u32>, ShaderLoadError> {
    let compiler = shaderc::Compiler::new().map_err(|error| ShaderLoadError::GlslCompile {
        path: path.to_path_buf(),
        error: error.to_string(),
    })?;
    let mut options =
        shaderc::CompileOptions::new().map_err(|error| ShaderLoadError::GlslCompile {
            path: path.to_path_buf(),
            error: error.to_string(),
        })?;
    options.set_source_language(shaderc::SourceLanguage::GLSL);
    options.set_target_env(
        shaderc::TargetEnv::Vulkan,
        shaderc::EnvVersion::Vulkan1_2 as u32,
    );
    options.set_target_spirv(shaderc::SpirvVersion::V1_5);

    compiler
        .compile_into_spirv(
            source,
            shaderc::ShaderKind::Fragment,
            &path.to_string_lossy(),
            "main",
            Some(&options),
        )
        .map(|artifact| artifact.as_binary().to_vec())
        .map_err(|error| ShaderLoadError::GlslCompile {
            path: path.to_path_buf(),
            error: error.to_string(),
        })
}

fn msl_from_spv(words: &[u32], path: &Path) -> Result<String, ShaderLoadError> {
    use spirv_cross2::compile::CompilableTarget;

    let module = spirv_cross2::Module::from_words(words);
    let compiler =
        spirv_cross2::Compiler::<spirv_cross2::targets::Msl>::new(module).map_err(|error| {
            ShaderLoadError::CrossCompile {
                path: path.to_path_buf(),
                error: error.to_string(),
            }
        })?;
    let mut options = spirv_cross2::targets::Msl::options();
    options.enable_decoration_binding = true;
    compiler
        .compile(&options)
        .map(|artifact| artifact.as_ref().to_string())
        .map_err(|error| ShaderLoadError::CrossCompile {
            path: path.to_path_buf(),
            error: error.to_string(),
        })
}

/// The uniform struct custom shaders read (upstream `shadertoy.Uniforms`). The
/// `#[repr(C, align(16))]` layout with explicit padding reproduces upstream's
/// `extern struct` `align(16)` field offsets (Rust's `[f32; 4]` has alignment 4,
/// so the padding — not the field alignment — places the vectors at their
/// 16-aligned offsets). `size_of == 4496`, `align_of == 16`.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CustomShaderUniforms {
    pub(crate) resolution: [f32; 3],
    pub(crate) time: f32,
    pub(crate) time_delta: f32,
    pub(crate) frame_rate: f32,
    pub(crate) frame: i32,
    _pad0: [u8; 4],
    pub(crate) channel_time: [[f32; 4]; 4],
    pub(crate) channel_resolution: [[f32; 4]; 4],
    pub(crate) mouse: [f32; 4],
    pub(crate) date: [f32; 4],
    pub(crate) sample_rate: f32,
    _pad1: [u8; 12],
    pub(crate) current_cursor: [f32; 4],
    pub(crate) previous_cursor: [f32; 4],
    pub(crate) current_cursor_color: [f32; 4],
    pub(crate) previous_cursor_color: [f32; 4],
    pub(crate) current_cursor_style: i32,
    pub(crate) previous_cursor_style: i32,
    pub(crate) cursor_visible: i32,
    pub(crate) cursor_change_time: f32,
    pub(crate) time_focus: f32,
    pub(crate) focus: i32,
    _pad2: [u8; 8],
    pub(crate) palette: [[f32; 4]; 256],
    pub(crate) background_color: [f32; 4],
    pub(crate) foreground_color: [f32; 4],
    pub(crate) cursor_color: [f32; 4],
    pub(crate) cursor_text: [f32; 4],
    pub(crate) selection_background_color: [f32; 4],
    pub(crate) selection_foreground_color: [f32; 4],
}

impl CustomShaderUniforms {
    /// The renderer-init defaults (upstream's `init` literal): all zero except
    /// `resolution = [0, 0, 1]`, `frame_rate = 60`, and `focus = 1`.
    pub(crate) fn new() -> Self {
        Self {
            resolution: [0.0, 0.0, 1.0],
            time: 0.0,
            time_delta: 0.0,
            frame_rate: 60.0,
            frame: 0,
            _pad0: [0; 4],
            channel_time: [[0.0; 4]; 4],
            channel_resolution: [[0.0; 4]; 4],
            mouse: [0.0; 4],
            date: [0.0; 4],
            sample_rate: 0.0,
            _pad1: [0; 12],
            current_cursor: [0.0; 4],
            previous_cursor: [0.0; 4],
            current_cursor_color: [0.0; 4],
            previous_cursor_color: [0.0; 4],
            current_cursor_style: 0,
            previous_cursor_style: 0,
            cursor_visible: 0,
            cursor_change_time: 0.0,
            time_focus: 0.0,
            focus: 1,
            _pad2: [0; 8],
            palette: [[0.0; 4]; 256],
            background_color: [0.0; 4],
            foreground_color: [0.0; 4],
            cursor_color: [0.0; 4],
            cursor_text: [0.0; 4],
            selection_background_color: [0.0; 4],
            selection_foreground_color: [0.0; 4],
        }
    }

    /// Update the per-frame time and resolution fields (the time/resolution
    /// group of upstream `updateCustomShaderUniformsForFrame`): `time` (seconds
    /// since the first frame), `time_delta` (seconds since the last frame), the
    /// `frame` counter (incremented), `resolution` (the screen size, `z = 1`),
    /// and `channel_resolution[0]`. The caller owns the clock (computes the
    /// seconds); the cursor-glyph update is a later slice.
    pub(crate) fn update_for_frame(
        &mut self,
        time_secs: f32,
        time_delta_secs: f32,
        screen_width: u32,
        screen_height: u32,
    ) {
        self.time = time_secs;
        self.time_delta = time_delta_secs;
        self.frame += 1;
        let (w, h) = (screen_width as f32, screen_height as f32);
        self.resolution = [w, h, 1.0];
        self.channel_resolution[0] = [w, h, 1.0, 0.0];
    }

    /// Update the cursor uniforms from the cursor glyph (upstream
    /// `updateCustomShaderUniformsForFrame`'s cursor half, Metal
    /// `custom_shader_y_is_down = true`): compute the cursor's pixel rect
    /// (`[left + bearingX, top + cellH - bearingY + glyphH, glyphW, glyphH]`) and
    /// its normalized color; on a change, shift `current` → `previous` and stamp
    /// `cursor_change_time = time`. No cursor glyph → no update.
    pub(crate) fn update_cursor(
        &mut self,
        cursor: Option<CellTextVertex>,
        cell_width: u32,
        cell_height: u32,
        padding_left: u32,
        padding_top: u32,
    ) {
        let Some(cursor) = cursor else {
            return;
        };
        let mut pixel_x = (cursor.grid_pos[0] as u32 * cell_width + padding_left) as f32;
        let mut pixel_y = (cursor.grid_pos[1] as u32 * cell_height + padding_top) as f32;
        pixel_x += f32::from(cursor.bearings[0]);
        // Metal: custom_shader_y_is_down = true.
        pixel_y += cell_height as f32;
        pixel_y -= f32::from(cursor.bearings[1]);
        pixel_y += cursor.glyph_size[1] as f32;

        let new_cursor = [
            pixel_x,
            pixel_y,
            cursor.glyph_size[0] as f32,
            cursor.glyph_size[1] as f32,
        ];
        let cursor_color = [
            f32::from(cursor.color[0]) / 255.0,
            f32::from(cursor.color[1]) / 255.0,
            f32::from(cursor.color[2]) / 255.0,
            f32::from(cursor.color[3]) / 255.0,
        ];

        if new_cursor != self.current_cursor || cursor_color != self.current_cursor_color {
            self.previous_cursor = self.current_cursor;
            self.previous_cursor_color = self.current_cursor_color;
            self.current_cursor = new_cursor;
            self.current_cursor_color = cursor_color;
            self.cursor_change_time = self.time;
        }
    }

    /// Update the focus uniforms (upstream `updateCustomShaderUniformsForFrame`'s
    /// focus block): `focus` is `1` when `focused`, else `0`; `time_focus` is
    /// stamped with the frame `time` when focus was just gained
    /// (`focus_changed && focused`). Returns the new `focus_changed` flag (cleared
    /// to `false` when consumed — upstream resets `custom_shader_focused_changed`).
    pub(crate) fn update_focus(&mut self, focused: bool, focus_changed: bool) -> bool {
        self.focus = i32::from(focused);
        if focus_changed && focused {
            self.time_focus = self.time;
            return false;
        }
        focus_changed
    }

    /// Update the 256-color palette uniform (the palette loop of upstream
    /// `updateCustomShaderUniformsFromState`): each palette color becomes a
    /// `vec4` of the normalized RGB (`channel / 255`) with an opaque alpha.
    pub(crate) fn update_palette(&mut self, palette: &Palette) {
        for (i, color) in palette.iter().enumerate() {
            self.palette[i] = [
                f32::from(color.r) / 255.0,
                f32::from(color.g) / 255.0,
                f32::from(color.b) / 255.0,
                1.0,
            ];
        }
    }

    /// Update the from-state color uniforms (the colors of upstream
    /// `updateCustomShaderUniformsFromState`): `background_color` and
    /// `foreground_color` always; `cursor_color`, `cursor_text`,
    /// `selection_background_color`, and `selection_foreground_color` only when
    /// their value is present (else the prior value is kept). Each is the RGB
    /// normalized to `[0, 1]` with an opaque alpha.
    pub(crate) fn update_state_colors(
        &mut self,
        background: Rgb,
        foreground: Rgb,
        cursor: Option<Rgb>,
        cursor_text: Option<Rgb>,
        selection_background: Option<Rgb>,
        selection_foreground: Option<Rgb>,
    ) {
        self.background_color = normalize_rgb(background);
        self.foreground_color = normalize_rgb(foreground);
        if let Some(c) = cursor {
            self.cursor_color = normalize_rgb(c);
        }
        if let Some(c) = cursor_text {
            self.cursor_text = normalize_rgb(c);
        }
        if let Some(c) = selection_background {
            self.selection_background_color = normalize_rgb(c);
        }
        if let Some(c) = selection_foreground {
            self.selection_foreground_color = normalize_rgb(c);
        }
    }

    /// Update the cursor visibility and style uniforms (the cursor
    /// visibility/style block of upstream `updateCustomShaderUniformsFromState`):
    /// `cursor_visible` is `1` when `visible`, else `0`; `previous_cursor_style`
    /// is always set to the prior `current_cursor_style`, then
    /// `current_cursor_style` to the new style's `shader_int` — unconditionally
    /// (upstream does not guard on a change).
    pub(crate) fn update_cursor_style(&mut self, visible: bool, style: Style) {
        self.cursor_visible = i32::from(visible);
        self.previous_cursor_style = self.current_cursor_style;
        self.current_cursor_style = style.shader_int();
    }
}

/// Normalize an `Rgb` to a `[0, 1]` `vec4` with an opaque alpha
/// (`@floatFromInt(channel) / 255.0`, alpha `1.0`).
fn normalize_rgb(c: Rgb) -> [f32; 4] {
    [
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
        1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        glsl_from_shader, load_from_files, CellTextVertex, CustomShaderUniforms, ShaderLoadError,
        Target,
    };
    use crate::config::ConfigFilePath;
    use std::mem::{align_of, offset_of, size_of};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(name: &str) -> PathBuf {
        let id = TEMP_ID.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "roastty-shadertoy-test-{}-{name}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_shader(value: &str) -> String {
        format!(
            r#"
void mainImage(out vec4 fragColor, in vec2 fragCoord) {{
    vec2 uv = fragCoord.xy / iResolution.xy;
    vec4 terminal = texture(iChannel0, uv);
    fragColor = vec4({value}, terminal.a);
}}
"#
        )
    }

    #[test]
    fn glsl_from_shader_prepends_upstream_prefix() {
        let glsl = glsl_from_shader("void mainImage(out vec4 c, in vec2 p) { c = vec4(p, 0, 1); }");

        assert!(glsl.starts_with("#version 430 core\n"));
        assert!(glsl.contains("layout(binding = 1, std140) uniform Globals"));
        assert!(glsl.contains("layout(binding = 0) uniform sampler2D iChannel0"));
        assert!(glsl.ends_with("void mainImage(out vec4 c, in vec2 p) { c = vec4(p, 0, 1); }"));
    }

    #[test]
    fn load_from_files_skips_optional_missing_and_preserves_order() {
        let dir = temp_dir("order");
        let first = dir.join("first.glsl");
        let second = dir.join("second.glsl");
        std::fs::write(&first, sample_shader("1.0, 0.0, 0.0")).unwrap();
        std::fs::write(&second, sample_shader("0.0, 1.0, 0.0")).unwrap();

        let paths = vec![
            ConfigFilePath::Required(first.to_string_lossy().into_owned()),
            ConfigFilePath::Optional(dir.join("missing.glsl").to_string_lossy().into_owned()),
            ConfigFilePath::Required(second.to_string_lossy().into_owned()),
        ];
        let sources = load_from_files(&paths, Target::Msl).unwrap();

        assert_eq!(sources.len(), 2);
        assert_ne!(sources[0], sources[1]);
        assert!(sources[0].contains("fragment"));
        assert!(sources[1].contains("fragment"));
    }

    #[test]
    fn load_from_files_errors_on_required_missing() {
        let dir = temp_dir("missing");
        let missing = dir.join("missing.glsl");
        let error = load_from_files(
            &[ConfigFilePath::Required(
                missing.to_string_lossy().into_owned(),
            )],
            Target::Msl,
        )
        .unwrap_err();

        assert!(matches!(error, ShaderLoadError::Io { path, error }
            if path == missing && error.kind() == std::io::ErrorKind::NotFound));
    }

    #[test]
    fn load_from_files_reports_invalid_shader_diagnostics() {
        let dir = temp_dir("invalid");
        let path = dir.join("invalid.glsl");
        std::fs::write(&path, "void notMainImage() {}").unwrap();

        let error = load_from_files(
            &[ConfigFilePath::Required(
                path.to_string_lossy().into_owned(),
            )],
            Target::Msl,
        )
        .unwrap_err();

        assert!(
            matches!(error, ShaderLoadError::GlslCompile { path: p, error }
            if p == path && error.contains("mainImage"))
        );
    }

    #[test]
    fn load_from_files_rejects_files_larger_than_four_mib() {
        let dir = temp_dir("too-large");
        let path = dir.join("large.glsl");
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(super::MAX_SHADER_BYTES + 1).unwrap();

        let error = load_from_files(
            &[ConfigFilePath::Required(
                path.to_string_lossy().into_owned(),
            )],
            Target::Msl,
        )
        .unwrap_err();

        assert!(
            matches!(error, ShaderLoadError::TooLarge { path: p, size, limit }
            if p == path && size == super::MAX_SHADER_BYTES + 1 && limit == super::MAX_SHADER_BYTES)
        );
    }

    #[test]
    fn custom_shader_uniforms_layout_matches_extern_struct() {
        assert_eq!(size_of::<CustomShaderUniforms>(), 4496);
        assert_eq!(align_of::<CustomShaderUniforms>(), 16);

        // The std140 field offsets (each `align(16)` field on a 16-multiple).
        assert_eq!(offset_of!(CustomShaderUniforms, resolution), 0);
        assert_eq!(offset_of!(CustomShaderUniforms, time), 12);
        assert_eq!(offset_of!(CustomShaderUniforms, frame), 24);
        assert_eq!(offset_of!(CustomShaderUniforms, channel_time), 32);
        assert_eq!(offset_of!(CustomShaderUniforms, channel_resolution), 96);
        assert_eq!(offset_of!(CustomShaderUniforms, mouse), 160);
        assert_eq!(offset_of!(CustomShaderUniforms, date), 176);
        assert_eq!(offset_of!(CustomShaderUniforms, sample_rate), 192);
        assert_eq!(offset_of!(CustomShaderUniforms, current_cursor), 208);
        assert_eq!(offset_of!(CustomShaderUniforms, current_cursor_style), 272);
        assert_eq!(offset_of!(CustomShaderUniforms, focus), 292);
        assert_eq!(offset_of!(CustomShaderUniforms, palette), 304);
        assert_eq!(offset_of!(CustomShaderUniforms, background_color), 4400);
        assert_eq!(
            offset_of!(CustomShaderUniforms, selection_foreground_color),
            4480
        );
    }

    #[test]
    fn custom_shader_uniforms_new_matches_init_defaults() {
        let u = CustomShaderUniforms::new();
        // All zero except resolution, frame_rate, and focus.
        assert_eq!(u.resolution, [0.0, 0.0, 1.0]);
        assert_eq!(u.frame_rate, 60.0);
        assert_eq!(u.focus, 1);
        // Representative zeroed fields.
        assert_eq!(u.time, 0.0);
        assert_eq!(u.frame, 0);
        assert_eq!(u.cursor_visible, 0);
        assert_eq!(u.palette[0], [0.0; 4]);
        assert_eq!(u.background_color, [0.0; 4]);
    }

    #[test]
    fn update_for_frame_sets_time_and_resolution() {
        let mut u = CustomShaderUniforms::new();

        u.update_for_frame(1.5, 0.016, 800, 600);
        assert_eq!(u.time, 1.5);
        assert_eq!(u.time_delta, 0.016);
        assert_eq!(u.frame, 1);
        assert_eq!(u.resolution, [800.0, 600.0, 1.0]);
        assert_eq!(u.channel_resolution[0], [800.0, 600.0, 1.0, 0.0]);

        // The frame counter increments across calls.
        u.update_for_frame(1.6, 0.016, 800, 600);
        assert_eq!(u.frame, 2);

        // The other fields are untouched.
        assert_eq!(u.focus, 1);
        assert_eq!(u.palette[0], [0.0; 4]);
        assert_eq!(u.channel_resolution[1], [0.0; 4]);
    }

    fn cursor_glyph(
        grid_pos: [u16; 2],
        glyph_size: [u32; 2],
        bearings: [i16; 2],
        color: [u8; 4],
    ) -> CellTextVertex {
        use crate::renderer::shader::{CellTextAtlas, CellTextFlags};
        CellTextVertex {
            glyph_pos: [0, 0],
            glyph_size,
            bearings,
            grid_pos,
            color,
            atlas: CellTextAtlas::Grayscale,
            flags: CellTextFlags::default(),
            _padding: [0, 0],
        }
    }

    #[test]
    fn update_cursor_computes_pixel_rect_and_tracks_changes() {
        let mut u = CustomShaderUniforms::new();
        u.time = 5.0;

        // grid_pos [2,3], glyph_size [10,20], bearings [1,2], red; cell 8×16,
        // padding left 4 / top 5. x = 2·8+4+1 = 21; y = 3·16+5+16−2+20 = 87.
        let glyph = cursor_glyph([2, 3], [10, 20], [1, 2], [255, 0, 0, 255]);
        u.update_cursor(Some(glyph), 8, 16, 4, 5);
        assert_eq!(u.current_cursor, [21.0, 87.0, 10.0, 20.0]);
        assert_eq!(u.current_cursor_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(u.previous_cursor, [0.0; 4]); // the old current
        assert_eq!(u.cursor_change_time, 5.0);

        // The same glyph at a later time → no change (previous and change time
        // stay).
        u.time = 6.0;
        u.update_cursor(Some(glyph), 8, 16, 4, 5);
        assert_eq!(u.previous_cursor, [0.0; 4]);
        assert_eq!(u.cursor_change_time, 5.0);

        // A different glyph → previous becomes the prior current, change time
        // updates.
        let glyph2 = cursor_glyph([0, 0], [10, 20], [0, 0], [0, 0, 255, 255]);
        u.update_cursor(Some(glyph2), 8, 16, 4, 5);
        assert_eq!(u.previous_cursor, [21.0, 87.0, 10.0, 20.0]);
        assert_eq!(u.cursor_change_time, 6.0);

        // None → no-op.
        let before = u;
        u.update_cursor(None, 8, 16, 4, 5);
        assert_eq!(u, before);
    }

    #[test]
    fn update_focus_sets_focus_and_stamps_on_gain() {
        let mut u = CustomShaderUniforms::new();
        u.time = 5.0;

        // Focus gained → focus 1, time_focus stamped, flag consumed.
        assert!(!u.update_focus(true, true));
        assert_eq!(u.focus, 1);
        assert_eq!(u.time_focus, 5.0);

        // Focused, no change → focus 1, time_focus unchanged, flag stays false.
        let mut u = CustomShaderUniforms::new();
        u.time = 5.0;
        assert!(!u.update_focus(true, false));
        assert_eq!(u.focus, 1);
        assert_eq!(u.time_focus, 0.0);

        // Unfocused but "changed" → focus 0, time_focus NOT stamped (not gained),
        // and the flag is NOT consumed (returns true).
        let mut u = CustomShaderUniforms::new();
        u.time = 5.0;
        assert!(u.update_focus(false, true));
        assert_eq!(u.focus, 0);
        assert_eq!(u.time_focus, 0.0);

        // Unfocused, no change → focus 0, returns false.
        assert!(!u.update_focus(false, false));
        assert_eq!(u.focus, 0);

        // The other fields are untouched.
        assert_eq!(u.frame, 0);
        assert_eq!(u.resolution, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn update_palette_normalizes_each_entry() {
        use crate::terminal::color::Rgb;

        let mut palette = [Rgb::new(0, 0, 0); 256];
        palette[5] = Rgb::new(255, 128, 0);
        palette[255] = Rgb::new(0, 0, 255);

        let mut u = CustomShaderUniforms::new();
        u.update_palette(&palette);

        assert_eq!(u.palette[0], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(u.palette[5], [1.0, 128.0 / 255.0, 0.0, 1.0]);
        assert_eq!(u.palette[255], [0.0, 0.0, 1.0, 1.0]);

        // The other fields are untouched.
        assert_eq!(u.background_color, [0.0; 4]);
        assert_eq!(u.focus, 1);
        assert_eq!(u.frame, 0);
    }

    #[test]
    fn update_state_colors_sets_required_and_optional() {
        use crate::terminal::color::Rgb;

        let mut u = CustomShaderUniforms::new();

        // First call: every optional color is `Some`, so each updates its field.
        u.update_state_colors(
            Rgb::new(10, 20, 30),
            Rgb::new(40, 50, 60),
            Some(Rgb::new(255, 0, 0)),
            Some(Rgb::new(0, 128, 255)),
            Some(Rgb::new(0, 255, 0)),
            Some(Rgb::new(64, 64, 64)),
        );
        assert_eq!(
            u.background_color,
            [10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 1.0]
        );
        assert_eq!(
            u.foreground_color,
            [40.0 / 255.0, 50.0 / 255.0, 60.0 / 255.0, 1.0]
        );
        assert_eq!(u.cursor_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(u.cursor_text, [0.0, 128.0 / 255.0, 1.0, 1.0]);
        assert_eq!(u.selection_background_color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(
            u.selection_foreground_color,
            [64.0 / 255.0, 64.0 / 255.0, 64.0 / 255.0, 1.0]
        );

        // Second call: every optional color is `None`, so each keeps its prior
        // (seeded) value; only background/foreground change.
        u.update_state_colors(Rgb::new(1, 2, 3), Rgb::new(4, 5, 6), None, None, None, None);
        assert_eq!(
            u.background_color,
            [1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0, 1.0]
        );
        assert_eq!(
            u.foreground_color,
            [4.0 / 255.0, 5.0 / 255.0, 6.0 / 255.0, 1.0]
        );
        assert_eq!(u.cursor_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(u.cursor_text, [0.0, 128.0 / 255.0, 1.0, 1.0]);
        assert_eq!(u.selection_background_color, [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(
            u.selection_foreground_color,
            [64.0 / 255.0, 64.0 / 255.0, 64.0 / 255.0, 1.0]
        );

        // Unrelated fields are untouched.
        assert_eq!(u.palette[0], [0.0; 4]);
        assert_eq!(u.focus, 1);
    }

    #[test]
    fn update_cursor_style_sets_visibility_and_shifts_unconditionally() {
        use crate::renderer::cursor::Style;

        let mut u = CustomShaderUniforms::new();
        assert_eq!(u.current_cursor_style, 0);
        assert_eq!(u.previous_cursor_style, 0);
        assert_eq!(u.cursor_visible, 0);

        // First update: previous takes the prior current (0), current = Bar (2).
        u.update_cursor_style(true, Style::Bar);
        assert_eq!(u.cursor_visible, 1);
        assert_eq!(u.previous_cursor_style, 0);
        assert_eq!(u.current_cursor_style, 2);

        // Repeated same style still shifts unconditionally: previous = prior
        // current (2), current = Bar (2).
        u.update_cursor_style(true, Style::Bar);
        assert_eq!(u.cursor_visible, 1);
        assert_eq!(u.previous_cursor_style, 2);
        assert_eq!(u.current_cursor_style, 2);

        // Changed style and hidden cursor: previous = prior current (2),
        // current = Underline (3), cursor_visible = 0.
        u.update_cursor_style(false, Style::Underline);
        assert_eq!(u.cursor_visible, 0);
        assert_eq!(u.previous_cursor_style, 2);
        assert_eq!(u.current_cursor_style, 3);

        // cursor_change_time is not touched here, and an unrelated field stays.
        assert_eq!(u.cursor_change_time, 0.0);
        assert_eq!(u.focus, 1);
    }

    #[test]
    fn target_variants_are_distinct() {
        assert_ne!(Target::Glsl, Target::Msl);
        // `Copy` + `Eq`: a trivial round-trip.
        let t = Target::Msl;
        let copied = t;
        assert_eq!(t, copied);
    }
}
