//! A texture atlas (<https://en.wikipedia.org/wiki/Texture_atlas>).
//!
//! Faithful port of upstream `font/Atlas.zig`. The packer is the skyline /
//! shelf-next-fit variant from Jukka Jylänki's "A Thousand Ways to Pack the
//! Bin" (as used by freetype-gl): the atlas hands out sub-rectangles of a square
//! texture for glyph sprites.
//!
//! Limitations carried over from upstream: written data must be packed (no
//! custom strides), and the texture is always square (regions written into it
//! need not be).
//!
//! This slice ports the allocation core (`new`/`clear`, `reserve` with `fit`
//! and `merge`, and `set`). `grow`, `set_from_larger`, and `dump` land in a
//! later experiment; the WASM bindings are out of scope (macOS-only).

use std::sync::atomic::{AtomicUsize, Ordering};

/// The pixel format of the texture data written into the atlas. This is uniform
/// for all textures in one atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Format {
    /// 1 byte per pixel grayscale.
    Grayscale = 0,
    /// 3 bytes per pixel BGR.
    Bgr = 1,
    /// 4 bytes per pixel BGRA.
    Bgra = 2,
}

impl Format {
    /// Bytes per pixel for this format. Returned as `u32` so it composes with the
    /// offset arithmetic without casts.
    pub(crate) fn depth(self) -> u32 {
        match self {
            Format::Grayscale => 1,
            Format::Bgr => 3,
            Format::Bgra => 4,
        }
    }
}

/// A node (rectangle) of available space on the skyline frontier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    x: u32,
    y: u32,
    width: u32,
}

/// A reserved region within the texture atlas, acquired from [`Atlas::reserve`].
/// A region reservation is required to write data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// An error reserving space in the atlas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AtlasError {
    /// The atlas cannot fit the desired region. The atlas must be enlarged.
    AtlasFull,
}

/// Number of nodes to preallocate in the node list on construction.
pub(crate) const NODE_PREALLOC: usize = 64;

/// A square texture atlas.
pub(crate) struct Atlas {
    /// The raw texture data.
    data: Vec<u8>,
    /// Width and height of the (always square) atlas texture.
    size: u32,
    /// The nodes (rectangles) of available space.
    nodes: Vec<Node>,
    /// The format of the texture data being written into the atlas.
    format: Format,
    /// Incremented on every data change, so a GPU-upload consumer can observe
    /// that the texture data changed since it was last sent. Read atomically.
    modified: AtomicUsize,
    /// Incremented on every resize, so a consumer can tell whether a GPU texture
    /// can be updated in-place or needs reallocation. Read atomically.
    resized: AtomicUsize,
}

impl Atlas {
    /// Create an atlas of `size` × `size` pixels in the given format. All data is
    /// zeroed and a single full-texture node (inside a 1px border) is seeded.
    ///
    /// Allocation is infallible in Rust (it aborts on OOM rather than returning
    /// an error), so unlike upstream `init` this returns an `Atlas` directly.
    pub(crate) fn new(size: u32, format: Format) -> Atlas {
        let depth = format.depth() as usize;
        let mut atlas = Atlas {
            data: vec![0u8; size as usize * size as usize * depth],
            size,
            nodes: Vec::with_capacity(NODE_PREALLOC),
            format,
            modified: AtomicUsize::new(0),
            resized: AtomicUsize::new(0),
        };

        // Sets up the initial node state.
        atlas.clear();

        atlas
    }

    /// The current value of the `modified` counter.
    pub(crate) fn modified(&self) -> usize {
        self.modified.load(Ordering::Relaxed)
    }

    /// The current value of the `resized` counter.
    pub(crate) fn resized(&self) -> usize {
        self.resized.load(Ordering::Relaxed)
    }

