import SwiftUI

/// The engine and the content database, opened once at launch.
struct AppServices: Sendable {
    let engine: EngineService
    let content: ContentRepository

    static func bundled() throws -> AppServices {
        AppServices(engine: EngineService(), content: try ContentRepository())
    }
}

@main
struct ChronosApp: App {
    @State private var profiles = ProfileStore()
    private let services = Result { try AppServices.bundled() }

    var body: some Scene {
        WindowGroup {
            Group {
                switch services {
                case .success(let services):
                    MainTabView(services: services)
                case .failure(let error):
                    UnavailableView(title: "Không mở được dữ liệu luận giải", message: String(describing: error))
                }
            }
            .environment(profiles)
            .tint(Theme.cinnabar)
        }
    }
}
