//! Custom (shadertoy-style) shader support.
//!
//! The `CustomShaderUniforms` value type — the uniform struct custom shaders
//! read — and its renderer-init defaults. A faithful port of upstream
//! `renderer/shadertoy.zig`'s `Uniforms` `extern struct`; the per-frame/state
//! update methods, the `Target` enum, and the shader loading are ported in later
//! slices.
#![allow(dead_code)]
// This shadertoy layer is consumed by later slices.

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
}

#[cfg(test)]
mod tests {
    use super::CustomShaderUniforms;
    use std::mem::{align_of, offset_of, size_of};

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
}
