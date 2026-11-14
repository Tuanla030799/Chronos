import Foundation

/// Which calendar a note's date is written on.
enum CalendarKind: String, Codable, Sendable, Hashable, CaseIterable, Identifiable {
    case lunar
    case solar

    var id: Self { self }

    var label: String {
        switch self {
        case .lunar: "Âm lịch"
        case .solar: "Dương lịch"
        }
    }
}

enum NoteCategory: String, Codable, Sendable, Hashable, CaseIterable, Identifiable {
    case anniversary
    case birthday
    case ceremony
    case important
    case note

    var id: Self { self }

    var label: String {
        switch self {
        case .anniversary: "Ngày giỗ"
        case .birthday: "Sinh nhật"
        case .ceremony: "Lễ, cúng"
        case .important: "Việc quan trọng"
        case .note: "Ghi chú"
        }
    }

    var systemImage: String {
        switch self {
        case .anniversary: "flame"
        case .birthday: "gift"
        case .ceremony: "leaf"
        case .important: "exclamationmark.circle"
        case .note: "note.text"
        }
    }

    /// A death anniversary, a birthday and a rite come back every year; a task or a note is
    /// for one day unless the person says otherwise.
    var repeatsYearlyByDefault: Bool {
        switch self {
        case .anniversary, .birthday, .ceremony: true
        case .important, .note: false
        }
    }
}

/// An event or a note on a day of the lunar or the solar calendar.
struct CalendarNote: Identifiable, Codable, Sendable, Hashable {
    var id = UUID()
    var title: String
    var notes = ""
    var calendarType: CalendarKind
    var day: Int
    var month: Int
    /// The year of the event itself; a yearly note comes back from this year on.
    var year: Int
    /// Read only for a lunar date, and only in `year`: later years use the regular month.
    var isLeapMonth = false
    var isYearlyRecurring = true
    var category = NoteCategory.anniversary
    /// Days before the event to remind at 08:00: 0 is the day itself.
    var reminderOffsets = [0, 1, 3]

    /// "15/10 Âm lịch", with the year when the note does not repeat.
    var dateLabel: String {
        var label = "\(day)/\(month)"
        if calendarType == .lunar && isLeapMonth && !isYearlyRecurring {
            label += " nhuận"
        }
        if !isYearlyRecurring {
            label += "/\(year)"
        }
        return "\(label) \(calendarType.label)"
    }
}

/// Where notes are kept: JSON in the `UserDefaults` of the App Group, which the app writes and
/// the widget reads.
enum NoteStorage {
    static let appGroup = "group.com.leanhtuan.chronos"
    static let key = "chronos.notes.v1"

    /// The suite of the App Group. Without the group entitlement the system still hands back a
    /// suite private to the process, so the app keeps working and only the widget sees nothing.
    static var shared: UserDefaults {
        UserDefaults(suiteName: appGroup) ?? .standard
    }

    static func load(from defaults: UserDefaults) -> [CalendarNote] {
        guard let data = defaults.data(forKey: key) else { return [] }
        return (try? JSONDecoder().decode([CalendarNote].self, from: data)) ?? []
    }

    static func save(_ notes: [CalendarNote], to defaults: UserDefaults) {
        guard let data = try? JSONEncoder().encode(notes) else { return }
        defaults.set(data, forKey: key)
    }
}