    /// Reserve a region of `width` × `height` within the atlas.
    ///
    /// May grow the internal node list. This does not enlarge the texture if it
    /// is full; it returns [`AtlasError::AtlasFull`] instead.
    pub(crate) fn reserve(&mut self, width: u32, height: u32) -> Result<Region, AtlasError> {
        // x, y are populated within the best-index search below.
        let mut region = Region {
            x: 0,
            y: 0,
            width,
            height,
        };

        // A zero-size region is returned as-is. This simplifies callers that
        // might write empty data.
        if width == 0 && height == 0 {
            return Ok(region);
        }

        // Find the node to insert the new region's node at.
        let best_idx = {
            let mut best_height: u32 = u32::MAX;
            let mut best_width: u32 = best_height;
            let mut chosen: Option<usize> = None;

            let mut i: usize = 0;
            while i < self.nodes.len() {
                // Check if our region fits within this node.
                if let Some(y) = self.fit(i, width, height) {
                    let node = self.nodes[i];
                    if (y + height) < best_height
                        || ((y + height) == best_height
                            && node.width > 0
                            && node.width < best_width)
                    {
                        chosen = Some(i);
                        best_width = node.width;
                        best_height = y + height;
                        region.x = node.x;
                        region.y = y;
                    }
                }
                i += 1;
            }

            // If we never found a chosen index, the atlas cannot fit our region.
            match chosen {
                Some(idx) => idx,
                None => return Err(AtlasError::AtlasFull),
            }
        };

        // Insert the new node for this rectangle at the exact best index.
        self.nodes.insert(
            best_idx,
            Node {
                x: region.x,
                y: region.y + height,
                width,
            },
        );

        // Optimize our rectangles: trim/remove nodes the new node overlaps.
        // `i` stays fixed: on removal the next node shifts into index `i` and is
        // reprocessed (upstream's `i -= 1; continue` over a `+= 1` loop step);
        // any surviving node breaks the loop.
        let i = best_idx + 1;
        while i < self.nodes.len() {
            let prev = self.nodes[i - 1];
            if self.nodes[i].x < prev.x + prev.width {
                let shrink = prev.x + prev.width - self.nodes[i].x;
                self.nodes[i].x += shrink;
                self.nodes[i].width = self.nodes[i].width.saturating_sub(shrink);
                if self.nodes[i].width == 0 {
                    self.nodes.remove(i);
                    // Reprocess the node that shifted into index `i`.
                    continue;
                }
            }

            break;
        }
        self.merge();

        Ok(region)
    }

    /// Attempt to fit a `width` × `height` rectangle into the node at `idx`.
    ///
    /// Returns the `y` within the texture where the rectangle can be placed (its
    /// `x` is the node's `x`), or `None` if it would cross the right/bottom 1px
    /// border.
    fn fit(&self, idx: usize, width: u32, height: u32) -> Option<u32> {
        // If the added width exceeds our texture size, it doesn't fit.
        let node = self.nodes[idx];
        if (node.x + width) > (self.size - 1) {
            return None;
        }

        // Go node by node looking for space that can fit our width.
        let mut y = node.y;
        let mut i = idx;
        let mut width_left = width;
        while width_left > 0 {
            let n = self.nodes[i];
            if n.y > y {
                y = n.y;
            }

            // If the added height exceeds our texture size, it doesn't fit.
            if (y + height) > (self.size - 1) {
                return None;
            }

            width_left = width_left.saturating_sub(n.width);
            i += 1;
        }

        Some(y)
    }

    /// Merge adjacent nodes with the same `y` value.
    fn merge(&mut self) {
        let mut i: usize = 0;
        while i + 1 < self.nodes.len() {
            let next = self.nodes[i + 1];
            if self.nodes[i].y == next.y {
                self.nodes[i].width += next.width;
                self.nodes.remove(i + 1);
                continue;
            }

            i += 1;
        }
    }

