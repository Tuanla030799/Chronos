import Foundation

enum ProfileGender: String, Codable, Sendable, Hashable, CaseIterable, Identifiable {
    case male
    case female

    var id: Self { self }

    var label: String {
        switch self {
        case .male: "Nam"
        case .female: "Nữ"
        }
    }

    var engineValue: Gender {
        switch self {
        case .male: .male
        case .female: .female
        }
    }
}

/// A birth date as the person knows it: on the solar or the lunar calendar, with clock time.
struct BirthDate: Codable, Sendable, Hashable {
    enum Kind: String, Codable, Sendable, Hashable, CaseIterable, Identifiable {
        case solar
        case lunar

        var id: Self { self }

        var label: String {
            switch self {
            case .solar: "Dương lịch"
            case .lunar: "Âm lịch"
            }
        }
    }

    var kind: Kind
    var year: Int
    var month: Int
    var day: Int
    /// Read only for a lunar date.
    var isLeapMonth: Bool = false
    var hour: Int
    var minute: Int
}

/// A person whose charts the app draws. Shared by every tab.
struct Profile: Identifiable, Codable, Sendable, Hashable {
    var id = UUID()
    var name: String
    var gender: ProfileGender
    var birth: BirthDate
    /// Offset of the birth clock from UTC, in hours: 7 for Vietnam.
    var tzOffset: Double = EngineService.vietnamOffset
    /// Birthplace longitude in degrees east. When set, Bát Tự and Tử Vi use true solar time
    /// there; when nil they use the clock time.
    var longitude: Double?

    /// Decision I1: the profile of a first launch, so no tab opens empty.
    static let sample = Profile(
        name: "Hồ sơ mẫu",
        gender: .male,
        birth: BirthDate(kind: .solar, year: 1990, month: 5, day: 15, hour: 9, minute: 30)
    )
}

extension Profile {
    /// "Nam, Dương lịch 15/05/1990 09:30".
    var summary: String {
        let date = String(format: "%02d/%02d/%04d %02d:%02d", birth.day, birth.month, birth.year, birth.hour, birth.minute)
        return "\(gender.label), \(birth.kind.label) \(date)\(birth.kind == .lunar && birth.isLeapMonth ? " (nhuận)" : "")"
    }
}
