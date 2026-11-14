import SwiftUI

/// `systemSmall`: the tear-off sheet folded small. Solar day, month and year, lunar date, the
/// day's Can Chi and whether it is Hoàng Đạo.
struct SmallDayView: View {
    let moment: DayMoment

    var body: some View {
        let day = moment.day
        VStack(alignment: .leading, spacing: 2) {
            Text(day.solar.weekdayName.uppercased())
                .font(.caption2.weight(.bold)).tracking(0.6)
                .foregroundStyle(Theme.cinnabar)
            Text("\(day.solar.day)")
                .font(.system(size: 50, weight: .bold, design: .serif))
                .foregroundStyle(day.solar.isSunday ? Theme.cinnabar : Theme.ink)
                .minimumScaleFactor(0.6)
                .lineLimit(1)
            Text("Tháng \(day.solar.month), \(String(day.solar.year))")
                .font(.caption2).foregroundStyle(Theme.inkSecondary)
            Spacer(minLength: 2)
            Text("Âm lịch \(day.lunarText)")
                .font(.caption.weight(.semibold)).foregroundStyle(Theme.ink)
            Text("Ngày \(canChiName(value: day.dayCanChi))")
                .font(.caption2).foregroundStyle(Theme.ink)
            PathGodLabel(isHoangDao: day.isHoangDao)
        }
        .lineLimit(1)
        .minimumScaleFactor(0.8)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        .accessibilityElement(children: .combine)
    }
}

/// `systemMedium`: the day of a Lịch Vạn Niên, and the event of today or the nearest one ahead.
struct MediumDayView: View {
    let moment: DayMoment
    var events: [NoteOccurrence] = []

    var body: some View {
        DayOverview(moment: moment, event: events.first)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

/// `systemLarge`: the full sheet of the day on top, then today's notes and the events ahead
/// with the days left.
struct LargeDayView: View {
    let moment: DayMoment
    var events: [NoteOccurrence] = []

    static let rows = 5

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            DayOverview(moment: moment, event: nil)
                .frame(height: 138, alignment: .top)
            Rectangle().fill(Theme.rule).frame(height: 1)
            Text("GHI CHÚ & SỰ KIỆN")
                .font(.system(size: 10, weight: .bold)).tracking(0.6)
                .foregroundStyle(Theme.inkSecondary)
            if events.isEmpty {
                Text("Không có sự kiện nào trong 30 ngày tới.")
                    .font(.caption).foregroundStyle(Theme.inkSecondary)
            } else {
                VStack(alignment: .leading, spacing: 7) {
                    ForEach(events.prefix(Self.rows)) { event in
                        EventRow(event: event)
                    }
                    if events.count > Self.rows {
                        Text("và \(events.count - Self.rows) sự kiện khác")
                            .font(.caption2).foregroundStyle(Theme.inkSecondary)
                    }
                }
            }
            Spacer(minLength: 0)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }
}

/// The sheet on the left; Can Chi, Tiết Khí, Trực, Sao and the Hoàng Đạo hours on the right,
/// with one event under them when there is one.
private struct DayOverview: View {
    let moment: DayMoment
    let event: NoteOccurrence?

