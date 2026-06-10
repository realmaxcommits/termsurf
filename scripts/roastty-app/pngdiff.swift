// Compare two PNGs and emit one JSON metrics object.
// Usage:
//   swift pngdiff.swift <expected.png> <actual.png>
//   swift pngdiff.swift <expected.png> <actual.png> --max-mismatch-ratio 0.01 --max-mean-channel-delta 2.0
import AppKit
import Foundation

struct Options {
    var expected: String
    var actual: String
    var maxMismatchRatio: Double = 0.0
    var maxMeanChannelDelta: Double = 0.0
}

func usage(exitCode: Int32) -> Never {
    FileHandle.standardError.write(
        """
        usage: pngdiff.swift <expected.png> <actual.png> [--max-mismatch-ratio N] [--max-mean-channel-delta N]
        emits one JSON object on stdout; diagnostics go to stderr
        """.data(using: .utf8)!
    )
    exit(exitCode)
}

func parseOptions(_ args: [String]) -> Options {
    if args.count == 2, args[1] == "--help" {
        usage(exitCode: 0)
    }
    guard args.count >= 3 else {
        usage(exitCode: 2)
    }

    var options = Options(expected: args[1], actual: args[2])
    var index = 3
    while index < args.count {
        let key = args[index]
        guard index + 1 < args.count else {
            FileHandle.standardError.write("missing value for \(key)\n".data(using: .utf8)!)
            usage(exitCode: 2)
        }
        let value = args[index + 1]
        switch key {
        case "--max-mismatch-ratio":
            guard let parsed = Double(value), parsed >= 0.0 else {
                FileHandle.standardError.write("invalid --max-mismatch-ratio: \(value)\n".data(using: .utf8)!)
                usage(exitCode: 2)
            }
            options.maxMismatchRatio = parsed
        case "--max-mean-channel-delta":
            guard let parsed = Double(value), parsed >= 0.0 else {
                FileHandle.standardError.write("invalid --max-mean-channel-delta: \(value)\n".data(using: .utf8)!)
                usage(exitCode: 2)
            }
            options.maxMeanChannelDelta = parsed
        default:
            FileHandle.standardError.write("unknown option: \(key)\n".data(using: .utf8)!)
            usage(exitCode: 2)
        }
        index += 2
    }
    return options
}

func loadBitmap(_ path: String) -> NSBitmapImageRep? {
    guard let image = NSImage(contentsOfFile: path),
          let tiff = image.tiffRepresentation,
          let bitmap = NSBitmapImageRep(data: tiff)
    else {
        return nil
    }
    return bitmap
}

func component(_ value: CGFloat) -> Int {
    Int((max(0.0, min(1.0, value)) * 255.0).rounded())
}

func rgba(_ bitmap: NSBitmapImageRep, x: Int, y: Int) -> [Int]? {
    guard let color = bitmap.colorAt(x: x, y: y)?.usingColorSpace(.sRGB) else {
        return nil
    }
    return [
        component(color.redComponent),
        component(color.greenComponent),
        component(color.blueComponent),
        component(color.alphaComponent),
    ]
}

func emit(_ object: [String: Any]) {
    let data = try! JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
    FileHandle.standardOutput.write(data)
    FileHandle.standardOutput.write("\n".data(using: .utf8)!)
}

let options = parseOptions(CommandLine.arguments)
guard let expected = loadBitmap(options.expected) else {
    FileHandle.standardError.write("failed to load expected PNG: \(options.expected)\n".data(using: .utf8)!)
    exit(2)
}
guard let actual = loadBitmap(options.actual) else {
    FileHandle.standardError.write("failed to load actual PNG: \(options.actual)\n".data(using: .utf8)!)
    exit(2)
}

let width = expected.pixelsWide
let height = expected.pixelsHigh
if actual.pixelsWide != width || actual.pixelsHigh != height {
    emit([
        "verdict": "FAIL",
        "error": "dimension_mismatch",
        "expected_width": width,
        "expected_height": height,
        "actual_width": actual.pixelsWide,
        "actual_height": actual.pixelsHigh,
        "compared_pixels": 0,
        "mismatched_pixels": 0,
        "mismatch_ratio": 1.0,
        "mean_channel_delta": 0.0,
        "max_channel_delta": 0,
        "max_mismatch_ratio": options.maxMismatchRatio,
        "max_mean_channel_delta": options.maxMeanChannelDelta,
    ])
    exit(1)
}

let comparedPixels = width * height
var mismatchedPixels = 0
var totalChannelDelta = 0
var maxChannelDelta = 0

for y in 0..<height {
    for x in 0..<width {
        guard let a = rgba(expected, x: x, y: y),
              let b = rgba(actual, x: x, y: y)
        else {
            FileHandle.standardError.write("failed to read pixel at \(x),\(y)\n".data(using: .utf8)!)
            exit(2)
        }

        var pixelDiffers = false
        for i in 0..<4 {
            let delta = abs(a[i] - b[i])
            totalChannelDelta += delta
            maxChannelDelta = max(maxChannelDelta, delta)
            if delta != 0 {
                pixelDiffers = true
            }
        }
        if pixelDiffers {
            mismatchedPixels += 1
        }
    }
}

let mismatchRatio = comparedPixels == 0 ? 0.0 : Double(mismatchedPixels) / Double(comparedPixels)
let meanChannelDelta = comparedPixels == 0 ? 0.0 : Double(totalChannelDelta) / Double(comparedPixels * 4)
let passed = mismatchRatio <= options.maxMismatchRatio
    && meanChannelDelta <= options.maxMeanChannelDelta

emit([
    "verdict": passed ? "PASS" : "FAIL",
    "width": width,
    "height": height,
    "compared_pixels": comparedPixels,
    "mismatched_pixels": mismatchedPixels,
    "mismatch_ratio": mismatchRatio,
    "mean_channel_delta": meanChannelDelta,
    "max_channel_delta": maxChannelDelta,
    "max_mismatch_ratio": options.maxMismatchRatio,
    "max_mean_channel_delta": options.maxMeanChannelDelta,
])

exit(passed ? 0 : 1)
