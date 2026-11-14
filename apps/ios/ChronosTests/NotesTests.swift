import Foundation
import Testing
@testable import Chronos

@MainActor
@Suite("Notes, yearly lunar events and reminders")
struct NotesTests {
    private let tz = 7.0

    private func lunarNote(_ title: String = "Giỗ Ông Nội", day: Int, month: Int, year: Int, leap: Bool = false, yearly: Bool = true) -> CalendarNote {
        CalendarNote(title: title, calendarType: .lunar, day: day, month: month, year: year, isLeapMonth: leap, isYearlyRecurring: yearly)
    }

    private func solar(_ year: Int, _ month: Int, _ day: Int) -> SolarDate {
        SolarDate(year: Int32(year), month: UInt8(month), day: UInt8(day))
    }

    private func lunarToSolarDate(_ year: Int, _ month: Int, _ day: Int, leap: Bool = false) throws -> SolarDate {
        try lunarToSolar(date: LunarDate(year: Int32(year), month: UInt8(month), day: UInt8(day), isLeapMonth: leap), tzOffset: tz)
    }

    /// A regular lunar month of `length` days in or after `year`.
    private func month(ofLength length: Int, from year: Int) throws -> (year: Int, month: Int) {
        for candidate in year...(year + 3) {
            let months = try lunarYear(year: Int32(candidate), tzOffset: tz).months
            if let found = months.first(where: { !$0.isLeap && Int($0.length) == length }) {
                return (candidate, Int(found.month))
            }
        }
        throw CocoaError(.featureUnsupported)
    }

    // MARK: Occurrences

    @Test func aYearlyLunarNoteFallsOnItsLunarDayEachYear() throws {
        let note = lunarNote(day: 15, month: 10, year: 2020)
        for year in [2020, 2026, 2027] {
            #expect(note.occurrence(inYear: year, tzOffset: tz) == (try lunarToSolarDate(year, 10, 15)))
        }
        #expect(note.occurrence(inYear: 2019, tzOffset: tz) == nil)
    }

    @Test func dayThirtyIsKeptOnDayTwentyNineInAShortMonth() throws {
        let short = try month(ofLength: 29, from: 2026)
        let long = try month(ofLength: 30, from: 2026)
        let inShort = lunarNote(day: 30, month: short.month, year: 2000)
        #expect(inShort.occurrence(inYear: short.year, tzOffset: tz) == (try lunarToSolarDate(short.year, short.month, 29)))
        let inLong = lunarNote(day: 30, month: long.month, year: 2000)
        #expect(inLong.occurrence(inYear: long.year, tzOffset: tz) == (try lunarToSolarDate(long.year, long.month, 30)))
        // The engine itself refuses day 30 of the short month: the note does not rely on that.
        #expect(throws: (any Error).self) { try lunarToSolarDate(short.year, short.month, 30) }
    }

    @Test func aYearlyNoteCanBeCreatedOnDayThirtyInAYearWhoseMonthIsShort() throws {
        let short = try month(ofLength: 29, from: 2026)
        let note = lunarNote(day: 30, month: short.month, year: short.year)
        #expect(note.occurrence(inYear: short.year, tzOffset: tz) == (try lunarToSolarDate(short.year, short.month, 29)))
        let once = lunarNote(day: 30, month: short.month, year: short.year, yearly: false)
        #expect(once.occurrence(inYear: short.year, tzOffset: tz) == nil)
    }

    @Test func aLeapYearKeepsTheRegularMonth() throws {
        // Quý Mão 2023 repeats its 2nd month.
        #expect(try lunarYear(year: 2023, tzOffset: tz).leapMonth == 2)
        let note = lunarNote(day: 10, month: 2, year: 2010)
        let regular = try lunarToSolarDate(2023, 2, 10)
        let leap = try lunarToSolarDate(2023, 2, 10, leap: true)
        #expect(regular != leap)
        #expect(note.occurrence(inYear: 2023, tzOffset: tz) == regular)
        #expect(note.occurrences(from: solar(2023, 1, 1), through: solar(2023, 12, 31), tzOffset: tz) == [regular])
    }

