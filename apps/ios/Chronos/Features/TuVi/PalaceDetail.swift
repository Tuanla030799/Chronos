import SwiftUI

/// The palace chosen on the board: its place in the year, its stars by kind, the overview of
/// the palace, the passages of *Tân Biên* that hold on this chart, then the conditional ones
/// folded under "Xem thêm" (D12).
struct PalaceDetail: View {
    let chart: TuViChart
    let palace: TuViPalace
    let year: TuViYearReading
    let content: ContentRepository
    let open: (TuViSheet) -> Void

    var body: some View {
        let groups = StarGroups(palace)
        VStack(spacing: 16) {
            header
            starsCard(groups)
            switch Result(catching: { try content.tuviPalace(chart: chart, branch: palace.branch) }) {
            case .success(let reading):
                overview(reading)
                ForEach(reading.primary) { group in
                    passageCard(group.section?.title ?? "Luận giải", group.passages)
                }
                if !reading.more.isEmpty {
                    MoreCard(groups: reading.more)
                }
                if reading.primary.isEmpty && reading.more.isEmpty {
                    Text("Sách không có đoạn luận riêng cho cung này với các sao hiện có.")
                        .font(.subheadline).foregroundStyle(Theme.inkSecondary)
                        .card()
                }
            case .failure(let error):
                UnavailableView(title: "Không đọc được luận giải", message: String(describing: error)).card()
            }
        }
    }

    // MARK: Header

    private var header: some View {
        let relations = PalaceRelations(of: palace.branch)
        return VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text("Cung \(tuviPalaceRoleName(value: palace.role))")
                    .font(.title2.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                Spacer()
                Text("\(stemName(stem: palace.stem)) \(branchName(branch: palace.branch))")
                    .font(.headline).fontDesign(.serif).foregroundStyle(Theme.inkSecondary)
            }
            FlowLayout(spacing: 6) {
                Chip(text: "Đại hạn \(palace.daiHanStartAge)–\(Int(palace.daiHanStartAge) + 9) tuổi")
                Chip(text: lifeStageName(value: palace.lifeStage))
                if palace.isThan { Chip(text: "Thân cư", color: Theme.gold) }
                if palace.hasTuan { Chip(text: "Tuần", color: Theme.cinnabar) }
                if palace.hasTriet { Chip(text: "Triệt", color: Theme.cinnabar) }
                if year.daiHan == palace.branch { Chip(text: "Đại hạn năm \(String(year.lunarYear))", color: Theme.gold) }
                if year.tieuHan == palace.branch { Chip(text: "Tiểu hạn năm \(String(year.lunarYear))", color: Theme.gold) }
            }
            Text("Tam hợp \(relations.trine.map { name(of: $0) }.joined(separator: ", ")) · Xung chiếu \(name(of: relations.opposite))")
                .font(.caption).foregroundStyle(Theme.inkSecondary)
        }
        .card()
    }

    private func name(of branch: Branch) -> String {
        let role = chart.palace(at: branch).map { tuviPalaceRoleName(value: $0.role) } ?? ""
        return "\(role) (\(branchName(branch: branch)))"
    }

    // MARK: Stars

    private func starsCard(_ groups: StarGroups) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Sao trong cung", systemImage: "sparkles")
            starGroup("Chính tinh", groups.main, empty: "Vô Chính Diệu")
            starGroup("Phụ tinh cát", groups.good, empty: nil)
            starGroup("Phụ tinh hung, sát", groups.bad, empty: nil)
            starGroup("Phụ tinh trung tính", groups.neutral, empty: nil)
            Text("Chạm vào một sao để đọc tính chất, độ sáng và Tứ Hóa của nó.")
                .font(.caption2).foregroundStyle(Theme.inkSecondary)
        }
        .card()
    }

    @ViewBuilder
    private func starGroup(_ title: String, _ stars: [TuViPlacedStar], empty: String?) -> some View {
        if !stars.isEmpty || empty != nil {
            VStack(alignment: .leading, spacing: 6) {
                Text("\(title.uppercased()) (\(stars.count))")
                    .font(.caption2.weight(.semibold)).tracking(0.6)
                    .foregroundStyle(Theme.inkSecondary)
                if stars.isEmpty, let empty {
                    Text(empty).font(.subheadline).italic().foregroundStyle(Theme.inkSecondary)
                } else {
                    FlowLayout(spacing: 6) {
                        ForEach(stars, id: \.star) { placed in
                            StarChip(placed: placed) { open(.star(placed, palace.branch)) }
                        }
                    }
                }
            }
        }
    }

    // MARK: Readings

    @ViewBuilder
    private func overview(_ reading: PalaceReading) -> some View {
        if let entry = reading.entries.first(where: { $0.key == tuviPalaceKey(role: palace.role) }) {
            VStack(alignment: .leading, spacing: 8) {
                CardTitle(text: "Nhận định chung", systemImage: "text.book.closed")
                Text(entry.summary).font(.subheadline).foregroundStyle(Theme.ink)
                Button("Đọc đầy đủ") { open(.entry(entry)) }
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(Theme.cinnabar)
            }
            .card()
        }
    }

    private func passageCard(_ title: String, _ passages: [Passage]) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: title, systemImage: "book")
            ForEach(passages) { passage in
                PassageView(passage: passage)
                if passage.id != passages.last?.id {
                    Divider().overlay(Theme.rule)
                }
            }
        }
        .card()
    }
}

