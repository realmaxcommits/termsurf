import AppKit
import Darwin
import GhosttyKit

@_cdecl("termsurf_clear_overlay")
func termsurf_clear_overlay(_ paneIDPointer: UnsafePointer<CChar>?) {
    guard let paneIDPointer else {
        termsurfLogOverlay("TermSurf overlay clear rejected: missing pane id")
        return
    }

    let paneID = String(cString: paneIDPointer)
    termsurfLogOverlay("TermSurf overlay clear request pane_id=\(paneID)")

    DispatchQueue.main.async {
        guard let appDelegate = NSApplication.shared.delegate as? AppDelegate else {
            termsurfLogOverlay("TermSurf overlay clear rejected: missing app delegate")
            return
        }
        guard let uuid = UUID(uuidString: paneID) else {
            termsurfLogOverlay("TermSurf overlay clear rejected: invalid pane id \(paneID)")
            return
        }
        guard let target = appDelegate.findSurface(forUUID: uuid) else {
            termsurfLogOverlay("TermSurf overlay clear rejected: no surface for pane id \(paneID)")
            return
        }

        target.clearTermSurfOverlay()
    }
}

@_cdecl("termsurf_present_overlay")
// swiftlint:disable:next function_parameter_count
func termsurf_present_overlay(
    _ paneIDPointer: UnsafePointer<CChar>?,
    _ contextID: UInt64,
    _ col: UInt64,
    _ row: UInt64,
    _ width: UInt64,
    _ height: UInt64,
    _ pixelWidth: UInt64,
    _ pixelHeight: UInt64
) {
    guard let paneIDPointer else {
        termsurfLogOverlay("TermSurf overlay rejected: missing pane id")
        return
    }

    let paneID = String(cString: paneIDPointer)
    termsurfLogOverlay(
        "TermSurf overlay request pane_id=\(paneID) context_id=\(contextID) grid=\(width)x\(height)+\(col)+\(row) pixel=\(pixelWidth)x\(pixelHeight)")

    DispatchQueue.main.async {
        guard let appDelegate = NSApplication.shared.delegate as? AppDelegate else {
            termsurfLogOverlay("TermSurf overlay rejected: missing app delegate")
            return
        }
        guard let uuid = UUID(uuidString: paneID) else {
            termsurfLogOverlay("TermSurf overlay rejected: invalid pane id \(paneID)")
            return
        }
        guard let target = appDelegate.findSurface(forUUID: uuid) else {
            termsurfLogOverlay("TermSurf overlay rejected: no surface for pane id \(paneID)")
            return
        }

        target.presentTermSurfOverlay(
            contextID: contextID,
            col: col,
            row: row,
            width: width,
            height: height,
            pixelWidth: pixelWidth,
            pixelHeight: pixelHeight)
    }
}

@_cdecl("termsurf_open_split")
func termsurf_open_split(
    _ paneIDPointer: UnsafePointer<CChar>?,
    _ directionPointer: UnsafePointer<CChar>?,
    _ commandPointer: UnsafePointer<CChar>?
) {
    guard let paneIDPointer, let directionPointer, let commandPointer else {
        termsurfLogOpenSplit("TermSurf OpenSplit rejected: missing C string")
        return
    }

    let paneID = String(cString: paneIDPointer)
    let direction = String(cString: directionPointer)
    let command = String(cString: commandPointer)

    termsurfLogOpenSplit("TermSurf OpenSplit request pane_id=\(paneID) direction=\(direction)")

    DispatchQueue.main.async {
        guard let appDelegate = NSApplication.shared.delegate as? AppDelegate else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: missing app delegate")
            return
        }
        guard let uuid = UUID(uuidString: paneID) else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: invalid pane id \(paneID)")
            return
        }
        guard let target = appDelegate.findSurface(forUUID: uuid) else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: no surface for pane id \(paneID)")
            return
        }
        guard let splitDirection = termsurfSplitDirection(direction) else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: invalid direction \(direction)")
            return
        }
        guard let surface = target.surface else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: target surface is unavailable")
            return
        }
        guard let controller = target.window?.windowController as? BaseTerminalController else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: target has no terminal controller")
            return
        }

        var config = Ghostty.SurfaceConfiguration(
            from: ghostty_surface_inherited_config(surface, GHOSTTY_SURFACE_CONTEXT_SPLIT))
        config.command = command

        guard controller.newSplit(at: target, direction: splitDirection, baseConfig: config) != nil else {
            termsurfLogOpenSplit("TermSurf OpenSplit rejected: split creation failed")
            return
        }

        termsurfLogOpenSplit("TermSurf OpenSplit created split pane_id=\(paneID) direction=\(direction)")
    }
}

private func termsurfLogOpenSplit(_ message: String) {
    AppDelegate.logger.info("\(message)")
    fputs("\(message)\n", stderr)
}

private func termsurfLogOverlay(_ message: String) {
    AppDelegate.logger.info("\(message)")
    fputs("\(message)\n", stderr)
}

private func termsurfSplitDirection(_ direction: String) -> SplitTree<Ghostty.SurfaceView>.NewDirection? {
    switch direction {
    case "right":
        return .right
    case "left":
        return .left
    case "down":
        return .down
    case "up":
        return .up
    default:
        return nil
    }
}
