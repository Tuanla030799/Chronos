import SwiftUI

/// The two choices the Four Pillars depend on, kept across launches: where the year and month
/// change, and what a birth at 23:00 to 23:59 belongs to.
struct BaziSettings: Equatable {
    var lunarNewYearBoundary = false
    var newDayAtRatHour = false

    var boundary: YearMonthBoundary { lunarNewYearBoundary ? .lunarNewYear : .solarTerms }
    var ratHour: RatHourConvention { newDayAtRatHour ? .newDayAtRatHour : .splitAtMidnight }
}

/// Everything the tab shows for one person: the chart and the entries of its keys.
struct BaziReading {
    let analysis: BaziAnalysis
    let entries: [ContentEntry]
    let currentYear: Int32
    private let byKey: [String: ContentEntry]
    private let content: ContentRepository

    init(profile: Profile, settings: BaziSettings, services: AppServices) throws {
        content = services.content
        analysis = try services.engine.bazi(for: profile, ratHour: settings.ratHour, boundary: settings.boundary)
        entries = try services.content.bazi(analysis)
        byKey = Dictionary(entries.map { ($0.key, $0) }, uniquingKeysWith: { first, _ in first })
        currentYear = services.engine.today().year
    }

    /// The entry of `key`. `baziKeys` lists the Thập Thần of the stems of the pillars only, so a
    /// key the chart shows elsewhere, such as the Thập Thần of a hidden stem, is read directly.
    func entry(_ key: String) -> ContentEntry? {
        byKey[key] ?? (try? content.entry(key))
    }
}

/// Bát Tự: the Four Pillars with their hidden stems, life stages, Nạp Âm and Thần Sát, the
/// luck cycles, then the readings. Every term opens its text.
struct BaziTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @AppStorage("chronos.bazi.lunarNewYearBoundary") private var lunarNewYearBoundary = false
    @AppStorage("chronos.bazi.newDayAtRatHour") private var newDayAtRatHour = false
    @State private var opened: ContentEntry?

    private var settings: BaziSettings {
        BaziSettings(lunarNewYearBoundary: lunarNewYearBoundary, newDayAtRatHour: newDayAtRatHour)
    }

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    ScrollView {
                        VStack(spacing: 16) {
                            BaziHeaderCard(
                                profile: profile,
                                lunarNewYearBoundary: $lunarNewYearBoundary,
                                newDayAtRatHour: $newDayAtRatHour
                            )
                            switch Result(catching: { try BaziReading(profile: profile, settings: settings, services: services) }) {
                            case .success(let reading):
                                DayMasterRow(reading: reading) { opened = $0 }
                                PillarTableCard(reading: reading, profile: profile) { opened = $0 }
                                LuckPillarsCard(daiVan: reading.analysis.daiVan, currentYear: reading.currentYear)
                                BaziEntriesCard(entries: reading.entries) { opened = $0 }
                            case .failure(let error):
                                UnavailableView(title: "Không lập được Tứ Trụ", message: String(describing: error))
                                    .card()
                            }
                        }
                        .padding(16)
                    }
                    .background(Theme.paper)
                } else {
                    NoProfileView()
                }
            }
            .navigationTitle("Bát Tự")
            .navigationBarTitleDisplayMode(.inline)
            .profileToolbar(services)
            .sheet(item: $opened) { EntrySheet(entry: $0, content: services.content) }
        }
    }
}

// MARK: - Header

/// Who is read, and the two boundary choices of the chart.
private struct BaziHeaderCard: View {
    let profile: Profile
    @Binding var lunarNewYearBoundary: Bool
    @Binding var newDayAtRatHour: Bool

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ProfileHeaderRow(profile: profile)

            Divider().overlay(Theme.rule)

