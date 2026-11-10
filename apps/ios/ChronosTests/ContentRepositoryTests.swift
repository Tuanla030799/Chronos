import Foundation
import Testing
@testable import Chronos

private final class TestBundleToken {}

@Suite("ContentRepository on the bundled chronos_content.sqlite")
struct ContentRepositoryTests {
    let content: ContentRepository
    let engine = EngineService()

    init() throws {
        content = try ContentRepository()
    }

    @Test func opensTheReleaseFileWithItsMetadata() {
        #expect(content.metadata.schemaVersion == ContentRepository.schemaVersion)
        #expect(content.metadata.contentHash.hasPrefix("sha256:"))
        #expect(!content.metadata.engineVersion.isEmpty)
    }

    @Test func aBundleWithoutTheDatabaseFailsClearly() {
        #expect(throws: ContentRepositoryError.missingResource("chronos_content.sqlite")) {
            try ContentRepository(bundle: Bundle(for: TestBundleToken.self))
        }
    }

    @Test func readsAnEntryWithItsExtraFields() throws {
        let kien = try #require(try content.entry("almanac.truc/Kien"))
        #expect(kien.title == "Kiến")
        #expect(kien.collection == "almanac.truc")
        #expect(kien.extra.goodFor.map(\.activity).contains("XuatHanh"))
        #expect(content.activityLabel("XuatHanh") == "Xuất hành")
        #expect(try content.entry("almanac.truc/KhongCo") == nil)
    }

    @Test func everyDayOf2026HasTextForAllItsAlmanacKeys() throws {
        var missing = Set<String>()
        var date = SolarDate(year: 2026, month: 1, day: 1)
        for _ in 0..<365 {
            let day = try engine.almanac(date)
            for key in almanacDayKeys(day: day) where try content.entry(key) == nil {
                missing.insert(key)
            }
            date = next(date)
        }
        #expect(missing.isEmpty, "\(missing.sorted())")
    }

    @Test func everyAgeRestrictionHasText() throws {
        var missing = Set<String>()
        for birth in Int32(1940)...2015 {
            let restrictions = ageRestrictions(birthLunarYear: birth, targetLunarYear: 2026)
            for key in ageRestrictionKeys(restrictions: restrictions) where try content.entry(key) == nil {
                missing.insert(key)
            }
        }
        #expect(missing.isEmpty, "\(missing.sorted())")
    }

    @Test func onlyTheCanChiGapsOfD18LackBaziText() throws {
        let profile = Profile(
            name: "Bát Tự",
            gender: .female,
            birth: BirthDate(kind: .solar, year: 1990, month: 6, day: 15, hour: 9, minute: 30)
        )
        let analysis = try engine.bazi(for: profile)
        let keys = baziKeys(analysis: analysis)
        let found = try content.bazi(analysis).map(\.key)
        let missing = keys.filter { !found.contains($0) }
        #expect(missing.allSatisfy { $0.hasPrefix("bazi.can_chi/") })
        #expect(found.first == "bazi.day_master/" + "\(analysis.day.canChi.stem)".capitalizedFirst)
    }

    @Test func everyFengShuiKeyHasText() throws {
        for star in EightMansionsStar.allCases {
            #expect(try content.eightStar(star) != nil)
            for usage in EightUse.allCases {
                #expect(try content.eightStarUse(star, usage) != nil)
            }
        }
        var trigrams = Set<Trigram>()
        for year in Int32(1960)...1980 {
            for gender in Gender.allCases {
                let kua = kuaInfo(birthLunarYear: year, gender: gender)
                trigrams.insert(kua.trigram)
                #expect(try content.kua(kua) != nil)
            }
        }
        #expect(trigrams.count == 8)
    }

    @Test func aStoveInvertsTheLuckOfItsStar() throws {
        let stove = try #require(try content.eightStarUse(.sinhKhi, .bep))
        #expect(stove.extra.verdict == .hung)
        #expect(stove.effectiveLuck == .hung)
    }

    @Test(arguments: NineStar.allCases)
    func aNineStarReadsTheTextOfItsPhase(star: NineStar) throws {
        for period in NineStar.allCases {
            let reading = try #require(try content.nineStar(star, period: period))
            let expected = reading.isAuspicious ? reading.entry.extra.auspiciousMeaning : reading.entry.extra.inauspiciousMeaning
            #expect(reading.meaning != nil)
            #expect(reading.meaning == expected)
        }
    }

    // MARK: Tử Vi

    /// Stable ids that `scan::select` of astro-data picks for this chart: the Rust reference.
    static let rustSelection: [TuViPalaceRole: [String]] = [
        .menh: [
            "menh/thuan-nghich/02", "menh/sinh-khac/03", "menh/chung/05", "menh/menh-than/01",
            "menh/menh-than/02", "menh/menh-than/03", "menh/menh-than/04", "menh/tuan-triet/01",
            "menh/thien-co/02", "menh/thien-co/03", "menh/thien-co/04", "menh/thien-co/09",
            "menh/thien-co/13", "menh/thien-co/14", "menh/thien-co/21", "menh/thien-co/22",
            "menh/thien-co/25", "menh/thien-co/26", "menh/thien-co/27", "menh/thien-co/28",
            "menh/thien-co/31", "menh/thien-co/32", "menh/thien-luong/06", "menh/thien-luong/08",
            "menh/thien-luong/10", "menh/thien-luong/16", "menh/thien-luong/25", "menh/ta-huu/23",
            "menh/ta-huu/24", "menh/ta-huu/25", "menh/kinh-duong/01", "menh/kinh-duong/02",
            "menh/kinh-duong/10", "menh/kinh-duong/16", "menh/kinh-duong/17", "menh/hoa-khoa/06",
        ],
        .quanLoc: [
            "menh/thai-duong/37", "menh/thai-duong/38", "menh/thien-co/31", "menh/cu-mon/31",
            "menh/cu-mon/35", "menh/thien-luong/16", "menh/xuong-khuc/39", "menh/khoi-viet/01",
            "menh/khoi-viet/02", "menh/khoi-viet/03", "menh/khoi-viet/04", "menh/khoi-viet/05",
            "menh/khoi-viet/08", "menh/khoi-viet/09", "menh/khoi-viet/10", "menh/khoi-viet/11",
            "menh/khoi-viet/13", "menh/ta-huu/01", "menh/ta-huu/02", "menh/ta-huu/04",
            "menh/ta-huu/05", "menh/ta-huu/06", "menh/ta-huu/07", "menh/ta-huu/10",
            "menh/ta-huu/14", "menh/ta-huu/15", "menh/hoa-khoa/08", "menh/hoa-khoa/09",
            "menh/hoa-khoa/10", "quan-loc/thai-duong/06", "quan-loc/khoi-viet/01",
            "quan-loc/khoi-viet/02", "quan-loc/ta-huu/01", "quan-loc/khoa-quyen-loc/01",
            "quan-loc/vo-chinh-dieu/01", "quan-loc/vo-chinh-dieu/03",
        ],
        .dienTrach: [
            "dien-trach/khong-kiep/02", "dien-trach/khoc-hu/01", "dien-trach/bo-sao/16",
            "dien-trach/bo-sao/20", "dien-trach/bo-sao/25", "dien-trach/bo-sao/26",
            "dien-trach/vo-chinh-dieu/01",
        ],
        .noBoc: ["no-boc/ket-hop/02", "no-boc/ket-hop/08"],
        .thienDi: ["thien-di/thien-co/03", "thien-di/tuan-triet/01"],
    ]

    static func referenceChart() throws -> TuViChart {
        try tuviChartFromLunar(
            lunar: LunarDate(year: 1931, month: 11, day: 30, isLeapMonth: false),
            hour: .dan,
            gender: .male
        )
    }

    @Test(arguments: Array(rustSelection.keys))
    func palaceSelectionMatchesRust(role: TuViPalaceRole) throws {
        let chart = try Self.referenceChart()
        let palace = try #require(chart.palaces.first { $0.role == role })
        let reading = try content.tuviPalace(chart: chart, branch: palace.branch)
        #expect(reading.passages.map(\.stableID) == Self.rustSelection[role])
        if let overview = try content.entry("tuvi.palace/" + "\(role)".capitalizedFirst) {
            #expect(reading.entries.first == overview)
        }
    }

    @Test func passagesSplitIntoPrimaryAndMoreInBookOrder() throws {
        let chart = try Self.referenceChart()
        for palace in chart.palaces {
            let reading = try content.tuviPalace(chart: chart, branch: palace.branch)
            let primary = reading.primary.flatMap(\.passages)
            let more = reading.more.flatMap(\.passages)
            #expect(primary.count + more.count == reading.passages.count)
            #expect(primary.allSatisfy { $0.tier == .primary })
            #expect(more.allSatisfy { $0.tier == .more && $0.conditionText != nil })
            #expect(reading.passages.map(\.id) == reading.passages.map(\.id).sorted())
            #expect(Set(reading.passages.map(\.id)).count == reading.passages.count)
        }
    }

    @Test func d9HidesDeathAndSexualityOnly() {
        #expect(Sensitivity.death.isHiddenByDefault)
        #expect(Sensitivity.sexuality.isHiddenByDefault)
        #expect(Sensitivity([.death, .legal]).isHiddenByDefault)
        #expect(!Sensitivity([.illness, .legal, .familyLoss]).isHiddenByDefault)
    }

    // MARK: Speed

    @Test func anEntryLookupTakesUnderAMillisecond() throws {
        let keys = try almanacDayKeys(day: engine.almanac(SolarDate(year: 2026, month: 11, day: 10)))
        let rounds = 1000
        let elapsed = try ContinuousClock().measure {
            for index in 0..<rounds {
                _ = try content.entry(keys[index % keys.count])
            }
        }
        let average = elapsed / rounds
        print("entry lookup: \(average) on average")
        #expect(average < .milliseconds(1))
    }

    @Test func aWholeChartReadsQuickly() throws {
        let chart = try Self.referenceChart()
        let elapsed = try ContinuousClock().measure {
            for palace in chart.palaces {
                _ = try content.tuviPalace(chart: chart, branch: palace.branch)
            }
        }
        print("12 palaces: \(elapsed)")
        #expect(elapsed < .milliseconds(250))
    }

    private func next(_ date: SolarDate) -> SolarDate {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .gmt
        let start = calendar.date(from: DateComponents(year: Int(date.year), month: Int(date.month), day: Int(date.day)))!
        let parts = calendar.dateComponents([.year, .month, .day], from: calendar.date(byAdding: .day, value: 1, to: start)!)
        return SolarDate(year: Int32(parts.year!), month: UInt8(parts.month!), day: UInt8(parts.day!))
    }
}

extension String {
    /// `quanLoc` to `QuanLoc`: Swift case names back to the Rust variant names of the keys.
    var capitalizedFirst: String {
        prefix(1).uppercased() + dropFirst()
    }
}
