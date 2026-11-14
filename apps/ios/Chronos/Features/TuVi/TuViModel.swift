import Foundation

/// The two choices a Tử Vi chart depends on, kept across launches.
struct TuViSettings: Equatable {
    var leapMonthAsCurrent = false
    var newDayAtRatHour = false

    var leapMonthRule: LeapMonthRule { leapMonthAsCurrent ? .sameAsCurrentMonth : .splitAt15th }
    var ratHour: RatHourConvention { newDayAtRatHour ? .newDayAtRatHour : .splitAtMidnight }
}

/// The fixed Địa Bàn: Tỵ to Thân across the top, Thìn and Mão down the left, Dậu and Tuất down
/// the right, Dần to Hợi across the bottom; the Thiên Bàn fills the 2×2 middle.
enum TuViBoardLayout {
    struct Cell: Hashable {
        let branch: Branch
        let row: Int
        let column: Int
    }

    static let cells: [Cell] = [
        Cell(branch: .ti, row: 0, column: 0),
        Cell(branch: .ngo, row: 0, column: 1),
        Cell(branch: .mui, row: 0, column: 2),
        Cell(branch: .than, row: 0, column: 3),
        Cell(branch: .thin, row: 1, column: 0),
        Cell(branch: .dau, row: 1, column: 3),
        Cell(branch: .mao, row: 2, column: 0),
        Cell(branch: .tuat, row: 2, column: 3),
        Cell(branch: .dan, row: 3, column: 0),
        Cell(branch: .suu, row: 3, column: 1),
        Cell(branch: .ty, row: 3, column: 2),
        Cell(branch: .hoi, row: 3, column: 3),
    ]

    static func cell(_ branch: Branch) -> Cell? {
        cells.first { $0.branch == branch }
    }
}

/// The palaces a palace meets: the other two of its Tam Hợp and the one in Xung Chiếu.
struct PalaceRelations: Equatable {
    let trine: [Branch]
    let opposite: Branch

    init(of branch: Branch) {
        trine = branchTrine(branch: branch).filter { $0 != branch }
        opposite = branchClash(branch: branch)
    }

    func role(of branch: Branch) -> Role? {
        if trine.contains(branch) { return .trine }
        if branch == opposite { return .opposite }
        return nil
    }

    enum Role {
        case trine, opposite
    }
}

/// The stars of a palace in the four groups the detail shows: Chính Tinh, then the minor stars
/// by their nature (section 4.9 of the Tử Vi spec, decision T9).
struct StarGroups: Equatable {
    let main: [TuViPlacedStar]
    let good: [TuViPlacedStar]
    let neutral: [TuViPlacedStar]
    let bad: [TuViPlacedStar]

    init(_ palace: TuViPalace) {
        main = palace.mainStars
        good = palace.minorStars.filter { tuviStarNature(value: $0.star) == .cat }
        neutral = palace.minorStars.filter { tuviStarNature(value: $0.star) == .trung }
        bad = palace.minorStars.filter { tuviStarNature(value: $0.star) == .hung }
    }

    /// Every star carrying one of the Tứ Hóa, Chính Tinh first.
    var transformed: [TuViPlacedStar] {
        (main + good + neutral + bad).filter { $0.transformation != nil }
    }
}

/// Where a lunar year falls on the chart: tuổi mụ, and the palaces of its Đại Hạn and Tiểu Hạn.
struct TuViYearReading: Equatable {
    let lunarYear: Int32
    let age: Int
    let daiHan: Branch?
    let tieuHan: Branch?

    init(chart: TuViChart, lunarYear: Int32) {
        self.lunarYear = lunarYear
        age = Int(lunarYear) - Int(chart.lunar.year) + 1
        if let age = UInt8(exactly: age) {
            daiHan = tuviDaiHanPalace(chart: chart, age: age)?.branch
            tieuHan = tuviTieuHanPalace(chart: chart, age: age)?.branch
        } else {
            daiHan = nil
            tieuHan = nil
        }
    }
}

extension TuViPlacedStar {
    /// "Tử Vi (M)": the name, with the brightness mark where the star has one.
    var label: String {
        var label = tuviStarName(value: star)
        if let brightness {
            label += " (\(starBrightnessMark(value: brightness)))"
        }
        return label
    }
}

extension TuViChart {
    func palace(at branch: Branch) -> TuViPalace? {
        palaces.first { $0.branch == branch }
    }
}

extension ContentLuck {
    /// The luck color of a star's nature.
    init(_ nature: StarNature) {
        switch nature {
        case .cat: self = .cat
        case .trung: self = .trung
        case .hung: self = .hung
        }
    }
}
