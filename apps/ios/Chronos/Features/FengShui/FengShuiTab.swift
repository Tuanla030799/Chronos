import SwiftUI

/// What the tab opens in a sheet.
enum FengShuiSheet: Identifiable {
    case entry(ContentEntry)
    case eightStar(EightMansionsStar, CompassDirection?)
    case nineStar(NineStarCell)

    var id: String {
        switch self {
        case .entry(let entry): "entry/\(entry.key)"
        case .eightStar(let star, let direction): "eight/\(star)/\(direction.map { "\($0)" } ?? "-")"
        case .nineStar(let cell): "nine/\(cell.layer)/\(cell.palace)"
        }
    }
}

/// Phong Thủy: the person's Mệnh Quái and eight directions, the match with a partner, and the
/// Nine Stars of a chosen moment on the Lạc Thư board.
struct FengShuiTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @State private var sheet: FengShuiSheet?

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    if let profile = store.selected {
                        ProfileHeaderRow(profile: profile).card()
                        switch Result(catching: { try services.engine.kua(for: profile) }) {
                        case .success(let kua):
                            KuaCard(kua: kua, content: services.content) { sheet = .entry($0) }
                            DirectionsCard(kua: kua) { sheet = .eightStar($0, $1) }
                            KuaCompatibilityCard(services: services, profile: profile, kua: kua) { sheet = $0 }
                        case .failure(let error):
                            UnavailableView(title: "Không tính được Mệnh Quái", message: String(describing: error))
                                .card()
                        }
                    } else {
                        ProfileEmptyStateView(
                            engine: services.engine,
                            systemImage: "safari",
                            message: "Tìm Mệnh Quái, bốn hướng tốt, bốn hướng xấu và xem tương hợp Bát Biến. Bàn Cửu Cung bên dưới không cần hồ sơ."
                        )
                        .card(padding: 0)
                    }
                    NineStarSection(services: services) { sheet = .nineStar($0) }
                }
                .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle("Phong Thủy")
            .navigationBarTitleDisplayMode(.inline)
            .profileToolbar(services)
            .sheet(item: $sheet) { sheet in
                switch sheet {
                case .entry(let entry):
                    EntrySheet(entry: entry, content: services.content)
                case .eightStar(let star, let direction):
                    EightStarSheet(star: star, direction: direction, content: services.content)
                case .nineStar(let cell):
                    NineStarSheet(cell: cell, content: services.content)
                }
            }
        }
    }
}

// MARK: - Mệnh Quái

private struct KuaCard: View {
    let kua: KuaInfo
    let content: ContentRepository
    let open: (ContentEntry) -> Void

    var body: some View {
        let entry = try? content.kua(kua)
        VStack(alignment: .leading, spacing: 10) {
            CardTitle(text: "Mệnh Quái", systemImage: "circle.hexagongrid", info: Glossary.kuaGroups)
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(trigramName(value: kua.trigram))
                    .font(.system(size: 34, weight: .bold, design: .serif)).foregroundStyle(Theme.ink)
                Text(trigramHanName(value: kua.trigram))
                    .font(.title3).foregroundStyle(Theme.inkSecondary)
                ElementBadge(element: trigramElement(value: kua.trigram))
                Spacer()
                VStack(spacing: 0) {
                    Text("\(kua.number)").font(.title.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.cinnabar)
                    Text("quái số").font(.caption2).foregroundStyle(Theme.inkSecondary)
                }
            }
            FlowLayout(spacing: 8) {
                Chip(text: "\(trigramGroupName(value: kua.group)) Mệnh", color: Theme.gold)
                Chip(text: "Hướng \(compassDirectionName(value: trigramDirection(value: kua.trigram)))")
                Chip(text: "Sinh năm âm lịch \(String(kua.birthLunarYear))")
            }
            if let entry {
                Text(entry.summary).font(.subheadline).foregroundStyle(Theme.ink)
                Button("Đọc đầy đủ") { open(entry) }
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(Theme.cinnabar)
            }
        }
        .card()
    }
}

// MARK: - Eight directions

/// The four good and four bad directions of the person's trigram.
private struct DirectionsCard: View {
    let kua: KuaInfo
    let open: (EightMansionsStar, CompassDirection?) -> Void

