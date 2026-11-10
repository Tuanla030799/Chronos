import Foundation
import Testing
@testable import Chronos

@Suite("Month grid and age reading")
struct CalendarAndAgeTests {
    let engine = EngineService()

    @Test func november2026StartsOnSundayAfterSixBlanks() throws {
        let page = try MonthPage.build(MonthKey(year: 2026, month: 11), engine: engine)
        #expect(page.days.count == 30)
        #expect(page.leadingBlanks == 6)
        #expect(page.days.first?.solar.weekdayName == "Chủ Nhật")
        let tenth = try #require(page.days.first { $0.solar.day == 10 })
        #expect(tenth.lunar == LunarDate(year: 2026, month: 10, day: 2, isLeapMonth: false))
    }

    @Test func everyCellAgreesWithTheEngine() throws {
        let page = try MonthPage.build(MonthKey(year: 2024, month: 2), engine: engine)
        #expect(page.days.count == 29)
        for day in page.days {
            let almanac = try engine.almanac(day.solar)
            #expect(day.lunar == almanac.lunar)
            #expect(day.isHoangDao == pathGodIsHoangDao(value: almanac.pathGod))
        }
        let tet = try #require(page.days.first { $0.solar.day == 10 })
        #expect(tet.lunarLabel == "1/1")
    }

    @Test func monthKeysWrapAroundTheYear() {
        #expect(MonthKey(year: 2026, month: 12).adding(months: 1) == MonthKey(year: 2027, month: 1))
        #expect(MonthKey(year: 2026, month: 1).adding(months: -1) == MonthKey(year: 2025, month: 12))
        #expect(MonthKey(year: 2026, month: 5).adding(months: -17) == MonthKey(year: 2024, month: 12))
    }

    @Test func weekdayNamesAreVietnamese() {
        #expect(SolarDate(year: 2026, month: 11, day: 10).weekdayName == "Thứ Ba")
    }

    @Test func theSampleProfileIsInItsOwnYearIn2026() throws {
        let reading = try engine.taiSui(for: .sample, targetLunarYear: 2026)
        #expect(reading.birthBranch == .ngo)
        #expect(reading.yearBranch == .ngo)
        #expect(reading.relations == [.truc, .hinh])
        #expect(taiSuiName(value: .truc) == "Trực Thái Tuế (năm tuổi)")
    }

    @Test func thaiTueUsesTheLunarBirthYear() throws {
        let beforeTet = Profile(
            name: "Trước Tết",
            gender: .female,
            birth: BirthDate(kind: .solar, year: 1990, month: 1, day: 20, hour: 12, minute: 0)
        )
        #expect(try engine.taiSui(for: beforeTet, targetLunarYear: 2026).birthBranch == .ti)
        #expect(try engine.yearCanChi(lunarYear: 2026) == CanChi(stem: .binh, branch: .ngo))
    }
}
