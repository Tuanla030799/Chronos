import Foundation
import Testing
@testable import Chronos

@Suite("Profile form: live conversion, leap months, birthplaces")
struct ProfileFormTests {
    let engine = EngineService()

    private func person(_ birth: BirthDate, longitude: Double? = nil) -> Profile {
        Profile(name: "Thử", gender: .male, birth: birth, longitude: longitude)
    }

    @Test func aSolarBirthShowsItsLunarDateAndCanChi() throws {
        let preview = try engine.birthPreview(for: .sample)
        #expect(preview.counterpart == "Âm lịch: 21 tháng 4 năm 1990")
        #expect(preview.canChi == "Năm Canh Ngọ, Tháng Tân Tỵ, Ngày Canh Thìn, Giờ Tân Tỵ")
        #expect(preview.solarTime == nil)
    }

    @Test func aLunarBirthShowsItsSolarDateAndTheSameCanChi() throws {
        let lunar = person(BirthDate(kind: .lunar, year: 1990, month: 4, day: 21, hour: 9, minute: 30))
        let preview = try engine.birthPreview(for: lunar)
        #expect(preview.counterpart == "Dương lịch: Thứ Ba, 15/05/1990")
        #expect(preview.canChi == (try engine.birthPreview(for: .sample).canChi))
    }

    @Test func thePreviewFollowsEveryChangeOfTheFields() throws {
        var profile = Profile.sample
        let before = try engine.birthPreview(for: profile)
        profile.birth.day = 16
        let nextDay = try engine.birthPreview(for: profile)
        #expect(nextDay.counterpart == "Âm lịch: 22 tháng 4 năm 1990")
        #expect(nextDay.canChi != before.canChi)
        profile.birth.hour = 23
        #expect(try engine.birthPreview(for: profile).canChi.contains("Giờ") )
        #expect(try engine.birthPreview(for: profile).canChi != nextDay.canChi)
    }

    @Test func aLeapMonthIsNamedInThePreview() throws {
        // 31 March 2023 is day 10 of the leap 2nd month of Quý Mão.
        let leap = person(BirthDate(kind: .lunar, year: 2023, month: 2, day: 10, isLeapMonth: true, hour: 8, minute: 0))
        let asSolar = try engine.birthPreview(for: leap)
        let solar = try engine.birth(of: leap)
        let back = person(BirthDate(kind: .solar, year: Int(solar.year), month: Int(solar.month), day: Int(solar.day), hour: 8, minute: 0))
        #expect(asSolar.counterpart.hasPrefix("Dương lịch:"))
        #expect(try engine.birthPreview(for: back).counterpart == "Âm lịch: 10 tháng 2 nhuận năm 2023")
    }

    @Test func aLongitudeAddsTheTrueSolarTime() throws {
        let preview = try engine.birthPreview(for: person(Profile.sample.birth, longitude: 105.85))
        let line = try #require(preview.solarTime)
        #expect(line.hasPrefix("Giờ Mặt Trời thực: 09:"))
        #expect(line.hasSuffix("ngày 15/05"))
    }

    @Test func anImpossibleDateHasNoPreview() {
        let missing = person(BirthDate(kind: .lunar, year: 2024, month: 2, day: 10, isLeapMonth: true, hour: 8, minute: 0))
        #expect(throws: (any Error).self) { try engine.birthPreview(for: missing) }
    }

    @Test func theEngineKnowsTheLeapMonthAndLengthOfALunarYear() throws {
        #expect(engine.leapMonth(ofLunarYear: 2023, tzOffset: 7) == 2)
        #expect(engine.leapMonth(ofLunarYear: 2024, tzOffset: 7) == nil)
        #expect(engine.leapMonth(ofLunarYear: 2025, tzOffset: 7) == 6)
        #expect(engine.leapMonth(ofLunarYear: 99_999, tzOffset: 7) == nil)
        let regular = try #require(engine.lunarMonthLength(year: 2023, month: 2, isLeap: false, tzOffset: 7))
        let leap = try #require(engine.lunarMonthLength(year: 2023, month: 2, isLeap: true, tzOffset: 7))
        #expect([29, 30].contains(regular) && [29, 30].contains(leap))
        #expect(engine.lunarMonthLength(year: 2024, month: 2, isLeap: true, tzOffset: 7) == nil)
    }

