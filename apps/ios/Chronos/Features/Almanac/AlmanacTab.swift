import SwiftUI

/// Trạch Nhật và Xem Tuổi: Tam Tai, Kim Lâu and Hoang Ốc of the selected person in a lunar year.
struct AlmanacTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @State private var targetYear: Int32

    init(services: AppServices) {
        self.services = services
        let today = services.engine.today()
        _targetYear = State(initialValue: (try? services.engine.lunar(today).year) ?? today.year)
    }

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    List {
                        Section {
                            Stepper("Năm âm lịch \(String(targetYear))", value: $targetYear, in: Int32(EngineService.supportedYears.lowerBound)...Int32(EngineService.supportedYears.upperBound))
                        }
                        switch Result(catching: { try reading(profile) }) {
                        case .success(let reading):
                            Section("Tuổi mụ \(reading.restrictions.lunarAge)") {
                                if reading.entries.isEmpty {
                                    Text("Năm \(String(targetYear)) không phạm Tam Tai, Kim Lâu, Hoang Ốc.")
                                        .foregroundStyle(Theme.ink)
                                } else {
                                    ForEach(reading.entries) { EntryView(entry: $0, content: services.content) }
                                }
                            }
                        case .failure(let error):
                            Text(String(describing: error)).foregroundStyle(Theme.cinnabar)
                        }
                    }
                    .chronosBackground()
                } else {
                    NoProfileView()
                }
            }
            .navigationTitle("Xem tuổi")
            .profileToolbar(services)
        }
    }

    private struct Reading {
        let restrictions: AgeRestrictions
        let entries: [ContentEntry]
    }

    private func reading(_ profile: Profile) throws -> Reading {
        let restrictions = try services.engine.ageRestrictions(for: profile, targetLunarYear: targetYear)
        return Reading(restrictions: restrictions, entries: try services.content.ageRestrictions(restrictions))
    }
}

/// Shown by the tabs that read a person until one is saved.
struct NoProfileView: View {
    var body: some View {
        UnavailableView(
            title: "Chưa chọn hồ sơ",
            message: "Mở menu hồ sơ ở góc trên để thêm người cần xem.",
            systemImage: "person.crop.circle.badge.questionmark"
        )
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.paper)
    }
}
