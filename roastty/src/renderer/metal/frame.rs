use objc2::runtime::ProtocolObject;
use objc2_metal::MTLBuffer;

use crate::font::atlas::Atlas;
use crate::renderer::cell::Contents;
use crate::renderer::metal::buffer::{
    FrameCells, MetalBuffer, MetalBufferError, MetalBufferOptions,
};
use crate::renderer::metal::shaders::MetalUniforms;
use crate::renderer::metal::texture::{FrameAtlasTexture, MetalTexture, MetalTextureError};

/// An error syncing the per-frame GPU resources — unifies the buffer and texture
/// error types so [`FrameState::sync`] can compose them with `?` (upstream's
/// single error union).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FrameStateError {
    Buffer(MetalBufferError),
    Texture(MetalTextureError),
}

impl From<MetalBufferError> for FrameStateError {
    fn from(error: MetalBufferError) -> Self {
        FrameStateError::Buffer(error)
    }
}

impl From<MetalTextureError> for FrameStateError {
    fn from(error: MetalTextureError) -> Self {
        FrameStateError::Texture(error)
    }
}

/// The per-frame GPU resources: the uniform buffer, the cell buffers
/// ([`FrameCells`]), and the grayscale/color atlas textures
/// ([`FrameAtlasTexture`]). Mirrors upstream's per-frame `FrameState`; [`sync`]
/// runs `drawFrame`'s per-frame sync block.
///
/// [`sync`]: FrameState::sync
pub(crate) struct FrameState {
    uniforms: MetalBuffer<MetalUniforms>,
    cells: FrameCells,
    grayscale: FrameAtlasTexture,
    color: FrameAtlasTexture,
}

impl FrameState {
    /// Create the per-frame resources: the uniform buffer and cell buffers at the
    /// initial capacity of one element, and the grayscale/color atlas textures
    /// sized to their atlases (not yet uploaded). Background and foreground share
    /// the buffer `options`; the atlas textures use its device and storage mode.
    pub(crate) fn new(
        options: MetalBufferOptions<'_>,
        grayscale_atlas: &Atlas,
        color_atlas: &Atlas,
    ) -> Result<Self, FrameStateError> {
        let device = options.device;
        let storage = options.resource_options.storage_mode;
        Ok(Self {
            uniforms: MetalBuffer::new(options, 1)?,
            cells: FrameCells::new(options)?,
            grayscale: FrameAtlasTexture::new(device, storage, grayscale_atlas)?,
            color: FrameAtlasTexture::new(device, storage, color_atlas)?,
        })
    }

    /// Sync the per-frame GPU resources (upstream's `drawFrame` sync block): the
    /// uniforms (one element), the cells (background + foreground), and both
    /// atlas textures (each gated on its `modified` counter). Returns the
    /// foreground cell count.
    pub(crate) fn sync(
        &mut self,
        options: MetalBufferOptions<'_>,
        uniforms: &MetalUniforms,
        contents: &Contents,
        grayscale_atlas: &Atlas,
        color_atlas: &Atlas,
    ) -> Result<usize, FrameStateError> {
        let device = options.device;
        let storage = options.resource_options.storage_mode;
        self.uniforms
            .sync(options, std::slice::from_ref(uniforms))?;
        let fg_count = self.cells.sync(options, contents)?;
        self.grayscale
            .sync_if_modified(device, storage, grayscale_atlas)?;
        self.color.sync_if_modified(device, storage, color_atlas)?;
        Ok(fg_count)
    }