            FlowLayout(spacing: 8) {
                OptionChip(title: "Năm, tháng đổi", value: lunarNewYearBoundary ? "Mùng 1 âm lịch" : "Tiết khí") {
                    Picker("Năm, tháng đổi tại", selection: $lunarNewYearBoundary) {
                        Text("Tiết khí (Lập Xuân)").tag(false)
                        Text("Mùng 1 âm lịch (Tết)").tag(true)
                    }
                }
                OptionChip(title: "Giờ Tý", value: newDayAtRatHour ? "Sang ngày từ 23h" : "Chia nửa đêm") {
                    Picker("Giờ Tý", selection: $newDayAtRatHour) {
                        Text("Chia ở nửa đêm (Tý sớm, Tý muộn)").tag(false)
                        Text("Sang ngày từ 23h").tag(true)
                    }
                }
                Chip(
                    text: profile.longitude.map { String(format: "Giờ mặt trời thực · %.1f°Đ", $0) } ?? "Giờ đồng hồ"
                )
            }
        }
        .card()
    }
}

/// A chip that opens a menu of choices.
private struct OptionChip<Choices: View>: View {
    let title: String
    let value: String
    @ViewBuilder let choices: Choices

    var body: some View {
        Menu {
            choices
        } label: {
            HStack(spacing: 4) {
                Text("\(title): \(value)")
                Image(systemName: "chevron.up.chevron.down").font(.system(size: 9, weight: .bold))
            }
            .font(.caption.weight(.medium))
            .foregroundStyle(Theme.cinnabar)
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .background(Theme.cinnabar.opacity(0.1), in: Capsule())
        }
    }
}

// MARK: - Day Master and Nạp Âm

/// The Day Master and the Nạp Âm of the birth year, side by side.
private struct DayMasterRow: View {
    let reading: BaziReading
    let open: (ContentEntry) -> Void

    var body: some View {
        let analysis = reading.analysis
        let stem = analysis.pillars.day.stem
        HStack(alignment: .top, spacing: 12) {
            tile(
                title: "Nhật Chủ",
                name: stemName(stem: stem),
                element: stemElement(stem: stem),
                entry: reading.entry(baziDayMasterKey(stem: stem))
            )
            tile(
                title: "Bản mệnh Nạp Âm",
                name: naYinName(value: analysis.year.naYin),
                element: naYinElement(value: analysis.year.naYin),
                entry: reading.entry(baziNaYinKey(naYin: analysis.year.naYin))
            )
        }
    }

    @ViewBuilder
    private func tile(title: String, name: String, element: FiveElements, entry: ContentEntry?) -> some View {
        let content = VStack(alignment: .leading, spacing: 6) {
            Text(title.uppercased())
                .font(.caption2.weight(.semibold)).tracking(0.6)
                .foregroundStyle(Theme.inkSecondary)
            HStack(spacing: 6) {
                Text(name).font(.title2.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                    .lineLimit(1).minimumScaleFactor(0.7)
                ElementBadge(element: element)
            }
            if let entry {
                Text(entry.summary)
                    .font(.caption).foregroundStyle(Theme.inkSecondary)
                    .lineLimit(4).multilineTextAlignment(.leading)
            }
        }
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .card(padding: 12)

        if let entry {
            Button { open(entry) } label: { content }
                .buttonStyle(.plain)
                .accessibilityHint("Mở giải nghĩa")
        } else {
            content
        }
    }
}

// MARK: - Readings

/// Index of the readings, by kind. The full text opens in a sheet.
private struct BaziEntriesCard: View {
    let entries: [ContentEntry]
    let open: (ContentEntry) -> Void

    private static let groups: [(title: String, collection: String, icon: String)] = [
        ("Nhật Chủ", "bazi.day_master", "sun.max"),
        ("Nạp Âm", "bazi.nayin", "waveform"),
        ("Thập Thần", "bazi.ten_god", "person.2"),
        ("Trường Sinh", "bazi.life_stage", "arrow.triangle.2.circlepath"),
        ("Thần Sát", "bazi.shen_sha", "sparkle"),
        ("Can Chi", "bazi.can_chi", "character"),
    ]

    var body: some View {
        ForEach(Self.groups, id: \.collection) { group in
            let items = entries.filter { $0.collection == group.collection }
            if !items.isEmpty {
                VStack(spacing: 10) {
                    CardTitle(text: group.title, systemImage: group.icon)
                    ForEach(items) { entry in
                        EntryRow(entry: entry) { open(entry) }
                        if entry.id != items.last?.id {
                            Divider().overlay(Theme.rule)
                        }
                    }
                }
                .card(padding: 14)
            }
        }
    }
}