    @Test func anEventInALeapMonthIsItselfThatYearAndRegularAfterwards() throws {
        let note = lunarNote(day: 10, month: 2, year: 2023, leap: true)
        #expect(note.occurrence(inYear: 2023, tzOffset: tz) == (try lunarToSolarDate(2023, 2, 10, leap: true)))
        #expect(note.occurrence(inYear: 2024, tzOffset: tz) == (try lunarToSolarDate(2024, 2, 10)))
    }

    @Test func aNoteThatDoesNotRepeatLivesInItsYearOnly() throws {
        let note = lunarNote(day: 15, month: 10, year: 2026, yearly: false)
        #expect(note.occurrence(inYear: 2026, tzOffset: tz) != nil)
        #expect(note.occurrence(inYear: 2027, tzOffset: tz) == nil)
        #expect(note.nextOccurrenceSolarDate(onOrAfter: solar(2027, 1, 1), tzOffset: tz) == nil)
    }

    @Test func aSolarNoteOnTheLeapDayIsKeptOnTheTwentyEighth() {
        let note = CalendarNote(title: "Sinh nhật", calendarType: .solar, day: 29, month: 2, year: 2024, category: .birthday)
        #expect(note.occurrence(inYear: 2024, tzOffset: tz) == solar(2024, 2, 29))
        #expect(note.occurrence(inYear: 2027, tzOffset: tz) == solar(2027, 2, 28))
        #expect(note.occurrence(inYear: 2028, tzOffset: tz) == solar(2028, 2, 29))
    }

    @Test func theNextOccurrenceIsThisYearsOrNextYears() throws {
        let note = lunarNote(day: 15, month: 10, year: 2020)
        let thisYear = try lunarToSolarDate(2026, 10, 15)
        let nextYear = try lunarToSolarDate(2027, 10, 15)
        #expect(note.nextOccurrenceSolarDate(onOrAfter: solar(2026, 11, 14), tzOffset: tz) == thisYear)
        #expect(note.nextOccurrenceSolarDate(onOrAfter: thisYear, tzOffset: tz) == thisYear)
        let dayAfter = try #require(thisYear.adding(days: 1))
        #expect(note.nextOccurrenceSolarDate(onOrAfter: dayAfter, tzOffset: tz) == nextYear)
        // Early January 2027 is still lunar year 2026, whose 15/10 has passed.
        #expect(note.nextOccurrenceSolarDate(onOrAfter: solar(2027, 1, 5), tzOffset: tz) == nextYear)
    }

    // MARK: Calendar

    @Test func upcomingListsTheNextThirtyDaysSoonestFirst() throws {
        let today = solar(2026, 11, 14)
        let gio = lunarNote(day: 15, month: 10, year: 2020)
        let sinhNhat = CalendarNote(title: "Sinh nhật Lan", calendarType: .solar, day: 16, month: 11, year: 1995, category: .birthday)
        let homNay = CalendarNote(title: "Họp lớp", calendarType: .solar, day: 14, month: 11, year: 2026, isYearlyRecurring: false, category: .important)
        let xa = CalendarNote(title: "Tết Dương", calendarType: .solar, day: 1, month: 1, year: 2000, category: .ceremony)
        let upcoming = NoteCalendar.upcoming([gio, sinhNhat, homNay, xa], from: today, days: 30, tzOffset: tz)
        #expect(upcoming.map(\.note.title) == ["Họp lớp", "Sinh nhật Lan", "Giỗ Ông Nội"])
        #expect(upcoming.map(\.daysLeft) == [0, 2, today.days(until: try lunarToSolarDate(2026, 10, 15))])
        #expect(upcoming.map(\.countdown).prefix(2) == ["Hôm nay", "Còn 2 ngày"])
        #expect(upcoming[2].countdown == "Còn \(upcoming[2].daysLeft) ngày")
    }

