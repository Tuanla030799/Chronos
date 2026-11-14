import Foundation
import SwiftUI
import Testing
import UIKit
@testable import Chronos

@Suite("Widget day, hours and timeline")
struct DayWidgetTests {
    let engine = EngineService()

    /// 14 November 2026 at `hour`:`minute` on Vietnam's clock.
    private func vietnam(_ hour: Int, _ minute: Int = 0, day: Int = 14) throws -> Date {
        try #require(Calendar.zone(7).date(from: DateComponents(year: 2026, month: 11, day: day, hour: hour, minute: minute)))
    }

    // MARK: Hours

    @Test func aDayHasThirteenContiguousSlotsWithTyAtBothEnds() {
        let slots = HourSlot.day
        #expect(slots.count == 13)
        #expect(slots.first == HourSlot(branch: .ty, startHour: 0, endHour: 1))
        #expect(slots.last == HourSlot(branch: .ty, startHour: 23, endHour: 24))
        for (earlier, later) in zip(slots, slots.dropFirst()) {
            #expect(earlier.endHour == later.startHour)
        }
        #expect(Set(slots.map(\.branch)).count == 12)
    }

    @Test func clockHoursFallInTheirDoubleHour() {
        #expect(HourSlot.at(hour: 0).branch == .ty)
        #expect(HourSlot.at(hour: 1).branch == .suu)
        #expect(HourSlot.at(hour: 2).branch == .suu)
        #expect(HourSlot.at(hour: 11).branch == .ngo)
        #expect(HourSlot.at(hour: 22).branch == .hoi)
        #expect(HourSlot.at(hour: 23) == HourSlot(branch: .ty, startHour: 23, endHour: 24))
    }

    @Test func hourRangesAreTheOnesTheCalendarPrints() {
        #expect(Branch.ty.hourRange == "23–01")
        #expect(Branch.suu.hourRange == "01–03")
        #expect(Branch.hoi.hourRange == "21–23")
    }

    // MARK: Day

    @Test func theSnapshotAgreesWithTheEngine() throws {
        let date = SolarDate(year: 2026, month: 11, day: 14)
        let day = try DaySnapshot(date, tzOffset: 7)
        let info = try engine.day(date)
        let almanac = try engine.almanac(date)
        #expect(day.lunar == info.lunar)
        #expect(day.dayCanChi == info.dayCanChi)
        #expect(day.yearCanChi == info.yearCanChi)
        #expect(day.monthCanChi == info.monthCanChi)
        #expect(day.solarTerm == info.solarTerm)
        #expect(day.truc == almanac.truc)
        #expect(day.mansion == almanac.mansion)
        #expect(day.hoangDaoHours == almanac.hoangDaoHours)
        #expect(day.hoangDaoHours.count == 6)
        #expect(day.isHoangDao == pathGodIsHoangDao(value: almanac.pathGod))
    }

    @Test func theMomentReadsTheDateOnVietnamsClock() throws {
        // 23:30 on 13 November in UTC is 06:30 on the 14th in Vietnam.
        let instant = try #require(ISO8601DateFormatter().date(from: "2026-11-13T23:30:00Z"))
        let moment = try DayMoment(at: instant, tzOffset: 7)
        #expect(moment.day.solar == SolarDate(year: 2026, month: 11, day: 14))
        #expect(moment.hour == 6 && moment.minute == 30)
    }

    @Test func theUpcomingHourIsTheCurrentOneWhenItIsHoangDao() throws {
        let day = try DaySnapshot(SolarDate(year: 2026, month: 11, day: 14), tzOffset: 7)
        for slot in HourSlot.day where day.hoangDaoHours.contains(slot.branch) {
            let upcoming = try #require(try DayMoment.upcoming(today: day, hour: slot.startHour, tzOffset: 7))
            #expect(upcoming.isNow)
            #expect(upcoming.branch == slot.branch)
            #expect(upcoming.startHour == slot.startHour)
        }
    }

    @Test func otherwiseItIsTheNextHoangDaoSlot() throws {
        let day = try DaySnapshot(SolarDate(year: 2026, month: 11, day: 14), tzOffset: 7)
        for slot in HourSlot.day where !day.hoangDaoHours.contains(slot.branch) {
            let upcoming = try #require(try DayMoment.upcoming(today: day, hour: slot.startHour, tzOffset: 7))
            #expect(!upcoming.isNow)
            let expected = HourSlot.day.first { $0.startHour > slot.startHour && day.hoangDaoHours.contains($0.branch) }
            if let expected {
                #expect(upcoming.startHour == expected.startHour)
                #expect(!upcoming.isTomorrow)
            } else {
                let tomorrow = try DaySnapshot(SolarDate(year: 2026, month: 11, day: 15), tzOffset: 7)
                #expect(upcoming.isTomorrow)
                #expect(tomorrow.hoangDaoHours.contains(upcoming.branch))
                #expect(upcoming.startHour == HourSlot.day.first { tomorrow.hoangDaoHours.contains($0.branch) }?.startHour)
            }
        }
    }

    @Test func aLateHourWithNoHoangDaoLeftLooksAtTomorrow() throws {
        // Find a day of November 2026 whose 23:00 Tý is Hắc Đạo: the next good hour is tomorrow's.
        var found = false
        for dayOfMonth in 1...29 {
            let day = try DaySnapshot(SolarDate(year: 2026, month: 11, day: UInt8(dayOfMonth)), tzOffset: 7)
            guard !day.hoangDaoHours.contains(.ty) else { continue }
            let upcoming = try #require(try DayMoment.upcoming(today: day, hour: 23, tzOffset: 7))
            #expect(upcoming.isTomorrow)
            #expect(upcoming.text.hasSuffix("(mai)"))
            found = true
        }
        #expect(found)
    }

