// swift-tools-version:5.9
//
// JovaSample — integration reference for jovawallet-core's Swift binding.
//
// This package depends on the jovawallet-core-swift satellite repo for the
// SDK proper (Phase 5 publishes the satellite at v1.0.0; before then, point
// the dependency at a local XCFramework built via
// `bindings/swift/scripts/build-xcframework.sh` on a Mac).
//
import PackageDescription

let package = Package(
    name: "JovaSample",
    platforms: [
        .iOS(.v14),
        .macOS(.v11),
    ],
    products: [
        .executable(name: "JovaSample", targets: ["JovaSample"]),
    ],
    dependencies: [
        // .package(url: "https://github.com/jovachain/jovawallet-core-swift.git", from: "1.0.0"),
        // Until the satellite repo is published, point at a sibling local checkout:
        .package(path: "../../bindings/swift"),
    ],
    targets: [
        .executableTarget(
            name: "JovaSample",
            dependencies: [
                .product(name: "JovaCore", package: "swift"),
            ]
        ),
    ]
)
