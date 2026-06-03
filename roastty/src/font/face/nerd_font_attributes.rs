//! Per-codepoint sizing/positioning constraints for Nerd Font glyphs.
//!
//! GENERATED from the upstream `nerd_font_attributes.zig` reference (vendored;
//! itself generated from the Nerd Fonts patcher script). DO NOT EDIT BY HAND.
//! Faithful port of upstream `nerd_font_attributes.getConstraint`. Regenerate by
//! reparsing that file's `getConstraint` switch.

use super::constraint::{Align, Constraint, Height, Size};

/// The sizing/positioning [`Constraint`] for the Nerd Font glyph at `cp`,
/// or `None` if `cp` is not a constrained Nerd Font codepoint.
pub(crate) fn get_constraint(cp: u32) -> Option<Constraint> {
    Some(match cp {
        0x2630 => Constraint {
            size: Size::Cover,
            height: Height::Icon,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            pad_left: 0.05,
            pad_right: 0.05,
            pad_top: 0.05,
            pad_bottom: 0.05,
            ..Default::default()
        },
        0x276c..=0x276d => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7142857142857143,
            relative_height: 0.8910614525139665,
            relative_x: 0.1428571428571428,
            relative_y: 0.0349162011173184,
            pad_top: 0.15,
            pad_bottom: 0.15,
            ..Default::default()
        },
        0x276e..=0x276f => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9885714285714285,
            relative_height: 0.8910614525139665,
            relative_x: 0.0057142857142857,
            relative_y: 0.0125698324022346,
            pad_top: 0.15,
            pad_bottom: 0.15,
            ..Default::default()
        },
        0x2770..=0x2771 => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            pad_top: 0.15,
            pad_bottom: 0.15,
            ..Default::default()
        },
        0xe0a0..=0xe0a3 | 0xe0cf => Constraint {
            size: Size::FitCover1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xe0b0 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.03,
            pad_right: -0.03,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0b1 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0b2 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: -0.03,
            pad_right: -0.03,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0b3 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0b4 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.03,
            pad_right: -0.03,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.59),
            ..Default::default()
        },
        0xe0b5 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            max_xy_ratio: Some(0.5),
            ..Default::default()
        },
        0xe0b6 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: -0.03,
            pad_right: -0.03,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.59),
            ..Default::default()
        },
        0xe0b7 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            max_xy_ratio: Some(0.5),
            ..Default::default()
        },
        0xe0b8 | 0xe0bc => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xe0b9 | 0xe0bd => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xe0ba | 0xe0be => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xe0bb | 0xe0bf => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xe0c0 | 0xe0c8 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xe0c1 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xe0c2 | 0xe0ca => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xe0c3 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xe0c4 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            max_xy_ratio: Some(0.86),
            ..Default::default()
        },
        0xe0c5 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            max_xy_ratio: Some(0.86),
            ..Default::default()
        },
        0xe0c6 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            max_xy_ratio: Some(0.78),
            ..Default::default()
        },
        0xe0c7 => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            max_xy_ratio: Some(0.78),
            ..Default::default()
        },
        0xe0cc => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.01,
            pad_right: -0.01,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.85),
            ..Default::default()
        },
        0xe0cd => Constraint {
            size: Size::Stretch,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            max_xy_ratio: Some(0.865),
            ..Default::default()
        },
        0xe0ce | 0xe0d0..=0xe0d1 => Constraint {
            size: Size::FitCover1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xe0d2 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.01,
            pad_right: -0.01,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0d4 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: -0.01,
            pad_right: -0.01,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0d6 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe0d7 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            max_xy_ratio: Some(0.7),
            ..Default::default()
        },
        0xe300 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8984375000000000,
            relative_y: 0.0986328125000000,
            ..Default::default()
        },
        0xe301 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8798828125000000,
            relative_y: 0.1171875000000000,
            ..Default::default()
        },
        0xe302 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7646484375000000,
            relative_y: 0.2314453125000000,
            ..Default::default()
        },
        0xe303 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8789062500000000,
            relative_y: 0.1171875000000000,
            ..Default::default()
        },
        0xe304 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9755859375000000,
            relative_y: 0.0244140625000000,
            ..Default::default()
        },
        0xe305 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9960937500000000,
            relative_y: 0.0019531250000000,
            ..Default::default()
        },
        0xe306 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9863281250000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe307 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9951171875000000,
            relative_y: 0.0039062500000000,
            ..Default::default()
        },
        0xe308 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9785156250000000,
            relative_y: 0.0195312500000000,
            ..Default::default()
        },
        0xe309 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9736328125000000,
            relative_y: 0.0214843750000000,
            ..Default::default()
        },
        0xe30a | 0xe35f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9648437500000000,
            relative_y: 0.0302734375000000,
            ..Default::default()
        },
        0xe30b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8437500000000000,
            relative_y: 0.1513671875000000,
            ..Default::default()
        },
        0xe30c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8027343750000000,
            relative_y: 0.1835937500000000,
            ..Default::default()
        },
        0xe30d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7753906250000000,
            relative_y: 0.1083984375000000,
            ..Default::default()
        },
        0xe30e | 0xe365 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9833984375000000,
            relative_y: 0.0166015625000000,
            ..Default::default()
        },
        0xe30f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9716796875000000,
            relative_y: 0.0263671875000000,
            ..Default::default()
        },
        0xe310 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6621093750000000,
            relative_y: 0.0986328125000000,
            ..Default::default()
        },
        0xe311 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6425781250000000,
            relative_y: 0.1171875000000000,
            ..Default::default()
        },
        0xe312 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5322265625000000,
            relative_y: 0.2314453125000000,
            ..Default::default()
        },
        0xe313 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6416015625000000,
            relative_y: 0.1181640625000000,
            ..Default::default()
        },
        0xe314 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7382812500000000,
            relative_y: 0.0195312500000000,
            ..Default::default()
        },
        0xe315 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6787109375000000,
            relative_y: 0.1357421875000000,
            ..Default::default()
        },
        0xe316 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7480468750000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe317 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7529296875000000,
            relative_y: 0.0048828125000000,
            ..Default::default()
        },
        0xe318 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7314453125000000,
            relative_y: 0.0263671875000000,
            ..Default::default()
        },
        0xe319 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7402343750000000,
            relative_y: 0.0195312500000000,
            ..Default::default()
        },
        0xe31a | 0xe35e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7294921875000000,
            relative_y: 0.0283203125000000,
            ..Default::default()
        },
        0xe31b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6074218750000000,
            relative_y: 0.1503906250000000,
            ..Default::default()
        },
        0xe31c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7363281250000000,
            relative_y: 0.0224609375000000,
            ..Default::default()
        },
        0xe31d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7460937500000000,
            relative_y: 0.0126953125000000,
            ..Default::default()
        },
        0xe31e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.2675781250000000,
            relative_y: 0.3310546875000000,
            ..Default::default()
        },
        0xe31f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7363281250000000,
            relative_y: 0.0986328125000000,
            ..Default::default()
        },
        0xe320 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7177734375000000,
            relative_y: 0.1171875000000000,
            ..Default::default()
        },
        0xe321 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8085937500000000,
            relative_y: 0.0253906250000000,
            ..Default::default()
        },
        0xe322 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7509765625000000,
            relative_y: 0.0839843750000000,
            ..Default::default()
        },
        0xe323 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8281250000000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe324 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8349609375000000,
            ..Default::default()
        },
        0xe325 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8154296875000000,
            relative_y: 0.0214843750000000,
            ..Default::default()
        },
        0xe326 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8144531250000000,
            relative_y: 0.0195312500000000,
            ..Default::default()
        },
        0xe327 | 0xe361 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8076171875000000,
            relative_y: 0.0273437500000000,
            ..Default::default()
        },
        0xe328 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6845703125000000,
            relative_y: 0.1503906250000000,
            ..Default::default()
        },
        0xe329 | 0xe367 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8173828125000000,
            relative_y: 0.0175781250000000,
            ..Default::default()
        },
        0xe32a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8105468750000000,
            relative_y: 0.0263671875000000,
            ..Default::default()
        },
        0xe32b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5175781250000000,
            relative_y: 0.2421875000000000,
            ..Default::default()
        },
        0xe32c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6992187500000000,
            relative_y: 0.1005859375000000,
            ..Default::default()
        },
        0xe32d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6787109375000000,
            relative_y: 0.1201171875000000,
            ..Default::default()
        },
        0xe32e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5654296875000000,
            relative_y: 0.2324218750000000,
            ..Default::default()
        },
        0xe32f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7714843750000000,
            relative_y: 0.0273437500000000,
            ..Default::default()
        },
        0xe330 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7148437500000000,
            relative_y: 0.0830078125000000,
            ..Default::default()
        },
        0xe331 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7919921875000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe332 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7871093750000000,
            relative_y: 0.0126953125000000,
            ..Default::default()
        },
        0xe333 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7714843750000000,
            relative_y: 0.0263671875000000,
            ..Default::default()
        },
        0xe334 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7773437500000000,
            relative_y: 0.0195312500000000,
            ..Default::default()
        },
        0xe335 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7714843750000000,
            relative_y: 0.0283203125000000,
            ..Default::default()
        },
        0xe336 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6503906250000000,
            relative_y: 0.1503906250000000,
            ..Default::default()
        },
        0xe337 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7753906250000000,
            relative_y: 0.0234375000000000,
            ..Default::default()
        },
        0xe338 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7792968750000000,
            relative_y: 0.0185546875000000,
            ..Default::default()
        },
        0xe339 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8445945945945946,
            ..Default::default()
        },
        0xe33a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5283203125000000,
            relative_y: 0.2324218750000000,
            ..Default::default()
        },
        0xe33b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5449218750000000,
            relative_y: 0.2148437500000000,
            ..Default::default()
        },
        0xe33c..=0xe33d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5273437500000000,
            relative_y: 0.2324218750000000,
            ..Default::default()
        },
        0xe33e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.3293918918918919,
            relative_y: 0.6706081081081081,
            ..Default::default()
        },
        0xe33f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5200000000000000,
            relative_y: 0.2707692307692308,
            ..Default::default()
        },
        0xe340 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8307692307692308,
            relative_y: 0.0861538461538462,
            ..Default::default()
        },
        0xe341 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8327702702702703,
            relative_y: 0.0050675675675676,
            ..Default::default()
        },
        0xe344 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5307692307692308,
            relative_y: 0.2092307692307692,
            ..Default::default()
        },
        0xe345 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5332112630208333,
            relative_y: 0.2040934244791667,
            ..Default::default()
        },
        0xe347 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8307692307692308,
            relative_y: 0.1246153846153846,
            ..Default::default()
        },
        0xe349 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5307967032967034,
            relative_y: 0.2615384615384616,
            ..Default::default()
        },
        0xe34c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8659995118379302,
            relative_y: 0.1340004881620698,
            ..Default::default()
        },
        0xe34d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9890163534293386,
            relative_y: 0.0002440810349036,
            ..Default::default()
        },
        0xe34f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5751953125000000,
            relative_y: 0.1142578125000000,
            ..Default::default()
        },
        0xe351 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6533203125000000,
            relative_y: 0.1328125000000000,
            ..Default::default()
        },
        0xe352 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5215384615384615,
            relative_y: 0.2846153846153846,
            ..Default::default()
        },
        0xe353 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8308012820512821,
            relative_y: 0.1230448717948718,
            ..Default::default()
        },
        0xe354..=0xe356 | 0xe358..=0xe359 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9935233160621761,
            relative_y: 0.0025906735751295,
            ..Default::default()
        },
        0xe357 | 0xe3a9 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9961139896373057,
            ..Default::default()
        },
        0xe35a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9935233160621761,
            relative_y: 0.0012953367875648,
            ..Default::default()
        },
        0xe35b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9987046632124352,
            relative_y: 0.0012953367875648,
            ..Default::default()
        },
        0xe360 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7695312500000000,
            relative_y: 0.0302734375000000,
            ..Default::default()
        },
        0xe362 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9902343750000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe363 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7900390625000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe364 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8251953125000000,
            relative_y: 0.0097656250000000,
            ..Default::default()
        },
        0xe366 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7832031250000000,
            relative_y: 0.0166015625000000,
            ..Default::default()
        },
        0xe369 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.4902343750000000,
            relative_y: 0.2548828125000000,
            ..Default::default()
        },
        0xe36b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9333658774713205,
            relative_y: 0.0266048328044911,
            ..Default::default()
        },
        0xe36c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7076171875000000,
            relative_y: 0.1083984375000000,
            ..Default::default()
        },
        0xe36d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8427734375000000,
            relative_y: 0.0625000000000000,
            ..Default::default()
        },
        0xe36e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7529721467391304,
            relative_y: 0.0956606657608696,
            ..Default::default()
        },
        0xe36f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6835937500000000,
            relative_y: 0.1250000000000000,
            ..Default::default()
        },
        0xe370 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8642578125000000,
            relative_y: 0.0625000000000000,
            ..Default::default()
        },
        0xe371 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6103515625000000,
            relative_y: 0.1933593750000000,
            ..Default::default()
        },
        0xe372 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7949218750000000,
            relative_y: 0.0576171875000000,
            ..Default::default()
        },
        0xe373 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8652343750000000,
            relative_y: 0.0058593750000000,
            ..Default::default()
        },
        0xe374 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.3154296875000000,
            relative_y: 0.2861328125000000,
            ..Default::default()
        },
        0xe375 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6772460937500000,
            relative_y: 0.1303710937500000,
            ..Default::default()
        },
        0xe376 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6992187500000000,
            relative_y: 0.1337890625000000,
            ..Default::default()
        },
        0xe377 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7314453125000000,
            relative_y: 0.1552734375000000,
            ..Default::default()
        },
        0xe378 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7314453125000000,
            relative_y: 0.1542968750000000,
            ..Default::default()
        },
        0xe379 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5751953125000000,
            relative_y: 0.1826171875000000,
            ..Default::default()
        },
        0xe37a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5263671875000000,
            relative_y: 0.2285156250000000,
            ..Default::default()
        },
        0xe37b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5751953125000000,
            relative_y: 0.1835937500000000,
            ..Default::default()
        },
        0xe37d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9003906250000000,
            relative_y: 0.0957031250000000,
            ..Default::default()
        },
        0xe37e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6015625000000000,
            relative_y: 0.2324218750000000,
            ..Default::default()
        },
        0xe37f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5200000000000000,
            relative_y: 0.2784615384615385,
            ..Default::default()
        },
        0xe380 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5200000000000000,
            relative_y: 0.2630769230769231,
            ..Default::default()
        },
        0xe38e..=0xe391 | 0xe394 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4990253411306043,
            relative_height: 0.9987012987012988,
            relative_x: 0.4996751137102014,
            ..Default::default()
        },
        0xe392..=0xe393 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4996751137102014,
            relative_height: 0.9987012987012988,
            relative_x: 0.4990253411306043,
            ..Default::default()
        },
        0xe395 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5471085120207927,
            relative_height: 0.9987012987012988,
            relative_x: 0.4515919428200130,
            ..Default::default()
        },
        0xe396 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5945419103313840,
            relative_height: 0.9987012987012988,
            relative_x: 0.4041585445094217,
            ..Default::default()
        },
        0xe397 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6426250812215725,
            relative_x: 0.3573749187784275,
            ..Default::default()
        },
        0xe398 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6900584795321637,
            relative_x: 0.3099415204678362,
            ..Default::default()
        },
        0xe399 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7381416504223521,
            relative_x: 0.2618583495776478,
            ..Default::default()
        },
        0xe39a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7855750487329435,
            relative_x: 0.2144249512670565,
            ..Default::default()
        },
        0xe39b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9987004548408057,
            relative_height: 0.9987012987012988,
            ..Default::default()
        },
        0xe39c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8323586744639376,
            relative_height: 0.9935064935064936,
            ..Default::default()
        },
        0xe39d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7855750487329435,
            relative_height: 0.9948051948051948,
            ..Default::default()
        },
        0xe39e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7381416504223521,
            relative_height: 0.9961038961038962,
            ..Default::default()
        },
        0xe39f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6907082521117609,
            relative_height: 0.9961038961038962,
            ..Default::default()
        },
        0xe3a0 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6426250812215725,
            relative_height: 0.9961038961038962,
            ..Default::default()
        },
        0xe3a1 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5945419103313840,
            relative_height: 0.9974025974025974,
            ..Default::default()
        },
        0xe3a2..=0xe3a3 | 0xe3a5 | 0xe3a7..=0xe3a8 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4990253411306043,
            relative_height: 0.9987012987012988,
            ..Default::default()
        },
        0xe3a4 | 0xe3a6 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4996751137102014,
            relative_height: 0.9987012987012988,
            ..Default::default()
        },
        0xe3aa => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9902343750000000,
            relative_y: 0.0078125000000000,
            ..Default::default()
        },
        0xe3ab => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7900390625000000,
            relative_y: 0.0058593750000000,
            ..Default::default()
        },
        0xe3ac => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8251953125000000,
            relative_y: 0.0078125000000000,
            ..Default::default()
        },
        0xe3ad => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7519531250000000,
            relative_y: 0.0068359375000000,
            ..Default::default()
        },
        0xe3ae => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6152343750000000,
            relative_y: 0.2324218750000000,
            ..Default::default()
        },
        0xe3af | 0xe3b3 | 0xe3b5..=0xe3bb => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9986072423398329,
            relative_y: 0.0013927576601671,
            ..Default::default()
        },
        0xe3b0..=0xe3b2 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9958217270194986,
            relative_y: 0.0041782729805014,
            ..Default::default()
        },
        0xe3c1 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6590187942396876,
            relative_y: 0.1349768123016842,
            ..Default::default()
        },
        0xe3c2 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7939956065413717,
            ..Default::default()
        },
        0x23fb..=0x23fe
        | 0x2665
        | 0x26a1
        | 0x2b58
        | 0xe000..=0xe00a
        | 0xe200..=0xe2a9
        | 0xe342..=0xe343
        | 0xe346
        | 0xe348
        | 0xe34a..=0xe34b
        | 0xe34e
        | 0xe350
        | 0xe35c..=0xe35d
        | 0xe368
        | 0xe36a
        | 0xe37c
        | 0xe381..=0xe38d
        | 0xe3b4
        | 0xe3bc..=0xe3c0
        | 0xe3c3..=0xe3e3
        | 0xe5fa..=0xe6b8
        | 0xe700..=0xe8ef
        | 0xea60
        | 0xea62..=0xea7c
        | 0xea7e..=0xea88
        | 0xea8a..=0xea8c
        | 0xea8f..=0xea98
        | 0xeaa3..=0xeab3
        | 0xeab8..=0xeac7
        | 0xeac9
        | 0xeacc..=0xead3
        | 0xead7..=0xeb09
        | 0xeb0b..=0xeb42
        | 0xeb44..=0xeb4e
        | 0xeb50..=0xeb6d
        | 0xeb72..=0xeb89
        | 0xeb8b..=0xeb99
        | 0xeb9b..=0xebd4
        | 0xebd7..=0xec06
        | 0xec08..=0xec0a
        | 0xec0d..=0xec1e
        | 0xed00..=0xedff
        | 0xee0c..=0xefce
        | 0xf000..=0xf004
        | 0xf006..=0xf025
        | 0xf028..=0xf02a
        | 0xf02c..=0xf030
        | 0xf034
        | 0xf036..=0xf043
        | 0xf045
        | 0xf047
        | 0xf053..=0xf05f
        | 0xf062
        | 0xf064..=0xf076
        | 0xf079..=0xf07d
        | 0xf07f..=0xf088
        | 0xf08a..=0xf0a3
        | 0xf0a6..=0xf0d6
        | 0xf0db
        | 0xf0df..=0xf0ff
        | 0xf108..=0xf12f
        | 0xf131..=0xf140
        | 0xf142..=0xf152
        | 0xf155
        | 0xf15a..=0xf174
        | 0xf176
        | 0xf179..=0xf181
        | 0xf183..=0xf220
        | 0xf223
        | 0xf22e..=0xf254
        | 0xf259
        | 0xf25c..=0xf381
        | 0xf400..=0xf415
        | 0xf417..=0xf423
        | 0xf425..=0xf430
        | 0xf435..=0xf437
        | 0xf439..=0xf43d
        | 0xf43f..=0xf442
        | 0xf446..=0xf449
        | 0xf44c..=0xf45b
        | 0xf45d..=0xf45f
        | 0xf462..=0xf466
        | 0xf468..=0xf46b
        | 0xf46d..=0xf46f
        | 0xf471..=0xf475
        | 0xf477..=0xf479
        | 0xf47f..=0xf48a
        | 0xf48c..=0xf492
        | 0xf494..=0xf499
        | 0xf49b..=0xf4c2
        | 0xf4c4..=0xf4ee
        | 0xf4f3..=0xf51c
        | 0xf51e..=0xf533
        | 0xf0001..=0xf1af0 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            ..Default::default()
        },
        0xea61 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7513020833333334,
            relative_height: 0.9291573452647278,
            relative_x: 0.0846354166666667,
            relative_y: 0.0708426547352722,
            ..Default::default()
        },
        0xea7d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8394854586129754,
            relative_height: 0.8751387347391787,
            relative_x: 0.0917225950782998,
            relative_y: 0.0416204217536071,
            ..Default::default()
        },
        0xea99 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9395973154362416,
            relative_height: 0.4778024417314096,
            relative_x: 0.0302013422818792,
            relative_y: 0.2269700332963374,
            ..Default::default()
        },
        0xea9a | 0xeaa1 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7673378076062640,
            relative_height: 0.8523862375138734,
            relative_x: 0.1526845637583893,
            relative_y: 0.0754716981132075,
            ..Default::default()
        },
        0xea9b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8590604026845637,
            relative_height: 0.7613762486126526,
            relative_x: 0.0721476510067114,
            relative_y: 0.0871254162042175,
            ..Default::default()
        },
        0xea9c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8590604026845637,
            relative_height: 0.7574916759156493,
            relative_x: 0.0721476510067114,
            relative_y: 0.0832408435072142,
            ..Default::default()
        },
        0xea9d | 0xeaa0 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4082774049217002,
            relative_height: 0.5077691453940066,
            relative_x: 0.2863534675615212,
            relative_y: 0.2763596004439512,
            ..Default::default()
        },
        0xea9e..=0xea9f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5117449664429530,
            relative_height: 0.4051054384017758,
            relative_x: 0.2136465324384788,
            relative_y: 0.3068812430632630,
            ..Default::default()
        },
        0xeaa2 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8061116965226555,
            relative_height: 0.9438247156716689,
            relative_x: 0.0679662802950474,
            relative_y: 0.0147523709167545,
            ..Default::default()
        },
        0xeab4 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9945482866043613,
            relative_height: 0.5264797507788161,
            relative_y: 0.2024922118380062,
            ..Default::default()
        },
        0xeab5 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5264797507788161,
            relative_height: 0.9945482866043613,
            relative_x: 0.2024922118380062,
            relative_y: 0.0054517133956386,
            ..Default::default()
        },
        0xeab6 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5264797507788161,
            relative_height: 0.9945482866043613,
            relative_x: 0.2710280373831775,
            ..Default::default()
        },
        0xeab7 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9945482866043613,
            relative_height: 0.5264797507788161,
            relative_x: 0.0054517133956386,
            relative_y: 0.2710280373831775,
            ..Default::default()
        },
        0xead4..=0xead5 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7069825436408977,
            relative_x: 0.1483790523690773,
            ..Default::default()
        },
        0xead6 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8780760626398211,
            relative_y: 0.0687919463087248,
            ..Default::default()
        },
        0xeb43 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7335766423357665,
            relative_height: 0.9996188152778837,
            relative_x: 0.1991657977059437,
            relative_y: 0.0003811847221163,
            ..Default::default()
        },
        0xeb6e | 0xeb71 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.4954604409857328,
            relative_y: 0.2522697795071336,
            ..Default::default()
        },
        0xeb6f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4973958333333333,
            relative_x: 0.2493489583333333,
            ..Default::default()
        },
        0xeb70 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4973958333333333,
            relative_height: 0.9961089494163424,
            relative_x: 0.2493489583333333,
            relative_y: 0.0038910505836576,
            ..Default::default()
        },
        0xeb8a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.3468834688346883,
            relative_height: 0.3353615785256410,
            relative_x: 0.2642276422764228,
            relative_y: 0.3313050881410256,
            ..Default::default()
        },
        0xeb9a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8740779768177028,
            relative_height: 0.9438247156716689,
            relative_x: 0.0679662802950474,
            relative_y: 0.0147523709167545,
            ..Default::default()
        },
        0xebd5 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9322210636079249,
            relative_height: 0.9318897917604415,
            relative_y: 0.0681102082395584,
            ..Default::default()
        },
        0xebd6 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9996446423917936,
            relative_y: 0.0003553576082064,
            ..Default::default()
        },
        0xec07 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.3495911047345768,
            relative_height: 0.3355179398148149,
            relative_x: 0.2615335565120357,
            relative_y: 0.3311487268518519,
            ..Default::default()
        },
        0xec0b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9327424400417101,
            relative_height: 0.9996188152778837,
            relative_y: 0.0003811847221163,
            ..Default::default()
        },
        0xec0c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8008342022940563,
            relative_x: 0.1991657977059437,
            ..Default::default()
        },
        0xee00 | 0xee03 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::End,
            align_vertical: Align::Center1,
            relative_width: 0.8681172291296625,
            relative_height: 0.8626692456479691,
            relative_x: 0.1314387211367673,
            relative_y: 0.0686653771760155,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xee01 | 0xee04 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8626692456479691,
            relative_y: 0.0686653771760155,
            pad_left: -0.05,
            pad_right: -0.05,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xee02 | 0xee05 => Constraint {
            size: Size::Stretch,
            max_constraint_width: 1,
            align_horizontal: Align::Start,
            align_vertical: Align::Center1,
            relative_width: 0.8685612788632326,
            relative_height: 0.8626692456479691,
            relative_y: 0.0686653771760155,
            pad_left: -0.025,
            pad_right: -0.025,
            pad_top: -0.005,
            pad_bottom: -0.005,
            ..Default::default()
        },
        0xee06 => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7059415911379657,
            relative_height: 0.2234524408656266,
            relative_x: 0.1470292044310171,
            relative_y: 0.7765475591343735,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            ..Default::default()
        },
        0xee07 => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5000000000000000,
            relative_height: 0.7498741821841973,
            relative_x: 0.5000000000000000,
            relative_y: 0.2501258178158027,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            ..Default::default()
        },
        0xee08 => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6299093655589124,
            relative_height: 0.8535480624056366,
            relative_x: 0.3700906344410876,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            ..Default::default()
        },
        0xee09 => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.4997483643683945,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            ..Default::default()
        },
        0xee0a => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6299093655589124,
            relative_height: 0.8535480624056366,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            ..Default::default()
        },
        0xee0b => Constraint {
            size: Size::Cover,
            max_constraint_width: 1,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5000000000000000,
            relative_height: 0.7498741821841973,
            relative_y: 0.2501258178158027,
            pad_left: 0.015,
            pad_right: 0.015,
            pad_top: 0.015,
            pad_bottom: 0.015,
            ..Default::default()
        },
        0xf005 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9999664113932554,
            relative_y: 0.0000335886067446,
            ..Default::default()
        },
        0xf026..=0xf027 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9786184354605580,
            relative_y: 0.0103951316192896,
            ..Default::default()
        },
        0xf02b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9758052740827267,
            relative_y: 0.0238869355863696,
            ..Default::default()
        },
        0xf031..=0xf033 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9987922705314010,
            relative_y: 0.0006038647342995,
            ..Default::default()
        },
        0xf035 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9989935587761675,
            relative_y: 0.0004025764895330,
            ..Default::default()
        },
        0xf044 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9925925925925926,
            ..Default::default()
        },
        0xf046 | 0xf153..=0xf154 | 0xf158 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8751322751322751,
            relative_y: 0.0624338624338624,
            ..Default::default()
        },
        0xf048 | 0xf04a | 0xf04e | 0xf051 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8577706898990622,
            relative_y: 0.0711892586341537,
            ..Default::default()
        },
        0xf049 | 0xf050 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8579450878868969,
            relative_y: 0.0710148606463189,
            ..Default::default()
        },
        0xf04b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9997041418532618,
            relative_y: 0.0002958581467381,
            ..Default::default()
        },
        0xf04c..=0xf04d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8572940020656472,
            relative_y: 0.0713404035569438,
            ..Default::default()
        },
        0xf04f => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7138835298072554,
            relative_y: 0.1433479295317200,
            ..Default::default()
        },
        0xf052 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9999748091795350,
            ..Default::default()
        },
        0xf060..=0xf061 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8567975830815709,
            relative_y: 0.0719033232628399,
            ..Default::default()
        },
        0xf063 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9987915407854985,
            relative_y: 0.0006042296072508,
            ..Default::default()
        },
        0xf077 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5700483091787439,
            relative_y: 0.2862318840579710,
            ..Default::default()
        },
        0xf078 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5700483091787439,
            relative_y: 0.1437198067632850,
            ..Default::default()
        },
        0xf07e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.4989429175475687,
            relative_y: 0.2505285412262157,
            ..Default::default()
        },
        0xf089 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9998488512696494,
            relative_y: 0.0001511487303507,
            ..Default::default()
        },
        0xf0a4..=0xf0a5 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7502645502645503,
            relative_y: 0.1248677248677249,
            ..Default::default()
        },
        0xf0d7 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.4281400966183575,
            relative_y: 0.2053140096618357,
            ..Default::default()
        },
        0xf0d8 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.4281400966183575,
            relative_y: 0.3472222222222222,
            ..Default::default()
        },
        0xf0d9 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7140772371750631,
            relative_y: 0.1333462732919255,
            ..Default::default()
        },
        0xf0da => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7140396210163651,
            relative_y: 0.1333838894506235,
            ..Default::default()
        },
        0xf0dc => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            ..Default::default()
        },
        0xf0dd => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            relative_height: 0.4275362318840580,
            relative_y: 0.0012077294685990,
            ..Default::default()
        },
        0xf0de => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            relative_height: 0.4287439613526570,
            relative_y: 0.5712560386473430,
            ..Default::default()
        },
        0xf100..=0xf101 | 0xf104..=0xf105 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8573155985489722,
            relative_y: 0.0713422007255139,
            ..Default::default()
        },
        0xf102 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9286577992744861,
            relative_y: 0.0713422007255139,
            ..Default::default()
        },
        0xf103 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9286577992744861,
            ..Default::default()
        },
        0xf106..=0xf107 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.5000000000000000,
            relative_y: 0.2853688029020556,
            ..Default::default()
        },
        0xf130 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9998602571268865,
            ..Default::default()
        },
        0xf141 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.2593984962406015,
            relative_y: 0.3696741854636592,
            ..Default::default()
        },
        0xf156 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8752505446623093,
            relative_y: 0.0623155929038282,
            ..Default::default()
        },
        0xf157 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8756468797564688,
            relative_y: 0.0624338624338624,
            ..Default::default()
        },
        0xf159 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8756067947646895,
            relative_y: 0.0623492063492063,
            ..Default::default()
        },
        0xf175 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9989423585404548,
            relative_y: 0.0005288207297726,
            ..Default::default()
        },
        0xf177..=0xf178 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6250661025912215,
            relative_y: 0.1877313590692755,
            ..Default::default()
        },
        0xf182 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9998046921689268,
            ..Default::default()
        },
        0xf221 | 0xf224..=0xf226 | 0xf228 | 0xf22a | 0xf22c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9994854643684076,
            ..Default::default()
        },
        0xf222 | 0xf227 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.8746819883943630,
            relative_y: 0.0624017379870223,
            ..Default::default()
        },
        0xf229 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9370837263813853,
            relative_y: 0.0624017379870223,
            ..Default::default()
        },
        0xf22b | 0xf22d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6874767744332962,
            relative_y: 0.1560043449675557,
            ..Default::default()
        },
        0xf255..=0xf256 | 0xf25a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9993997599039616,
            ..Default::default()
        },
        0xf257 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7810124049619848,
            relative_y: 0.0935945806894186,
            ..Default::default()
        },
        0xf258 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.7498142113988452,
            relative_y: 0.1247927742525582,
            ..Default::default()
        },
        0xf25b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.9975006099019084,
            ..Default::default()
        },
        0xf416 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_height: 0.6090604026845637,
            relative_y: 0.2119686800894855,
            ..Default::default()
        },
        0xf424 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5019531250000000,
            relative_height: 0.5755033557046980,
            relative_x: 0.2480468750000000,
            relative_y: 0.2108501118568233,
            ..Default::default()
        },
        0xf431 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6240234375000000,
            relative_height: 0.7695749440715883,
            relative_x: 0.2031250000000000,
            relative_y: 0.1420581655480984,
            ..Default::default()
        },
        0xf432 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6718750000000000,
            relative_height: 0.7147651006711410,
            relative_x: 0.1875000000000000,
            relative_y: 0.1610738255033557,
            ..Default::default()
        },
        0xf433 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6240234375000000,
            relative_height: 0.7695749440715883,
            relative_x: 0.2041015625000000,
            relative_y: 0.0883668903803132,
            ..Default::default()
        },
        0xf434 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6718750000000000,
            relative_height: 0.7147651006711410,
            relative_x: 0.1406250000000000,
            relative_y: 0.1599552572706935,
            ..Default::default()
        },
        0xf438 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.2436523437500000,
            relative_height: 0.4560546875000000,
            relative_x: 0.3813476562500000,
            relative_y: 0.2719726562500000,
            ..Default::default()
        },
        0xf43e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5029296875000000,
            relative_height: 0.5755033557046980,
            relative_x: 0.2500000000000000,
            relative_y: 0.2136465324384788,
            ..Default::default()
        },
        0xf443 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7500000000000000,
            relative_x: 0.1250000000000000,
            ..Default::default()
        },
        0xf444..=0xf445 | 0xf4c3 | 0xf51d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5000000000000000,
            relative_height: 0.5000000000000000,
            relative_x: 0.2500000000000000,
            relative_y: 0.2500000000000000,
            ..Default::default()
        },
        0xf44a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.2436523437500000,
            relative_height: 0.4560546875000000,
            relative_x: 0.3750000000000000,
            relative_y: 0.2719726562500000,
            ..Default::default()
        },
        0xf44b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4560546875000000,
            relative_height: 0.2436523437500000,
            relative_x: 0.2719726562500000,
            relative_y: 0.3188476562500000,
            ..Default::default()
        },
        0xf45c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5019531250000000,
            relative_height: 0.5749440715883669,
            relative_x: 0.2480468750000000,
            relative_y: 0.2114093959731544,
            ..Default::default()
        },
        0xf460 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.3593750000000000,
            relative_height: 0.6240234375000000,
            relative_x: 0.3750000000000000,
            relative_y: 0.1884765625000000,
            ..Default::default()
        },
        0xf461 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6237816764132553,
            relative_height: 0.9988851727982163,
            relative_x: 0.1881091617933723,
            ..Default::default()
        },
        0xf467 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5639648437500000,
            relative_height: 0.5649414062500000,
            relative_x: 0.2187500000000000,
            relative_y: 0.2177734375000000,
            ..Default::default()
        },
        0xf46c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5039062500000000,
            relative_height: 0.5771812080536913,
            relative_x: 0.2490234375000000,
            relative_y: 0.2091722595078300,
            ..Default::default()
        },
        0xf470 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9926757812500000,
            relative_height: 0.2690429687500000,
            relative_y: 0.6865234375000000,
            ..Default::default()
        },
        0xf476 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8732325694783033,
            relative_x: 0.0633837152608484,
            ..Default::default()
        },
        0xf47a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.5843079922027290,
            relative_height: 0.9509476031215162,
            relative_x: 0.2066276803118908,
            relative_y: 0.0234113712374582,
            ..Default::default()
        },
        0xf47b..=0xf47c => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.6250000000000000,
            relative_height: 0.3593750000000000,
            relative_x: 0.1875000000000000,
            relative_y: 0.3281250000000000,
            ..Default::default()
        },
        0xf47d => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.3593750000000000,
            relative_height: 0.6240234375000000,
            relative_x: 0.2656250000000000,
            relative_y: 0.1875000000000000,
            ..Default::default()
        },
        0xf47e => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.4560546875000000,
            relative_height: 0.2436523437500000,
            relative_x: 0.2719726562500000,
            relative_y: 0.3750000000000000,
            ..Default::default()
        },
        0xf48b => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7187500000000000,
            relative_height: 0.0937500000000000,
            relative_x: 0.1250000000000000,
            relative_y: 0.4687500000000000,
            ..Default::default()
        },
        0xf493 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8313840155945419,
            relative_height: 0.9509476031215162,
            relative_x: 0.0843079922027290,
            relative_y: 0.0234113712374582,
            ..Default::default()
        },
        0xf49a => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.8727450024378351,
            relative_x: 0.0633837152608484,
            ..Default::default()
        },
        0xf4ef | 0xf4f2 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.7142857142857143,
            relative_x: 0.1428571428571428,
            ..Default::default()
        },
        0xf4f0 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9642857142857143,
            relative_height: 0.7407407407407407,
            relative_y: 0.1111111111111111,
            ..Default::default()
        },
        0xf4f1 => Constraint {
            size: Size::FitCover1,
            height: Height::Icon,
            align_horizontal: Align::Center1,
            align_vertical: Align::Center1,
            relative_width: 0.9642857142857143,
            relative_height: 0.7407407407407407,
            relative_x: 0.0357142857142857,
            relative_y: 0.1111111111111111,
            ..Default::default()
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_constraint_known() {
        // The hexagram (0x2630): cover + icon, one cell, centered, 0.05 pads.
        assert_eq!(
            get_constraint(0x2630),
            Some(Constraint {
                size: Size::Cover,
                height: Height::Icon,
                max_constraint_width: 1,
                align_horizontal: Align::Center1,
                align_vertical: Align::Center1,
                pad_left: 0.05,
                pad_right: 0.05,
                pad_top: 0.05,
                pad_bottom: 0.05,
                ..Default::default()
            })
        );
        // 0xE0B0 (powerline left solid): stretch, start-aligned, negative pads,
        // a max aspect ratio.
        assert_eq!(
            get_constraint(0xE0B0),
            Some(Constraint {
                size: Size::Stretch,
                max_constraint_width: 1,
                align_horizontal: Align::Start,
                align_vertical: Align::Center1,
                pad_left: -0.03,
                pad_right: -0.03,
                pad_top: -0.005,
                pad_bottom: -0.005,
                max_xy_ratio: Some(0.7),
                ..Default::default()
            })
        );
        // 0xE0A0 (powerline branch): fit_cover1, centered (a range arm).
        assert_eq!(
            get_constraint(0xE0A0),
            Some(Constraint {
                size: Size::FitCover1,
                align_horizontal: Align::Center1,
                align_vertical: Align::Center1,
                ..Default::default()
            })
        );
        // The two codepoints upstream's own face tests probe resolve.
        assert!(get_constraint(0xEA61).is_some());
        assert!(get_constraint(0xE0C0).is_some());
    }

    #[test]
    fn get_constraint_none() {
        for cp in [0x41u32, 0x2500, 0x1F600, 0x0] {
            assert_eq!(get_constraint(cp), None, "{cp:#x} is not a Nerd glyph");
        }
    }

    #[test]
    fn get_constraint_ranges() {
        // 0xE0A0..=0xE0A3 | 0xE0CF share one constraint: both ends and the
        // joined singleton match, and the neighbor 0xE0A4 differs.
        let c = get_constraint(0xE0A0);
        assert!(c.is_some());
        assert_eq!(get_constraint(0xE0A3), c, "range end matches");
        assert_eq!(get_constraint(0xE0CF), c, "joined singleton matches");
        assert_ne!(get_constraint(0xE0A4), c, "the neighbor is a different arm");
    }

    #[test]
    fn get_constraint_defaults_match() {
        // 0xE0A0's arm sets no relative_* or max_xy_ratio, so those take the
        // struct defaults — matching upstream's unset fields.
        let c = get_constraint(0xE0A0).unwrap();
        assert_eq!(c.relative_width, 1.0);
        assert_eq!(c.relative_height, 1.0);
        assert_eq!(c.relative_x, 0.0);
        assert_eq!(c.relative_y, 0.0);
        assert_eq!(c.max_xy_ratio, None);
        assert_eq!(c.max_constraint_width, 2);
        assert_eq!(c.height, Height::Cell);
    }
}
