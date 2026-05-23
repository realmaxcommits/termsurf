use crate::termwindow::render::TripleLayerQuadAllocator;
use crate::termwindow::{UIItem, UIItemType};
use mux::pane::Pane;
use mux::tab::{PositionedPane, PositionedSplit, SplitDirection};
use std::sync::Arc;

impl crate::TermWindow {
    pub fn paint_split(
        &mut self,
        layers: &mut TripleLayerQuadAllocator,
        split: &PositionedSplit,
        pane: &Arc<dyn Pane>,
        active_pos: Option<&PositionedPane>,
    ) -> anyhow::Result<()> {
        let palette = pane.palette();
        let adjacent_to_active = active_pos
            .map(|pos| split_is_adjacent_to_pane(split, pos))
            .unwrap_or(false);
        let foreground = if adjacent_to_active {
            self.config
                .focused_split_border_color
                .map(|c| c.to_linear())
                .unwrap_or_else(|| palette.split.to_linear())
        } else {
            self.config
                .unfocused_split_border_color
                .map(|c| c.to_linear())
                .unwrap_or_else(|| palette.split.to_linear())
        };
        let cell_width = self.render_metrics.cell_size.width as f32;
        let cell_height = self.render_metrics.cell_size.height as f32;

        let border = self.get_os_border();
        let first_row_offset = if self.show_tab_bar && !self.config.tab_bar_at_bottom {
            self.tab_bar_pixel_height()?
        } else {
            0.
        } + border.top.get() as f32;

        let (padding_left, padding_top) = self.padding_left_top();

        let pos_y = split.top as f32 * cell_height + first_row_offset + padding_top;
        let pos_x = split.left as f32 * cell_width + padding_left + border.left.get() as f32;

        if split.direction == SplitDirection::Horizontal {
            self.filled_rectangle(
                layers,
                2,
                euclid::rect(
                    pos_x,
                    pos_y,
                    cell_width,
                    split.size as f32 * cell_height,
                ),
                foreground,
            )?;
            self.ui_items.push(UIItem {
                x: border.left.get() as usize
                    + padding_left as usize
                    + (split.left * cell_width as usize),
                width: cell_width as usize,
                y: padding_top as usize
                    + first_row_offset as usize
                    + split.top * cell_height as usize,
                height: split.size * cell_height as usize,
                item_type: UIItemType::Split(split.clone()),
            });
        } else {
            self.filled_rectangle(
                layers,
                2,
                euclid::rect(
                    pos_x,
                    pos_y,
                    split.size as f32 * cell_width,
                    cell_height,
                ),
                foreground,
            )?;
            self.ui_items.push(UIItem {
                x: border.left.get() as usize
                    + padding_left as usize
                    + (split.left * cell_width as usize),
                width: split.size * cell_width as usize,
                y: padding_top as usize
                    + first_row_offset as usize
                    + split.top * cell_height as usize,
                height: cell_height as usize,
                item_type: UIItemType::Split(split.clone()),
            });
        }

        Ok(())
    }
}

fn split_is_adjacent_to_pane(split: &PositionedSplit, pane: &PositionedPane) -> bool {
    match split.direction {
        SplitDirection::Horizontal => {
            let split_left = split.left;
            let split_top = split.top;
            let split_bottom = split.top + split.size;
            let pane_top = pane.top;
            let pane_bottom = pane.top + pane.height;
            let vertical_overlap = pane_top < split_bottom && pane_bottom > split_top;
            vertical_overlap
                && (pane.left + pane.width == split_left || pane.left == split_left + 1)
        }
        SplitDirection::Vertical => {
            let split_left = split.left;
            let split_right = split.left + split.size;
            let split_top = split.top;
            let pane_left = pane.left;
            let pane_right = pane.left + pane.width;
            let horizontal_overlap = pane_left < split_right && pane_right > split_left;
            horizontal_overlap && (pane.top + pane.height == split_top || pane.top == split_top + 1)
        }
    }
}
