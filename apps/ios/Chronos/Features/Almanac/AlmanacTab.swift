import SwiftUI

/// Trạch Nhật và Xem Tuổi: Tam Tai, Kim Lâu, Hoang Ốc and Thái Tuế of the selected person in a
/// lunar year, with their readings.
struct AlmanacTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @State private var targetYear: Int32
    private let currentYear: Int32

    init(services: AppServices) {
        self.services = services
        let today = services.engine.today()
        let year = (try? services.engine.lunar(today).year) ?? today.year
        currentYear = year
        _targetYear = State(initialValue: year)
    }

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    ScrollView {
                        VStack(spacing: 16) {
                            yearSwitcher
                            switch Result(catching: { try AgeReading(profile: profile, year: targetYear, services: services) }) {
                            case .success(let reading):
                                PersonCard(profile: profile, reading: reading)
                                StatusGrid(reading: reading)
                                if let current = reading.restrictions.hoangOc {
                                    HoangOcRing(current: current, age: reading.restrictions.lunarAge).card()
                                }
                                TaiSuiCard(reading: reading.taiSui)
                                ExplanationsCard(entries: reading.entries, content: services.content)
                            case .failure(let error):
                                Text(String(describing: error)).foregroundStyle(Theme.cinnabar).card()
                            }
                        }
                        .padding(16)
                    }
                    .background(Theme.paper)
                } else {
                    ProfileEmptyStateView(
                        engine: services.engine,
                        systemImage: "checkmark.seal",
                        message: "Xem Tam Tai, Kim Lâu, Hoang Ốc và quan hệ với Thái Tuế của một người trong từng năm."
                    )
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Theme.paper)
                }
            }
            .navigationTitle("Xem tuổi")
            .navigationBarTitleDisplayMode(.inline)
            .profileToolbar(services)
        }
    }

    private var yearSwitcher: some View {
        HStack {
            Button { targetYear -= 1 } label: { Image(systemName: "chevron.left") }
                .accessibilityLabel("Năm trước")
                .disabled(Int(targetYear) <= EngineService.supportedYears.lowerBound)
            Spacer()
            VStack(spacing: 2) {
                Text("Năm \(String(targetYear))").font(.title3.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                Text(((try? services.engine.yearCanChi(lunarYear: targetYear)).map { canChiName(value: $0) } ?? "")
                    + (targetYear == currentYear ? " · năm nay" : ""))
                    .font(.caption).foregroundStyle(Theme.inkSecondary)
            }
            Spacer()
            Button { targetYear += 1 } label: { Image(systemName: "chevron.right") }
                .accessibilityLabel("Năm sau")
                .disabled(Int(targetYear) >= EngineService.supportedYears.upperBound)
        }
        .overlay(alignment: .bottomTrailing) {
            if targetYear != currentYear {
                Button("Năm nay") { targetYear = currentYear }.font(.caption2.weight(.semibold)).offset(y: 14)
            }
        }
        .card(padding: 14)
    }
}

/// Everything the tab shows for one person in one lunar year.
struct AgeReading {
    let birthYear: Int32
    let birthCanChi: CanChi
    let restrictions: AgeRestrictions
    let taiSui: TaiSuiReading
    let entries: [ContentEntry]

    init(profile: Profile, year: Int32, services: AppServices) throws {
        birthYear = try services.engine.lunarBirthDate(of: profile).year
        birthCanChi = try services.engine.yearCanChi(lunarYear: birthYear)
        restrictions = try services.engine.ageRestrictions(for: profile, targetLunarYear: year)
        taiSui = try services.engine.taiSui(for: profile, targetLunarYear: year)
        entries = try services.content.ageRestrictions(restrictions)
    }
}

private struct PersonCard: View {
    let profile: Profile
    let reading: AgeReading