    var body: some View {
        let chart = eightMansionsChart(trigram: kua.trigram)
        let good = chart.filter { eightMansionsStarLuck(value: $0.star) == .cat }.sorted(by: Self.order)
        let bad = chart.filter { eightMansionsStarLuck(value: $0.star) == .hung }.sorted(by: Self.order)
        VStack(alignment: .leading, spacing: 14) {
            CardTitle(text: "Bát Trạch · tám hướng", systemImage: "safari")
            group("Bốn hướng tốt", good, color: Theme.gold)
            group("Bốn hướng xấu", bad, color: Theme.cinnabar)
            Text("Chạm vào một hướng để đọc ý nghĩa và cách dùng cho hướng nhà, bếp, bàn làm việc, kết hôn.")
                .font(.caption2).foregroundStyle(Theme.inkSecondary)
        }
        .card()
    }

    private static func order(_ first: DirectionStar, _ second: DirectionStar) -> Bool {
        let all = EightMansionsStar.allCases
        return (all.firstIndex(of: first.star) ?? 0) < (all.firstIndex(of: second.star) ?? 0)
    }

    private func group(_ title: String, _ items: [DirectionStar], color: Color) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title).font(.subheadline.weight(.semibold)).foregroundStyle(color)
            Grid(horizontalSpacing: 8, verticalSpacing: 8) {
                ForEach(Array(stride(from: 0, to: items.count, by: 2)), id: \.self) { start in
                    GridRow {
                        ForEach(items[start..<min(start + 2, items.count)], id: \.direction) { item in
                            tile(item, color: color)
                        }
                    }
                }
            }
        }
    }

    private func tile(_ item: DirectionStar, color: Color) -> some View {
        Button { open(item.star, item.direction) } label: {
            VStack(alignment: .leading, spacing: 2) {
                Text(compassDirectionName(value: item.direction))
                    .font(.caption).foregroundStyle(Theme.inkSecondary)
                Text(eightMansionsStarName(value: item.star))
                    .font(.headline).fontDesign(.serif).foregroundStyle(color)
                Text(eightMansionsStarCorrespondingName(value: item.star))
                    .font(.caption2).foregroundStyle(Theme.inkSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(10)
            .background(color.opacity(0.1), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 12, style: .continuous).strokeBorder(color.opacity(0.35), lineWidth: 1))
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityHint("Mở ý nghĩa của sao")
    }
}

/// A star of Bát Du Niên: its general meaning, then how it reads for a house, a stove, a desk
/// and a marriage.
struct EightStarSheet: View {
    let star: EightMansionsStar
    let direction: CompassDirection?
    let content: ContentRepository
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(alignment: .firstTextBaseline, spacing: 8) {
                            Text(eightMansionsStarName(value: star))
                                .font(.system(size: 30, weight: .bold, design: .serif)).foregroundStyle(Theme.ink)
                            Text(eightMansionsStarHanName(value: star)).font(.title3).foregroundStyle(Theme.inkSecondary)
                        }
                        FlowLayout(spacing: 8) {
                            if let direction {
                                Chip(text: "Hướng \(compassDirectionName(value: direction))")
                            }
                            Chip(
                                text: eightMansionsStarLuck(value: star) == .cat ? "Cát" : "Hung",
                                color: eightMansionsStarLuck(value: star) == .cat ? Theme.gold : Theme.cinnabar
                            )
                            Chip(text: eightMansionsStarCorrespondingName(value: star))
                        }
                    }
                    .card()

                    if let entry = try? content.eightStar(star) {
                        EntryView(entry: entry, content: content).card()
                    }

                    VStack(alignment: .leading, spacing: 12) {
                        CardTitle(text: "Ứng dụng", systemImage: "house")
                        ForEach(EightUse.allCases, id: \.self) { usage in
                            if let entry = try? content.eightStarUse(star, usage) {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(eightUseName(usage: usage).uppercased())
                                        .font(.caption2.weight(.semibold)).tracking(0.6)
                                        .foregroundStyle(Theme.inkSecondary)
                                    EntryView(entry: entry, content: content)
                                }
                                if usage != EightUse.allCases.last {
                                    Divider().overlay(Theme.rule)
                                }
                            }
                        }
                    }
                    .card()
                }
                .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle("Sao \(eightMansionsStarName(value: star))")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Đóng") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }
}
