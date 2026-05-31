#include <assert.h>
#include <stdint.h>
#include <string.h>

#include "ghostty.h"

static void wakeup_cb(void *userdata) {
  (void)userdata;
}

static bool action_cb(ghostty_app_t app,
                      ghostty_target_s target,
                      ghostty_action_s action) {
  (void)app;
  (void)target;
  (void)action;
  return false;
}

static bool read_clipboard_cb(void *userdata,
                              ghostty_clipboard_e clipboard,
                              void *state) {
  (void)userdata;
  (void)clipboard;
  (void)state;
  return false;
}

static void confirm_read_clipboard_cb(void *userdata,
                                      const char *str,
                                      void *state,
                                      ghostty_clipboard_request_e request) {
  (void)userdata;
  (void)str;
  (void)state;
  (void)request;
}

static void write_clipboard_cb(void *userdata,
                               ghostty_clipboard_e clipboard,
                               const ghostty_clipboard_content_s *content,
                               size_t len,
                               bool confirm) {
  (void)userdata;
  (void)clipboard;
  (void)content;
  (void)len;
  (void)confirm;
}

static void close_surface_cb(void *userdata, bool process_alive) {
  (void)userdata;
  (void)process_alive;
}

int main(int argc, char **argv) {
  assert(ghostty_init((uintptr_t)argc, argv) == GHOSTTY_SUCCESS);

  ghostty_config_free(NULL);
  ghostty_app_free(NULL);
  ghostty_surface_free(NULL);
  ghostty_config_load_cli_args(NULL);
  ghostty_config_load_default_files(NULL);
  ghostty_config_load_recursive_files(NULL);
  ghostty_config_load_file(NULL, NULL);
  ghostty_config_finalize(NULL);
  assert(ghostty_config_diagnostics_count(NULL) == 0);
  assert(ghostty_config_get_diagnostic(NULL, 0).message != NULL);
  assert(ghostty_app_userdata(NULL) == NULL);
  ghostty_app_tick(NULL);
  ghostty_app_set_focus(NULL, true);
  ghostty_app_set_color_scheme(NULL, GHOSTTY_COLOR_SCHEME_DARK);
  ghostty_app_update_config(NULL, NULL);
  assert(!ghostty_app_needs_confirm_quit(NULL));
  assert(!ghostty_app_has_global_keybinds(NULL));
  assert(ghostty_surface_userdata(NULL) == NULL);
  assert(ghostty_surface_app(NULL) == NULL);
  ghostty_surface_update_config(NULL, NULL);
  assert(!ghostty_surface_needs_confirm_quit(NULL));
  assert(!ghostty_surface_process_exited(NULL));
  ghostty_surface_set_content_scale(NULL, 1.0, 1.0);
  ghostty_surface_set_focus(NULL, true);
  ghostty_surface_set_occlusion(NULL, true);
  ghostty_surface_set_color_scheme(NULL, GHOSTTY_COLOR_SCHEME_DARK);
  ghostty_surface_set_size(NULL, 1, 1);
  ghostty_surface_size_s null_size = ghostty_surface_size(NULL);
  assert(null_size.width_px == 0);
  assert(null_size.height_px == 0);
  assert(null_size.columns == 0);
  assert(null_size.rows == 0);
  assert(null_size.cell_width_px == 0);
  assert(null_size.cell_height_px == 0);
  assert(ghostty_surface_foreground_pid(NULL) == 0);
  ghostty_surface_request_close(NULL);

  ghostty_info_s info = ghostty_info();
  assert(info.version != NULL);
  assert(info.version_len > 0);

  ghostty_config_t config = ghostty_config_new();
  assert(config != NULL);
  ghostty_config_load_cli_args(config);
  ghostty_config_load_default_files(config);
  ghostty_config_load_recursive_files(config);
  ghostty_config_load_file(config, "/tmp/nonexistent-roastty-config");
  ghostty_config_finalize(config);
  assert(ghostty_config_diagnostics_count(config) == 0);
  ghostty_diagnostic_s diagnostic = ghostty_config_get_diagnostic(config, 0);
  assert(diagnostic.message != NULL);

  ghostty_config_t clone = ghostty_config_clone(config);
  assert(clone != NULL);
  ghostty_config_free(clone);

  ghostty_string_s open_path = ghostty_config_open_path();
  assert(open_path.ptr != NULL);
  assert(open_path.len == strlen("roastty-config"));
  assert(open_path.sentinel == false);
  ghostty_string_free(open_path);

  uintptr_t app_userdata = 0xA991;
  ghostty_runtime_config_s runtime = {
      .userdata = (void *)app_userdata,
      .supports_selection_clipboard = true,
      .wakeup_cb = wakeup_cb,
      .action_cb = action_cb,
      .read_clipboard_cb = read_clipboard_cb,
      .confirm_read_clipboard_cb = confirm_read_clipboard_cb,
      .write_clipboard_cb = write_clipboard_cb,
      .close_surface_cb = close_surface_cb,
  };

  ghostty_app_t app = ghostty_app_new(&runtime, config);
  assert(app != NULL);
  assert((uintptr_t)ghostty_app_userdata(app) == app_userdata);
  ghostty_app_tick(app);
  ghostty_app_set_focus(app, true);
  ghostty_app_set_color_scheme(app, GHOSTTY_COLOR_SCHEME_DARK);
  ghostty_app_update_config(app, config);
  assert(!ghostty_app_needs_confirm_quit(app));
  assert(!ghostty_app_has_global_keybinds(app));

  uintptr_t surface_userdata = 0x5A5A;
  ghostty_surface_config_s surface_config = ghostty_surface_config_new();
  surface_config.userdata = (void *)surface_userdata;
  surface_config.scale_factor = 2.0;
  surface_config.context = GHOSTTY_SURFACE_CONTEXT_WINDOW;

  ghostty_app_t app_with_null_runtime = ghostty_app_new(NULL, config);
  assert(app_with_null_runtime != NULL);
  assert(ghostty_app_userdata(app_with_null_runtime) == NULL);
  ghostty_app_free(app_with_null_runtime);

  ghostty_surface_t surface_with_null_config = ghostty_surface_new(app, NULL);
  assert(surface_with_null_config != NULL);
  assert(ghostty_surface_app(surface_with_null_config) == app);
  ghostty_surface_free(surface_with_null_config);

  assert(ghostty_surface_new(NULL, &surface_config) == NULL);

  ghostty_surface_t surface = ghostty_surface_new(app, &surface_config);
  assert(surface != NULL);
  assert(ghostty_surface_app(surface) == app);
  assert((uintptr_t)ghostty_surface_userdata(surface) == surface_userdata);

  ghostty_surface_set_content_scale(surface, 2.0, 2.0);
  ghostty_surface_set_focus(surface, true);
  ghostty_surface_set_occlusion(surface, false);
  ghostty_surface_set_color_scheme(surface, GHOSTTY_COLOR_SCHEME_LIGHT);
  ghostty_surface_set_size(surface, 1024, 768);

  ghostty_surface_size_s size = ghostty_surface_size(surface);
  assert(size.width_px == 1024);
  assert(size.height_px == 768);
  assert(size.columns == 0);
  assert(size.rows == 0);
  assert(size.cell_width_px == 0);
  assert(size.cell_height_px == 0);

  assert(ghostty_surface_foreground_pid(surface) == 0);
  assert(!ghostty_surface_needs_confirm_quit(surface));
  assert(!ghostty_surface_process_exited(surface));

  ghostty_string_s tty_name = ghostty_surface_tty_name(surface);
  assert(tty_name.ptr != NULL);
  assert(tty_name.len == strlen("roastty-skeleton-tty"));
  assert(tty_name.sentinel == true);
  ghostty_string_free(tty_name);

  ghostty_surface_request_close(surface);
  ghostty_surface_free(surface);

  ghostty_string_s empty_tty = ghostty_surface_tty_name(NULL);
  assert(empty_tty.ptr == NULL);
  assert(empty_tty.len == 0);
  assert(empty_tty.sentinel == false);
  ghostty_string_free(empty_tty);

  for (int i = 0; i < 16; i++) {
    ghostty_surface_t loop_surface = ghostty_surface_new(app, &surface_config);
    assert(loop_surface != NULL);
    ghostty_surface_free(loop_surface);
  }

  ghostty_app_free(app);
  ghostty_config_free(config);
  return 0;
}
