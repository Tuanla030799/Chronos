import Foundation
import Testing
@testable import Chronos

@Suite("EngineService")
struct EngineServiceTests {
    let engine = EngineService()

    private func profile(_ birth: BirthDate, gender: ProfileGender = .male) -> Profile {
        Profile(name: "Thử", gender: gender, birth: birth)
    }

    @Test func todayIsTheCivilDateInVietnam() throws {
        let instant = try #require(ISO8601DateFormatter().date(from: "2026-11-10T18:30:00Z"))
        #expect(engine.today(now: instant) == SolarDate(year: 2026, month: 11, day: 11))
        let now = engine.now(instant)
        #expect(now.hour == 1 && now.minute == 30)
        #expect(EngineService(tzOffset: 0).today(now: instant) == SolarDate(year: 2026, month: 11, day: 10))
    }

    @Test func aLunarBirthResolvesThroughLunarToSolar() throws {
        let tet = profile(BirthDate(kind: .lunar, year: 2024, month: 1, day: 1, hour: 8, minute: 30))
        let time = try engine.birth(of: tet)
        #expect(time == SolarDateTime(year: 2024, month: 2, day: 10, hour: 8, minute: 30, second: 0))
        #expect(try engine.lunarBirthDate(of: tet) == LunarDate(year: 2024, month: 1, day: 1, isLeapMonth: false))
    }

    @Test func aSolarBirthBeforeTetBelongsToThePreviousLunarYear() throws {
        let person = profile(BirthDate(kind: .solar, year: 2024, month: 2, day: 9, hour: 12, minute: 0))
        #expect(try engine.lunarBirthDate(of: person).year == 2023)
    }

    @Test func aLeapMonthTheYearLacksIsRefused() {
        let wrong = profile(BirthDate(kind: .lunar, year: 2024, month: 5, day: 1, isLeapMonth: true, hour: 8, minute: 0))
        #expect(throws: AstroError.self) { try engine.birth(of: wrong) }
    }

    @Test func anImpossibleClockTimeIsRefused() {
        let wrong = profile(BirthDate(kind: .solar, year: 2000, month: 1, day: 1, hour: 25, minute: 0))
        #expect(throws: AstroError.self) { try engine.bazi(for: wrong) }
    }

    @Test func aProfileDrawsTheSameTuViChartAsItsLunarDate() throws {
        let person = profile(BirthDate(kind: .lunar, year: 1931, month: 11, day: 30, hour: 4, minute: 0))
        let fromProfile = try engine.tuviChart(for: person)
        let reference = try ContentRepositoryTests.referenceChart()
        #expect(fromProfile.menhBranch == reference.menhBranch)
        #expect(fromProfile.thanBranch == reference.thanBranch)
        #expect(fromProfile.palaces == reference.palaces)
    }

    @Test func longitudeTurnsOnTrueSolarTime() throws {
        var person = profile(BirthDate(kind: .solar, year: 1990, month: 6, day: 15, hour: 23, minute: 10))
        let clock = try engine.bazi(for: person)
        person.longitude = 106.7
        let solar = try engine.bazi(for: person)
        #expect(clock.pillars.effectiveTime != solar.pillars.effectiveTime)
    }

    @Test func kuaAndAgeRestrictionsUseTheLunarBirthYear() throws {
        let person = profile(BirthDate(kind: .solar, year: 1990, month: 1, day: 20, hour: 12, minute: 0), gender: .female)
        let lunarYear = try engine.lunarBirthDate(of: person).year
        #expect(lunarYear == 1989)
        #expect(try engine.kua(for: person) == kuaInfo(birthLunarYear: 1989, gender: .female))
        #expect(
            try engine.ageRestrictions(for: person, targetLunarYear: 2026)
                == ageRestrictions(birthLunarYear: 1989, targetLunarYear: 2026)
        )
    }
}
