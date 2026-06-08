// Issue 802 / Exp 5 — CGEvent mouse injector (keyboard goes via osascript).
// Coordinates are global points (same units as CGWindowBounds from winid.swift).
//   inject.swift move  <x> <y>
//   inject.swift click <x> <y> [left|right|middle] [count]
//   inject.swift drag  <x1> <y1> <x2> <y2>
//   inject.swift scroll <x> <y> <lines>      (positive = up, negative = down)
// Requires Accessibility for the responsible app (the host terminal).
import CoreGraphics
import Foundation

let a = CommandLine.arguments
func n(_ i: Int) -> CGFloat { CGFloat(Double(a[i]) ?? 0) }
func post(_ e: CGEvent?) { e?.post(tap: .cghidEventTap) }
func ev(_ t: CGEventType, _ p: CGPoint, _ b: CGMouseButton) -> CGEvent? {
    CGEvent(mouseEventSource: nil, mouseType: t, mouseCursorPosition: p, mouseButton: b)
}
guard a.count >= 2 else {
    FileHandle.standardError.write("usage: inject <move|click|drag|scroll> ...\n".data(using: .utf8)!)
    exit(2)
}

switch a[1] {
case "move":
    post(ev(.mouseMoved, CGPoint(x: n(2), y: n(3)), .left))

case "click":
    let p = CGPoint(x: n(2), y: n(3))
    let which = a.count > 4 ? a[4] : "left"
    let count = a.count > 5 ? (Int(a[5]) ?? 1) : 1
    let (down, up, b): (CGEventType, CGEventType, CGMouseButton) =
        which == "right" ? (.rightMouseDown, .rightMouseUp, .right)
        : which == "middle" ? (.otherMouseDown, .otherMouseUp, .center)
        : (.leftMouseDown, .leftMouseUp, .left)
    post(ev(.mouseMoved, p, b))
    for i in 1...count {
        let d = ev(down, p, b); d?.setIntegerValueField(.mouseEventClickState, value: Int64(i)); post(d)
        let u = ev(up, p, b); u?.setIntegerValueField(.mouseEventClickState, value: Int64(i)); post(u)
    }

case "drag":
    let p1 = CGPoint(x: n(2), y: n(3)), p2 = CGPoint(x: n(4), y: n(5))
    post(ev(.mouseMoved, p1, .left))
    post(ev(.leftMouseDown, p1, .left))
    let steps = 12
    for i in 1...steps {
        let t = CGFloat(i) / CGFloat(steps)
        post(ev(.leftMouseDragged, CGPoint(x: p1.x + (p2.x - p1.x) * t, y: p1.y + (p2.y - p1.y) * t), .left))
    }
    post(ev(.leftMouseUp, p2, .left))

case "scroll":
    post(ev(.mouseMoved, CGPoint(x: n(2), y: n(3)), .left))
    let e = CGEvent(scrollWheelEvent2Source: nil, units: .line, wheelCount: 1,
                    wheel1: Int32(Double(a[4]) ?? 0), wheel2: 0, wheel3: 0)
    e?.post(tap: .cghidEventTap)

default:
    FileHandle.standardError.write("unknown subcommand: \(a[1])\n".data(using: .utf8)!)
    exit(2)
}