    // MARK: Timeline

    @Test func theTimelineTurnsAtEachDoubleHourAndAtMidnight() throws {
        let now = try vietnam(10, 30)
        let dates = DayTimeline.dates(after: now, tzOffset: 7)
        #expect(dates.first == now)
        let hours = dates.dropFirst().map { Calendar.zone(7).component(.hour, from: $0) }
        #expect(hours == [11, 13, 15, 17, 19, 21, 23, 0])
        #expect(dates.last == (try vietnam(0, day: 15)))
        #expect(zip(dates, dates.dropFirst()).allSatisfy { $0 < $1 })
    }

    @Test func theMidnightEntryShowsTheNextDay() throws {
        let midnight = try #require(DayTimeline.dates(after: try vietnam(22), tzOffset: 7).last)
        let moment = try DayMoment(at: midnight, tzOffset: 7)
        #expect(moment.day.solar == SolarDate(year: 2026, month: 11, day: 15))
        #expect(moment.hour == 0)
    }

    @Test func dayArithmeticCrossesMonthsAndYears() {
        #expect(SolarDate(year: 2026, month: 12, day: 31).adding(days: 1) == SolarDate(year: 2027, month: 1, day: 1))
        #expect(SolarDate(year: 2024, month: 2, day: 28).adding(days: 1) == SolarDate(year: 2024, month: 2, day: 29))
    }

    // MARK: Events

    /// Notes as the widget reads them: one today, one tomorrow, a death anniversary on 15/10.
    static func sampleEvents(from today: SolarDate) -> [NoteOccurrence] {
        let notes = [
            CalendarNote(title: "Giỗ Ông Nội", calendarType: .lunar, day: 15, month: 10, year: 2020),
            CalendarNote(title: "Họp lớp", calendarType: .solar, day: Int(today.day), month: Int(today.month), year: Int(today.year), isYearlyRecurring: false, category: .important),
            CalendarNote(title: "Sinh nhật Lan", calendarType: .solar, day: Int(today.day) + 1, month: Int(today.month), year: 1995, category: .birthday),
        ]
        return NoteCalendar.upcoming(notes, from: today, days: 30, tzOffset: 7)
    }

    @Test func theWidgetListsTodayThenTheEventsAhead() throws {
        let events = Self.sampleEvents(from: SolarDate(year: 2026, month: 11, day: 14))
        #expect(events.map(\.note.title) == ["Họp lớp", "Sinh nhật Lan", "Giỗ Ông Nội"])
        #expect(events.map(\.countdown) == ["Hôm nay", "Ngày mai", "Còn 9 ngày"])
        #expect(events[2].date == SolarDate(year: 2026, month: 11, day: 23))
    }

    @Test func theLockScreenPrefersAnEventOfTodayOrTomorrow() throws {
        let today = SolarDate(year: 2026, month: 11, day: 14)
        let events = Self.sampleEvents(from: today)
        #expect(RectangularDayView.reminder(in: events)?.note.title == "Họp lớp")
        #expect(RectangularDayView.reminder(in: Array(events.dropFirst()))?.note.title == "Sinh nhật Lan")
        #expect(RectangularDayView.reminder(in: Array(events.dropFirst(2))) == nil)
        #expect(RectangularDayView.reminder(in: []) == nil)
    }

    @Test func theWidgetReadsTheNotesTheAppSaved() throws {
        let suite = "ChronosTests.widget.\(UUID().uuidString)"
        let defaults = try #require(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        #expect(NoteStorage.load(from: defaults).isEmpty)
        let note = CalendarNote(title: "Giỗ Ông Nội", calendarType: .lunar, day: 15, month: 10, year: 2020)
        NoteStorage.save([note], to: defaults)
        let read = NoteStorage.load(from: try #require(UserDefaults(suiteName: suite)))
        #expect(read == [note])
        #expect(NoteCalendar.upcoming(read, from: SolarDate(year: 2026, month: 11, day: 14), days: 30, tzOffset: 7).first?.daysLeft == 9)
    }

    // MARK: Views

    @MainActor
    @Test(arguments: [ColorScheme.light, .dark])
    func everyWidgetViewRendersAtItsFamilySize(scheme: ColorScheme) throws {
        let moment = try DayMoment(at: try vietnam(9, 15), tzOffset: 7)
        let events = Self.sampleEvents(from: moment.day.solar)
        let views: [(String, AnyView, CGSize)] = [
            ("small", AnyView(SmallDayView(moment: moment)), CGSize(width: 170, height: 170)),
            ("medium", AnyView(MediumDayView(moment: moment, events: events)), CGSize(width: 364, height: 170)),
            ("large", AnyView(LargeDayView(moment: moment, events: events)), CGSize(width: 364, height: 382)),
            ("large-empty", AnyView(LargeDayView(moment: moment)), CGSize(width: 364, height: 382)),
            ("rectangular", AnyView(RectangularDayView(moment: moment, events: events)), CGSize(width: 172, height: 76)),
        ]
        for (name, view, size) in views {
            let framed = view
                .padding(name == "rectangular" ? 4 : 16)
                .frame(width: size.width, height: size.height)
                .background(Theme.paper)
                .environment(\.colorScheme, scheme)
            let renderer = ImageRenderer(content: framed)
            renderer.scale = 3
            let image = try #require(renderer.uiImage, "\(name)")
            #expect(image.size == size, "\(name)")
        }
    }
}
