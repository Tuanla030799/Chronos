import Foundation

extension Calendar {
    /// Gregorian in UTC: date arithmetic on civil dates, free of the device's zone.
    static let civil: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .gmt
        return calendar
    }()
}

extension SolarDate {
    /// "Thứ Hai" … "Chủ Nhật".
    var weekdayName: String {
        let names = ["Chủ Nhật", "Thứ Hai", "Thứ Ba", "Thứ Tư", "Thứ Năm", "Thứ Sáu", "Thứ Bảy"]
        let components = DateComponents(year: Int(year), month: Int(month), day: Int(day))
        guard let date = Calendar.civil.date(from: components) else { return "" }
        return names[Calendar.civil.component(.weekday, from: date) - 1]
    }

    var isSunday: Bool { weekdayName == "Chủ Nhật" }
}

extension Branch {
    /// Clock hours of the double-hour as almanacs print them: "23–01" for Tý, "01–03" for Sửu.
    var hourRange: String {
        let index = Branch.allCases.firstIndex(of: self) ?? 0
        return String(format: "%02d–%02d", (2 * index + 23) % 24, (2 * index + 1) % 24)
    }
}
