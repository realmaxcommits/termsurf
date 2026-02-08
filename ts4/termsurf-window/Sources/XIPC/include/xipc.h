#ifndef XIPC_H
#define XIPC_H

#include <stdint.h>
#include <mach/mach.h>

/// Callback invoked when a "frame" message is received via XPC.
/// The callback receives the IOSurface Mach port, width, height, and user context.
typedef void (*xipc_frame_callback)(mach_port_t port, uint32_t width, uint32_t height, void *context);

/// Connect to a named XPC Mach service (client mode).
///
/// The service must be registered with launchd. When the service sends a
/// message with action "frame", the callback is invoked with the IOSurface
/// Mach port and dimensions.
///
/// This function returns immediately. Events are dispatched on the main queue.
void xipc_connect(const char *service_name, xipc_frame_callback callback, void *context);

/// Import an IOSurface from a Mach port received from another process.
/// Returns the IOSurfaceRef as void*, or NULL on failure.
void *xipc_import_iosurface(mach_port_t port);

/// Get the width of an IOSurface.
size_t xipc_iosurface_width(void *surface);

/// Get the height of an IOSurface.
size_t xipc_iosurface_height(void *surface);

#endif
