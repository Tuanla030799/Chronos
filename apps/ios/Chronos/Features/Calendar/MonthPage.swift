import Foundation

/// A month on the solar calendar: what each cell of the grid shows.
struct MonthKey: Hashable, Sendable {
    let year: Int32
    let month: UInt8

    init(year: Int32, month: UInt8) {
        self.year = year
        self.month = month
    }

    init(_ date: SolarDate) {
        self.init(year: date.year, month: date.month)
    }

    func adding(months: Int) -> MonthKey {
        let index = Int(year) * 12 + Int(month) - 1 + months
        return MonthKey(year: Int32(index.floorDiv(12)), month: UInt8(index.floorMod(12) + 1))
    }
}

struct CalendarDay: Identifiable, Hashable, Sendable {
    let solar: SolarDate
    let lunar: LunarDate
    /// The day's path god is Hoàng Đạo; otherwise it is Hắc Đạo.
    let isHoangDao: Bool

    var id: SolarDate { solar }

    /// Lunar day under the solar one: the month too on day 1, and "n" for a leap month.
    var lunarLabel: String {
        guard lunar.day == 1 else { return "\(lunar.day)" }
        return "1/\(lunar.month)\(lunar.isLeapMonth ? "n" : "")"
    }
}

struct MonthPage: Sendable {
    let key: MonthKey
    /// Empty cells before day 1 in a week that starts on Monday, as Vietnamese calendars do.
    let leadingBlanks: Int
    let days: [CalendarDay]

    static func build(_ key: MonthKey, engine: EngineService) throws -> MonthPage {
        let calendar = Calendar.civil
        let first = DateComponents(year: Int(key.year), month: Int(key.month), day: 1)
        guard let start = calendar.date(from: first),
              let range = calendar.range(of: .day, in: .month, for: start)
        else {
            return MonthPage(key: key, leadingBlanks: 0, days: [])
        }
        let weekday = calendar.component(.weekday, from: start)
        let days = try range.map { day in
            let almanac = try engine.almanac(SolarDate(year: key.year, month: key.month, day: UInt8(day)))
            return CalendarDay(solar: almanac.solar, lunar: almanac.lunar, isHoangDao: pathGodIsHoangDao(value: almanac.pathGod))
        }
        return MonthPage(key: key, leadingBlanks: (weekday + 5) % 7, days: days)
    }
}

private extension Int {
    func floorDiv(_ divisor: Int) -> Int {
        (self - floorMod(divisor)) / divisor
    }

    func floorMod(_ divisor: Int) -> Int {
        ((self % divisor) + divisor) % divisor
    }
}