/// The conditional passages, folded until opened, with how many there are.
private struct MoreCard: View {
    let groups: [PassageGroup]
    @State private var expanded = false

    var body: some View {
        let count = groups.reduce(0) { $0 + $1.passages.count }
        VStack(alignment: .leading, spacing: 12) {
            Button {
                withAnimation(.snappy) { expanded.toggle() }
            } label: {
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Xem thêm các trường hợp phối hợp sao")
                            .font(.subheadline.weight(.semibold)).foregroundStyle(Theme.ink)
                        Text("\(count) đoạn có điều kiện, bạn tự đối chiếu với lá số")
                            .font(.caption).foregroundStyle(Theme.inkSecondary)
                    }
                    Spacer()
                    Image(systemName: expanded ? "chevron.up" : "chevron.down")
                        .foregroundStyle(Theme.cinnabar)
                }
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .accessibilityHint(expanded ? "Thu gọn" : "Mở rộng")

            if expanded {
                ForEach(groups) { group in
                    if let section = group.section {
                        Text(section.title.uppercased())
                            .font(.caption2.weight(.semibold)).tracking(0.6)
                            .foregroundStyle(Theme.gold)
                    }
                    ForEach(group.passages) { passage in
                        PassageView(passage: passage)
                        Divider().overlay(Theme.rule)
                    }
                }
            }
        }
        .card()
    }
}

/// A star as a tappable chip: name, brightness mark and Tứ Hóa, tinted by its nature.
private struct StarChip: View {
    let placed: TuViPlacedStar
    let open: () -> Void

    var body: some View {
        let nature = tuviStarNature(value: placed.star)
        let element = tuviStarElement(value: placed.star)
        let color = element.map(Theme.element) ?? Theme.luck(ContentLuck(nature))
        Button(action: open) {
            HStack(spacing: 4) {
                Text(placed.label).font(.subheadline.weight(element == nil ? .medium : .bold))
                if let hoa = placed.transformation {
                    Text(fourTransformationName(value: hoa))
                        .font(.caption2.weight(.bold))
                        .foregroundStyle(Theme.luck(ContentLuck(fourTransformationNature(value: hoa))))
                }
            }
            .foregroundStyle(color)
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(color.opacity(0.1), in: Capsule())
            .overlay(Capsule().strokeBorder(color.opacity(0.3), lineWidth: 1))
        }
        .buttonStyle(.plain)
        .accessibilityHint("Mở tính chất của sao")
    }
}

/// One star: its kind, nature, element and brightness here, then the texts of the star, of its
/// brightness and of its Hóa.
struct TuViStarSheet: View {
    let placed: TuViPlacedStar
    let branch: Branch
    let content: ContentRepository
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        let star = placed.star
        let entry = try? content.entry(tuviStarKey(star: star))
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(placed.label)
                            .font(.system(size: 28, weight: .bold, design: .serif))
                            .foregroundStyle(tuviStarElement(value: star).map(Theme.element) ?? Theme.ink)
                        FlowLayout(spacing: 6) {
                            Chip(text: tuviStarCategory(value: star) == .chinhTinh ? "Chính tinh" : "Phụ tinh")
                            Chip(
                                text: starNatureName(value: tuviStarNature(value: star)),
                                color: Theme.luck(ContentLuck(tuviStarNature(value: star)))
                            )
                            if let element = tuviStarElement(value: star) {
                                ElementBadge(element: element)
                            }
                            if let brightness = placed.brightness {
                                Chip(text: "\(starBrightnessName(value: brightness)) tại \(branchName(branch: branch))")
                            }
                            if let hoa = placed.transformation {
                                Chip(text: fourTransformationName(value: hoa), color: Theme.gold)
                            }
                        }
                        if let profile = entry?.extra.profile {
                            if let element = profile.element {
                                Text("Ngũ hành: \(element)").font(.caption).foregroundStyle(Theme.inkSecondary)
                            }
                            if !profile.labels.isEmpty {
                                Text(profile.labels.joined(separator: " · ")).font(.caption).foregroundStyle(Theme.inkSecondary)
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .card()

                    if let entry {
                        EntryView(entry: entry, content: content).card()
                    }
                    if let brightness = placed.brightness,
                       let entry = try? content.entry(tuviStarBrightnessKey(star: star, level: brightness)) {
                        EntryView(entry: entry, content: content).card()
                    }
                    if let hoa = placed.transformation,
                       let entry = try? content.entry(tuviTransformationKey(transformation: hoa)) {
                        EntryView(entry: entry, content: content).card()
                    }
                }
                .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle(tuviStarName(value: star))
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
