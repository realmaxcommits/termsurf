#![allow(dead_code)]
// Pipeline descriptor values are consumed by later renderer slices.

use crate::renderer::metal::api::{
    MetalBlendFactor, MetalBlendOperation, MetalPixelFormat, MetalVertexFormat,
    MetalVertexStepFunction,
};
use crate::renderer::shader::ImageVertex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MetalVertexAttribute {
    pub(crate) format: MetalVertexFormat,
    pub(crate) offset: usize,
    pub(crate) buffer_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MetalVertexLayout {
    pub(crate) stride: usize,
    pub(crate) step_function: MetalVertexStepFunction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MetalVertexDescriptor {
    pub(crate) attributes: Vec<MetalVertexAttribute>,
    pub(crate) layout: MetalVertexLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetalPipelineAttachmentOptions {
    pub(crate) pixel_format: MetalPixelFormat,
    pub(crate) blending_enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetalPipelineAttachmentDescriptor {
    pub(crate) pixel_format: MetalPixelFormat,
    pub(crate) blending_enabled: bool,
    pub(crate) blend: Option<MetalBlendDescriptor>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetalBlendDescriptor {
    pub(crate) rgb_operation: MetalBlendOperation,
    pub(crate) alpha_operation: MetalBlendOperation,
    pub(crate) source_rgb_factor: MetalBlendFactor,
    pub(crate) source_alpha_factor: MetalBlendFactor,
    pub(crate) destination_rgb_factor: MetalBlendFactor,
    pub(crate) destination_alpha_factor: MetalBlendFactor,
}

pub(crate) fn pipeline_attachment_descriptor(
    options: MetalPipelineAttachmentOptions,
) -> MetalPipelineAttachmentDescriptor {
    MetalPipelineAttachmentDescriptor {
        pixel_format: options.pixel_format,
        blending_enabled: options.blending_enabled,
        blend: options
            .blending_enabled
            .then_some(premultiplied_alpha_blend()),
    }
}

fn premultiplied_alpha_blend() -> MetalBlendDescriptor {
    MetalBlendDescriptor {
        rgb_operation: MetalBlendOperation::Add,
        alpha_operation: MetalBlendOperation::Add,
        source_rgb_factor: MetalBlendFactor::One,
        source_alpha_factor: MetalBlendFactor::One,
        destination_rgb_factor: MetalBlendFactor::OneMinusSourceAlpha,
        destination_alpha_factor: MetalBlendFactor::OneMinusSourceAlpha,
    }
}

pub(crate) trait MetalVertexInput {
    fn vertex_descriptor(step_function: MetalVertexStepFunction) -> MetalVertexDescriptor;
}

impl MetalVertexInput for ImageVertex {
    fn vertex_descriptor(step_function: MetalVertexStepFunction) -> MetalVertexDescriptor {
        MetalVertexDescriptor {
            attributes: vec![
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float2,
                    offset: std::mem::offset_of!(ImageVertex, grid_pos),
                    buffer_index: 0,
                },
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float2,
                    offset: std::mem::offset_of!(ImageVertex, cell_offset),
                    buffer_index: 0,
                },
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float4,
                    offset: std::mem::offset_of!(ImageVertex, source_rect),
                    buffer_index: 0,
                },
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float2,
                    offset: std::mem::offset_of!(ImageVertex, dest_size),
                    buffer_index: 0,
                },
            ],
            layout: MetalVertexLayout {
                stride: std::mem::size_of::<ImageVertex>(),
                step_function,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_vertex_descriptor_maps_fields_to_upstream_attributes() {
        let descriptor = ImageVertex::vertex_descriptor(MetalVertexStepFunction::PerVertex);

        assert_eq!(
            descriptor.attributes,
            vec![
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float2,
                    offset: std::mem::offset_of!(ImageVertex, grid_pos),
                    buffer_index: 0,
                },
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float2,
                    offset: std::mem::offset_of!(ImageVertex, cell_offset),
                    buffer_index: 0,
                },
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float4,
                    offset: std::mem::offset_of!(ImageVertex, source_rect),
                    buffer_index: 0,
                },
                MetalVertexAttribute {
                    format: MetalVertexFormat::Float2,
                    offset: std::mem::offset_of!(ImageVertex, dest_size),
                    buffer_index: 0,
                },
            ]
        );
        assert_eq!(
            descriptor.layout,
            MetalVertexLayout {
                stride: std::mem::size_of::<ImageVertex>(),
                step_function: MetalVertexStepFunction::PerVertex,
            }
        );
    }

    #[test]
    fn image_vertex_descriptor_preserves_attributes_for_per_instance_step() {
        let per_vertex = ImageVertex::vertex_descriptor(MetalVertexStepFunction::PerVertex);
        let per_instance = ImageVertex::vertex_descriptor(MetalVertexStepFunction::PerInstance);

        assert_eq!(per_instance.attributes, per_vertex.attributes);
        assert_eq!(
            per_instance.layout.stride,
            std::mem::size_of::<ImageVertex>()
        );
        assert_eq!(
            per_instance.layout.step_function,
            MetalVertexStepFunction::PerInstance
        );
    }

    #[test]
    fn enabled_attachment_uses_upstream_premultiplied_alpha_blend() {
        let descriptor = pipeline_attachment_descriptor(MetalPipelineAttachmentOptions {
            pixel_format: MetalPixelFormat::Rgba8Unorm,
            blending_enabled: true,
        });

        assert_eq!(
            descriptor,
            MetalPipelineAttachmentDescriptor {
                pixel_format: MetalPixelFormat::Rgba8Unorm,
                blending_enabled: true,
                blend: Some(MetalBlendDescriptor {
                    rgb_operation: MetalBlendOperation::Add,
                    alpha_operation: MetalBlendOperation::Add,
                    source_rgb_factor: MetalBlendFactor::One,
                    source_alpha_factor: MetalBlendFactor::One,
                    destination_rgb_factor: MetalBlendFactor::OneMinusSourceAlpha,
                    destination_alpha_factor: MetalBlendFactor::OneMinusSourceAlpha,
                }),
            }
        );
    }

    #[test]
    fn disabled_attachment_has_no_blend_descriptor() {
        let descriptor = pipeline_attachment_descriptor(MetalPipelineAttachmentOptions {
            pixel_format: MetalPixelFormat::Bgra8Unorm,
            blending_enabled: false,
        });

        assert_eq!(
            descriptor,
            MetalPipelineAttachmentDescriptor {
                pixel_format: MetalPixelFormat::Bgra8Unorm,
                blending_enabled: false,
                blend: None,
            }
        );
    }

    #[test]
    fn attachment_pixel_formats_pass_through_unchanged() {
        assert_eq!(
            pipeline_attachment_descriptor(MetalPipelineAttachmentOptions {
                pixel_format: MetalPixelFormat::Rgba8Unorm,
                blending_enabled: true,
            })
            .pixel_format,
            MetalPixelFormat::Rgba8Unorm
        );
        assert_eq!(
            pipeline_attachment_descriptor(MetalPipelineAttachmentOptions {
                pixel_format: MetalPixelFormat::Bgra8Unorm,
                blending_enabled: true,
            })
            .pixel_format,
            MetalPixelFormat::Bgra8Unorm
        );
    }
}
