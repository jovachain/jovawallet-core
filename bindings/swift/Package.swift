// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "JovaCore",
    platforms: [.iOS(.v14), .macOS(.v11)],
    products: [
        .library(name: "JovaCore", targets: ["JovaCore"]),
    ],
    targets: [
        .binaryTarget(
            name: "JovaCoreFFI",
            // Phase 0: local-only build. CI replaces this with the XCFramework path.
            path: "JovaCoreFFI.xcframework"
        ),
        .target(
            name: "JovaCore",
            dependencies: ["JovaCoreFFI"],
            path: "Sources/JovaCore"
        ),
        .testTarget(
            name: "JovaCoreTests",
            dependencies: ["JovaCore"],
            path: "Tests/JovaCoreTests"
        ),
    ]
)
