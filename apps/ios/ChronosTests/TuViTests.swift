import Foundation
import Testing
@testable import Chronos

@Suite("Tử Vi tab")
struct TuViTests {
    let services: AppServices
    let chart: TuViChart
    var content: ContentRepository { services.content }

    init() throws {
        services = try AppServices.bundled()
        chart = try services.engine.tuviChart(for: .sample)
    }

    // MARK: Board

    @Test func theBoardPlacesTheTwelveBranchesOnItsRim() {
        let cells = TuViBoardLayout.cells
        #expect(Set(cells.map(\.branch)).count == 12)
        let rows = (0..<4).map { row in cells.filter { $0.row == row }.sorted { $0.column < $1.column }.map(\.branch) }
        #expect(rows[0] == [.ti, .ngo, .mui, .than])
        #expect(rows[1] == [.thin, .dau])
        #expect(rows[2] == [.mao, .tuat])
        #expect(rows[3] == [.dan, .suu, .ty, .hoi])
        for cell in cells {
            #expect(cell.row == 0 || cell.row == 3 || cell.column == 0 || cell.column == 3, "\(cell.branch)")
        }
    }

    @Test func aPalaceMeetsTwoTrinesAndOneOpposite() {
        let ty = PalaceRelations(of: .ty)
        #expect(Set(ty.trine) == [.than, .thin])
        #expect(ty.opposite == .ngo)
        #expect(ty.role(of: .than) == .trine)
        #expect(ty.role(of: .ngo) == .opposite)
        #expect(ty.role(of: .suu) == nil)
        #expect(ty.role(of: .ty) == nil)
        for branch in Branch.allCases {
            let relations = PalaceRelations(of: branch)
            #expect(relations.trine.count == 2)
            #expect(!relations.trine.contains(branch))
            #expect(relations.opposite != branch)
            #expect(PalaceRelations(of: relations.opposite).opposite == branch)
            for other in relations.trine {
                #expect(Set(PalaceRelations(of: other).trine) == Set(relations.trine.filter { $0 != other } + [branch]))
            }
        }
    }

    // MARK: Stars

    @Test func everyStarOfAPalaceFallsInExactlyOneGroup() {
        for palace in chart.palaces {
            let groups = StarGroups(palace)
            let all = (groups.main + groups.good + groups.neutral + groups.bad).map(\.star)
            #expect(all.count == palace.mainStars.count + palace.minorStars.count)
            #expect(Set(all) == Set((palace.mainStars + palace.minorStars).map(\.star)))
            #expect(groups.main == palace.mainStars)
            #expect(groups.good.allSatisfy { tuviStarNature(value: $0.star) == .cat })
            #expect(groups.bad.allSatisfy { tuviStarNature(value: $0.star) == .hung })
            #expect(groups.neutral.allSatisfy { tuviStarNature(value: $0.star) == .trung })
            #expect(groups.transformed.allSatisfy { $0.transformation != nil })
        }
    }

    @Test func theFourTransformationsOfACanhYearAreOnTheBoard() {
        // Canh: Thái Dương Hóa Lộc, Vũ Khúc Hóa Quyền, Thái Âm Hóa Khoa, Thiên Đồng Hóa Kỵ.
        let transformed = chart.palaces.flatMap { StarGroups($0).transformed }
        let byHoa = Dictionary(uniqueKeysWithValues: transformed.compactMap { placed in placed.transformation.map { ($0, placed.star) } })
        #expect(byHoa[.hoaLoc] == .thaiDuong)
        #expect(byHoa[.hoaQuyen] == .vuKhuc)
        #expect(byHoa[.hoaKhoa] == .thaiAm)
        #expect(byHoa[.hoaKy] == .thienDong)
    }

    @Test func aStarLabelCarriesItsBrightnessMark() {
        #expect(TuViPlacedStar(star: .phaQuan, brightness: .mieu, transformation: nil).label == "Phá Quân (M)")
        #expect(TuViPlacedStar(star: .locTon, brightness: nil, transformation: nil).label == "Lộc Tồn")
    }

