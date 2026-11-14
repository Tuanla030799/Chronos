import SwiftUI

/// The palaces of the Lạc Thư board as `LoShuGrid` rows lay them out, south at the top
/// (section 4.7 of docs/FENGSHUI_SPEC.md).
enum LoShuLayout {
    static let rows: [[Palace]] = [
        [.ton, .ly, .khon],
        [.chan, .trungCung, .doai],
        [.canTho, .kham, .canKim],
    ]
}

/// The four boards of a moment.
enum NineStarLayer: String, CaseIterable, Identifiable {
    case year = "Năm"
    case month = "Tháng"
    case day = "Ngày"
    case hour = "Giờ"

    var id: Self { self }

    /// The star in the center and the direction it flies: year and month fly forward, day and
    /// hour as their own chain says.
    func center(of stars: NineStars) -> (star: NineStar, forward: Bool) {
        switch self {
        case .year: (stars.year, true)
        case .month: (stars.month, true)
        case .day: (stars.day.star, stars.day.forward)
        case .hour: (stars.hour.star, stars.hour.forward)
        }
    }
}

/// One cell of a board, as the sheet needs it.
struct NineStarCell: Hashable {
    let layer: NineStarLayer
    let palace: Palace
    let star: NineStar
    let period: NineStar
}

/// Cửu Cung Phi Tinh of a chosen moment (now by default): the period, four center stars and the
/// 3×3 board of the one selected.
struct NineStarSection: View {
    let services: AppServices
    let open: (NineStarCell) -> Void
    @State private var date = Date.now
    @State private var layer = NineStarLayer.year

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Cửu Cung Phi Tinh", systemImage: "square.grid.3x3", info: Glossary.nineStarPhases)
            DatePicker("Lúc", selection: $date, displayedComponents: [.date, .hourAndMinute])
                .environment(\.timeZone, TimeZone(secondsFromGMT: Int((services.engine.tzOffset * 3600).rounded())) ?? .gmt)
                .tint(Theme.cinnabar)
                .foregroundStyle(Theme.ink)
            if abs(date.timeIntervalSinceNow) > 60 {
                Button("Bây giờ") { date = .now }
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(Theme.cinnabar)
            }

            switch Result(catching: { try services.engine.nineStars(at: services.engine.now(date)) }) {
            case .success(let stars):
                content(stars)
            case .failure(let error):
                Text(String(describing: error)).foregroundStyle(Theme.cinnabar)
            }
        }
        .card()
    }

    @ViewBuilder
    private func content(_ stars: NineStars) -> some View {
        Divider().overlay(Theme.rule)
        HStack {
            Text("Vận \(nineStarNumber(value: stars.van)) · \(tamNguyenName(value: stars.nguyen))")
                .font(.headline).fontDesign(.serif).foregroundStyle(Theme.ink)
            Spacer()
            Text(nineStarName(value: stars.van)).font(.caption).foregroundStyle(Theme.inkSecondary)
        }

        HStack(spacing: 6) {
            ForEach(NineStarLayer.allCases) { item in
                layerTile(item, stars: stars)
            }
        }

        let center = layer.center(of: stars)
        let grid = generateLoshuGrid(centerStar: center.star, forward: center.forward)
        LoShuBoard(grid: grid, layer: layer, period: stars.van, open: open)

        Text("Nam ở trên. Chạm vào một cung để đọc luận giải của sao ở đó.")
            .font(.caption2).foregroundStyle(Theme.inkSecondary)
    }

    private func layerTile(_ item: NineStarLayer, stars: NineStars) -> some View {
        let star = item.center(of: stars).star
        let selected = item == layer
        return Button { layer = item } label: {
            VStack(spacing: 2) {
                Text(item.rawValue).font(.caption2.weight(.semibold))
                    .foregroundStyle(selected ? Theme.paper : Theme.inkSecondary)
                Text("\(nineStarNumber(value: star))")
                    .font(.title3.weight(.bold)).fontDesign(.serif)
                    .foregroundStyle(selected ? Theme.paper : Theme.element(nineStarElement(value: star)))
                Text(nineStarPhaseName(value: nineStarPhase(star: star, period: stars.van)))
                    .font(.system(size: 9, weight: .medium))
                    .foregroundStyle(selected ? Theme.paper.opacity(0.9) : Theme.inkSecondary)
                    .lineLimit(1).minimumScaleFactor(0.8)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .background(selected ? Theme.cinnabar : Theme.paper, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 10, style: .continuous).strokeBorder(Theme.rule, lineWidth: selected ? 0 : 1))
        }
        .buttonStyle(.plain)
        .accessibilityLabel("Bàn \(item.rawValue), sao \(nineStarNumber(value: star)) ở giữa")
        .accessibilityAddTraits(selected ? .isSelected : [])
    }
}

