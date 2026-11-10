import SwiftUI

/// Lịch Vạn Niên: the lunar date, Can Chi, solar term, Giờ Hoàng Đạo and the almanac of a day.
struct CalendarTab: View {
    let services: AppServices
    @State private var date: SolarDate

    init(services: AppServices) {
        self.services = services
        _date = State(initialValue: services.engine.today())
    }

    var body: some View {
        NavigationStack {
            List {
                Section {
                    DatePicker("Ngày dương lịch", selection: pickerDate, displayedComponents: .date)
                }
                switch Result(catching: { try reading() }) {
                case .success(let reading):
                    Section("Âm lịch") {
                        LabeledContent("Ngày âm", value: lunarText(reading.info.lunar))
                        LabeledContent("Năm", value: canChiName(value: reading.info.yearCanChi))
                        LabeledContent("Tháng", value: canChiName(value: reading.info.monthCanChi))
                        LabeledContent("Ngày", value: canChiName(value: reading.info.dayCanChi))
                        LabeledContent(
                            "Tiết khí",
                            value: solarTermName(term: reading.info.solarTerm) + (reading.info.solarTermBegins ? " (bắt đầu hôm nay)" : "")
                        )
                    }
                    Section("Giờ Hoàng Đạo") {
                        Text(reading.almanac.hoangDaoHours.map { branchName(branch: $0) }.joined(separator: ", "))
                            .foregroundStyle(Theme.ink)
                    }
                    Section("Trực, Sao, Thần Sát") {
                        ForEach(reading.entries) { EntryView(entry: $0, content: services.content) }
                    }
                case .failure(let error):
                    Text(String(describing: error)).foregroundStyle(Theme.cinnabar)
                }
            }
            .chronosBackground()
            .navigationTitle("Lịch Vạn Niên")
            .profileToolbar(services)
        }
    }

    private struct Reading {
        let info: DayInfo
        let almanac: AlmanacDay
        let entries: [ContentEntry]
    }

    private func reading() throws -> Reading {
        let almanac = try services.engine.almanac(date)
        return Reading(info: try services.engine.day(date), almanac: almanac, entries: try services.content.almanacDay(almanac))
    }

    private func lunarText(_ lunar: LunarDate) -> String {
        "\(lunar.day)/\(lunar.month)\(lunar.isLeapMonth ? " nhuận" : "")/\(lunar.year)"
    }

    /// The picker works in the device's calendar; only the civil date is kept.
    private var pickerDate: Binding<Date> {
        Binding(
            get: {
                Calendar.current.date(from: DateComponents(year: Int(date.year), month: Int(date.month), day: Int(date.day))) ?? .now
            },
            set: { newValue in
                let parts = Calendar.current.dateComponents([.year, .month, .day], from: newValue)
                date = SolarDate(year: Int32(parts.year ?? 0), month: UInt8(parts.month ?? 1), day: UInt8(parts.day ?? 1))
            }
        )
    }
}