    @Test func aDayAndAMonthFindTheirNotes() throws {
        let gio = lunarNote(day: 15, month: 10, year: 2020)
        let day = try lunarToSolarDate(2026, 10, 15)
        #expect(NoteCalendar.notes([gio], on: day, tzOffset: tz) == [gio])
        #expect(NoteCalendar.notes([gio], on: try #require(day.adding(days: 1)), tzOffset: tz).isEmpty)
        let marked = NoteCalendar.markedDays([gio], from: solar(2026, 11, 1), through: solar(2026, 11, 30), tzOffset: tz)
        #expect(marked == [day])
        #expect(day.month == 11)
    }

    @Test func aDraftOnADayIsWrittenOnTheLunarCalendarAndConvertsBack() throws {
        let day = solar(2026, 11, 14)
        let draft = CalendarNote.draft(on: day, tzOffset: tz)
        #expect(draft.calendarType == .lunar)
        #expect((draft.day, draft.month, draft.year) == (6, 10, 2026))
        #expect(draft.isYearlyRecurring && draft.category == .anniversary)
        #expect(draft.occurrence(inYear: 2026, tzOffset: tz) == day)
        let asSolar = draft.rewritten(as: .solar, tzOffset: tz)
        #expect((asSolar.day, asSolar.month, asSolar.year) == (14, 11, 2026))
        #expect(asSolar.rewritten(as: .lunar, tzOffset: tz) == draft)
    }

    @Test func labelsNameTheCalendar() {
        #expect(lunarNote(day: 15, month: 10, year: 2020).dateLabel == "15/10 Âm lịch")
        #expect(lunarNote(day: 10, month: 2, year: 2023, leap: true, yearly: false).dateLabel == "10/2 nhuận/2023 Âm lịch")
        #expect(NoteCategory.anniversary.label == "Ngày giỗ")
        #expect(NoteCategory.allCases.filter(\.repeatsYearlyByDefault) == [.anniversary, .birthday, .ceremony])
    }

    @Test func thePreviewOfTheFormSaysWhenAndWhy() throws {
        let short = try month(ofLength: 29, from: 2026)
        let note = lunarNote(day: 30, month: short.month, year: short.year)
        let lines = NoteFormView.preview(note, today: solar(short.year, 1, 1), tzOffset: tz)
        #expect(lines.first?.hasPrefix("Dương lịch năm \(short.year):") == true)
        #expect(lines.contains { $0.hasPrefix("Lần tới:") })
        #expect(lines.contains("Năm nào tháng \(short.month) thiếu (29 ngày) thì tính vào ngày 29."))
        let invalid = CalendarNote(title: "x", calendarType: .solar, day: 31, month: 2, year: 2026, isYearlyRecurring: false)
        #expect(NoteFormView.preview(invalid, today: solar(2026, 1, 1), tzOffset: tz) == ["Ngày này không có trên lịch."])
    }

    // MARK: Store

    private func withDefaults(_ body: (UserDefaults) throws -> Void) rethrows {
        let suite = "ChronosTests.notes.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suite)!
        defer { defaults.removePersistentDomain(forName: suite) }
        try body(defaults)
    }

    @Test func theStoreSavesReplacesDeletesAndTellsItsListeners() {
        withDefaults { defaults in
            var heard: [[CalendarNote]] = []
            let store = NoteStore(defaults: defaults) { heard.append($0) }
            #expect(store.notes.isEmpty)
            var note = lunarNote(day: 15, month: 10, year: 2020)
            store.save(note)
            note.title = "Giỗ Bà Nội"
            store.save(note)
            #expect(store.notes.map(\.title) == ["Giỗ Bà Nội"])
            #expect(NoteStore(defaults: defaults) { _ in }.notes == [note])
            #expect(NoteStorage.load(from: defaults) == [note])
            store.delete(note.id)
            #expect(NoteStorage.load(from: defaults).isEmpty)
            #expect(heard.map(\.count) == [1, 1, 0])
        }
    }

    @Test func theSharedSuiteIsTheAppGroup() {
        #expect(NoteStorage.appGroup == "group.com.leanhtuan.chronos")
        #expect(CalendarNote(title: "x", calendarType: .lunar, day: 1, month: 1, year: 2026).reminderOffsets == [0, 1, 3])
    }

