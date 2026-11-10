import Foundation
import SwiftUI
import Testing
import UIKit
@testable import Chronos

@Suite("Bát Tự and Phong Thủy tabs")
struct BaziAndFengShuiTests {
    let services: AppServices
    var engine: EngineService { services.engine }
    var content: ContentRepository { services.content }

    init() throws {
        services = try AppServices.bundled()
    }

    private func person(_ year: Int, _ month: Int, _ day: Int, hour: Int = 12, minute: Int = 0) -> Profile {
        Profile(
            name: "Thử",
            gender: .male,
            birth: BirthDate(kind: .solar, year: year, month: month, day: day, hour: hour, minute: minute)
        )
    }

    // MARK: Bát Tự

    @Test func theSampleProfileHasTheFourPillarsOfItsBirth() throws {
        let pillars = try engine.bazi(for: .sample).pillars
        #expect(canChiName(value: pillars.year) == "Canh Ngọ")
        #expect(canChiName(value: pillars.month) == "Tân Tỵ")
        #expect(canChiName(value: pillars.day) == "Canh Thìn")
        #expect(canChiName(value: pillars.hour) == "Tân Tỵ")
    }

    @Test func theYearBoundaryChangesTheYearPillarBetweenTetAndLapXuan() throws {
        // 1 February 1990 is after Tết (27 January) and before Lập Xuân (4 February).
        let between = person(1990, 2, 1)
        let atTerms = try engine.bazi(for: between, boundary: .solarTerms).pillars.year
        let atTet = try engine.bazi(for: between, boundary: .lunarNewYear).pillars.year
        #expect(canChiName(value: atTerms) == "Kỷ Tỵ")
        #expect(canChiName(value: atTet) == "Canh Ngọ")
        #expect(try engine.bazi(for: between).pillars.year == atTerms)
    }

    @Test func theRatHourConventionChangesTheDayPillarAtTwentyThree() throws {
        let late = person(1990, 5, 15, hour: 23, minute: 30)
        let split = try engine.bazi(for: late, ratHour: .splitAtMidnight).pillars.day
        let newDay = try engine.bazi(for: late, ratHour: .newDayAtRatHour).pillars.day
        #expect(split != newDay)
        #expect(try engine.bazi(for: late).pillars.day == split)
    }

    @Test func theTabSettingsMapToTheEngineConventions() {
        #expect(BaziSettings().boundary == .solarTerms)
        #expect(BaziSettings().ratHour == .splitAtMidnight)
        #expect(BaziSettings(lunarNewYearBoundary: true, newDayAtRatHour: true).boundary == .lunarNewYear)
        #expect(BaziSettings(lunarNewYearBoundary: true, newDayAtRatHour: true).ratHour == .newDayAtRatHour)
    }

    @Test func everyTermOfTheTableHasATextEntryFromTheKeysOfTheEngine() throws {
        var missing: [String] = []
        for stem in Stem.allCases where try content.entry(baziDayMasterKey(stem: stem)) == nil {
            missing.append(baziDayMasterKey(stem: stem))
        }
        for god in TenGod.allCases where try content.entry(baziTenGodKey(god: god)) == nil {
            missing.append(baziTenGodKey(god: god))
        }
        for stage in LifeStage.allCases where try content.entry(baziLifeStageKey(stage: stage)) == nil {
            missing.append(baziLifeStageKey(stage: stage))
        }
        for naYin in NaYin.allCases where try content.entry(baziNaYinKey(naYin: naYin)) == nil {
            missing.append(baziNaYinKey(naYin: naYin))
        }
        for star in BaziStar.allCases where try content.entry(baziShenShaKey(star: star)) == nil {
            missing.append(baziShenShaKey(star: star))
        }
        for key in [baziKhongVongKey(), baziKhoiCanhKey()] where try content.entry(key) == nil {
            missing.append(key)
        }
        #expect(missing.isEmpty, "\(missing)")
    }

    @Test func theReadingFindsTheTenGodOfAHiddenStemThatBaziKeysDoNotList() throws {
        let reading = try BaziReading(profile: .sample, settings: BaziSettings(), services: services)
        let listed = Set(reading.entries.map(\.key))
        let hidden = reading.analysis.year.hiddenStems + reading.analysis.month.hiddenStems
            + reading.analysis.day.hiddenStems + reading.analysis.hour.hiddenStems
        for stem in hidden {
            #expect(reading.entry(baziTenGodKey(god: stem.tenGod)) != nil)
        }
        #expect(hidden.contains { !listed.contains(baziTenGodKey(god: $0.tenGod)) })
    }

    @Test func theLuckCycleOfTheCurrentYearIsGiapThanFor1990() throws {
        let daiVan = try engine.bazi(for: .sample).daiVan
        #expect(daiVan.cycles.count == 10)
        #expect(daiVan.cycles.first?.canChi == nil)
        let current = try #require(daiVan.cycles.first { ($0.startYear...$0.endYear).contains(2026) })
        #expect(current.canChi.map { canChiName(value: $0) } == "Giáp Thân")
        #expect(current.startAge == 28 && current.endAge == 37)
    }