/// The 3×3 board. Each cell: the palace, the star's number and name, and its phase in the period.
struct LoShuBoard: View {
    let grid: LoShuGrid
    let layer: NineStarLayer
    let period: NineStar
    let open: (NineStarCell) -> Void

    var body: some View {
        VStack(spacing: 6) {
            ForEach(LoShuLayout.rows, id: \.self) { row in
                HStack(spacing: 6) {
                    ForEach(row, id: \.self) { palace in
                        cell(palace)
                    }
                }
            }
        }
    }

    private func cell(_ palace: Palace) -> some View {
        let star = loShuStarAt(grid: grid, palace: palace)
        let phase = nineStarPhase(star: star, period: period)
        let good = nineStarPhaseIsAuspicious(value: phase)
        let color = good ? Theme.gold : Theme.cinnabar
        let direction = palaceDirection(value: palace).map { compassDirectionName(value: $0) } ?? "Trung Cung"
        return Button {
            open(NineStarCell(layer: layer, palace: palace, star: star, period: period))
        } label: {
            VStack(spacing: 2) {
                Text(direction).font(.system(size: 10, weight: .medium)).foregroundStyle(Theme.inkSecondary)
                Text("\(nineStarNumber(value: star))")
                    .font(.system(size: 30, weight: .bold, design: .serif))
                    .foregroundStyle(Theme.element(nineStarElement(value: star)))
                Text(nineStarName(value: star))
                    .font(.system(size: 10)).foregroundStyle(Theme.ink)
                    .lineLimit(1).minimumScaleFactor(0.7)
                Text(nineStarPhaseName(value: phase))
                    .font(.system(size: 10, weight: .semibold)).foregroundStyle(color)
                    .padding(.horizontal, 6).padding(.vertical, 2)
                    .background(color.opacity(0.12), in: Capsule())
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 8)
            .background(Theme.paper, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).strokeBorder(palace == .trungCung ? Theme.cinnabar : Theme.rule, lineWidth: palace == .trungCung ? 1.5 : 1))
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(direction), sao \(nineStarName(value: star)), \(nineStarPhaseName(value: phase))")
        .accessibilityHint("Mở luận giải")
    }
}

/// The reading of one star on the board: the text of its phase in the period first.
struct NineStarSheet: View {
    let cell: NineStarCell
    let content: ContentRepository
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    header
                    if let reading = try? content.nineStar(cell.star, period: cell.period) {
                        VStack(alignment: .leading, spacing: 8) {
                            CardTitle(text: reading.phaseName, systemImage: reading.isAuspicious ? "sun.max" : "cloud")
                            if let meaning = reading.meaning {
                                Text(meaning).font(.body).foregroundStyle(Theme.ink)
                            }
                        }
                        .card()
                        EntryView(entry: reading.entry, content: content).card()
                    }
                }
                .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle("Bàn \(cell.layer.rawValue)")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Đóng") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }

    private var header: some View {
        let star = cell.star
        let direction = palaceDirection(value: cell.palace).map { "cung \(compassDirectionName(value: $0))" } ?? "Trung Cung"
        return VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text("\(nineStarNumber(value: star))")
                    .font(.system(size: 44, weight: .bold, design: .serif))
                    .foregroundStyle(Theme.element(nineStarElement(value: star)))
                VStack(alignment: .leading, spacing: 0) {
                    Text(nineStarName(value: star)).font(.title3.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                    Text("\(nineStarHanName(value: star)) · \(nineStarXuanKongName(value: star))")
                        .font(.caption).foregroundStyle(Theme.inkSecondary)
                }
            }
            FlowLayout(spacing: 8) {
                Chip(text: "Bay vào \(direction)")
                Chip(text: nineStarColorName(value: star))
                Chip(
                    text: nineStarPhaseName(value: nineStarPhase(star: star, period: cell.period)),
                    color: nineStarPhaseIsAuspicious(value: nineStarPhase(star: star, period: cell.period)) ? Theme.gold : Theme.cinnabar
                )
            }
        }
        .card()
    }
}