    @Test func aLunarBirthIsFittedToItsYear() throws {
        // Moving a leap birth to a year without that leap month turns the switch off.
        var birth = BirthDate(kind: .lunar, year: 2023, month: 2, day: 10, isLeapMonth: true, hour: 8, minute: 0)
        #expect(engine.fitted(birth, tzOffset: 7) == birth)
        birth.year = 2024
        #expect(!engine.fitted(birth, tzOffset: 7).isLeapMonth)
        birth.year = 2023
        birth.month = 3
        #expect(!engine.fitted(birth, tzOffset: 7).isLeapMonth)

        // Day 30 does not outlive a month of 29 days.
        let months = try lunarYear(year: 2026, tzOffset: 7).months
        let short = try #require(months.first { !$0.isLeap && $0.length == 29 })
        let thirty = BirthDate(kind: .lunar, year: 2026, month: Int(short.month), day: 30, hour: 8, minute: 0)
        #expect(engine.fitted(thirty, tzOffset: 7).day == 29)
        #expect(engine.fitted(Profile.sample.birth, tzOffset: 7) == Profile.sample.birth)
    }

    @Test func theLeapNoteSaysWhyTheSwitchIsOnOfferOrNot() {
        #expect(ProfileFormView.leapNote(year: 2024, month: 2, leap: nil) == "Năm 2024 âm lịch không có tháng nhuận.")
        #expect(ProfileFormView.leapNote(year: 2023, month: 2, leap: 2) == "Năm 2023 nhuận tháng 2: bật nếu sinh vào tháng 2 nhuận.")
        #expect(ProfileFormView.leapNote(year: 2023, month: 5, leap: 2) == "Năm 2023 nhuận tháng 2, không nhuận tháng 5.")
    }

    @Test func theCommonBirthplacesCarryTheGivenLongitudes() {
        let places = Dictionary(uniqueKeysWithValues: Birthplace.common.map { ($0.name, $0.longitude) })
        #expect(places == ["Hà Nội": 105.85, "TP. Hồ Chí Minh": 106.66, "Đà Nẵng": 108.20, "Hải Phòng": 106.68, "Cần Thơ": 105.78])
    }

    // MARK: Glossary

    @Test func theGlossaryListsTheTwoGroupsFromTheEngine() {
        let topic = Glossary.kuaGroups
        #expect(topic.items[0].text == "Gồm các quái Khảm, Chấn, Tốn, Ly.")
        #expect(topic.items[1].text == "Gồm các quái Khôn, Càn, Đoài, Cấn.")
    }

    @Test func goodDirectionsBelongToTheGroupOfTheTrigramAsTheGlossarySays() {
        for trigram in Trigram.allCases {
            for item in eightMansionsChart(trigram: trigram) {
                let sameGroup = trigramGroup(value: directionTrigram(direction: item.direction)) == trigramGroup(value: trigram)
                #expect(sameGroup == (eightMansionsStarLuck(value: item.star) == .cat), "\(trigram) \(item.direction)")
            }
        }
    }

    @Test func onlyAStarOutOfItsPeriodReadsItsInauspiciousText() {
        #expect(nineStarPhaseIsAuspicious(value: .duongVan))
        #expect(nineStarPhaseIsAuspicious(value: .sinhKhi))
        #expect(nineStarPhaseIsAuspicious(value: .thoaiKhi))
        #expect(!nineStarPhaseIsAuspicious(value: .thatVan))
        #expect(nineStarPhase(star: .cuuTu, period: .cuuTu) == .duongVan)
        #expect(nineStarPhase(star: .nhatBach, period: .cuuTu) == .sinhKhi)
        #expect(nineStarPhase(star: .batBach, period: .cuuTu) == .thoaiKhi)
    }

    @Test func theSymbolLegendNamesTheFiveBrightnessMarksOfTheEngine() {
        let terms = Glossary.tuviSymbols.items.map(\.term)
        for level in StarBrightness.allCases {
            let mark = "(\(starBrightnessMark(value: level)))"
            #expect(terms.contains { $0.hasPrefix(mark) && $0.contains(starBrightnessName(value: level)) }, "\(mark)")
        }
        for mark in ["<THÂN>", "ĐH", "TH", "TUẦN, TRIỆT"] {
            #expect(terms.contains(mark))
        }
    }
}
