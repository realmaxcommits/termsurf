use super::page::{page_layout, Capacity, CapacityAdjustment, STD_CAPACITY};
use super::size::CellCountInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Viewport {
    Active,
    Top,
    Pin,
}

fn standard_page_size() -> usize {
    page_layout(STD_CAPACITY).total_size()
}

fn initial_capacity(cols: CellCountInt) -> Capacity {
    if let Ok(capacity) = STD_CAPACITY.adjust(CapacityAdjustment::cols(cols)) {
        return capacity;
    }

    STD_CAPACITY.with_cols(cols)
}

fn min_max_size(cols: CellCountInt, rows: CellCountInt) -> usize {
    let capacity = initial_capacity(cols);
    let capacity_rows = capacity.rows() as usize;
    let rows = rows as usize;
    let pages_exact = if capacity_rows >= rows {
        1
    } else {
        rows.div_ceil(capacity_rows)
    };
    let pages = pages_exact + 1;
    debug_assert!(pages >= 2);

    standard_page_size() * pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::page::page_layout;

    #[test]
    fn viewport_variants_compare_as_expected() {
        assert_eq!(Viewport::Active, Viewport::Active);
        assert_eq!(Viewport::Top, Viewport::Top);
        assert_eq!(Viewport::Pin, Viewport::Pin);
        assert_ne!(Viewport::Active, Viewport::Top);
        assert_ne!(Viewport::Active, Viewport::Pin);
        assert_ne!(Viewport::Top, Viewport::Pin);
    }

    #[test]
    fn initial_capacity_normal_width_preserves_standard_size() {
        let standard_size = standard_page_size();
        let capacity = initial_capacity(80);

        assert_eq!(capacity.cols(), 80);
        assert!(capacity.rows() >= 1);
        assert_eq!(page_layout(capacity).total_size(), standard_size);
    }

    #[test]
    fn initial_capacity_max_standard_width_preserves_standard_size() {
        let standard_size = standard_page_size();
        let max_cols = STD_CAPACITY
            .max_cols()
            .expect("standard capacity should fit at least one row");
        let capacity = initial_capacity(max_cols);

        assert_eq!(capacity.cols(), max_cols);
        assert_eq!(capacity.rows(), 1);
        assert_eq!(page_layout(capacity).total_size(), standard_size);
    }

    #[test]
    fn initial_capacity_too_wide_uses_non_standard_page() {
        let standard_size = standard_page_size();
        let max_cols = STD_CAPACITY
            .max_cols()
            .expect("standard capacity should fit at least one row");
        assert!(max_cols < CellCountInt::MAX);
        let requested_cols = max_cols + 1;
        let capacity = initial_capacity(requested_cols);

        assert_eq!(capacity.cols(), requested_cols);
        assert_eq!(capacity.rows(), STD_CAPACITY.rows());
        assert!(page_layout(capacity).total_size() > standard_size);
    }

    #[test]
    fn initial_capacity_max_columns_lays_out() {
        let capacity = initial_capacity(CellCountInt::MAX);
        let layout = page_layout(capacity);

        assert_eq!(capacity.cols(), CellCountInt::MAX);
        assert!(capacity.rows() >= 1);
        assert!(layout.total_size() >= standard_page_size());
    }

    #[test]
    fn min_max_size_normal_dimensions_are_two_standard_pages() {
        assert_eq!(min_max_size(80, 24), standard_page_size() * 2);
    }

    #[test]
    fn min_max_size_adds_extra_page_for_multi_page_active_area() {
        let cols = 80;
        let capacity = initial_capacity(cols);
        let rows = capacity.rows() + 1;
        let expected_pages = (rows as usize).div_ceil(capacity.rows() as usize) + 1;

        assert!(expected_pages > 2);
        assert_eq!(
            min_max_size(cols, rows),
            standard_page_size() * expected_pages
        );
    }
}