    /// Set the data for a reserved region. The data must fit the region exactly
    /// and be packed in the atlas's format.
    pub(crate) fn set(&mut self, reg: Region, data: &[u8]) {
        debug_assert!(reg.x < (self.size - 1));
        debug_assert!((reg.x + reg.width) <= (self.size - 1));
        debug_assert!(reg.y < (self.size - 1));
        debug_assert!((reg.y + reg.height) <= (self.size - 1));

        let depth = self.format.depth() as usize;
        let size = self.size as usize;
        let rx = reg.x as usize;
        let ry = reg.y as usize;
        let row = reg.width as usize * depth;
        for i in 0..reg.height as usize {
            let tex_offset = ((ry + i) * size + rx) * depth;
            let data_offset = i * row;
            self.data[tex_offset..tex_offset + row]
                .copy_from_slice(&data[data_offset..data_offset + row]);
        }

        self.modified.fetch_add(1, Ordering::Relaxed);
    }

    /// Reset the atlas: zero the data and re-seed the single full-texture node
    /// inside the 1px border.
    pub(crate) fn clear(&mut self) {
        self.modified.fetch_add(1, Ordering::Relaxed);
        self.data.fill(0);
        self.nodes.clear();

        // The initial rectangle is the full texture inside a 1px border, which
        // avoids artifacting when sampling the texture.
        self.nodes.push(Node {
            x: 1,
            y: 1,
            width: self.size - 2,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_depth() {
        assert_eq!(Format::Grayscale.depth(), 1);
        assert_eq!(Format::Bgr.depth(), 3);
        assert_eq!(Format::Bgra.depth(), 4);
    }

    #[test]
    fn exact_fit() {
        let mut atlas = Atlas::new(34, Format::Grayscale); // +2 for 1px border
        let modified = atlas.modified();
        atlas.reserve(32, 32).unwrap();
        // reserve does not change the texture data.
        assert_eq!(modified, atlas.modified());
        assert_eq!(atlas.reserve(1, 1), Err(AtlasError::AtlasFull));
    }

    #[test]
    fn doesnt_fit() {
        let mut atlas = Atlas::new(32, Format::Grayscale);
        // Doesn't fit due to the border (only 30×30 usable).
        assert_eq!(atlas.reserve(32, 32), Err(AtlasError::AtlasFull));
    }

    #[test]
    fn fit_multiple() {
        let mut atlas = Atlas::new(32, Format::Grayscale);
        atlas.reserve(15, 30).unwrap();
        atlas.reserve(15, 30).unwrap();
        assert_eq!(atlas.reserve(1, 1), Err(AtlasError::AtlasFull));
    }

    #[test]
    fn writing_data() {
        let mut atlas = Atlas::new(32, Format::Grayscale);
        let reg = atlas.reserve(2, 2).unwrap();
        let old = atlas.modified();
        atlas.set(reg, &[1, 2, 3, 4]);
        assert!(atlas.modified() > old);

        // 33 because of the 1px border and so on.
        assert_eq!(atlas.data[33], 1);
        assert_eq!(atlas.data[34], 2);
        assert_eq!(atlas.data[65], 3);
        assert_eq!(atlas.data[66], 4);
    }

    #[test]
    fn writing_bgr_data() {
        let mut atlas = Atlas::new(32, Format::Bgr);
        // BGR is 3 bytes per pixel.
        let reg = atlas.reserve(1, 2).unwrap();
        atlas.set(
            reg,
            &[
                1, 2, 3, //
                4, 5, 6,
            ],
        );

        let depth = atlas.format.depth() as usize;
        assert_eq!(atlas.data[33 * depth], 1);
        assert_eq!(atlas.data[33 * depth + 1], 2);
        assert_eq!(atlas.data[33 * depth + 2], 3);
        assert_eq!(atlas.data[65 * depth], 4);
        assert_eq!(atlas.data[65 * depth + 1], 5);
        assert_eq!(atlas.data[65 * depth + 2], 6);
    }
}
