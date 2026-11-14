import SwiftUI

/// Đại Vận: when luck starts, then the cycles of ten years in a row that scrolls sideways. The
/// cycle that holds the current year is marked.
struct LuckPillarsCard: View {
    let daiVan: DaiVan
    let currentYear: Int32

    private var currentIndex: UInt8? {
        daiVan.cycles.first { $0.startYear...$0.endYear ~= currentYear }?.index
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Đại Vận", systemImage: "arrow.right.circle", info: Glossary.baziTerms)
            HelpNote(text: "Vận trình đổi mỗi mười năm; mỗi ô là một bước, ghi tuổi và năm của nó.")

            VStack(alignment: .leading, spacing: 4) {
                Text(startText)
                    .font(.headline).fontDesign(.serif).foregroundStyle(Theme.ink)
                Text(daiVan.forward ? "Vận đi thuận" : "Vận đi nghịch")
                    .font(.caption).foregroundStyle(Theme.inkSecondary)
            }

            ScrollViewReader { proxy in
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(daiVan.cycles, id: \.index) { cycle in
                            CycleCard(cycle: cycle, isCurrent: cycle.index == currentIndex)
                                .id(cycle.index)
                        }
                    }
                    .padding(.vertical, 2)
                }
                .onAppear {
                    if let currentIndex {
                        proxy.scrollTo(currentIndex, anchor: .center)
                    }
                }
            }
        }
        .card()
    }

    private var startText: String {
        var parts = ["\(daiVan.startYears) tuổi"]
        if daiVan.startMonths > 0 { parts.append("\(daiVan.startMonths) tháng") }
        if daiVan.startDays > 0 { parts.append("\(daiVan.startDays) ngày") }
        return "Khởi vận lúc " + parts.joined(separator: " ")
    }
}

private struct CycleCard: View {
    let cycle: LuckCycle
    let isCurrent: Bool

    var body: some View {
        VStack(spacing: 6) {
            Text(isCurrent ? "HIỆN TẠI" : " ")
                .font(.system(size: 9, weight: .bold)).tracking(0.6)
                .foregroundStyle(Theme.cinnabar)
            Text("\(cycle.startAge)–\(cycle.endAge)")
                .font(.caption.weight(.semibold)).foregroundStyle(Theme.inkSecondary)
            if let canChi = cycle.canChi {
                Text(stemName(stem: canChi.stem))
                    .font(.system(size: 24, weight: .bold, design: .serif))
                    .foregroundStyle(Theme.element(stemElement(stem: canChi.stem)))
                Text(branchName(branch: canChi.branch))
                    .font(.system(size: 24, weight: .bold, design: .serif))
                    .foregroundStyle(Theme.element(branchElement(branch: canChi.branch)))
            } else {
                Text("Trước\nvận")
                    .font(.subheadline.weight(.medium)).foregroundStyle(Theme.inkSecondary)
                    .multilineTextAlignment(.center)
                    .frame(height: 58)
            }
            Text("\(String(cycle.startYear))–\(String(cycle.endYear))")
                .font(.system(size: 10)).foregroundStyle(Theme.inkSecondary)
        }
        .padding(.vertical, 10)
        .frame(width: 84)
        .background(isCurrent ? Theme.cinnabar.opacity(0.1) : Theme.paper, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        .overlay {
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(isCurrent ? Theme.cinnabar : Theme.rule, lineWidth: isCurrent ? 2 : 1)
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(label)
    }

    private var label: String {
        let range = "từ \(cycle.startAge) đến \(cycle.endAge) tuổi, năm \(cycle.startYear) đến \(cycle.endYear)"
        let name = cycle.canChi.map { canChiName(value: $0) } ?? "trước vận"
        return "Đại vận \(name), \(range)\(isCurrent ? ", hiện tại" : "")"
    }
}
