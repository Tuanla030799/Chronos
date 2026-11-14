import SwiftUI

/// What the tab opens in a sheet.
enum TuViSheet: Identifiable {
    case entry(ContentEntry)
    case star(TuViPlacedStar, Branch)

    var id: String {
        switch self {
        case .entry(let entry): "entry/\(entry.key)"
        case .star(let placed, let branch): "star/\(placed.star)/\(branch)"
        }
    }
}

/// Tử Vi: the chart on the Địa Bàn with the Thiên Bàn in the middle, a year to read its Đại Hạn
/// and Tiểu Hạn, and the reading of the palace chosen on the board.
struct TuViTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @AppStorage("chronos.tuvi.leapMonthAsCurrent") private var leapMonthAsCurrent = false
    @AppStorage("chronos.tuvi.newDayAtRatHour") private var newDayAtRatHour = false
    @State private var targetYear: Int32
    @State private var selected: Branch?
    @State private var sheet: TuViSheet?
    private let currentYear: Int32

    init(services: AppServices) {
        self.services = services
        let today = services.engine.today()
        let year = (try? services.engine.lunar(today).year) ?? today.year
        currentYear = year
        _targetYear = State(initialValue: year)
    }

    private var settings: TuViSettings {
        TuViSettings(leapMonthAsCurrent: leapMonthAsCurrent, newDayAtRatHour: newDayAtRatHour)
    }

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    ScrollView {
                        VStack(spacing: 16) {
                            content(profile)
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 16)
                    }
                    .background(Theme.paper)
                } else {
                    NoProfileView()
                }
            }
            .navigationTitle("Tử Vi")
            .navigationBarTitleDisplayMode(.inline)
            .profileToolbar(services)
            .onChange(of: store.selectedID) { selected = nil }
            .sheet(item: $sheet) { sheet in
                switch sheet {
                case .entry(let entry):
                    EntrySheet(entry: entry, content: services.content)
                case .star(let placed, let branch):
                    TuViStarSheet(placed: placed, branch: branch, content: services.content)
                }
            }
        }
    }

    @ViewBuilder
    private func content(_ profile: Profile) -> some View {
        let chart = Result {
            try services.engine.tuviChart(for: profile, leapMonthRule: settings.leapMonthRule, ratHour: settings.ratHour)
        }
        switch chart {
        case .success(let chart):
            let year = TuViYearReading(chart: chart, lunarYear: targetYear)
            let branch = selected ?? chart.menhBranch
            header(profile, year: year).padding(.horizontal, 8)
            TuViBoard(chart: chart, name: profile.name, year: year, selected: branch) { tapped in
                withAnimation(.snappy) { selected = tapped }
            }
            BoardLegend().padding(.horizontal, 8)
            if let palace = chart.palace(at: branch) {
                PalaceDetail(chart: chart, palace: palace, year: year, content: services.content) { sheet = $0 }
                    .padding(.horizontal, 8)
            }
        case .failure(let error):
            UnavailableView(title: "Không lập được lá số", message: String(describing: error))
                .card()
                .padding(.horizontal, 8)
        }
    }

    // MARK: Header

    private func header(_ profile: Profile, year: TuViYearReading) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            ProfileHeaderRow(profile: profile)
            Divider().overlay(Theme.rule)
            HStack {
                Button { targetYear -= 1 } label: { Image(systemName: "chevron.left") }
                    .accessibilityLabel("Năm trước")
                    .disabled(Int(targetYear) <= EngineService.supportedYears.lowerBound)
                Spacer()
                VStack(spacing: 2) {
                    Text("Năm xem \(String(targetYear))")
                        .font(.headline).fontDesign(.serif).foregroundStyle(Theme.ink)
                    Text(yearSubtitle(year))
                        .font(.caption).foregroundStyle(Theme.inkSecondary)
                }
                Spacer()
                Button { targetYear += 1 } label: { Image(systemName: "chevron.right") }
                    .accessibilityLabel("Năm sau")
                    .disabled(Int(targetYear) >= EngineService.supportedYears.upperBound)
            }
            FlowLayout(spacing: 8) {
                OptionChip(title: "Tháng nhuận", value: leapMonthAsCurrent ? "Như tháng trước" : "Chia ở ngày 15") {
                    Picker("Tháng nhuận", selection: $leapMonthAsCurrent) {
                        Text("Mùng 1–15 tính tháng trước, 16 trở đi tính tháng sau").tag(false)
                        Text("Cả tháng tính như tháng trước").tag(true)
                    }
                }
                OptionChip(title: "Giờ Tý", value: newDayAtRatHour ? "Sang ngày từ 23h" : "Chia nửa đêm") {
                    Picker("Giờ Tý", selection: $newDayAtRatHour) {
                        Text("Chia ở nửa đêm (Tý sớm, Tý muộn)").tag(false)
                        Text("Sang ngày từ 23h").tag(true)
                    }
                }
                if targetYear != currentYear {
                    Button("Năm nay") { targetYear = currentYear }
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(Theme.cinnabar)
                        .padding(.vertical, 5)
                }
            }
        }
        .card()
    }

    private func yearSubtitle(_ year: TuViYearReading) -> String {
        var parts: [String] = []
        if let canChi = try? services.engine.yearCanChi(lunarYear: year.lunarYear) {
            parts.append(canChiName(value: canChi))
        }
        parts.append(year.age >= 1 ? "\(year.age) tuổi mụ" : "trước năm sinh")
        if year.lunarYear == currentYear {
            parts.append("năm nay")
        }
        return parts.joined(separator: " · ")
    }
}

/// What the colors of the board mean.
private struct BoardLegend: View {
    var body: some View {
        FlowLayout(spacing: 12) {
            item("Đang chọn", color: Theme.cinnabar, dashed: false)
            item("Tam hợp", color: Theme.gold, dashed: false)
            item("Xung chiếu", color: Theme.cinnabar, dashed: true)
            Text("ĐH, TH: Đại hạn, Tiểu hạn của năm xem. Góc dưới mỗi cung: năm có Tiểu hạn đến cung đó.")
                .font(.caption2).foregroundStyle(Theme.inkSecondary)
        }
    }

    private func item(_ title: String, color: Color, dashed: Bool) -> some View {
        HStack(spacing: 4) {
            RoundedRectangle(cornerRadius: 3)
                .strokeBorder(color, style: StrokeStyle(lineWidth: 1.5, dash: dashed ? [3, 2] : []))
                .frame(width: 14, height: 10)
            Text(title).font(.caption2).foregroundStyle(Theme.inkSecondary)
        }
    }
}
