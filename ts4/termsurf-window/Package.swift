// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "termsurf-window",
    platforms: [.macOS(.v13)],
    targets: [
        .target(
            name: "XIPC",
            path: "Sources/XIPC",
            linkerSettings: [
                .linkedFramework("IOSurface"),
            ]
        ),
        .executableTarget(
            name: "termsurf-window",
            dependencies: ["XIPC"],
            path: "Sources/Window"
        ),
    ]
)
