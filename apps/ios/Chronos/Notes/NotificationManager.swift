import Foundation
@preconcurrency import UserNotifications

/// One local notification to schedule.
struct ReminderRequest: Hashable, Sendable {
    let id: String
    let title: String
    let body: String
    /// Year, month, day, hour and minute on the clock of the plan's time zone.
    let fire: DateComponents
}

/// Which reminders the notes need: at 08:00 on the day of each coming occurrence and on the
/// days before it that the note asks for.
enum ReminderPlan {
    static let hour = 8
    /// iOS keeps at most 64 pending local notifications for an app.
    static let limit = 60
    /// How far ahead occurrences are scheduled; the app plans again at each launch and change.
    static let horizonDays = 400
    static let prefix = "note."

    static func requests(for notes: [CalendarNote], now: Date, tzOffset: Double) -> [ReminderRequest] {
        let calendar = Calendar.zone(tzOffset)
        let parts = calendar.dateComponents([.year, .month, .day], from: now)
        let today = SolarDate(year: Int32(parts.year ?? 0), month: UInt8(parts.month ?? 1), day: UInt8(parts.day ?? 1))
        guard let horizon = today.adding(days: horizonDays) else { return [] }

        var planned: [(date: Date, request: ReminderRequest)] = []
        for note in notes where !note.reminderOffsets.isEmpty {
            for occurrence in note.occurrences(from: today, through: horizon, tzOffset: tzOffset) {
                for offset in Set(note.reminderOffsets) where offset >= 0 {
                    guard let day = occurrence.adding(days: -offset) else { continue }
                    var fire = DateComponents(
                        calendar: calendar,
                        timeZone: calendar.timeZone,
                        year: Int(day.year),
                        month: Int(day.month),
                        day: Int(day.day),
                        hour: hour,
                        minute: 0
                    )
                    guard let date = calendar.date(from: fire), date > now else { continue }
                    fire.calendar = nil
                    planned.append((date, ReminderRequest(
                        id: "\(prefix)\(note.id.uuidString).\(occurrence.ordinal).\(offset)",
                        title: note.category.label,
                        body: body(note, on: occurrence, offset: offset),
                        fire: fire
                    )))
                }
            }
        }
        return planned
            .sorted { ($0.date, $0.request.id) < ($1.date, $1.request.id) }
            .prefix(limit)
            .map(\.request)
    }

    /// "Ngày mai là Giỗ Ông Nội (15/10 Âm lịch), Thứ Ba 24/11/2026."
    static func body(_ note: CalendarNote, on date: SolarDate, offset: Int) -> String {
        let lead: String
        switch offset {
        case 0: lead = "Hôm nay là"
        case 1: lead = "Ngày mai là"
        default: lead = "Còn \(offset) ngày nữa là"
        }
        let day = String(format: "%@ %02d/%02d/%04d", date.weekdayName, Int(date.day), Int(date.month), Int(date.year))
        return "\(lead) \(note.title) (\(note.dateLabel)), \(day)."
    }
}

/// Asks for permission and keeps the pending local notifications equal to the plan.
enum NotificationManager {
    /// Asks once; later calls return what the person chose.
    static func requestAuthorization() async -> Bool {
        (try? await UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound])) ?? false
    }

    /// Replaces every pending reminder of the notes with the plan for `notes`. Schedules
    /// nothing until the person has allowed notifications.
    static func sync(_ notes: [CalendarNote], now: Date = .now, tzOffset: Double = EngineService.vietnamOffset) async {
        let center = UNUserNotificationCenter.current()
        let stale = await center.pendingNotificationRequests().map(\.identifier).filter { $0.hasPrefix(ReminderPlan.prefix) }
        center.removePendingNotificationRequests(withIdentifiers: stale)

        let status = await center.notificationSettings().authorizationStatus
        guard status == .authorized || status == .provisional else { return }
        for planned in ReminderPlan.requests(for: notes, now: now, tzOffset: tzOffset) {
            let content = UNMutableNotificationContent()
            content.title = planned.title
            content.body = planned.body
            content.sound = .default
            let trigger = UNCalendarNotificationTrigger(dateMatching: planned.fire, repeats: false)
            try? await center.add(UNNotificationRequest(identifier: planned.id, content: content, trigger: trigger))
        }
    }
}
