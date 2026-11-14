import SwiftUI

/// Lịch Vạn Niên: a month grid, then the sheet of the chosen day.
struct CalendarTab: View {
    let services: AppServices
    private let today: SolarDate
    @State private var selected: SolarDate
    @State private var page: Result<MonthPage, Error>
    @State private var opened: ContentEntry?
    @State private var slide: Edge = .trailing
    @Environment(NoteStore.self) private var notes

    init(services: AppServices) {
        self.services = services
        let today = services.engine.today()
        self.today = today
        _selected = State(initialValue: today)
        _page = State(initialValue: Result { try MonthPage.build(MonthKey(today), engine: services.engine) })
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 16) {
                    monthCard
                    switch Result(catching: { try DayReading(selected, services: services) }) {
                    case .success(let reading):
                        DaySheetCard(reading: reading)
                        NotesCard(selected: selected, today: today, engine: services.engine)
                        HoursCard(hoangDao: reading.almanac.hoangDaoHours)
                        ActivitiesCard(entries: reading.activityEntries, content: services.content)
                        GodsCard(reading: reading) { opened = $0 }
                    case .failure(let error):
                        Text(String(describing: error)).foregroundStyle(Theme.cinnabar).card()
                    }
                }
                .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle("Lịch Vạn Niên")
            .navigationBarTitleDisplayMode(.inline)
            .profileToolbar(services)
            .sheet(item: $opened) { EntrySheet(entry: $0, content: services.content) }
        }
    }

    // MARK: Month

    @ViewBuilder
    private var monthCard: some View {
        VStack(spacing: 12) {
            HStack {
                Button { show(months: -1) } label: { Image(systemName: "chevron.left") }
                    .accessibilityLabel("Tháng trước")
                Spacer()
                Text(title).font(.headline.weight(.semibold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                Spacer()
                Button { show(months: 1) } label: { Image(systemName: "chevron.right") }
                    .accessibilityLabel("Tháng sau")
            }
            .padding(.horizontal, 4)

            HStack(spacing: 0) {
                ForEach(["T2", "T3", "T4", "T5", "T6", "T7", "CN"], id: \.self) { name in
                    Text(name)
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(name == "CN" ? Theme.cinnabar : Theme.inkSecondary)
                        .frame(maxWidth: .infinity)
                }
            }

            switch page {
            case .success(let page):
                MonthGrid(page: page, selected: selected, today: today, marked: marked(page)) { day in
                    withAnimation(.snappy) { selected = day.solar }
                }
                .id(page.key)
                .transition(.push(from: slide))
            case .failure(let error):
                Text(String(describing: error)).foregroundStyle(Theme.cinnabar)
            }

            HStack(spacing: 16) {
                Label("Hoàng Đạo", systemImage: "circle.fill").foregroundStyle(Theme.gold)
                Label("Hắc Đạo", systemImage: "circle.fill").foregroundStyle(Theme.cinnabar)
                InfoButton(topic: Glossary.calendarMarks)
                Spacer()
                if selected != today {
                    Button("Hôm nay") { go(to: today) }.font(.caption.weight(.semibold))
                }
            }
            .font(.caption2)
            .labelStyle(DotLabelStyle())
        }
        .card(padding: 12)
        .clipped()
        .gesture(
            DragGesture(minimumDistance: 24).onEnded { value in
                if value.translation.width < -50 { show(months: 1) }
                if value.translation.width > 50 { show(months: -1) }
            }
        )
    }

    /// Days of the month that have a note or an event.
    private func marked(_ page: MonthPage) -> Set<SolarDate> {
        guard let first = page.days.first?.solar, let last = page.days.last?.solar else { return [] }
        return NoteCalendar.markedDays(notes.notes, from: first, through: last, tzOffset: services.engine.tzOffset)
    }

    private var title: String {
        let key = (try? page.get().key) ?? MonthKey(selected)
        return "Tháng \(key.month), \(String(key.year))"
    }

    /// Moves the grid by `months`, keeping the day of month where the new month has it.
    private func show(months: Int) {
        let next = Result { try MonthPage.build(MonthKey(selected).adding(months: months), engine: services.engine) }
        guard let built = try? next.get(), let last = built.days.last else {
            page = next
            return
        }
        let day = SolarDate(year: built.key.year, month: built.key.month, day: min(selected.day, last.solar.day))
        slide = months > 0 ? .trailing : .leading
        withAnimation(.snappy) {
            page = next
            selected = day
        }
    }

    private func go(to date: SolarDate) {
        let key = MonthKey(date)
        guard (try? page.get().key) != key else {
            withAnimation(.snappy) { selected = date }
            return
        }
        let current = (try? page.get().key) ?? MonthKey(selected)
        slide = (key.year, key.month) > (current.year, current.month) ? .trailing : .leading
        withAnimation(.snappy) {
            selected = date
            page = Result { try MonthPage.build(key, engine: services.engine) }
        }
    }
}

/// Everything shown for one day.
struct DayReading {
    let info: DayInfo
    let almanac: AlmanacDay
    let entries: [ContentEntry]

    init(_ date: SolarDate, services: AppServices) throws {
        almanac = try services.engine.almanac(date)
        info = try services.engine.day(date)
        entries = try services.content.almanacDay(almanac)
    }

    func entries(_ collection: String) -> [ContentEntry] {
        entries.filter { $0.collection == collection }
    }

    var isHoangDao: Bool { pathGodIsHoangDao(value: almanac.pathGod) }

    /// Entries that say what to do or avoid, in key order: Trực, Tú, then the gods.
    var activityEntries: [ContentEntry] {
        entries.filter { !$0.extra.goodFor.isEmpty || !$0.extra.avoid.isEmpty }
    }
}

private struct DotLabelStyle: LabelStyle {
    func makeBody(configuration: Configuration) -> some View {
        HStack(spacing: 4) {
            configuration.icon.font(.system(size: 6))
            configuration.title.foregroundStyle(Theme.inkSecondary)
        }
    }
}

// MARK: - Grid

private struct MonthGrid: View {
    let page: MonthPage
    let selected: SolarDate
    let today: SolarDate
    let marked: Set<SolarDate>
    let select: (CalendarDay) -> Void

    /// Weeks of seven cells; nil pads before day 1 and after the last day. Laid out eagerly so
    /// the scroll view knows the grid's height on the first pass.
    private var weeks: [[CalendarDay?]] {
        let cells: [CalendarDay?] = Array(repeating: nil, count: page.leadingBlanks) + page.days.map { $0 }
        let padded = cells + Array(repeating: nil, count: (7 - cells.count % 7) % 7)
        return stride(from: 0, to: padded.count, by: 7).map { Array(padded[$0..<$0 + 7]) }
    }

    var body: some View {
        VStack(spacing: 4) {
            ForEach(Array(weeks.enumerated()), id: \.offset) { _, week in
                HStack(spacing: 2) {
                    ForEach(Array(week.enumerated()), id: \.offset) { _, day in
                        if let day {
                            DayCell(day: day, isSelected: day.solar == selected, isToday: day.solar == today, hasNote: marked.contains(day.solar))
                                .onTapGesture { select(day) }
                                .accessibilityAddTraits(.isButton)
                        } else {
                            Color.clear.frame(maxWidth: .infinity, minHeight: 50)
                        }
                    }
                }
            }
        }
    }
}

private struct DayCell: View {
    let day: CalendarDay
    let isSelected: Bool
    let isToday: Bool
    let hasNote: Bool

    var body: some View {
        VStack(spacing: 1) {
            Text("\(day.solar.day)")
                .font(.callout.weight(isSelected || isToday ? .bold : .regular))
                .foregroundStyle(isSelected ? Theme.paper : day.solar.isSunday ? Theme.cinnabar : Theme.ink)
            Text(day.lunarLabel)
                .font(.system(size: 9, weight: day.lunar.day == 1 ? .bold : .regular))
                .foregroundStyle(isSelected ? Theme.paper.opacity(0.85) : day.lunar.day == 1 ? Theme.cinnabar : Theme.inkSecondary)
            Circle()
                .fill(day.isHoangDao ? Theme.gold : Theme.cinnabar.opacity(0.55))
                .frame(width: 5, height: 5)
        }
        .frame(maxWidth: .infinity, minHeight: 50)
        .background {
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(isSelected ? Theme.cinnabar : .clear)
        }
        .overlay {
            if isToday && !isSelected {
                RoundedRectangle(cornerRadius: 10, style: .continuous).strokeBorder(Theme.cinnabar, lineWidth: 1.5)
            }
        }
        .overlay(alignment: .topTrailing) {
            if hasNote {
                Image(systemName: "bookmark.fill")
                    .font(.system(size: 8))
                    .foregroundStyle(isSelected ? Theme.paper : Theme.cinnabar)
                    .padding(4)
            }
        }
        .contentShape(Rectangle())
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Ngày \(day.solar.day), âm lịch \(day.lunar.day) tháng \(day.lunar.month), \(day.isHoangDao ? "Hoàng Đạo" : "Hắc Đạo")\(hasNote ? ", có ghi chú" : "")")
    }
}

// MARK: - Day sheet

/// The classic tear-off sheet: a large solar day, the lunar date and the three Can Chi.
private struct DaySheetCard: View {
    let reading: DayReading

    var body: some View {
        let info = reading.info
        VStack(spacing: 10) {
            Text("THÁNG \(info.solar.month) · \(String(info.solar.year))")
                .font(.caption.weight(.semibold)).tracking(1.2)
                .foregroundStyle(Theme.inkSecondary)
            Text("\(info.solar.day)")
                .font(.system(size: 96, weight: .bold, design: .serif))
                .foregroundStyle(info.solar.isSunday ? Theme.cinnabar : Theme.ink)
                .contentTransition(.numericText())
            Text(info.solar.weekdayName).font(.title3.weight(.semibold)).foregroundStyle(Theme.cinnabar)

            Divider().overlay(Theme.rule)

            Text("Ngày \(info.lunar.day) tháng \(info.lunar.month)\(info.lunar.isLeapMonth ? " nhuận" : "") năm \(canChiName(value: info.yearCanChi))")
                .font(.headline).fontDesign(.serif).foregroundStyle(Theme.ink)
                .multilineTextAlignment(.center)

            HStack(spacing: 0) {
                pillar("Năm", info.yearCanChi)
                pillar("Tháng", info.monthCanChi)
                pillar("Ngày", info.dayCanChi)
            }

            HStack(spacing: 8) {
                Chip(
                    text: (reading.isHoangDao ? "Hoàng Đạo · " : "Hắc Đạo · ") + pathGodName(value: reading.almanac.pathGod),
                    color: reading.isHoangDao ? Theme.gold : Theme.cinnabar
                )
                Chip(text: "Tiết " + solarTermName(term: info.solarTerm) + (info.solarTermBegins ? " (giao tiết)" : ""))
            }
        }
        .frame(maxWidth: .infinity)
        .card(padding: 20)
    }

    private func pillar(_ title: String, _ value: CanChi) -> some View {
        VStack(spacing: 2) {
            Text(title).font(.caption2).foregroundStyle(Theme.inkSecondary)
            Text(canChiName(value: value)).font(.title3.weight(.semibold)).fontDesign(.serif).foregroundStyle(Theme.ink)
        }
        .frame(maxWidth: .infinity)
    }
}

/// The twelve double-hours, the Hoàng Đạo ones marked.
private struct HoursCard: View {
    let hoangDao: [Branch]

    var body: some View {
        VStack(spacing: 10) {
            CardTitle(text: "Giờ Hoàng Đạo", systemImage: "clock")
            Grid(horizontalSpacing: 6, verticalSpacing: 6) {
                ForEach(0..<3, id: \.self) { row in
                    GridRow {
                        ForEach(row * 4..<row * 4 + 4, id: \.self) { index in
                            cell(index)
                        }
                    }
                }
            }
        }
        .card(padding: 12)
    }

    private func cell(_ index: Int) -> some View {
        let branch = Branch.allCases[index]
        let good = hoangDao.contains(branch)
        return VStack(spacing: 2) {
            Image(systemName: good ? "sun.max.fill" : "moon")
                .font(.caption)
                .foregroundStyle(good ? Theme.gold : Theme.inkSecondary.opacity(0.6))
            Text(branchName(branch: branch))
                .font(.subheadline.weight(good ? .semibold : .regular))
                .foregroundStyle(good ? Theme.ink : Theme.inkSecondary)
            Text(branch.hourRange).font(.system(size: 10)).foregroundStyle(Theme.inkSecondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 8)
        .background(good ? Theme.gold.opacity(0.12) : Theme.paper.opacity(0.5), in: RoundedRectangle(cornerRadius: 10))
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Giờ \(branchName(branch: branch)), \(branch.hourRange), \(good ? "Hoàng Đạo" : "Hắc Đạo")")
    }
}

/// What to do and what to avoid, as each source says it.
private struct ActivitiesCard: View {
    let entries: [ContentEntry]
    let content: ContentRepository

    var body: some View {
        if !entries.isEmpty {
            VStack(alignment: .leading, spacing: 12) {
                CardTitle(text: "Việc nên làm, kiêng kỵ", systemImage: "checklist")
                ForEach(entries) { entry in
                    VStack(alignment: .leading, spacing: 6) {
                        Text(entry.title).font(.subheadline.weight(.semibold)).foregroundStyle(Theme.ink)
                        FlowLayout {
                            ForEach(entry.extra.goodFor, id: \.self) { item in
                                Chip(text: "✓ " + content.activityLabel(item.activity), color: Theme.gold)
                            }
                            ForEach(entry.extra.avoid, id: \.self) { item in
                                Chip(text: "✕ " + content.activityLabel(item.activity), color: Theme.cinnabar)
                            }
                        }
                    }
                    if entry.id != entries.last?.id {
                        Divider().overlay(Theme.rule)
                    }
                }
            }
            .card()
        }
    }
}

/// Trực, Tú, the path god, the almanac gods and the Vietnamese taboo days; a row opens its text.
private struct GodsCard: View {
    let reading: DayReading
    let open: (ContentEntry) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            group("Trực", "almanac.truc", icon: "seal")
            group("Nhị Thập Bát Tú", "almanac.mansion", icon: "sparkles")
            group("Thần trực nhật", "almanac.path_god", icon: "sun.horizon", luck: reading.isHoangDao ? .cat : .hung)
            group("Thần Sát", "almanac.god", icon: "shield.lefthalf.filled")
            group("Ngày kỵ dân gian", "almanac.taboo", icon: "exclamationmark.triangle")
        }
    }

    @ViewBuilder
    private func group(_ title: String, _ collection: String, icon: String, luck: ContentLuck? = nil) -> some View {
        let entries = reading.entries(collection)
        if !entries.isEmpty {
            VStack(spacing: 10) {
                CardTitle(text: title, systemImage: icon)
                ForEach(entries) { entry in
                    EntryRow(entry: entry, luck: luck) { open(entry) }
                }
            }
            .card(padding: 14)
        }
    }
}