    @Test func hiddenStemsAreNamedByPosition() {
        #expect(PillarTableCard.qiName(0) == "Bản khí")
        #expect(PillarTableCard.qiName(1) == "Trung khí")
        #expect(PillarTableCard.qiName(2) == "Dư khí")
    }

    // MARK: Phong Thủy

    @Test func everyStarAndUseOfTheEightMansionsHasText() throws {
        for star in EightMansionsStar.allCases {
            #expect(try content.eightStar(star) != nil, "\(star)")
            for usage in EightUse.allCases {
                #expect(try content.eightStarUse(star, usage) != nil, "\(star) \(usage)")
            }
        }
        #expect(eightUseName(usage: .banLamViec) == "Bàn làm việc")
    }

    @Test func aStoveAtSinhKhiReadsHungWhereTheStarItselfIsCat() throws {
        let star = try #require(try content.eightStar(.sinhKhi))
        let stove = try #require(try content.eightStarUse(.sinhKhi, .bep))
        #expect(star.effectiveLuck == .cat)
        #expect(stove.effectiveLuck == .hung)
    }

    @Test func eachTrigramHasFourGoodAndFourBadDirections() {
        for trigram in Trigram.allCases {
            let chart = eightMansionsChart(trigram: trigram)
            #expect(chart.count == 8)
            #expect(chart.filter { eightMansionsStarLuck(value: $0.star) == .cat }.count == 4)
            #expect(chart.filter { eightMansionsStarLuck(value: $0.star) == .hung }.count == 4)
        }
    }

    @Test func theBoardLayoutMatchesTheRowsOfTheGrid() {
        for center in NineStar.allCases {
            for forward in [true, false] {
                let grid = generateLoshuGrid(centerStar: center, forward: forward)
                let rows = loShuRows(grid: grid)
                for (rowIndex, palaces) in LoShuLayout.rows.enumerated() {
                    for (columnIndex, palace) in palaces.enumerated() {
                        #expect(rows[rowIndex][columnIndex] == loShuStarAt(grid: grid, palace: palace))
                    }
                }
            }
        }
        #expect(Set(LoShuLayout.rows.flatMap { $0 }).count == 9)
    }

    @Test func theLayersFlyTheWayTheSpecSays() throws {
        let stars = try engine.nineStars(at: SolarDateTime(year: 2026, month: 11, day: 11, hour: 0, minute: 21, second: 0))
        #expect(stars.van == .cuuTu)
        #expect(NineStarLayer.year.center(of: stars) == (stars.year, true))
        #expect(NineStarLayer.month.center(of: stars) == (stars.month, true))
        #expect(NineStarLayer.day.center(of: stars) == (stars.day.star, stars.day.forward))
        #expect(NineStarLayer.hour.center(of: stars) == (stars.hour.star, stars.hour.forward))
    }

    @Test func aCellOfTheBoardReadsTheTextOfItsPhase() throws {
        // Cửu Tử rules the period, so Nhất Bạch is in Sinh Khí and reads its auspicious text.
        let reading = try #require(try content.nineStar(.nhatBach, period: .cuuTu))
        #expect(reading.phaseName == nineStarPhaseName(value: nineStarPhase(star: .nhatBach, period: .cuuTu)))
        #expect(reading.isAuspicious)
        #expect(reading.meaning == reading.entry.extra.auspiciousMeaning)
        let lost = try #require(try content.nineStar(.nguHoang, period: .cuuTu))
        #expect(!lost.isAuspicious)
        #expect(lost.meaning == lost.entry.extra.inauspiciousMeaning)
    }

    @Test func aMarriageBetweenTwoTrigramsReadsTheStarBetweenThem() throws {
        let first = kuaInfo(birthLunarYear: 1990, gender: .male)
        let second = kuaInfo(birthLunarYear: 1992, gender: .female)
        let compatibility = kuaCompatibility(first: first, second: second)
        let entries = try content.kuaCompatibility(compatibility)
        #expect(entries.count == 2)
        #expect(entries.first?.key == eightStarKey(star: compatibility.star))
        #expect(entries.last?.key == eightStarUseKey(star: compatibility.star, usage: .ketHon))
    }

    // MARK: Colors

    @Test func everyElementHasItsOwnColor() {
        let elements: [FiveElements] = [.kim, .moc, .thuy, .hoa, .tho]
        for style in [UIUserInterfaceStyle.light, .dark] {
            let traits = UITraitCollection(userInterfaceStyle: style)
            let resolved = elements.map { UIColor(Theme.element($0)).resolvedColor(with: traits).description }
            #expect(Set(resolved).count == 5)
        }
        #expect(stemElement(stem: .giap) == .moc)
        #expect(branchElement(branch: .mui) == .tho)
    }
}
