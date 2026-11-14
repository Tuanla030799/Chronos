import Foundation

/// The birth of a profile on the other calendar, with the Can Chi of its lunar date and hour.
struct BirthPreview: Equatable, Sendable {
    /// "Âm lịch: 21 tháng 4 năm 1990" or "Dương lịch: Thứ Ba, 15/05/1990".
    let counterpart: String
    /// "Năm Canh Ngọ, Tháng Tân Tỵ, Ngày Canh Thìn, Giờ Tân Tỵ": year, month and day of the
    /// lunar calendar, hour by the clock with Tý split at midnight.
    let canChi: String
    /// The clock corrected to the birthplace, when the profile has a longitude.
    let solarTime: String?
}

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

    /// The same birth written on the other calendar, so switching the editor between Dương lịch
    /// and Âm lịch keeps the person's birthday instead of reusing its numbers. A date that does
    /// not exist on its calendar keeps its numbers, for the editor to refuse on saving.
    func birth(_ birth: BirthDate, rewrittenAs kind: BirthDate.Kind, tzOffset: Double) -> BirthDate {
        var rewritten = birth
        rewritten.kind = kind
        switch (birth.kind, kind) {
        case (.solar, .lunar):
            let solar = SolarDate(year: Int32(clamping: birth.year), month: UInt8(clamping: birth.month), day: UInt8(clamping: birth.day))
            guard let lunar = try? solarToLunar(date: solar, tzOffset: tzOffset) else { return rewritten }
            rewritten.year = Int(lunar.year)
            rewritten.month = Int(lunar.month)
            rewritten.day = Int(lunar.day)
            rewritten.isLeapMonth = lunar.isLeapMonth
        case (.lunar, .solar):
            guard let solar = try? lunarToSolar(date: lunarBirth(birth), tzOffset: tzOffset) else { return rewritten }
            rewritten.year = Int(solar.year)
            rewritten.month = Int(solar.month)
            rewritten.day = Int(solar.day)
            rewritten.isLeapMonth = false
        case (.solar, .solar), (.lunar, .lunar):
            break
        }
        return rewritten
    }

    /// The leap month of a lunar year; nil when it has none or lies outside the engine's range.
    func leapMonth(ofLunarYear year: Int, tzOffset: Double) -> Int? {
        (try? lunarYear(year: Int32(clamping: year), tzOffset: tzOffset))?.leapMonth.map(Int.init)
    }

    /// Days of a lunar month, 29 or 30; nil when the year has no such month.
    func lunarMonthLength(year: Int, month: Int, isLeap: Bool, tzOffset: Double) -> Int? {
        (try? lunarYear(year: Int32(clamping: year), tzOffset: tzOffset))?
            .months
            .first { Int($0.month) == month && $0.isLeap == isLeap }
            .map { Int($0.length) }
    }

    /// A lunar birth made to fit its year: the leap flag off unless the year repeats that very
    /// month, and the day no later than the last day of the month. A solar birth is returned
    /// as it is.
    func fitted(_ birth: BirthDate, tzOffset: Double) -> BirthDate {
        guard birth.kind == .lunar else { return birth }
        var fitted = birth
        if fitted.isLeapMonth && leapMonth(ofLunarYear: birth.year, tzOffset: tzOffset) != birth.month {
            fitted.isLeapMonth = false
        }
        if let length = lunarMonthLength(year: fitted.year, month: fitted.month, isLeap: fitted.isLeapMonth, tzOffset: tzOffset) {
            fitted.day = min(fitted.day, length)
        }
        return fitted
    }

    /// What the profile form shows under the birth fields as they change.
    func birthPreview(for profile: Profile) throws -> BirthPreview {
        let time = try birth(of: profile)
        let date = SolarDate(year: time.year, month: time.month, day: time.day)
        let info = try dayInfo(date: date, tzOffset: profile.tzOffset)
        var options = defaultFourPillarsOptions()
        options.tzOffset = profile.tzOffset
        let hour = try fourPillars(birth: time, options: options).hour

        let counterpart: String
        switch profile.birth.kind {
        case .solar:
            let lunar = info.lunar
            counterpart = "Âm lịch: \(lunar.day) tháng \(lunar.month)\(lunar.isLeapMonth ? " nhuận" : "") năm \(String(lunar.year))"
        case .lunar:
            counterpart = String(format: "Dương lịch: %@, %02d/%02d/%04d", date.weekdayName, Int(date.day), Int(date.month), Int(date.year))
        }
        let canChi = "Năm \(canChiName(value: info.yearCanChi)), Tháng \(canChiName(value: info.monthCanChi)), "
            + "Ngày \(canChiName(value: info.dayCanChi)), Giờ \(canChiName(value: hour))"

        var solarTime: String?
        if let longitude = profile.longitude {
            let real = try trueSolarTime(clock: time, longitude: longitude, tzOffset: profile.tzOffset)
            solarTime = String(
                format: "Giờ Mặt Trời thực: %02d:%02d ngày %02d/%02d",
                Int(real.hour), Int(real.minute), Int(real.day), Int(real.month)
            )
        }
        return BirthPreview(counterpart: counterpart, canChi: canChi, solarTime: solarTime)
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

    /// `ratHour` and `boundary` default to the engine's own: Tý split at midnight, year and month
    /// changing at the solar terms.
    func bazi(
        for profile: Profile,
        ratHour: RatHourConvention? = nil,
        boundary: YearMonthBoundary? = nil
    ) throws -> BaziAnalysis {
        var options = defaultFourPillarsOptions()
        options.tzOffset = profile.tzOffset
        options.longitude = profile.longitude
        options.ratHour = ratHour ?? options.ratHour
        options.boundary = boundary ?? options.boundary
        return try analyzeBazi(birth: birth(of: profile), options: options, gender: profile.gender.engineValue)
    }

    /// `leapMonthRule` and `ratHour` default to the engine's own: a leap month split at day 15,
    /// Tý split at midnight.
    func tuviChart(
        for profile: Profile,
        leapMonthRule: LeapMonthRule? = nil,
        ratHour: RatHourConvention? = nil
    ) throws -> TuViChart {
        var config = defaultTuviConfig()
        config.tzOffset = profile.tzOffset
        config.leapMonthRule = leapMonthRule ?? config.leapMonthRule
        config.ratHour = ratHour ?? config.ratHour
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
