// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "TwoProfilesSwift",
    platforms: [.macOS(.v14)],
    targets: [
        .executableTarget(
            name: "Receiver",
            path: "Sources/Receiver",
            exclude: ["Shaders.metal"],
            linkerSettings: [
                .linkedFramework("Cocoa"),
                .linkedFramework("Metal"),
                .linkedFramework("QuartzCore"),
                .linkedFramework("IOSurface"),
            ]
        )
    ]
)