    var body: some View {
        HStack(spacing: 14) {
            Text(String(branchName(branch: reading.birthCanChi.branch).prefix(1)))
                .font(.title.weight(.bold)).fontDesign(.serif)
                .foregroundStyle(Theme.paper)
                .frame(width: 56, height: 56)
                .background(Theme.cinnabar, in: Circle())
            VStack(alignment: .leading, spacing: 3) {
                Text(profile.name).font(.headline).foregroundStyle(Theme.ink)
                Text("\(profile.gender.label) · tuổi \(canChiName(value: reading.birthCanChi)) (\(String(reading.birthYear)))")
                    .font(.subheadline).foregroundStyle(Theme.inkSecondary)
            }
            Spacer()
            VStack(spacing: 0) {
                Text("\(reading.restrictions.lunarAge)").font(.title.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                Text("tuổi mụ").font(.caption2).foregroundStyle(Theme.inkSecondary)
            }
        }
        .card()
    }
}

/// Four tiles: phạm or not, each in its luck color.
private struct StatusGrid: View {
    let reading: AgeReading

    var body: some View {
        let restrictions = reading.restrictions
        LazyVGrid(columns: [GridItem(.flexible(), spacing: 10), GridItem(.flexible(), spacing: 10)], spacing: 10) {
            tile(
                "Tam Tai",
                restrictions.tamTai.map { "Năm " + tamTaiName(value: $0) } ?? "Không phạm",
                restrictions.tamTai == nil ? nil : ContentLuck.hung,
                Glossary.tamTaiNote
            )
            tile(
                "Kim Lâu",
                restrictions.kimLau.map { kimLauName(value: $0) } ?? "Không phạm",
                restrictions.kimLau == nil ? nil : ContentLuck.hung,
                Glossary.kimLauNote
            )
            tile(
                "Hoang Ốc",
                restrictions.hoangOc.map { hoangOcName(value: $0) } ?? "Chưa xét (dưới 10 tuổi)",
                restrictions.hoangOc.map { hoangOcLuck(value: $0) == .cat ? ContentLuck.cat : .hung },
                Glossary.hoangOcNote
            )
            tile(
                "Thái Tuế",
                reading.taiSui.relations.isEmpty
                    ? "Không xung, không hợp"
                    : reading.taiSui.relations.map { taiSuiName(value: $0) }.joined(separator: ", "),
                TaiSuiCard.luck(reading.taiSui),
                Glossary.taiSuiNote
            )
        }
    }

    private func tile(_ title: String, _ value: String, _ luck: ContentLuck?, _ note: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text(title.uppercased()).font(.caption2.weight(.semibold)).tracking(0.6).foregroundStyle(Theme.inkSecondary)
                Spacer()
                Circle().fill(Theme.luck(luck)).frame(width: 8, height: 8)
            }
            Text(value)
                .font(.subheadline.weight(.semibold))
                .foregroundStyle(luck == .hung ? Theme.cinnabar : Theme.ink)
                .fixedSize(horizontal: false, vertical: true)
            Spacer(minLength: 0)
            HelpNote(text: note)
        }
        .frame(minHeight: 72, maxHeight: .infinity, alignment: .topLeading)
        .card(padding: 12)
        .accessibilityElement(children: .combine)
    }
}

/// The six palaces of Hoang Ốc on a ring, the one of this age marked.
struct HoangOcRing: View {
    let current: HoangOc
    let age: Int32

