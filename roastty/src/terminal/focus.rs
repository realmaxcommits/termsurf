//! Terminal focus in/out report encoding.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FocusEvent {
    Gained,
    Lost,
}

pub(crate) const MAX_ENCODE_SIZE: usize = 3;

pub(crate) const fn encode(event: FocusEvent) -> &'static [u8] {
    match event {
        FocusEvent::Gained => b"\x1b[I",
        FocusEvent::Lost => b"\x1b[O",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_encode_gained_and_lost() {
        assert_eq!(encode(FocusEvent::Gained), b"\x1b[I");
        assert_eq!(encode(FocusEvent::Lost), b"\x1b[O");
    }

    #[test]
    fn focus_max_encode_size_matches_outputs() {
        assert_eq!(MAX_ENCODE_SIZE, 3);
        assert_eq!(encode(FocusEvent::Gained).len(), MAX_ENCODE_SIZE);
        assert_eq!(encode(FocusEvent::Lost).len(), MAX_ENCODE_SIZE);
    }
}