    var body: some View {
        let day = moment.day
        HStack(alignment: .top, spacing: 12) {
            VStack(alignment: .leading, spacing: 1) {
                Text(day.solar.weekdayName.uppercased())
                    .font(.caption2.weight(.bold)).tracking(0.5)
                    .foregroundStyle(Theme.cinnabar)
                Text("\(day.solar.day)")
                    .font(.system(size: 46, weight: .bold, design: .serif))
                    .foregroundStyle(day.solar.isSunday ? Theme.cinnabar : Theme.ink)
                Text("Tháng \(day.solar.month), \(String(day.solar.year))")
                    .font(.caption2).foregroundStyle(Theme.inkSecondary)
                Spacer(minLength: 2)
                Text("Âm lịch").font(.caption2).foregroundStyle(Theme.inkSecondary)
                Text(day.lunarText)
                    .font(.headline).fontDesign(.serif).foregroundStyle(Theme.ink)
            }
            .frame(width: 92, alignment: .leading)

            Rectangle().fill(Theme.rule).frame(width: 1)

            VStack(alignment: .leading, spacing: 3) {
                Text("Năm \(canChiName(value: day.yearCanChi)) · Tháng \(canChiName(value: day.monthCanChi))")
                    .font(.caption2).foregroundStyle(Theme.inkSecondary)
                HStack(spacing: 6) {
                    Text("Ngày \(canChiName(value: day.dayCanChi))")
                        .font(.caption.weight(.bold)).foregroundStyle(Theme.ink)
                    PathGodLabel(isHoangDao: day.isHoangDao)
                }
                Text("Tiết \(solarTermName(term: day.solarTerm)) · Trực \(trucName(value: day.truc)) · Sao \(mansionName(value: day.mansion))")
                    .font(.caption2).foregroundStyle(Theme.ink)
                Text("GIỜ HOÀNG ĐẠO")
                    .font(.system(size: 9, weight: .bold)).tracking(0.5)
                    .foregroundStyle(Theme.gold)
                    .padding(.top, 2)
                Grid(alignment: .leading, horizontalSpacing: 8, verticalSpacing: 2) {
                    ForEach(Array(stride(from: 0, to: day.hoangDaoHours.count, by: 3)), id: \.self) { start in
                        GridRow {
                            ForEach(day.hoangDaoHours[start..<min(start + 3, day.hoangDaoHours.count)], id: \.self) { branch in
                                hour(branch)
                            }
                        }
                    }
                }
                if let event {
                    Label {
                        Text("\(event.countdown): \(event.note.title)")
                    } icon: {
                        Image(systemName: event.note.category.systemImage)
                    }
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(Theme.cinnabar)
                    .padding(.top, 2)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .lineLimit(1)
        .minimumScaleFactor(0.75)
        .accessibilityElement(children: .combine)
    }

    /// One Hoàng Đạo hour; the one running now, or the next one, in bold Kim.
    private func hour(_ branch: Branch) -> some View {
        let marked = moment.upcoming.map { !$0.isTomorrow && $0.branch == branch } ?? false
        return Text("\(branchName(branch: branch)) \(branch.hourRange)")
            .font(.system(size: 11, weight: marked ? .bold : .regular))
            .foregroundStyle(marked ? Theme.gold : Theme.ink)
    }
}

/// One event of the large widget: what, on which date, and how far ahead.
private struct EventRow: View {
    let event: NoteOccurrence

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: event.note.category.systemImage)
                .font(.caption)
                .foregroundStyle(Theme.cinnabar)
                .frame(width: 16)
            VStack(alignment: .leading, spacing: 0) {
                Text(event.note.title)
                    .font(.caption.weight(.semibold)).foregroundStyle(Theme.ink)
                Text("\(event.note.category.label) · \(event.note.dateLabel)")
                    .font(.system(size: 10)).foregroundStyle(Theme.inkSecondary)
            }
            Spacer(minLength: 4)
            Text(event.countdown)
                .font(.caption2.weight(.semibold))
                .foregroundStyle(event.daysLeft == 0 ? Theme.cinnabar : Theme.gold)
        }
        .lineLimit(1)
        .accessibilityElement(children: .combine)
    }
}

/// `accessoryRectangular`, on the Lock Screen: solar and lunar date, the day's Can Chi, and an
/// event of today or tomorrow when there is one, else the Hoàng Đạo hour running now or coming
/// next. The system tints it, so it sets no colors.
struct RectangularDayView: View {
    let moment: DayMoment
    var events: [NoteOccurrence] = []

    /// The event the Lock Screen shows instead of the hour: the first of today or tomorrow.
    nonisolated static func reminder(in events: [NoteOccurrence]) -> NoteOccurrence? {
        events.first { $0.daysLeft <= 1 }
    }

    var body: some View {
        let day = moment.day
        VStack(alignment: .leading, spacing: 1) {
            Text("\(day.solar.day)/\(day.solar.month) · Âm \(day.lunarText)")
                .font(.headline)
            Text("Ngày \(canChiName(value: day.dayCanChi)) · \(day.isHoangDao ? "Hoàng Đạo" : "Hắc Đạo")")
                .font(.caption)
            if let event = Self.reminder(in: events) {
                Label {
                    Text("\(event.countdown): \(event.note.title)")
                } icon: {
                    Image(systemName: event.note.category.systemImage)
                }
                .font(.caption.weight(.semibold))
            } else if let upcoming = moment.upcoming {
                Label {
                    Text("\(upcoming.isNow ? "Đang giờ" : "Giờ tới") \(upcoming.text)")
                } icon: {
                    Image(systemName: "sun.max")
                }
                .font(.caption)
            }
        }
        .lineLimit(1)
        .minimumScaleFactor(0.8)
        .frame(maxWidth: .infinity, alignment: .leading)
        .accessibilityElement(children: .combine)
    }
}

/// Hoàng Đạo in Kim or Hắc Đạo in Chu Sa, with a dot: the word carries the meaning, not only the
/// color.
struct PathGodLabel: View {
    let isHoangDao: Bool

    var body: some View {
        HStack(spacing: 4) {
            Circle().frame(width: 6, height: 6)
            Text(isHoangDao ? "Hoàng Đạo" : "Hắc Đạo").font(.caption2.weight(.semibold))
        }
        .foregroundStyle(isHoangDao ? Theme.gold : Theme.cinnabar)
    }
}

/// What a widget shows when the engine cannot read the date.
struct DayUnavailableView: View {
    var body: some View {
        VStack(spacing: 4) {
            Image(systemName: "calendar.badge.exclamationmark")
            Text("Không đọc được ngày").font(.caption)
        }
        .foregroundStyle(Theme.inkSecondary)
    }
}
