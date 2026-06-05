//! Mouse-related OS helpers.

pub(crate) fn click_interval() -> Option<u32> {
    system_double_click_interval_seconds().map(seconds_to_millis_ceil)
}

#[cfg(target_os = "macos")]
fn system_double_click_interval_seconds() -> Option<f64> {
    Some(objc2_app_kit::NSEvent::doubleClickInterval())
}

#[cfg(not(target_os = "macos"))]
fn system_double_click_interval_seconds() -> Option<f64> {
    None
}

fn seconds_to_millis_ceil(seconds: f64) -> u32 {
    (seconds * 1000.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_to_millis_ceil_matches_upstream_conversion() {
        assert_eq!(seconds_to_millis_ceil(0.001), 1);
        assert_eq!(seconds_to_millis_ceil(0.0011), 2);
        assert_eq!(seconds_to_millis_ceil(0.5), 500);
        assert_eq!(seconds_to_millis_ceil(0.0), 0);
    }

    #[test]
    fn click_interval_matches_platform_shape() {
        let interval = click_interval();

        if cfg!(target_os = "macos") {
            assert!(interval.is_some_and(|value| value > 0));
        } else {
            assert_eq!(interval, None);
        }
    }
}
