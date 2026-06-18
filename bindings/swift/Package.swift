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
            // DEV (this branch): local path to the freshly-built XCFramework. The
            // iOS app adds this package by LOCAL PATH (../jovawallet-core-swift) so
            // it links this artifact directly — see ios-wallet feature/jova-core-sdk.
            // RELEASE (merge gate): replace with the published v0.4.0 URL + checksum:
            //   .binaryTarget(name: "JovaCoreFFI",
            //                 url: "https://github.com/jovachain/jovawallet-core-swift/releases/download/v0.4.0/JovaCoreFFI.xcframework.zip",
            //                 checksum: "<sha256 of the zip>")
            // To get the checksum: swift package compute-checksum bindings/swift/JovaCoreFFI.xcframework.zip
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
