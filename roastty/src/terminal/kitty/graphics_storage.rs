//! Kitty graphics image storage.

use std::collections::HashMap;

use super::graphics_image::{Image, ImageLoadError, LoadingImage, LoadingImageLimits};

pub(crate) const DEFAULT_NEXT_IMAGE_ID: u32 = 2_147_483_647;
pub(crate) const DEFAULT_TOTAL_LIMIT: usize = 320 * 1000 * 1000;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ImageStorage {
    pub(crate) dirty: bool,
    pub(crate) next_image_id: u32,
    pub(crate) loading: Option<Box<LoadingImage>>,
    pub(crate) image_limits: LoadingImageLimits,
    pub(crate) total_bytes: usize,
    pub(crate) total_limit: usize,
    images: HashMap<u32, Image>,
}

impl Default for LoadingImageLimits {
    fn default() -> Self {
        Self::DIRECT
    }
}

impl Default for ImageStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageStorage {
    pub(crate) fn new() -> Self {
        Self {
            dirty: false,
            next_image_id: DEFAULT_NEXT_IMAGE_ID,
            loading: None,
            image_limits: LoadingImageLimits::DIRECT,
            total_bytes: 0,
            total_limit: DEFAULT_TOTAL_LIMIT,
            images: HashMap::new(),
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.total_limit != 0
    }

    pub(crate) fn len(&self) -> usize {
        self.images.len()
    }

    pub(crate) fn set_limit(&mut self, limit: usize) {
        if limit == 0 {
            let image_limits = self.image_limits;
            self.images.clear();
            self.loading = None;
            self.total_bytes = 0;
            self.total_limit = 0;
            self.image_limits = image_limits;
            self.dirty = true;
            return;
        }

        if limit < self.total_bytes {
            let required_bytes = self.total_bytes - limit;
            let _ = self.evict_image(required_bytes);
        }

        self.total_limit = limit;
    }

    pub(crate) fn add_image(&mut self, image: Image) -> Result<(), ImageLoadError> {
        let image_bytes = image.data.len();
        if image_bytes > self.total_limit {
            return Err(ImageLoadError::OutOfMemory);
        }

        let existing_bytes = self
            .images
            .get(&image.id)
            .map(|stored| stored.data.len())
            .unwrap_or(0);
        let final_bytes_without_eviction = self
            .total_bytes
            .checked_sub(existing_bytes)
            .and_then(|bytes| bytes.checked_add(image_bytes))
            .ok_or(ImageLoadError::OutOfMemory)?;

        if final_bytes_without_eviction > self.total_limit {
            let required_bytes = final_bytes_without_eviction - self.total_limit;
            if !self.evict_image_excluding(required_bytes, image.id) {
                return Err(ImageLoadError::OutOfMemory);
            }
        }

        if let Some(old) = self.images.insert(image.id, image) {
            self.total_bytes -= old.data.len();
        }
        self.total_bytes += image_bytes;
        self.dirty = true;
        Ok(())
    }

    pub(crate) fn image_by_id(&self, image_id: u32) -> Option<&Image> {
        self.images.get(&image_id)
    }

    pub(crate) fn image_by_number(&self, image_number: u32) -> Option<&Image> {
        self.images
            .values()
            .filter(|image| image.number == image_number)
            .max_by(|lhs, rhs| compare_newest(lhs, rhs))
    }

    pub(crate) fn evict_image(&mut self, required_bytes: usize) -> bool {
        self.evict_image_excluding(required_bytes, u32::MAX)
    }

    fn evict_image_excluding(&mut self, required_bytes: usize, excluded_id: u32) -> bool {
        if required_bytes == 0 {
            return true;
        }

        let mut candidates: Vec<(u32, Option<std::time::Instant>)> = self
            .images
            .values()
            .filter(|image| image.id != excluded_id)
            .map(|image| (image.id, image.transmit_time))
            .collect();
        candidates.sort_by(|(lhs_id, lhs_time), (rhs_id, rhs_time)| {
            compare_oldest_parts(*lhs_time, *lhs_id, *rhs_time, *rhs_id)
        });

        let mut evicted = 0usize;
        for (id, _) in candidates {
            let Some(image) = self.images.remove(&id) else {
                continue;
            };
            evicted += image.data.len();
            self.total_bytes -= image.data.len();
            self.dirty = true;
            if evicted >= required_bytes {
                return true;
            }
        }

        false
    }
}

fn compare_newest(lhs: &Image, rhs: &Image) -> std::cmp::Ordering {
    match (lhs.transmit_time, rhs.transmit_time) {
        (Some(lhs_time), Some(rhs_time)) => {
            lhs_time.cmp(&rhs_time).then_with(|| lhs.id.cmp(&rhs.id))
        }
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => lhs.id.cmp(&rhs.id),
    }
}

fn compare_oldest_parts(
    lhs_time: Option<std::time::Instant>,
    lhs_id: u32,
    rhs_time: Option<std::time::Instant>,
    rhs_id: u32,
) -> std::cmp::Ordering {
    match (lhs_time, rhs_time) {
        (Some(lhs_time), Some(rhs_time)) => {
            lhs_time.cmp(&rhs_time).then_with(|| lhs_id.cmp(&rhs_id))
        }
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, None) => lhs_id.cmp(&rhs_id),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::super::graphics_command::{TransmissionCompression, TransmissionFormat};
    use super::*;

    fn image(id: u32, number: u32, bytes: usize, transmit_time: Instant) -> Image {
        Image {
            id,
            number,
            width: bytes as u32,
            height: 1,
            format: TransmissionFormat::Rgb,
            compression: TransmissionCompression::None,
            data: vec![id as u8; bytes],
            transmit_time: Some(transmit_time),
            implicit_id: false,
        }
    }

    #[test]
    fn kitty_graphics_storage_defaults_and_enabled_state() {
        let storage = ImageStorage::new();
        let default_storage = ImageStorage::default();
        assert_eq!(default_storage.next_image_id, storage.next_image_id);
        assert_eq!(default_storage.total_limit, storage.total_limit);
        assert_eq!(default_storage.image_limits, storage.image_limits);
        assert_eq!(default_storage.enabled(), storage.enabled());

        assert!(!storage.dirty);
        assert_eq!(storage.next_image_id, DEFAULT_NEXT_IMAGE_ID);
        assert!(storage.loading.is_none());
        assert_eq!(storage.image_limits, LoadingImageLimits::DIRECT);
        assert_eq!(storage.total_bytes, 0);
        assert_eq!(storage.total_limit, DEFAULT_TOTAL_LIMIT);
        assert!(storage.enabled());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn kitty_graphics_storage_set_limit_zero_clears_images_and_preserves_limits() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.image_limits = LoadingImageLimits::ALL;
        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage.dirty = false;

        storage.set_limit(0);

        assert!(!storage.enabled());
        assert_eq!(storage.image_limits, LoadingImageLimits::ALL);
        assert_eq!(storage.total_bytes, 0);
        assert_eq!(storage.len(), 0);
        assert!(storage.image_by_id(1).is_none());
        assert!(storage.dirty);
    }

    #[test]
    fn kitty_graphics_storage_add_image_updates_bytes_lookup_and_dirty() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.add_image(image(1, 9, 10, base)).unwrap();

        assert_eq!(storage.total_bytes, 10);
        assert_eq!(storage.len(), 1);
        assert!(storage.dirty);
        assert_eq!(storage.image_by_id(1).unwrap().number, 9);
    }

