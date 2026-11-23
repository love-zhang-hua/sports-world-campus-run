import SwiftUI

@main struct CampusRunApp: App {
    @StateObject private var model = RouteModel()
    @Environment(\.scenePhase) private var phase
    var body: some Scene {
        WindowGroup {
            ContentView(model: model)
                .tint(Color(red: 0.07, green: 0.41, blue: 0.33))
                .task { await model.startSession() }
                .onChange(of: phase) { state in if state != .active { model.background() } }
        }
    }
}
