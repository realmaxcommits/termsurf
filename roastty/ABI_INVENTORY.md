# Roastty ABI Inventory

This inventory tracks the `ghostty_*` C ABI surface at the start of the Roastty
rewrite. Experiment 2 implements only the lifecycle skeleton needed to prove
opaque handles, string ownership, exported symbols, and C header compatibility.

## Implemented In Experiment 2

- `ghostty_init`
- `ghostty_info`
- `ghostty_string_free`
- `ghostty_config_new`
- `ghostty_config_free`
- `ghostty_config_clone`
- `ghostty_config_load_cli_args`
- `ghostty_config_load_file`
- `ghostty_config_load_default_files`
- `ghostty_config_load_recursive_files`
- `ghostty_config_finalize`
- `ghostty_config_diagnostics_count`
- `ghostty_config_get_diagnostic`
- `ghostty_config_open_path`
- `ghostty_app_new`
- `ghostty_app_free`
- `ghostty_app_tick`
- `ghostty_app_userdata`
- `ghostty_app_set_focus`
- `ghostty_app_update_config`
- `ghostty_app_needs_confirm_quit`
- `ghostty_app_has_global_keybinds`
- `ghostty_app_set_color_scheme`
- `ghostty_surface_config_new`
- `ghostty_surface_new`
- `ghostty_surface_free`
- `ghostty_surface_userdata`
- `ghostty_surface_app`
- `ghostty_surface_update_config`
- `ghostty_surface_needs_confirm_quit`
- `ghostty_surface_process_exited`
- `ghostty_surface_set_content_scale`
- `ghostty_surface_set_focus`
- `ghostty_surface_set_occlusion`
- `ghostty_surface_set_size`
- `ghostty_surface_size`
- `ghostty_surface_foreground_pid`
- `ghostty_surface_tty_name`
- `ghostty_surface_set_color_scheme`
- `ghostty_surface_request_close`

## Used By Swift But Deferred

These symbols are referenced under `vendor/ghostty/macos/Sources/` but require
behavior semantics beyond the inert lifecycle skeleton.

- `ghostty_app_key`
- `ghostty_app_keyboard_changed`
- `ghostty_app_open_config`
- `ghostty_config_get`
- `ghostty_config_key_is_binding`
- `ghostty_config_trigger`
- `ghostty_inspector_free`
- `ghostty_inspector_key`
- `ghostty_inspector_metal_init`
- `ghostty_inspector_metal_render`
- `ghostty_inspector_metal_shutdown`
- `ghostty_inspector_mouse_button`
- `ghostty_inspector_mouse_pos`
- `ghostty_inspector_mouse_scroll`
- `ghostty_inspector_set_content_scale`
- `ghostty_inspector_set_focus`
- `ghostty_inspector_set_size`
- `ghostty_inspector_text`
- `ghostty_surface_binding_action`
- `ghostty_surface_complete_clipboard_request`
- `ghostty_surface_free_text`
- `ghostty_surface_has_selection`
- `ghostty_surface_ime_point`
- `ghostty_surface_inherited_config`
- `ghostty_surface_inspector`
- `ghostty_surface_key`
- `ghostty_surface_key_is_binding`
- `ghostty_surface_key_translation_mods`
- `ghostty_surface_mouse_button`
- `ghostty_surface_mouse_captured`
- `ghostty_surface_mouse_pos`
- `ghostty_surface_mouse_pressure`
- `ghostty_surface_mouse_scroll`
- `ghostty_surface_preedit`
- `ghostty_surface_quicklook_font`
- `ghostty_surface_quicklook_word`
- `ghostty_surface_read_selection`
- `ghostty_surface_read_text`
- `ghostty_surface_refresh`
- `ghostty_surface_set_display_id`
- `ghostty_surface_split`
- `ghostty_surface_split_equalize`
- `ghostty_surface_split_focus`
- `ghostty_surface_split_resize`
- `ghostty_surface_text`
- `ghostty_surface_draw`

## Not Relevant To The Current Skeleton

These symbols are published by `vendor/ghostty/include/ghostty.h` but are not
needed to prove the app/config/surface lifecycle skeleton.

- `ghostty_benchmark_cli`
- `ghostty_cli_try_action`
- `ghostty_set_window_background_blur`
- `ghostty_translate`
