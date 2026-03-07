pub fn wezboard_version() -> &'static str {
    // See build.rs
    env!("WEZBOARD_CI_TAG")
}

pub fn wezboard_target_triple() -> &'static str {
    // See build.rs
    env!("WEZBOARD_TARGET_TRIPLE")
}
