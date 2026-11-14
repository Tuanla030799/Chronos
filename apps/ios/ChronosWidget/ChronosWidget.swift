import SwiftUI
import WidgetKit

/// Vietnam's clock, as everywhere in the app.
private let tzOffset = 7.0

struct DayEntry: TimelineEntry {
    let date: Date
    let moment: DayMoment?
    /// Notes of the App Group falling in the next 30 days, soonest first.
    let events: [NoteOccurrence]

    init(at date: Date, notes: [CalendarNote]) {
        self.date = date
        moment = try? DayMoment(at: date, tzOffset: tzOffset)
        events = moment.map { NoteCalendar.upcoming(notes, from: $0.day.solar, days: 30, tzOffset: tzOffset) } ?? []
    }
}

/// One entry now, then one at each double-hour boundary left in the day and one at midnight,
/// after which WidgetKit asks for the next day. The app reloads the timeline when notes change.
struct DayProvider: TimelineProvider {
    func placeholder(in context: Context) -> DayEntry {
        DayEntry(at: .now, notes: [])
    }

    func getSnapshot(in context: Context, completion: @escaping (DayEntry) -> Void) {
        completion(DayEntry(at: .now, notes: NoteStorage.load(from: NoteStorage.shared)))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<DayEntry>) -> Void) {
        let notes = NoteStorage.load(from: NoteStorage.shared)
        let entries = DayTimeline.dates(after: .now, tzOffset: tzOffset).map { DayEntry(at: $0, notes: notes) }
        completion(Timeline(entries: entries, policy: .atEnd))
    }
}

struct DayWidgetView: View {
    let entry: DayEntry
    @Environment(\.widgetFamily) private var family

    var body: some View {
        Group {
            if let moment = entry.moment {
                switch family {
                case .systemMedium:
                    MediumDayView(moment: moment, events: entry.events)
                case .systemLarge:
                    LargeDayView(moment: moment, events: entry.events)
                case .accessoryRectangular:
                    RectangularDayView(moment: moment, events: entry.events)
                default:
                    SmallDayView(moment: moment)
                }
            } else {
                DayUnavailableView()
            }
        }
        .containerBackground(Theme.paper, for: .widget)
    }
}

struct DayWidget: Widget {
    var body: some WidgetConfiguration {
        StaticConfiguration(kind: "ChronosDay", provider: DayProvider()) { entry in
            DayWidgetView(entry: entry)
        }
        .configurationDisplayName("Lịch Vạn Niên")
        .description("Ngày dương, ngày âm, Can Chi, giờ Hoàng Đạo, ghi chú và ngày giỗ sắp tới.")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge, .accessoryRectangular])
    }
}

@main
struct ChronosWidgetBundle: WidgetBundle {
    var body: some Widget {
        DayWidget()
    }
}
