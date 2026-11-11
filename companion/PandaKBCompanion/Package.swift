// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "PandaKBCompanion",
    platforms: [
        .macOS(.v13),
    ],
    products: [
        .executable(name: "PandaKBCompanion", targets: ["PandaKBCompanion"]),
    ],
    targets: [
        .executableTarget(
            name: "PandaKBCompanion",
            path: "Sources/PandaKBCompanion"
        ),
    ]
)