    /// The uniform buffer (bound at every draw step).
    pub(crate) fn uniforms_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        self.uniforms.buffer()
    }

    /// The cell buffers (the background and cell-text buffers).
    pub(crate) fn cells(&self) -> &FrameCells {
        &self.cells
    }

    /// The grayscale atlas texture (bound at the cell-text draw step).
    pub(crate) fn grayscale_texture(&self) -> &MetalTexture {
        self.grayscale.texture()
    }

    /// The color atlas texture (bound at the cell-text draw step).
    pub(crate) fn color_texture(&self) -> &MetalTexture {
        self.color.texture()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::atlas::Format;
    use crate::renderer::cell::Key;
    use crate::renderer::metal::api::{MetalResourceOptions, MetalStorageMode};
    use crate::renderer::shader::{CellBg, CellTextAtlas, CellTextFlags, CellTextVertex};
    use crate::renderer::size::GridSize;
    use objc2::rc::Retained;
    use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

    fn metal_device() -> Retained<ProtocolObject<dyn MTLDevice>> {
        MTLCreateSystemDefaultDevice().expect("Roastty requires a Metal device")
    }

    fn as_bytes<T>(value: &T) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>())
        }
    }

    fn buffer_bytes(buffer: &ProtocolObject<dyn MTLBuffer>, len: usize) -> Vec<u8> {
        let ptr = buffer.contents().as_ptr().cast::<u8>();
        let mut bytes = vec![0u8; len];
        unsafe {
            std::ptr::copy_nonoverlapping(ptr, bytes.as_mut_ptr(), len);
        }
        bytes
    }

    fn atlas_with_pixel(size: u32, format: Format, data: &[u8]) -> Atlas {
        let mut atlas = Atlas::new(size, format);
        let region = atlas.reserve(1, 1).expect("reserve a 1×1 region");
        atlas.set(region, data);
        atlas
    }

    #[test]
    fn frame_state_sync_syncs_uniforms_cells_and_atlas_textures() {
        let device = metal_device();
        let options = MetalBufferOptions {
            device: &device,
            resource_options: MetalResourceOptions::image(MetalStorageMode::Shared),
        };

        // A grayscale atlas (1 byte/pixel) and a bgra color atlas (4 bytes/pixel),
        // each with a written pixel.
        let grayscale_atlas = atlas_with_pixel(4, Format::Grayscale, &[200]);
        let color_atlas = atlas_with_pixel(4, Format::Bgra, &[10, 20, 30, 40]);

        // A 1×1 Contents: an explicit background cell and one foreground vertex.
        let mut contents = Contents::default();
        contents.resize(GridSize {
            columns: 1,
            rows: 1,
        });
        let bg = CellBg([1, 2, 3, 4]);
        *contents.bg_cell_mut(0, 0) = bg;
        let vertex = CellTextVertex {
            glyph_pos: [0, 0],
            glyph_size: [0, 0],
            bearings: [0, 0],
            grid_pos: [0, 0],
            color: [9, 9, 9, 9],
            atlas: CellTextAtlas::Grayscale,
            flags: CellTextFlags::new(false, false),
            _padding: [0, 0],
        };
        contents.add(Key::Text, vertex);

        let uniforms =
            MetalUniforms::test_with_grid([2, 2], [1, 1], [2.0, 2.0], [0.0; 4], 0, [0, 0, 0, 0]);

        let mut state = FrameState::new(options, &grayscale_atlas, &color_atlas)
            .expect("frame state should be created");
        let fg_count = state
            .sync(
                options,
                &uniforms,
                &contents,
                &grayscale_atlas,
                &color_atlas,
            )
            .expect("frame state sync should succeed");

        // The cell sync returned the foreground count (one vertex).
        assert_eq!(fg_count, 1);

        // The uniforms were synced (one element).
        assert_eq!(
            buffer_bytes(
                state.uniforms_buffer(),
                std::mem::size_of::<MetalUniforms>()
            ),
            as_bytes(&uniforms)
        );

        // The cells' background buffer holds the explicit background cell.
        assert_eq!(
            buffer_bytes(state.cells().bg_buffer(), std::mem::size_of::<CellBg>()),
            as_bytes(&bg)
        );

        // The cells' cell-text buffer holds the foreground vertex.
        assert_eq!(
            buffer_bytes(
                state.cells().text_buffer(),
                std::mem::size_of::<CellTextVertex>()
            ),
            as_bytes(&vertex)
        );

        // Both atlas textures hold their atlas data.
        assert_eq!(
            state.grayscale_texture().read_bytes(),
            grayscale_atlas.data()
        );
        assert_eq!(state.color_texture().read_bytes(), color_atlas.data());
    }
}
