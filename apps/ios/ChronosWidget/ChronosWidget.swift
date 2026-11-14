import SwiftUI
import WidgetKit

/// Vietnam's clock, as everywhere in the app.
private let tzOffset = 7.0

struct DayEntry: TimelineEntry {
    let date: Date
    let moment: DayMoment?

    init(at date: Date) {
        self.date = date
        moment = try? DayMoment(at: date, tzOffset: tzOffset)
    }
}

/// One entry now, then one at each double-hour boundary left in the day and one at midnight,
/// after which WidgetKit asks for the next day.
struct DayProvider: TimelineProvider {
    func placeholder(in context: Context) -> DayEntry {
        DayEntry(at: .now)
    }

    func getSnapshot(in context: Context, completion: @escaping (DayEntry) -> Void) {
        completion(DayEntry(at: .now))
    }

    func getTimeline(in context: Context, completion: @escaping (Timeline<DayEntry>) -> Void) {
        let entries = DayTimeline.dates(after: .now, tzOffset: tzOffset).map(DayEntry.init(at:))
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
                    MediumDayView(moment: moment)
                case .accessoryRectangular:
                    RectangularDayView(moment: moment)
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
        .description("Ngày dương, ngày âm, Can Chi, Hoàng Đạo và giờ tốt trong ngày.")
        .supportedFamilies([.systemSmall, .systemMedium, .accessoryRectangular])
    }
}

@main
struct ChronosWidgetBundle: WidgetBundle {
    var body: some Widget {
        DayWidget()
    }
}