    // MARK: Reminders

    private func vietnam(_ year: Int, _ month: Int, _ day: Int, _ hour: Int, _ minute: Int = 0) throws -> Date {
        try #require(Calendar.zone(7).date(from: DateComponents(year: year, month: month, day: day, hour: hour, minute: minute)))
    }

    @Test func remindersFireAtEightOnTheDayAndTheDaysBefore() throws {
        let once = CalendarNote(title: "Giỗ Ông Nội", calendarType: .lunar, day: 15, month: 10, year: 2026, isYearlyRecurring: false)
        let day = try lunarToSolarDate(2026, 10, 15)
        let requests = ReminderPlan.requests(for: [once], now: try vietnam(2026, 11, 14, 9), tzOffset: tz)
        #expect(requests.count == 3)
        let days = try [3, 1, 0].map { try #require(day.adding(days: -$0)) }
        #expect(requests.map { $0.fire.day } == days.map { Int($0.day) })
        #expect(requests.allSatisfy { $0.fire.hour == 8 && $0.fire.minute == 0 && $0.fire.timeZone == TimeZone(secondsFromGMT: 7 * 3600) })
        #expect(requests.allSatisfy { $0.title == "Ngày giỗ" && $0.id.hasPrefix(ReminderPlan.prefix) })
        #expect(requests[0].body.hasPrefix("Còn 3 ngày nữa là Giỗ Ông Nội (15/10/2026 Âm lịch)"))
        #expect(requests[1].body.hasPrefix("Ngày mai là"))
        #expect(requests[2].body.hasPrefix("Hôm nay là"))
        #expect(Set(requests.map(\.id)).count == 3)
    }

    @Test func aReminderWhoseHourHasPassedIsNotScheduled() throws {
        let today = CalendarNote(title: "Họp lớp", calendarType: .solar, day: 14, month: 11, year: 2026, isYearlyRecurring: false, category: .important)
        let before = ReminderPlan.requests(for: [today], now: try vietnam(2026, 11, 14, 7, 59), tzOffset: tz)
        #expect(before.map(\.fire.day) == [14])
        let after = ReminderPlan.requests(for: [today], now: try vietnam(2026, 11, 14, 8), tzOffset: tz)
        #expect(after.isEmpty)
        var silent = today
        silent.reminderOffsets = []
        #expect(ReminderPlan.requests(for: [silent], now: try vietnam(2026, 11, 1, 7), tzOffset: tz).isEmpty)
    }

    @Test func thePlanStaysUnderThePendingLimitAndTakesTheSoonest() throws {
        let notes = (1...28).map { CalendarNote(title: "Việc \($0)", calendarType: .solar, day: $0, month: 12, year: 2026, isYearlyRecurring: false, category: .important) }
        let requests = ReminderPlan.requests(for: notes, now: try vietnam(2026, 11, 14, 9), tzOffset: tz)
        #expect(requests.count == ReminderPlan.limit)
        #expect(requests.first?.fire.month == 11 && requests.first?.fire.day == 28)
        let days = requests.map { ($0.fire.month ?? 0) * 100 + ($0.fire.day ?? 0) }
        #expect(days == days.sorted())
    }

    @Test func aYearlyNoteIsPlannedForItsNextOccurrence() throws {
        let note = lunarNote(day: 15, month: 10, year: 2020)
        let thisYear = try lunarToSolarDate(2026, 10, 15)
        let now = try #require(Calendar.zone(7).date(from: DateComponents(year: Int(thisYear.year), month: Int(thisYear.month), day: Int(thisYear.day), hour: 12)))
        let requests = ReminderPlan.requests(for: [note], now: now, tzOffset: tz)
        let nextYear = try lunarToSolarDate(2027, 10, 15)
        #expect(requests.count == 3)
        #expect(requests.last?.fire.year == Int(nextYear.year))
        #expect(requests.last?.fire.day == Int(nextYear.day))
    }
}