    #[test]
    fn kitty_graphics_storage_replace_same_id_updates_accounting_for_sizes() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.total_limit = 25;

        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage
            .add_image(image(1, 0, 10, base + Duration::from_secs(1)))
            .unwrap();
        assert_eq!(storage.total_bytes, 10);
        assert_eq!(storage.len(), 1);

        storage
            .add_image(image(1, 0, 5, base + Duration::from_secs(2)))
            .unwrap();
        assert_eq!(storage.total_bytes, 5);
        assert_eq!(storage.len(), 1);

        storage
            .add_image(image(1, 0, 25, base + Duration::from_secs(3)))
            .unwrap();
        assert_eq!(storage.total_bytes, 25);
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn kitty_graphics_storage_same_id_replacement_does_not_over_evict() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.total_limit = 20;
        storage.add_image(image(1, 0, 15, base)).unwrap();
        storage
            .add_image(image(2, 0, 5, base + Duration::from_secs(1)))
            .unwrap();

        storage
            .add_image(image(1, 0, 15, base + Duration::from_secs(2)))
            .unwrap();

        assert_eq!(storage.total_bytes, 20);
        assert!(storage.image_by_id(1).is_some());
        assert!(storage.image_by_id(2).is_some());
    }

    #[test]
    fn kitty_graphics_storage_rejects_image_larger_than_limit_without_mutation() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.total_limit = 10;
        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage.dirty = false;

        assert_eq!(
            storage.add_image(image(2, 0, 11, base + Duration::from_secs(1))),
            Err(ImageLoadError::OutOfMemory)
        );
        assert_eq!(storage.total_bytes, 10);
        assert_eq!(storage.len(), 1);
        assert!(storage.image_by_id(1).is_some());
        assert!(storage.image_by_id(2).is_none());
        assert!(!storage.dirty);
    }

    #[test]
    fn kitty_graphics_storage_lowering_limit_evicts_oldest_images() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage
            .add_image(image(2, 0, 10, base + Duration::from_secs(1)))
            .unwrap();
        storage
            .add_image(image(3, 0, 10, base + Duration::from_secs(2)))
            .unwrap();

        storage.set_limit(15);

        assert_eq!(storage.total_limit, 15);
        assert_eq!(storage.total_bytes, 10);
        assert!(storage.image_by_id(1).is_none());
        assert!(storage.image_by_id(2).is_none());
        assert!(storage.image_by_id(3).is_some());
    }

    #[test]
    fn kitty_graphics_storage_lowering_limit_exact_fit_succeeds() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage
            .add_image(image(2, 0, 10, base + Duration::from_secs(1)))
            .unwrap();

        storage.set_limit(10);

        assert_eq!(storage.total_limit, 10);
        assert_eq!(storage.total_bytes, 10);
        assert!(storage.image_by_id(1).is_none());
        assert!(storage.image_by_id(2).is_some());
    }

    #[test]
    fn kitty_graphics_storage_add_image_evicts_enough_old_images_to_fit() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.total_limit = 25;
        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage
            .add_image(image(2, 0, 10, base + Duration::from_secs(1)))
            .unwrap();

        storage
            .add_image(image(3, 0, 15, base + Duration::from_secs(2)))
            .unwrap();

        assert_eq!(storage.total_bytes, 25);
        assert!(storage.image_by_id(1).is_none());
        assert!(storage.image_by_id(2).is_some());
        assert!(storage.image_by_id(3).is_some());
    }

    #[test]
    fn kitty_graphics_storage_add_image_exact_fit_eviction_succeeds() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.total_limit = 20;
        storage.add_image(image(1, 0, 10, base)).unwrap();
        storage
            .add_image(image(2, 0, 10, base + Duration::from_secs(1)))
            .unwrap();

        storage
            .add_image(image(3, 0, 10, base + Duration::from_secs(2)))
            .unwrap();

        assert_eq!(storage.total_bytes, 20);
        assert!(storage.image_by_id(1).is_none());
        assert!(storage.image_by_id(2).is_some());
        assert!(storage.image_by_id(3).is_some());
    }

    #[test]
    fn kitty_graphics_storage_image_by_id_borrows_stored_image() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.add_image(image(1, 0, 10, base)).unwrap();

        let stored = storage.image_by_id(1).unwrap();
        assert_eq!(
            stored.data.as_ptr(),
            storage.image_by_id(1).unwrap().data.as_ptr()
        );
        assert_eq!(stored.data.len(), 10);
    }

    #[test]
    fn kitty_graphics_storage_image_by_number_picks_newest_with_id_tie_break() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.add_image(image(1, 7, 1, base)).unwrap();
        storage
            .add_image(image(3, 7, 1, base + Duration::from_secs(1)))
            .unwrap();
        storage
            .add_image(image(2, 7, 1, base + Duration::from_secs(1)))
            .unwrap();

        assert_eq!(storage.image_by_number(7).unwrap().id, 3);
    }

    #[test]
    fn kitty_graphics_storage_eviction_moves_images_without_payload_clones() {
        let base = Instant::now();
        let mut storage = ImageStorage::new();
        storage.total_limit = 20;
        storage.add_image(image(1, 0, 10, base)).unwrap();
        let survivor = image(2, 0, 10, base + Duration::from_secs(1));
        let survivor_ptr = survivor.data.as_ptr();
        storage.add_image(survivor).unwrap();

        assert!(storage.evict_image(10));

        let stored = storage.image_by_id(2).unwrap();
        assert_eq!(stored.data.as_ptr(), survivor_ptr);
        assert_eq!(storage.total_bytes, 10);
        assert!(storage.image_by_id(1).is_none());
    }
}
