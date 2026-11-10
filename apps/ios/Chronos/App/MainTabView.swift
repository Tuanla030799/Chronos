import SwiftUI

/// The five modules of the app.
struct MainTabView: View {
    let services: AppServices

    var body: some View {
        TabView {
            CalendarTab(services: services)
                .tabItem { Label("Lịch", systemImage: "calendar") }
            AlmanacTab(services: services)
                .tabItem { Label("Xem tuổi", systemImage: "checkmark.seal") }
            BaziTab(services: services)
                .tabItem { Label("Bát Tự", systemImage: "square.grid.2x2") }
            TuViTab(services: services)
                .tabItem { Label("Tử Vi", systemImage: "circle.grid.3x3") }
            FengShuiTab(services: services)
                .tabItem { Label("Phong Thủy", systemImage: "safari") }
        }
    }
}

extension View {
    /// The profile menu every tab shows in its navigation bar.
    func profileToolbar(_ services: AppServices) -> some View {
        modifier(ProfileToolbar(engine: services.engine))
    }
}

private struct ProfileToolbar: ViewModifier {
    let engine: EngineService
    @Environment(ProfileStore.self) private var store
    @State private var managing = false

    func body(content: Content) -> some View {
        content
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Menu {
                        ForEach(store.profiles) { profile in
                            Button {
                                store.select(profile.id)
                            } label: {
                                if profile.id == store.selectedID {
                                    Label(profile.name, systemImage: "checkmark")
                                } else {
                                    Text(profile.name)
                                }
                            }
                        }
                        Divider()
                        Button("Quản lý hồ sơ", systemImage: "person.2") { managing = true }
                    } label: {
                        Label(store.selected?.name ?? "Hồ sơ", systemImage: "person.crop.circle")
                            .labelStyle(.titleAndIcon)
                    }
                }
            }
            .sheet(isPresented: $managing) {
                ProfileListView(engine: engine)
            }
    }
}
