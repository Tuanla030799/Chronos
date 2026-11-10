import SwiftUI

/// The chart: Giờ, Ngày, Tháng, Năm from left to right, as a printed Bát Tự reads. A Thập Thần,
/// Trường Sinh, Nạp Âm or Thần Sát opens its text.
struct PillarTableCard: View {
    let reading: BaziReading
    let profile: Profile
    let open: (ContentEntry) -> Void

    private struct Column: Identifiable {
        let title: String
        let position: PillarPosition
        let pillar: PillarReading

        var id: String { title }
    }

    private var columns: [Column] {
        let analysis = reading.analysis
        return [
            Column(title: "Giờ", position: .hour, pillar: analysis.hour),
            Column(title: "Ngày", position: .day, pillar: analysis.day),
            Column(title: "Tháng", position: .month, pillar: analysis.month),
            Column(title: "Năm", position: .year, pillar: analysis.year),
        ]
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Tứ Trụ", systemImage: "square.grid.2x2")

            row(nil) { column in
                Text(column.title)
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(column.position == .day ? Theme.cinnabar : Theme.ink)
                    .frame(maxWidth: .infinity)
                    .padding(.bottom, 4)
                    .overlay(alignment: .bottom) {
                        Rectangle()
                            .fill(column.position == .day ? Theme.cinnabar : Theme.rule)
                            .frame(height: column.position == .day ? 2 : 1)
                    }
            }

            row("Thập Thần") { column in
                if let god = column.pillar.tenGod {
                    tappable(reading.entry(baziTenGodKey(god: god))) {
                        Text(tenGodName(value: god)).font(.caption.weight(.medium)).foregroundStyle(Theme.ink)
                    }
                } else {
                    let stem = reading.analysis.pillars.day.stem
                    tappable(reading.entry(baziDayMasterKey(stem: stem))) {
                        Text("Nhật Chủ").font(.caption.weight(.bold)).foregroundStyle(Theme.cinnabar)
                    }
                }
            }

            row("Thiên Can") { column in
                let stem = column.pillar.canChi.stem
                VStack(spacing: 4) {
                    Text(stemName(stem: stem))
                        .font(.system(size: 28, weight: .bold, design: .serif))
                        .foregroundStyle(Theme.element(stemElement(stem: stem)))
                    ElementBadge(element: stemElement(stem: stem))
                }
                .frame(maxWidth: .infinity)
                .accessibilityElement(children: .combine)
            }

            row("Địa Chi") { column in
                let branch = column.pillar.canChi.branch
                VStack(spacing: 4) {
                    Text(branchName(branch: branch))
                        .font(.system(size: 28, weight: .bold, design: .serif))
                        .foregroundStyle(Theme.element(branchElement(branch: branch)))
                    ElementBadge(element: branchElement(branch: branch))
                    ForEach(voidSources(of: column), id: \.self) { source in
                        Text("Không · \(source)")
                            .font(.system(size: 10, weight: .semibold))
                            .foregroundStyle(Theme.cinnabar)
                    }
                }
                .frame(maxWidth: .infinity)
                .accessibilityElement(children: .combine)
            }

            row("Tàng Can") { column in
                VStack(spacing: 6) {
                    ForEach(Array(column.pillar.hiddenStems.enumerated()), id: \.offset) { index, hidden in
                        tappable(reading.entry(baziTenGodKey(god: hidden.tenGod)), fill: false) {
                            VStack(spacing: 0) {
                                Text(stemName(stem: hidden.stem))
                                    .font(.footnote.weight(.semibold))
                                    .foregroundStyle(Theme.element(stemElement(stem: hidden.stem)))
                                Text(tenGodName(value: hidden.tenGod))
                                    .font(.caption2).foregroundStyle(Theme.ink)
                                Text(Self.qiName(index))
                                    .font(.system(size: 9)).foregroundStyle(Theme.inkSecondary)
                            }
                        }
                    }
                }
            }

            row("Trường Sinh") { column in
                tappable(reading.entry(baziLifeStageKey(stage: column.pillar.lifeStage))) {
                    Text(lifeStageName(value: column.pillar.lifeStage))
                        .font(.caption.weight(.medium)).foregroundStyle(Theme.ink)
                }
            }

            row("Nạp Âm") { column in
                tappable(reading.entry(baziNaYinKey(naYin: column.pillar.naYin))) {
                    Text(naYinName(value: column.pillar.naYin))
                        .font(.caption.weight(.medium))
                        .foregroundStyle(Theme.element(naYinElement(value: column.pillar.naYin)))
                        .multilineTextAlignment(.center)
                }
            }

            row("Thần Sát") { column in
                let stars = stars(of: column.position)
                if stars.isEmpty {
                    Text("—").font(.caption).foregroundStyle(Theme.inkSecondary)
                } else {
                    VStack(spacing: 4) {
                        ForEach(stars, id: \.self) { star in
                            tappable(reading.entry(baziShenShaKey(star: star)), fill: false) {
                                Text(baziStarName(value: star))
                                    .font(.caption2.weight(.medium)).foregroundStyle(Theme.ink)
                                    .multilineTextAlignment(.center)
                            }
                        }
                    }
                }
            }

            Divider().overlay(Theme.rule)
            voidSection
            footnote
        }
        .card()
    }

    // MARK: Rows

    /// A caption over four equal cells.
    private func row<Cell: View>(_ title: String?, @ViewBuilder cell: @escaping (Column) -> Cell) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            if let title {
                Text(title.uppercased())
                    .font(.system(size: 10, weight: .semibold)).tracking(0.6)
                    .foregroundStyle(Theme.inkSecondary)
            }
            HStack(alignment: .top, spacing: 6) {
                ForEach(columns) { column in
                    cell(column).frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
                }
            }
            .fixedSize(horizontal: false, vertical: true)
        }
    }

    /// A cell that opens `entry`. Without an entry it is plain text, a gap of the data. A cell
    /// alone in its column fills the row so the cells of a row are as tall as each other.
    @ViewBuilder
    private func tappable<Label: View>(
        _ entry: ContentEntry?,
        fill: Bool = true,
        @ViewBuilder label: () -> Label
    ) -> some View {
        let label = label()
            .frame(maxWidth: .infinity, maxHeight: fill ? .infinity : nil)
            .padding(.vertical, 5)
            .padding(.horizontal, 2)
        if let entry {
            Button { open(entry) } label: {
                label.background(Theme.paper, in: RoundedRectangle(cornerRadius: 8, style: .continuous))
            }
            .buttonStyle(.plain)
            .accessibilityHint("Mở giải nghĩa")
        } else {
            label
        }
    }

    // MARK: Không Vong

    @ViewBuilder
    private var voidSection: some View {
        let analysis = reading.analysis
        VStack(alignment: .leading, spacing: 8) {
            voidLine("Không Vong theo Ngày", analysis.khongVongDay)
            voidLine("Không Vong theo Năm", analysis.khongVongYear)
            if analysis.isKhoiCanh {
                Button {
                    if let entry = reading.entry(baziKhoiCanhKey()) { open(entry) }
                } label: {
                    Chip(text: "Khôi Canh · trụ Ngày \(canChiName(value: analysis.pillars.day))", color: Theme.gold)
                }
                .buttonStyle(.plain)
            }
        }
    }

    private func voidLine(_ title: String, _ void: KhongVong) -> some View {
        Button {
            if let entry = reading.entry(baziKhongVongKey()) { open(entry) }
        } label: {
            HStack {
                Text(title).font(.subheadline).foregroundStyle(Theme.inkSecondary)
                Spacer()
                Text("\(branchName(branch: void.first)), \(branchName(branch: void.second))")
                    .font(.subheadline.weight(.semibold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                Image(systemName: "chevron.right").font(.caption).foregroundStyle(Theme.inkSecondary)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    private var footnote: some View {
        let time = reading.analysis.pillars.effectiveTime
        let clock = String(
            format: "%02d/%02d/%04d %02d:%02d",
            Int(time.day), Int(time.month), Int(time.year), Int(time.hour), Int(time.minute)
        )
        return VStack(alignment: .leading, spacing: 2) {
            Text("Lập trụ theo \(profile.longitude == nil ? "giờ đồng hồ" : "giờ mặt trời thực") \(clock).")
            Text("Chạm vào Thập Thần, Tàng Can, Trường Sinh, Nạp Âm hay Thần Sát để đọc giải nghĩa.")
        }
        .font(.caption2)
        .foregroundStyle(Theme.inkSecondary)
    }

    // MARK: Data

    /// Bản khí, Trung khí, Dư khí by position, as the source orders its hidden stems.
    nonisolated static func qiName(_ index: Int) -> String {
        ["Bản khí", "Trung khí", "Dư khí"][min(index, 2)]
    }

    /// Thần Sát of a pillar, each once even when two bases give it.
    private func stars(of position: PillarPosition) -> [BaziStar] {
        var seen: [BaziStar] = []
        for placement in reading.analysis.stars where placement.pillar == position && !seen.contains(placement.star) {
            seen.append(placement.star)
        }
        return seen
    }

    /// Which Không Vong, of the Day or of the Year, names the branch of this pillar. The pillar
    /// a Không Vong was read from is not tested against its own.
    private func voidSources(of column: Column) -> [String] {
        let analysis = reading.analysis
        let branch = column.pillar.canChi.branch
        var sources: [String] = []
        if column.position != .day, [analysis.khongVongDay.first, analysis.khongVongDay.second].contains(branch) {
            sources.append("Ngày")
        }
        if column.position != .year, [analysis.khongVongYear.first, analysis.khongVongYear.second].contains(branch) {
            sources.append("Năm")
        }
        return sources
    }
}
