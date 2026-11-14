import SwiftUI

/// The twelve palaces on a 4×4 board with the Thiên Bàn in the 2×2 middle. The chosen palace is
/// outlined in Chu Sa, its Tam Hợp in Kim and its Xung Chiếu dashed, joined by thin lines.
struct TuViBoard: View {
    let chart: TuViChart
    let name: String
    let year: TuViYearReading
    let selected: Branch
    let select: (Branch) -> Void

    nonisolated static let spacing: CGFloat = 3
    nonisolated static let cellHeight: CGFloat = 136

    var body: some View {
        let relations = PalaceRelations(of: selected)
        GeometryReader { geometry in
            let width = (geometry.size.width - 3 * Self.spacing) / 4
            ZStack(alignment: .topLeading) {
                lines(relations, cellWidth: width)
                ThienBan(chart: chart, name: name)
                    .frame(width: 2 * width + Self.spacing, height: 2 * Self.cellHeight + Self.spacing)
                    .offset(x: width + Self.spacing, y: Self.cellHeight + Self.spacing)
                ForEach(TuViBoardLayout.cells, id: \.self) { cell in
                    if let palace = chart.palace(at: cell.branch) {
                        PalaceCell(
                            palace: palace,
                            isSelected: cell.branch == selected,
                            relation: relations.role(of: cell.branch),
                            isDaiHan: year.daiHan == cell.branch,
                            isTieuHan: year.tieuHan == cell.branch
                        )
                        .frame(width: width, height: Self.cellHeight)
                        .offset(x: CGFloat(cell.column) * (width + Self.spacing), y: CGFloat(cell.row) * (Self.cellHeight + Self.spacing))
                        .onTapGesture { select(cell.branch) }
                    }
                }
            }
        }
        .frame(height: 4 * Self.cellHeight + 3 * Self.spacing)
    }

    /// The Tam Hợp triangle and the line to the Xung Chiếu, behind the cells: they show across
    /// the Thiên Bàn and in the gaps.
    private func lines(_ relations: PalaceRelations, cellWidth: CGFloat) -> some View {
        Canvas { context, _ in
            func center(_ branch: Branch) -> CGPoint? {
                guard let cell = TuViBoardLayout.cell(branch) else { return nil }
                return CGPoint(
                    x: CGFloat(cell.column) * (cellWidth + Self.spacing) + cellWidth / 2,
                    y: CGFloat(cell.row) * (Self.cellHeight + Self.spacing) + Self.cellHeight / 2
                )
            }
            guard let origin = center(selected) else { return }
            let corners = relations.trine.compactMap(center)
            var triangle = Path()
            triangle.move(to: origin)
            for corner in corners {
                triangle.addLine(to: corner)
            }
            triangle.closeSubpath()
            context.stroke(triangle, with: .color(Theme.gold.opacity(0.7)), lineWidth: 1.5)
            if let opposite = center(relations.opposite) {
                var line = Path()
                line.move(to: origin)
                line.addLine(to: opposite)
                context.stroke(line, with: .color(Theme.cinnabar.opacity(0.7)), style: StrokeStyle(lineWidth: 1.5, dash: [5, 4]))
            }
        }
        .allowsHitTesting(false)
        .accessibilityHidden(true)
    }
}

/// One palace: role, Can Chi and Đại Hạn on top; Chính Tinh and Tứ Hóa in the middle; counts,
/// Tuần, Triệt, the year's hạn, Trường Sinh and the Tiểu Hạn branch at the foot.
private struct PalaceCell: View {
    let palace: TuViPalace
    let isSelected: Bool
    let relation: PalaceRelations.Role?
    let isDaiHan: Bool
    let isTieuHan: Bool

    var body: some View {
        let groups = StarGroups(palace)
        VStack(alignment: .leading, spacing: 1) {
            HStack(alignment: .firstTextBaseline, spacing: 2) {
                Text(tuviPalaceRoleName(value: palace.role).uppercased())
                    .font(.system(size: 9, weight: .heavy))
                    .foregroundStyle(palace.role == .menh ? Theme.cinnabar : Theme.ink)
                    .lineLimit(1).minimumScaleFactor(0.7)
                Spacer(minLength: 0)
                Text("\(palace.daiHanStartAge)")
                    .font(.system(size: 9, weight: .bold)).foregroundStyle(Theme.inkSecondary)
            }
            HStack(spacing: 3) {
                Text("\(stemName(stem: palace.stem)) \(branchName(branch: palace.branch))")
                    .font(.system(size: 8.5)).foregroundStyle(Theme.inkSecondary)
                if palace.isThan {
                    Text("<THÂN>").font(.system(size: 8, weight: .heavy)).foregroundStyle(Theme.gold)
                }
            }
            .lineLimit(1)

            if groups.main.isEmpty {
                Text("Vô Chính Diệu").font(.system(size: 9.5)).italic().foregroundStyle(Theme.inkSecondary)
                    .padding(.top, 2)
            }
            ForEach(groups.main, id: \.star) { placed in
                Text(placed.label)
                    .font(.system(size: 10.5, weight: .bold))
                    .foregroundStyle(tuviStarElement(value: placed.star).map(Theme.element) ?? Theme.ink)
                    .lineLimit(1).minimumScaleFactor(0.7)
            }
            ForEach(groups.transformed, id: \.star) { placed in
                if let hoa = placed.transformation {
                    Text(fourTransformationName(value: hoa))
                        .font(.system(size: 8.5, weight: .semibold))
                        .foregroundStyle(Theme.luck(ContentLuck(fourTransformationNature(value: hoa))))
                }
            }

            Spacer(minLength: 0)

            Text("Cát \(groups.good.count) · Hung \(groups.bad.count)")
                .font(.system(size: 8)).foregroundStyle(Theme.inkSecondary)
            tags
            HStack(spacing: 2) {
                Text(lifeStageName(value: palace.lifeStage))
                Spacer(minLength: 0)
                Text("năm \(branchName(branch: palace.tieuHanYearBranch))")
            }
            .font(.system(size: 8)).foregroundStyle(Theme.inkSecondary)
            .lineLimit(1)
        }
        .padding(4)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background {
            RoundedRectangle(cornerRadius: 6, style: .continuous)
                .fill(Theme.surface)
                .overlay(RoundedRectangle(cornerRadius: 6, style: .continuous).fill(tint))
        }
        .overlay { border }
        .contentShape(Rectangle())
        .accessibilityElement(children: .ignore)
        .accessibilityLabel(accessibilityText(groups))
        .accessibilityAddTraits(isSelected ? [.isButton, .isSelected] : .isButton)
    }

