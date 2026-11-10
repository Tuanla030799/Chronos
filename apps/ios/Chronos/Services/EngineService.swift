import Foundation

/// The app's entry to AstroCore. Turns a `Profile` into engine inputs so every tab reads a
/// person the same way, and keeps the GMT+7 default in one place.
struct EngineService: Sendable {
    static let vietnamOffset = 7.0
    /// `MIN_YEAR...MAX_YEAR` of astro-core.
    static let supportedYears = 1600...2400

    var tzOffset: Double = vietnamOffset

    // MARK: Calendar

    /// The civil date at `now` in the zone of `tzOffset`.
    func today(now: Date = .now) -> SolarDate {
        let parts = components(of: now)
        return SolarDate(year: Int32(parts.year ?? 0), month: UInt8(parts.month ?? 1), day: UInt8(parts.day ?? 1))
    }

    /// The civil date and time at `now` in the zone of `tzOffset`.
    func now(_ now: Date = .now) -> SolarDateTime {
        let parts = components(of: now)
        return SolarDateTime(
            year: Int32(parts.year ?? 0),
            month: UInt8(parts.month ?? 1),
            day: UInt8(parts.day ?? 1),
            hour: UInt8(parts.hour ?? 0),
            minute: UInt8(parts.minute ?? 0),
            second: UInt8(parts.second ?? 0)
        )
    }

    func day(_ date: SolarDate) throws -> DayInfo {
        try dayInfo(date: date, tzOffset: tzOffset)
    }

    func almanac(_ date: SolarDate) throws -> AlmanacDay {
        try almanacDay(date: date, tzOffset: tzOffset)
    }

    func lunar(_ date: SolarDate) throws -> LunarDate {
        try solarToLunar(date: date, tzOffset: tzOffset)
    }

    func nineStars(at time: SolarDateTime) throws -> NineStars {
        try Chronos.nineStars(time: time, tzOffset: tzOffset)
    }

    // MARK: Profiles

    /// The clock time of birth on the solar calendar. A lunar birth date goes through
    /// `lunarToSolar` in the profile's own zone; invalid dates throw `AstroError`.
    func birth(of profile: Profile) throws -> SolarDateTime {
        let birth = profile.birth
        let date: SolarDate
        switch birth.kind {
        case .solar:
            date = SolarDate(year: Int32(clamping: birth.year), month: UInt8(clamping: birth.month), day: UInt8(clamping: birth.day))
        case .lunar:
            date = try lunarToSolar(date: lunarBirth(birth), tzOffset: profile.tzOffset)
        }
        return SolarDateTime(
            year: date.year,
            month: date.month,
            day: date.day,
            hour: UInt8(clamping: birth.hour),
            minute: UInt8(clamping: birth.minute),
            second: 0
        )
    }

    /// The lunar date of birth: as entered, or converted from the solar date.
    func lunarBirthDate(of profile: Profile) throws -> LunarDate {
        switch profile.birth.kind {
        case .lunar:
            let lunar = lunarBirth(profile.birth)
            _ = try lunarToSolar(date: lunar, tzOffset: profile.tzOffset)
            return lunar
        case .solar:
            let time = try birth(of: profile)
            return try solarToLunar(
                date: SolarDate(year: time.year, month: time.month, day: time.day),
                tzOffset: profile.tzOffset
            )
        }
    }

    func bazi(for profile: Profile) throws -> BaziAnalysis {
        var options = defaultFourPillarsOptions()
        options.tzOffset = profile.tzOffset
        options.longitude = profile.longitude
        return try analyzeBazi(birth: birth(of: profile), options: options, gender: profile.gender.engineValue)
    }

    func tuviChart(for profile: Profile) throws -> TuViChart {
        var config = defaultTuviConfig()
        config.tzOffset = profile.tzOffset
        config.useTrueSolarTime = profile.longitude != nil
        config.longitude = profile.longitude
        return try Chronos.tuviChart(birth: birth(of: profile), gender: profile.gender.engineValue, config: config)
    }

    /// Tam Tai, Kim Lâu and Hoang Ốc of the person in the lunar year `targetLunarYear`.
    func ageRestrictions(for profile: Profile, targetLunarYear: Int32) throws -> AgeRestrictions {
        Chronos.ageRestrictions(
            birthLunarYear: try lunarBirthDate(of: profile).year,
            targetLunarYear: targetLunarYear
        )
    }

    /// Decision I5: the person's relation to the Thái Tuế of `targetLunarYear`.
    func taiSui(for profile: Profile, targetLunarYear: Int32) throws -> TaiSuiReading {
        Chronos.taiSui(birthLunarYear: try lunarBirthDate(of: profile).year, targetLunarYear: targetLunarYear)
    }

    /// Can Chi of a lunar year, read on its first day.
    func yearCanChi(lunarYear: Int32) throws -> CanChi {
        let newYear = try lunarToSolar(date: LunarDate(year: lunarYear, month: 1, day: 1, isLeapMonth: false), tzOffset: tzOffset)
        return try day(newYear).yearCanChi
    }

    func kua(for profile: Profile) throws -> KuaInfo {
        kuaInfo(birthLunarYear: try lunarBirthDate(of: profile).year, gender: profile.gender.engineValue)
    }

    // MARK: Helpers

    private func lunarBirth(_ birth: BirthDate) -> LunarDate {
        LunarDate(
            year: Int32(clamping: birth.year),
            month: UInt8(clamping: birth.month),
            day: UInt8(clamping: birth.day),
            isLeapMonth: birth.isLeapMonth
        )
    }

    private func components(of date: Date) -> DateComponents {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = TimeZone(secondsFromGMT: Int((tzOffset * 3600).rounded())) ?? .gmt
        return calendar.dateComponents([.year, .month, .day, .hour, .minute, .second], from: date)
    }
}