    @Test func everyStarOfTheChartHasItsTextAndOnlyChinhTinhHaveAnElement() throws {
        for palace in chart.palaces {
            for placed in palace.mainStars + palace.minorStars {
                #expect(try content.entry(tuviStarKey(star: placed.star)) != nil, "\(placed.star)")
                let isMain = tuviStarCategory(value: placed.star) == .chinhTinh
                #expect((tuviStarElement(value: placed.star) != nil) == isMain, "\(placed.star)")
            }
            for placed in palace.mainStars + palace.minorStars {
                if let hoa = placed.transformation {
                    #expect(try content.entry(tuviTransformationKey(transformation: hoa)) != nil)
                }
            }
        }
    }

    // MARK: Year

    @Test func theSampleIsThirtySevenIn2026WithItsHanOnTheBoard() throws {
        let year = TuViYearReading(chart: chart, lunarYear: 2026)
        #expect(year.age == 37)
        let daiHan = try #require(year.daiHan.flatMap { chart.palace(at: $0) })
        #expect((Int(daiHan.daiHanStartAge)...Int(daiHan.daiHanStartAge) + 9).contains(37))
        #expect(year.tieuHan == tuviTieuHanPalace(chart: chart, age: 37)?.branch)
        // A Ngọ year person starts the Tiểu Hạn at Thìn, so each Ngọ year comes back there.
        #expect(chart.palace(at: try #require(year.tieuHan))?.tieuHanYearBranch == .ngo)
        #expect(year.tieuHan == .thin)
    }

    @Test func aYearBeforeBirthHasNoHan() {
        let year = TuViYearReading(chart: chart, lunarYear: 1980)
        #expect(year.age < 1)
        #expect(year.daiHan == nil)
        #expect(year.tieuHan == nil)
    }

    @Test func theTabSettingsMapToTheEngineConventions() {
        #expect(TuViSettings().leapMonthRule == .splitAt15th)
        #expect(TuViSettings().ratHour == .splitAtMidnight)
        let other = TuViSettings(leapMonthAsCurrent: true, newDayAtRatHour: true)
        #expect(other.leapMonthRule == .sameAsCurrentMonth)
        #expect(other.ratHour == .newDayAtRatHour)
    }

    @Test func theLeapMonthRuleChangesAChartBornLateInALeapMonth() throws {
        // 2023 has a leap 2nd month; day 20 of it reads as month 2 or as month 3.
        let person = Profile(
            name: "Nhuận",
            gender: .female,
            birth: BirthDate(kind: .lunar, year: 2023, month: 2, day: 20, isLeapMonth: true, hour: 10, minute: 0)
        )
        let split = try services.engine.tuviChart(for: person, leapMonthRule: .splitAt15th)
        let same = try services.engine.tuviChart(for: person, leapMonthRule: .sameAsCurrentMonth)
        #expect(split.effectiveMonth == 3)
        #expect(same.effectiveMonth == 2)
        #expect(split.menhBranch != same.menhBranch)
    }

    // MARK: Reading

    @Test func choosingMenhReadsItsOverviewAndPassagesWithCorrections() throws {
        let reading = try content.tuviPalace(chart: chart, branch: chart.menhBranch)
        #expect(reading.entries.first?.key == tuviPalaceKey(role: .menh))
        #expect(!reading.primary.isEmpty)
        #expect(reading.passages.allSatisfy { $0.correction.map { !$0.isEmpty } ?? true })
        #expect(reading.passages.contains { $0.correction != nil })
        #expect(reading.more.allSatisfy { group in group.passages.allSatisfy { $0.tier == .more } })
    }

    @Test func everyPalaceOfTheSampleCanBeRead() throws {
        for palace in chart.palaces {
            let reading = try content.tuviPalace(chart: chart, branch: palace.branch)
            #expect(Set(reading.passages.map(\.id)).count == reading.passages.count)
        }
    }
}
