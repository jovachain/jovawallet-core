// JovaSample — entry point.
//
// Minimal SwiftUI host so the integration can be exercised on an iPhone
// simulator or device. Build on a Mac with Xcode 15+; this Linux dev VM
// can't build Swift code.

import SwiftUI

@main
struct JovaSampleApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}
