import SwiftUI

/// Tử Vi: the twelve palaces on the Địa Bàn around the Thiên Bàn; a palace opens its reading.
struct TuViTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @State private var opened: OpenedPalace?

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    switch Result(catching: { try services.engine.tuviChart(for: profile) }) {
                    case .success(let chart):
                        ScrollView {
                            TuViBoard(chart: chart, name: profile.name) { palace in
                                opened = OpenedPalace(chart: chart, palace: palace)
                            }
                            .padding(8)
                        }
                        .background(Theme.paper)
                    case .failure(let error):
                        UnavailableView(title: "Không lập được lá số", message: String(describing: error))
                    }
                } else {
                    NoProfileView()
                }
            }
            .navigationTitle("Tử Vi")
            .navigationBarTitleDisplayMode(.inline)
            .profileToolbar(services)
            .sheet(item: $opened) { opened in
                PalaceSheet(opened: opened, content: services.content)
                    .presentationDetents([.medium, .large])
            }
        }
    }
}

struct OpenedPalace: Identifiable {
    let chart: TuViChart
    let palace: TuViPalace

    var id: String { "\(palace.branch)" }
}

/// The fixed Địa Bàn: Tỵ to Thân across the top, Dần to Hợi across the bottom.
private struct TuViBoard: View {
    let chart: TuViChart
    let name: String
    let open: (TuViPalace) -> Void

    var body: some View {
        VStack(spacing: 2) {
            HStack(spacing: 2) { cells([.ti, .ngo, .mui, .than]) }
            HStack(spacing: 2) {
                VStack(spacing: 2) { cells([.thin, .mao]) }
                center
                VStack(spacing: 2) { cells([.dau, .tuat]) }
            }
            HStack(spacing: 2) { cells([.dan, .suu, .ty, .hoi]) }
        }
        .background(Theme.rule)
    }

    private func cells(_ branches: [Branch]) -> some View {
        ForEach(branches, id: \.self) { branch in
            if let palace = chart.palaces.first(where: { $0.branch == branch }) {
                PalaceCell(palace: palace)
                    .onTapGesture { open(palace) }
            }
        }
    }

    private var center: some View {
        VStack(spacing: 4) {
            Text(name).font(.headline).foregroundStyle(Theme.ink)
            Text(tuviPolarityName(value: chart.polarity))
            Text("Bản mệnh: " + naYinName(value: chart.banMenh))
            Text(fiveElementsBureauName(value: chart.bureau))
            Text("Mệnh chủ \(tuviStarName(value: chart.menhChu)), Thân chủ \(tuviStarName(value: chart.thanChu))")
        }
        .font(.caption)
        .foregroundStyle(Theme.inkSecondary)
        .multilineTextAlignment(.center)
        .padding(6)
        .frame(maxWidth: .infinity, minHeight: 236)
        .background(Theme.paper)
    }
}

private struct PalaceCell: View {
    let palace: TuViPalace

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            HStack {
                Text(tuviPalaceRoleName(value: palace.role))
                    .font(.caption.weight(.bold))
                    .foregroundStyle(Theme.cinnabar)
                if palace.isThan {
                    Text("Thân").font(.caption2).foregroundStyle(Theme.gold)
                }
            }
            ForEach(palace.mainStars, id: \.star) { placed in
                Text(starLabel(placed))
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(Theme.ink)
            }
            if !palace.minorStars.isEmpty {
                Text("+\(palace.minorStars.count) sao")
                    .font(.caption2)
                    .foregroundStyle(Theme.inkSecondary)
            }
            Spacer(minLength: 0)
            Text("\(stemName(stem: palace.stem)) \(branchName(branch: palace.branch)) · \(palace.daiHanStartAge)")
                .font(.caption2)
                .foregroundStyle(Theme.inkSecondary)
        }
        .padding(4)
        .frame(maxWidth: .infinity, minHeight: 117, alignment: .topLeading)
        .background(Theme.surface)
        .contentShape(Rectangle())
    }
}

func starLabel(_ placed: TuViPlacedStar) -> String {
    var label = tuviStarName(value: placed.star)
    if let brightness = placed.brightness {
        label += " " + starBrightnessMark(value: brightness)
    }
    if let transformation = placed.transformation {
        label += " · " + fourTransformationName(value: transformation)
    }
    return label
}

/// The reading of one palace: entries of its keys, the firm passages, then "Xem thêm".
private struct PalaceSheet: View {
    let opened: OpenedPalace
    let content: ContentRepository

    var body: some View {
        NavigationStack {
            switch Result(catching: { try content.tuviPalace(chart: opened.chart, branch: opened.palace.branch) }) {
            case .success(let reading):
                List {
                    Section("Sao") {
                        ForEach(opened.palace.mainStars + opened.palace.minorStars, id: \.star) { placed in
                            Text(starLabel(placed)).foregroundStyle(Theme.ink)
                        }
                    }
                    ForEach(reading.primary) { group in
                        Section(group.section?.title ?? "Luận giải") {
                            ForEach(group.passages) { PassageView(passage: $0) }
                        }
                    }
                    if !reading.more.isEmpty {
                        Section {
                            DisclosureGroup("Xem thêm (\(reading.more.reduce(0) { $0 + $1.passages.count }) đoạn có điều kiện)") {
                                ForEach(reading.more) { group in
                                    if let section = group.section {
                                        Text(section.title).font(.caption.weight(.semibold)).foregroundStyle(Theme.gold)
                                    }
                                    ForEach(group.passages) { PassageView(passage: $0) }
                                }
                            }
                        }
                    }
                    Section("Tra cứu") {
                        ForEach(reading.entries) { EntryView(entry: $0, content: content) }
                    }
                }
                .chronosBackground()
                .navigationTitle("Cung " + tuviPalaceRoleName(value: opened.palace.role))
                .navigationBarTitleDisplayMode(.inline)
            case .failure(let error):
                UnavailableView(title: "Không đọc được luận giải", message: String(describing: error))
            }
        }
    }
}
