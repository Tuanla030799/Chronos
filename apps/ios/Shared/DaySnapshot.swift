import Foundation

/// What the widget shows for one civil day, read from AstroCore alone (decision I3: no SQLite).
struct DaySnapshot: Hashable, Sendable {
    let solar: SolarDate
    let lunar: LunarDate
    let yearCanChi: CanChi
    let monthCanChi: CanChi
    let dayCanChi: CanChi
    let solarTerm: SolarTerm
    let solarTermBegins: Bool
    let truc: Truc
    let mansion: Mansion
    let pathGod: PathGod
    let hoangDaoHours: [Branch]

    init(_ date: SolarDate, tzOffset: Double) throws {
        let info = try dayInfo(date: date, tzOffset: tzOffset)
        let almanac = try almanacDay(date: date, tzOffset: tzOffset)
        solar = date
        lunar = info.lunar
        yearCanChi = info.yearCanChi
        monthCanChi = info.monthCanChi
        dayCanChi = info.dayCanChi
        solarTerm = info.solarTerm
        solarTermBegins = info.solarTermBegins
        truc = almanac.truc
        mansion = almanac.mansion
        pathGod = almanac.pathGod
        hoangDaoHours = almanac.hoangDaoHours
    }

    var isHoangDao: Bool { pathGodIsHoangDao(value: pathGod) }

    /// "25/9" or "1/10 nhuận".
    var lunarText: String {
        "\(lunar.day)/\(lunar.month)\(lunar.isLeapMonth ? " nhuận" : "")"
    }
}

/// One of the double-hours of a civil day. The day changes at midnight, as the engine's default
/// `SplitAtMidnight` has it, so Tý covers both 00:00–00:59 and 23:00–23:59 of the same day and a
/// day has thirteen slots.
struct HourSlot: Hashable, Sendable {
    let branch: Branch
    let startHour: Int
    let endHour: Int

    static let day: [HourSlot] = {
        let branches = Branch.allCases
        var slots = [HourSlot(branch: branches[0], startHour: 0, endHour: 1)]
        for index in 1..<12 {
            slots.append(HourSlot(branch: branches[index], startHour: 2 * index - 1, endHour: 2 * index + 1))
        }
        slots.append(HourSlot(branch: branches[0], startHour: 23, endHour: 24))
        return slots
    }()

    /// The slot holding clock hour `hour` (0–23).
    static func at(hour: Int) -> HourSlot {
        day.last { $0.startHour <= hour } ?? day[0]
    }
}

/// A Hoàng Đạo double-hour the reader can still use: the one running now, or the next one.
struct UpcomingHour: Hashable, Sendable {
    let branch: Branch
    let startHour: Int
    let endHour: Int
    let isNow: Bool
    let isTomorrow: Bool

    /// "Giờ Dần 03–05", with "mai" when it falls on the next day.
    var text: String {
        let hours = String(format: "%02d–%02d", startHour, endHour % 24)
        return "\(branchName(branch: branch)) \(hours)\(isTomorrow ? " (mai)" : "")"
    }
}

/// The day and the time of a widget entry.
struct DayMoment: Hashable, Sendable {
    let day: DaySnapshot
    let hour: Int
    let minute: Int
    let upcoming: UpcomingHour?

    /// The moment at `date` on the civil clock of `tzOffset`.
    init(at date: Date, tzOffset: Double) throws {
        let calendar = Calendar.zone(tzOffset)
        let parts = calendar.dateComponents([.year, .month, .day, .hour, .minute], from: date)
        let today = SolarDate(year: Int32(parts.year ?? 0), month: UInt8(parts.month ?? 1), day: UInt8(parts.day ?? 1))
        day = try DaySnapshot(today, tzOffset: tzOffset)
        hour = parts.hour ?? 0
        minute = parts.minute ?? 0
        upcoming = try Self.upcoming(today: day, hour: hour, tzOffset: tzOffset)
    }

    /// The Hoàng Đạo slot running at `hour`, else the next one today, else the first of tomorrow.
    static func upcoming(today: DaySnapshot, hour: Int, tzOffset: Double) throws -> UpcomingHour? {
        let current = HourSlot.at(hour: hour)
        for slot in HourSlot.day where slot.startHour >= current.startHour && today.hoangDaoHours.contains(slot.branch) {
            return UpcomingHour(
                branch: slot.branch,
                startHour: slot.startHour,
                endHour: slot.endHour,
                isNow: slot == current,
                isTomorrow: false
            )
        }
        guard let next = today.solar.adding(days: 1) else { return nil }
        let tomorrow = try DaySnapshot(next, tzOffset: tzOffset)
        guard let slot = HourSlot.day.first(where: { tomorrow.hoangDaoHours.contains($0.branch) }) else { return nil }
        return UpcomingHour(branch: slot.branch, startHour: slot.startHour, endHour: slot.endHour, isNow: false, isTomorrow: true)
    }
}

/// When the widget redraws: at every double-hour boundary left today, then at midnight, so the
/// day turns at 00:00 and the Hoàng Đạo hour stays current.
enum DayTimeline {
    static func dates(after now: Date, tzOffset: Double) -> [Date] {
        let calendar = Calendar.zone(tzOffset)
        let start = calendar.startOfDay(for: now)
        let boundaries = HourSlot.day.dropFirst().map(\.startHour) + [24]
        let later = boundaries.compactMap { calendar.date(byAdding: .hour, value: $0, to: start) }.filter { $0 > now }
        return [now] + later
    }
}

extension Calendar {
    /// Gregorian on the fixed clock of `tzOffset` hours east of UTC.
    static func zone(_ tzOffset: Double) -> Calendar {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: Int((tzOffset * 3600).rounded())) ?? .gmt
        return calendar
    }
}

extension SolarDate {
    /// The civil date `days` later, or nil past the range of `Calendar`.
    func adding(days: Int) -> SolarDate? {
        let components = DateComponents(year: Int(year), month: Int(month), day: Int(day))
        guard let date = Calendar.civil.date(from: components),
              let next = Calendar.civil.date(byAdding: .day, value: days, to: date)
        else { return nil }
        let parts = Calendar.civil.dateComponents([.year, .month, .day], from: next)
        return SolarDate(year: Int32(parts.year ?? 0), month: UInt8(parts.month ?? 1), day: UInt8(parts.day ?? 1))
    }
}
