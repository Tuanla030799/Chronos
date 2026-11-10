import SwiftUI

enum AppTab: Int, CaseIterable {
    case calendar, almanac, bazi, tuvi, fengshui

    /// The tab to open first: `-ChronosInitialTab 3` on the command line opens Tử Vi.
    static var initial: AppTab {
        AppTab(rawValue: UserDefaults.standard.integer(forKey: "ChronosInitialTab")) ?? .calendar
    }
}

/// The five modules of the app.
struct MainTabView: View {
    let services: AppServices
    @State private var selection = AppTab.initial

    var body: some View {
        TabView(selection: $selection) {
            CalendarTab(services: services)
                .tabItem { Label("Lịch", systemImage: "calendar") }
                .tag(AppTab.calendar)
            AlmanacTab(services: services)
                .tabItem { Label("Xem tuổi", systemImage: "checkmark.seal") }
                .tag(AppTab.almanac)
            BaziTab(services: services)
                .tabItem { Label("Bát Tự", systemImage: "square.grid.2x2") }
                .tag(AppTab.bazi)
            TuViTab(services: services)
                .tabItem { Label("Tử Vi", systemImage: "circle.grid.3x3") }
                .tag(AppTab.tuvi)
            FengShuiTab(services: services)
                .tabItem { Label("Phong Thủy", systemImage: "safari") }
                .tag(AppTab.fengshui)
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
