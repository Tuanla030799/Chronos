import Foundation

/// A note falling on a solar day, seen from a reference day.
struct NoteOccurrence: Identifiable, Hashable, Sendable {
    let note: CalendarNote
    let date: SolarDate
    /// Days from the reference day: 0 is that day.
    let daysLeft: Int

    var id: String { "\(note.id)/\(date.ordinal)" }

    /// "Hôm nay", "Ngày mai" or "Còn 9 ngày".
    var countdown: String {
        switch daysLeft {
        case 0: "Hôm nay"
        case 1: "Ngày mai"
        default: "Còn \(daysLeft) ngày"
        }
    }
}

extension CalendarNote {
    /// The solar day of the note in `year` of its own calendar, or nil when it has none there:
    /// a year before the event, another year of a note that does not repeat, or a date its
    /// calendar does not have.
    ///
    /// A yearly lunar note follows the custom of the death anniversary: in a year whose month
    /// is short, day 30 is kept on day 29; and in a year that repeats its month as a leap
    /// month, it stays in the regular month. Only the year of the event itself reads
    /// `isLeapMonth`, and a yearly note whose exact date that year lacks follows the same
    /// custom there. A yearly solar note on 29 February is kept on the 28th in a common year.
    func occurrence(inYear year: Int, tzOffset: Double) -> SolarDate? {
        guard year == self.year || (isYearlyRecurring && year > self.year) else { return nil }
        switch calendarType {
        case .solar:
            guard let length = Self.solarMonthLength(year: year, month: month), (1...31).contains(day) else { return nil }
            guard day <= length || isYearlyRecurring else { return nil }
            return SolarDate(year: Int32(year), month: UInt8(month), day: UInt8(min(day, length)))
        case .lunar:
            guard (1...12).contains(month), (1...30).contains(day) else { return nil }
            if year == self.year {
                let exact = LunarDate(year: Int32(year), month: UInt8(month), day: UInt8(day), isLeapMonth: isLeapMonth)
                if let date = try? lunarToSolar(date: exact, tzOffset: tzOffset) {
                    return date
                }
                guard isYearlyRecurring else { return nil }
            }
            guard let months = try? lunarYear(year: Int32(year), tzOffset: tzOffset).months,
                  let regular = months.first(where: { Int($0.month) == month && !$0.isLeap })
            else { return nil }
            let kept = LunarDate(year: Int32(year), month: UInt8(month), day: UInt8(min(day, Int(regular.length))), isLeapMonth: false)
            return try? lunarToSolar(date: kept, tzOffset: tzOffset)
        }
    }

    /// Every solar day of the note from `first` through `last`.
    func occurrences(from first: SolarDate, through last: SolarDate, tzOffset: Double) -> [SolarDate] {
        guard first.ordinal <= last.ordinal else { return [] }
        let years: ClosedRange<Int>
        switch calendarType {
        case .solar:
            years = Int(first.year)...Int(last.year)
        case .lunar:
            guard let from = try? solarToLunar(date: first, tzOffset: tzOffset),
                  let to = try? solarToLunar(date: last, tzOffset: tzOffset)
            else { return [] }
            years = Int(from.year)...Int(to.year)
        }
        return years
            .compactMap { occurrence(inYear: $0, tzOffset: tzOffset) }
            .filter { (first.ordinal...last.ordinal).contains($0.ordinal) }
    }

    /// The next solar day of the note on or after `today`: this year's if it is still ahead,
    /// else next year's. Nil for a past note that does not repeat.
    func nextOccurrenceSolarDate(onOrAfter today: SolarDate, tzOffset: Double) -> SolarDate? {
        let current: Int
        switch calendarType {
        case .solar:
            current = Int(today.year)
        case .lunar:
            guard let lunar = try? solarToLunar(date: today, tzOffset: tzOffset) else { return nil }
            current = Int(lunar.year)
        }
        let start = max(current, year)
        return [start, start + 1]
            .compactMap { occurrence(inYear: $0, tzOffset: tzOffset) }
            .first { $0.ordinal >= today.ordinal }
    }

    private static func solarMonthLength(year: Int, month: Int) -> Int? {
        guard (1...12).contains(month),
              let date = Calendar.civil.date(from: DateComponents(year: year, month: month, day: 1))
        else { return nil }
        return Calendar.civil.range(of: .day, in: .month, for: date)?.count
    }
}

/// Notes on the days of a calendar.
enum NoteCalendar {
    /// The notes that fall on `date`, in the order they were saved.
    static func notes(_ notes: [CalendarNote], on date: SolarDate, tzOffset: Double) -> [CalendarNote] {
        notes.filter { !$0.occurrences(from: date, through: date, tzOffset: tzOffset).isEmpty }
    }

    /// The days from `first` through `last` that have a note.
    static func markedDays(_ notes: [CalendarNote], from first: SolarDate, through last: SolarDate, tzOffset: Double) -> Set<SolarDate> {
        Set(notes.flatMap { $0.occurrences(from: first, through: last, tzOffset: tzOffset) })
    }

    /// What falls from `today` through `days` days later, soonest first.
    static func upcoming(_ notes: [CalendarNote], from today: SolarDate, days: Int, tzOffset: Double) -> [NoteOccurrence] {
        guard let last = today.adding(days: days) else { return [] }
        return notes
            .flatMap { note in
                note.occurrences(from: today, through: last, tzOffset: tzOffset).map { date in
                    NoteOccurrence(note: note, date: date, daysLeft: today.days(until: date))
                }
            }
            .sorted { ($0.daysLeft, $0.note.title) < ($1.daysLeft, $1.note.title) }
    }
}

extension SolarDate {
    /// Orders civil dates: 20261114 for 14 November 2026.
    var ordinal: Int {
        Int(year) * 10_000 + Int(month) * 100 + Int(day)
    }

    /// Whole days from this date to `other`; negative when `other` is earlier.
    func days(until other: SolarDate) -> Int {
        let calendar = Calendar.civil
        guard let from = calendar.date(from: DateComponents(year: Int(year), month: Int(month), day: Int(day))),
              let to = calendar.date(from: DateComponents(year: Int(other.year), month: Int(other.month), day: Int(other.day)))
        else { return 0 }
        return calendar.dateComponents([.day], from: from, to: to).day ?? 0
    }
}

extension CalendarNote {
    /// The same day written on the other calendar, read in the note's own year. A date its
    /// calendar lacks keeps its numbers, for the form to refuse.
    func rewritten(as kind: CalendarKind, tzOffset: Double) -> CalendarNote {
        var rewritten = self
        rewritten.calendarType = kind
        guard kind != calendarType, let solar = occurrence(inYear: year, tzOffset: tzOffset) else { return rewritten }
        switch kind {
        case .solar:
            rewritten.year = Int(solar.year)
            rewritten.month = Int(solar.month)
            rewritten.day = Int(solar.day)
            rewritten.isLeapMonth = false
        case .lunar:
            guard let lunar = try? solarToLunar(date: solar, tzOffset: tzOffset) else { return rewritten }
            rewritten.year = Int(lunar.year)
            rewritten.month = Int(lunar.month)
            rewritten.day = Int(lunar.day)
            rewritten.isLeapMonth = lunar.isLeapMonth
        }
        return rewritten
    }

    /// A new note on `date`, written on the lunar calendar as a death anniversary is.
    static func draft(on date: SolarDate, tzOffset: Double) -> CalendarNote {
        let solar = CalendarNote(title: "", calendarType: .solar, day: Int(date.day), month: Int(date.month), year: Int(date.year))
        return solar.rewritten(as: .lunar, tzOffset: tzOffset)
    }
}
