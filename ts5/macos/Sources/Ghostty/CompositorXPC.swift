// Copyright 2025 TermSurf
// Issue 505: XPC Mach service listener for the compositor.
// Receives overlay coordinates from `web` processes running in terminal panes.

import Foundation
import GhosttyKit
import os.log

/// Manages the XPC Mach service listener for the TermSurf compositor.
///
/// `web` processes connect to `com.termsurf.compositor` and send `set_overlay`
/// messages containing grid coordinates. The compositor looks up the
/// corresponding pane by UUID and passes the coordinates to the Zig renderer
/// via the C API.
private let logger = Logger(subsystem: "com.termsurf.compositor", category: "xpc")

class CompositorXPC {
    static let shared = CompositorXPC()

    /// The XPC listener connection (must be retained).
    private var listener: xpc_connection_t?

    /// Active peer connections (must be retained to prevent ARC release).
    private var peers: [xpc_connection_t] = []

    /// Maps peer connections to their pane UUID (for cleanup on disconnect).
    private var peerPaneIds: [ObjectIdentifier: UUID] = [:]

    /// Weak reference to the app delegate for surface lookup.
    private weak var appDelegate: GhosttyAppDelegate?

    private init() {}

    /// Start the XPC Mach service listener.
    ///
    /// Call this once during app startup (e.g., in applicationDidFinishLaunching).
    /// The service name must be registered with launchd via a LaunchAgent plist.
    func start(appDelegate: GhosttyAppDelegate) {
        self.appDelegate = appDelegate
        logger.info("Compositor XPC listener starting on com.termsurf.compositor")

        let queue = DispatchQueue(label: "com.termsurf.compositor.xpc")
        let conn = xpc_connection_create_mach_service(
            "com.termsurf.compositor",
            queue,
            UInt64(XPC_CONNECTION_MACH_SERVICE_LISTENER))

        listener = conn

        xpc_connection_set_event_handler(conn) { [weak self] peer in
            guard let self = self else { return }
            if xpc_get_type(peer) == XPC_TYPE_CONNECTION {
                let peerConn = peer as xpc_connection_t
                self.peers.append(peerConn)
                fputs("[Compositor] Client connected (\(self.peers.count) total)\n", stderr)

                xpc_connection_set_event_handler(peerConn) { [weak self] event in
                    guard let self = self else { return }
                    if xpc_get_type(event) == XPC_TYPE_DICTIONARY {
                        self.handleMessage(event, from: peerConn)
                    } else if xpc_get_type(event) == XPC_TYPE_ERROR {
                        if event === XPC_ERROR_CONNECTION_INVALID {
                            self.handleDisconnect(peerConn)
                        } else {
                            fputs("[Compositor] XPC error\n", stderr)
                        }
                    }
                }
                xpc_connection_resume(peerConn)
            } else if xpc_get_type(peer) == XPC_TYPE_ERROR {
                fputs("[Compositor] Listener error\n", stderr)
            }
        }

        xpc_connection_resume(conn)
        logger.info("Compositor XPC listener active")
        fputs("[Compositor] Listening on com.termsurf.compositor...\n", stderr)
    }

    // MARK: - Message handling

    private func handleMessage(_ msg: xpc_object_t, from peer: xpc_connection_t) {
        guard let actionPtr = xpc_dictionary_get_string(msg, "action") else { return }
        let action = String(cString: actionPtr)

        switch action {
        case "set_overlay":
            guard let paneIdPtr = xpc_dictionary_get_string(msg, "pane_id") else {
                fputs("[Compositor] set_overlay missing pane_id\n", stderr)
                return
            }
            let paneIdStr = String(cString: paneIdPtr)
            guard let uuid = UUID(uuidString: paneIdStr) else {
                fputs("[Compositor] invalid pane_id: \(paneIdStr)\n", stderr)
                return
            }

            let col = UInt32(xpc_dictionary_get_uint64(msg, "col"))
            let row = UInt32(xpc_dictionary_get_uint64(msg, "row"))
            let width = UInt32(xpc_dictionary_get_uint64(msg, "width"))
            let height = UInt32(xpc_dictionary_get_uint64(msg, "height"))

            // Remember which pane this peer controls (for cleanup on disconnect).
            let peerId = ObjectIdentifier(peer as AnyObject)
            peerPaneIds[peerId] = uuid

            // Look up the surface and set the overlay.
            DispatchQueue.main.async { [weak self] in
                guard let self = self,
                      let surface = self.appDelegate?.findSurface(forUUID: uuid),
                      let cSurface = surface.surface else {
                    fputs("[Compositor] surface not found for pane \(paneIdStr)\n", stderr)
                    return
                }
                ghostty_surface_set_overlay(cSurface, col, row, width, height)
            }

        default:
            fputs("[Compositor] unknown action: \(action)\n", stderr)
        }
    }

    private func handleDisconnect(_ peer: xpc_connection_t) {
        fputs("[Compositor] Client disconnected\n", stderr)

        // Remove from peers list.
        peers.removeAll { $0 === peer }

        // Clear overlay for the pane this peer was controlling.
        let peerId = ObjectIdentifier(peer as AnyObject)
        if let uuid = peerPaneIds.removeValue(forKey: peerId) {
            DispatchQueue.main.async { [weak self] in
                guard let self = self,
                      let surface = self.appDelegate?.findSurface(forUUID: uuid),
                      let cSurface = surface.surface else { return }
                ghostty_surface_clear_overlay(cSurface)
            }
        }
    }
}