    var body: some View {
        VStack(spacing: 12) {
            CardTitle(text: "Vòng Hoang Ốc", systemImage: "house")
            GeometryReader { geometry in
                let size = min(geometry.size.width, geometry.size.height)
                let radius = size / 2 - 38
                let center = CGPoint(x: geometry.size.width / 2, y: geometry.size.height / 2)
                ZStack {
                    Circle()
                        .stroke(Theme.rule, style: StrokeStyle(lineWidth: 1, dash: [3, 4]))
                        .frame(width: radius * 2, height: radius * 2)
                        .position(center)
                    VStack(spacing: 0) {
                        Text("\(age)").font(.title.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                        Text("tuổi mụ").font(.caption2).foregroundStyle(Theme.inkSecondary)
                    }
                    .position(center)
                    ForEach(Array(HoangOc.allCases.enumerated()), id: \.offset) { index, palace in
                        let angle = Angle.degrees(Double(index) * 60 - 90)
                        node(palace, number: index + 1)
                            .position(
                                x: center.x + radius * CGFloat(cos(angle.radians)),
                                y: center.y + radius * CGFloat(sin(angle.radians))
                            )
                    }
                }
            }
            .frame(height: 260)
        }
    }

    private func node(_ palace: HoangOc, number: Int) -> some View {
        let isCurrent = palace == current
        let color = hoangOcLuck(value: palace) == .cat ? Theme.gold : Theme.cinnabar
        return VStack(spacing: 2) {
            Text("\(number)")
                .font(.caption.weight(.bold))
                .foregroundStyle(isCurrent ? Theme.paper : color)
                .frame(width: 28, height: 28)
                .background(isCurrent ? color : color.opacity(0.14), in: Circle())
                .overlay(Circle().strokeBorder(color, lineWidth: isCurrent ? 0 : 1))
            Text(hoangOcName(value: palace))
                .font(.system(size: 11, weight: isCurrent ? .bold : .regular))
                .foregroundStyle(isCurrent ? Theme.ink : Theme.inkSecondary)
                .fixedSize()
        }
        .scaleEffect(isCurrent ? 1.08 : 1)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(hoangOcName(value: palace))\(isCurrent ? ", tuổi năm nay" : "")")
    }
}

/// Decision I5: relation of the birth branch to the Thái Tuế of the year.
private struct TaiSuiCard: View {
    let reading: TaiSuiReading

    /// Hung when any relation is Hung, Cát when only Hợp holds, none when nothing holds.
    static func luck(_ reading: TaiSuiReading) -> ContentLuck? {
        guard !reading.relations.isEmpty else { return nil }
        return reading.relations.contains { taiSuiLuck(value: $0) == .hung } ? .hung : .cat
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Thái Tuế", systemImage: "star.circle")
            HStack(spacing: 10) {
                branch("Tuổi", reading.birthBranch)
                Image(systemName: "arrow.left.and.right").foregroundStyle(Theme.inkSecondary)
                branch("Thái Tuế năm", reading.yearBranch)
            }
            if reading.relations.isEmpty {
                Text("Chi năm sinh không xung, hại, hình, phá hay hợp với chi của năm.")
                    .font(.subheadline).foregroundStyle(Theme.inkSecondary)
            } else {
                FlowLayout {
                    ForEach(reading.relations, id: \.self) { relation in
                        Chip(
                            text: taiSuiName(value: relation),
                            color: taiSuiLuck(value: relation) == .cat ? Theme.gold : Theme.cinnabar
                        )
                    }
                }
            }
        }
        .card()
    }

    private func branch(_ title: String, _ value: Branch) -> some View {
        VStack(spacing: 2) {
            Text(title).font(.caption2).foregroundStyle(Theme.inkSecondary)
            Text(branchName(branch: value)).font(.title3.weight(.semibold)).fontDesign(.serif).foregroundStyle(Theme.ink)
        }
        .frame(maxWidth: .infinity)
    }
}

/// The readings of the restrictions in force, from the content database.
private struct ExplanationsCard: View {
    let entries: [ContentEntry]
    let content: ContentRepository

    var body: some View {
        if !entries.isEmpty {
            VStack(alignment: .leading, spacing: 12) {
                CardTitle(text: "Lời giải", systemImage: "book")
                ForEach(entries) { entry in
                    EntryView(entry: entry, content: content)
                    if entry.id != entries.last?.id {
                        Divider().overlay(Theme.rule)
                    }
                }
            }
            .card()
        }
    }
}