    @ViewBuilder
    private var tags: some View {
        let flags: [(text: String, color: Color, on: Bool)] = [
            ("TUẦN", Theme.cinnabar, palace.hasTuan),
            ("TRIỆT", Theme.cinnabar, palace.hasTriet),
            ("ĐH", Theme.gold, isDaiHan),
            ("TH", Theme.gold, isTieuHan),
        ]
        let items = flags.filter(\.on).map { ($0.text, $0.color) }
        if !items.isEmpty {
            HStack(spacing: 2) {
                ForEach(items, id: \.0) { item in
                    Text(item.0)
                        .font(.system(size: 7.5, weight: .bold))
                        .foregroundStyle(item.1)
                        .padding(.horizontal, 3)
                        .padding(.vertical, 1)
                        .background(item.1.opacity(0.14), in: Capsule())
                }
            }
        }
    }

    /// Drawn over an opaque surface, so the lines behind the cells never show through.
    private var tint: Color {
        if isSelected { return Theme.cinnabar.opacity(0.08) }
        if relation == .trine { return Theme.gold.opacity(0.08) }
        return .clear
    }

    @ViewBuilder
    private var border: some View {
        let shape = RoundedRectangle(cornerRadius: 6, style: .continuous)
        if isSelected {
            shape.strokeBorder(Theme.cinnabar, lineWidth: 2)
        } else if relation == .trine {
            shape.strokeBorder(Theme.gold, lineWidth: 1.5)
        } else if relation == .opposite {
            shape.strokeBorder(Theme.cinnabar, style: StrokeStyle(lineWidth: 1.5, dash: [4, 3]))
        } else {
            shape.strokeBorder(Theme.rule, lineWidth: 1)
        }
    }

    private func accessibilityText(_ groups: StarGroups) -> String {
        var parts = ["Cung \(tuviPalaceRoleName(value: palace.role))", "\(stemName(stem: palace.stem)) \(branchName(branch: palace.branch))"]
        if palace.isThan { parts.append("Thân cư") }
        parts.append(groups.main.isEmpty ? "Vô Chính Diệu" : groups.main.map(\.label).joined(separator: ", "))
        if palace.hasTuan { parts.append("Tuần") }
        if palace.hasTriet { parts.append("Triệt") }
        if isDaiHan { parts.append("Đại hạn năm xem") }
        if isTieuHan { parts.append("Tiểu hạn năm xem") }
        if isSelected { parts.append("đang chọn") }
        if relation == .trine { parts.append("tam hợp") }
        if relation == .opposite { parts.append("xung chiếu") }
        return parts.joined(separator: ", ")
    }
}

/// The Thiên Bàn: who, when, Bản Mệnh, Cục, their relation, Âm Dương and the two ruling stars.
private struct ThienBan: View {
    let chart: TuViChart
    let name: String

    var body: some View {
        let lunar = chart.lunar
        VStack(spacing: 3) {
            Text(name).font(.subheadline.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                .lineLimit(1).minimumScaleFactor(0.7)
            Text(tuviPolarityName(value: chart.polarity))
                .font(.caption2.weight(.semibold)).foregroundStyle(Theme.cinnabar)
            Text(String(format: "Dương lịch %02d/%02d/%04d", Int(chart.solarDate.day), Int(chart.solarDate.month), Int(chart.solarDate.year)))
            Text("Âm lịch \(lunar.day)/\(lunar.month)\(lunar.isLeapMonth ? " nhuận" : "")")
            Divider().overlay(Theme.rule).padding(.horizontal, 12)
            Text("Năm \(canChiName(value: chart.yearCanChi)) · Tháng \(canChiName(value: chart.monthCanChi))")
            Text("Ngày \(canChiName(value: chart.dayCanChi)) · Giờ \(canChiName(value: chart.hourCanChi))")
            Divider().overlay(Theme.rule).padding(.horizontal, 12)
            Text("Bản mệnh \(naYinName(value: chart.banMenh))")
                .foregroundStyle(Theme.element(naYinElement(value: chart.banMenh)))
            Text(fiveElementsBureauName(value: chart.bureau))
                .fontWeight(.semibold)
                .foregroundStyle(Theme.element(fiveElementsBureauElement(value: chart.bureau)))
            Text("\(menhCucRelationName(value: chart.menhCuc)) · \(yinYangHarmonyName(value: chart.harmony))")
            Text("Mệnh chủ \(tuviStarName(value: chart.menhChu))")
            Text("Thân chủ \(tuviStarName(value: chart.thanChu))")
        }
        .font(.system(size: 9.5))
        .foregroundStyle(Theme.ink)
        .multilineTextAlignment(.center)
        .lineLimit(1)
        .minimumScaleFactor(0.75)
        .padding(6)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.paper.opacity(0.72), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 8, style: .continuous).strokeBorder(Theme.rule, lineWidth: 1))
        .accessibilityElement(children: .combine)
    }
}
