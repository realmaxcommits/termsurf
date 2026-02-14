// Copyright 2025 TermSurf
// Minimal XPC receiver that accepts IOSurface Mach ports and logs them.
// Part of Issue 414 Experiment 2: XPC frame delivery.

#import <Foundation/Foundation.h>
#import <IOSurface/IOSurface.h>
#import <mach/mach.h>
#import <xpc/xpc.h>

#include <stdio.h>
#include <time.h>

static int g_frame_count = 0;
static struct timespec g_last_log_time;

static void handle_message(xpc_object_t msg) {
  const char *action = xpc_dictionary_get_string(msg, "action");
  if (!action)
    return;

  if (strcmp(action, "display_surface") == 0) {
    mach_port_t port = xpc_dictionary_copy_mach_send(msg, "iosurface_port");
    if (port == MACH_PORT_NULL) {
      fprintf(stderr, "[Receiver] null Mach port\n");
      return;
    }

    IOSurfaceRef surface = IOSurfaceLookupFromMachPort(port);
    if (!surface) {
      fprintf(stderr,
              "[Receiver] IOSurfaceLookupFromMachPort failed (port=%u)\n",
              port);
      mach_port_deallocate(mach_task_self(), port);
      return;
    }

    size_t width = IOSurfaceGetWidth(surface);
    size_t height = IOSurfaceGetHeight(surface);

    g_frame_count++;

    // Log FPS once per second.
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    double elapsed = (now.tv_sec - g_last_log_time.tv_sec) +
                     (now.tv_nsec - g_last_log_time.tv_nsec) / 1e9;
    if (elapsed >= 1.0) {
      double fps = g_frame_count / elapsed;
      fprintf(stderr,
              "[Receiver] %d frames in %.2fs (%.1f fps) | IOSurface %zux%zu\n",
              g_frame_count, elapsed, fps, width, height);
      g_frame_count = 0;
      g_last_log_time = now;
    }

    CFRelease(surface);
    mach_port_deallocate(mach_task_self(), port);
  } else if (strcmp(action, "register") == 0) {
    const char *session_id = xpc_dictionary_get_string(msg, "session_id");
    fprintf(stderr, "[Receiver] Profile server registered: %s\n",
            session_id ? session_id : "(no session_id)");
  }
}

int main(int argc, const char *argv[]) {
  fprintf(stderr,
          "[Receiver] Starting XPC Mach service listener: "
          "com.termsurf.two-profiles\n");

  clock_gettime(CLOCK_MONOTONIC, &g_last_log_time);

  // Use a serial background queue for XPC — not the main queue — to avoid
  // conflicts with any GUI event loop (learned from cef-test).
  dispatch_queue_t queue = dispatch_queue_create(
      "com.termsurf.two-profiles.xpc", DISPATCH_QUEUE_SERIAL);

  xpc_connection_t listener = xpc_connection_create_mach_service(
      "com.termsurf.two-profiles", queue,
      XPC_CONNECTION_MACH_SERVICE_LISTENER);

  if (!listener) {
    fprintf(stderr, "[Receiver] Failed to create Mach service listener\n");
    return 1;
  }

  xpc_connection_set_event_handler(listener, ^(xpc_object_t peer) {
    if (xpc_get_type(peer) == XPC_TYPE_CONNECTION) {
      fprintf(stderr, "[Receiver] Profile server connected\n");

      xpc_connection_set_event_handler(
          (xpc_connection_t)peer, ^(xpc_object_t event) {
            if (xpc_get_type(event) == XPC_TYPE_DICTIONARY) {
              handle_message(event);
            } else if (xpc_get_type(event) == XPC_TYPE_ERROR) {
              if (event == XPC_ERROR_CONNECTION_INVALID) {
                fprintf(stderr, "[Receiver] Connection closed\n");
              } else {
                fprintf(stderr, "[Receiver] XPC error\n");
              }
            }
          });

      xpc_connection_resume((xpc_connection_t)peer);
    } else if (xpc_get_type(peer) == XPC_TYPE_ERROR) {
      fprintf(stderr, "[Receiver] Listener error\n");
    }
  });

  xpc_connection_resume(listener);
  fprintf(stderr, "[Receiver] Listening...\n");

  dispatch_main();
  return 0;
}
