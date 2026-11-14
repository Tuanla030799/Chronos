import SwiftUI

/// A note being added or changed.
struct NoteDraft: Identifiable {
    let note: CalendarNote
    let isNew: Bool

    var id: CalendarNote.ID { note.id }
}

/// Form for one event or note: what, on which calendar and day, whether it comes back every
/// year, and when to remind. The card under the date shows the day on the other calendar and
/// the next time it falls.
struct NoteFormView: View {
    @State var note: CalendarNote
    let isNew: Bool
    let engine: EngineService
    @Environment(NoteStore.self) private var store
    @Environment(\.dismiss) private var dismiss

    private var tzOffset: Double { engine.tzOffset }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Tên sự kiện, ví dụ Giỗ Ông Nội", text: $note.title)
                    Picker("Loại", selection: $note.category) {
                        ForEach(NoteCategory.allCases) { Label($0.label, systemImage: $0.systemImage).tag($0) }
                    }
                }
                .listRowBackground(Theme.surface)

                Section {
                    Picker("Lịch", selection: $note.calendarType) {
                        ForEach(CalendarKind.allCases) { Text($0.label).tag($0) }
                    }
                    .pickerStyle(.segmented)
                    switch note.calendarType {
                    case .solar:
                        DatePicker("Ngày", selection: solarDate, displayedComponents: .date)
                            .environment(\.timeZone, .gmt)
                    case .lunar:
                        lunarFields
                    }
                    Toggle("Lặp lại hằng năm", isOn: $note.isYearlyRecurring)
                    NotePreviewCard(lines: Self.preview(note, today: engine.today(), tzOffset: tzOffset))
                } header: {
                    Text("Ngày")
                }
                .listRowBackground(Theme.surface)

                Section {
                    reminder("Sáng ngày diễn ra", offset: 0)
                    reminder("Trước 1 ngày", offset: 1)
                    reminder("Trước 3 ngày", offset: 3)
                } header: {
                    Text("Nhắc lúc 08:00")
                } footer: {
                    Text("Ứng dụng sẽ xin quyền gửi thông báo khi bạn lưu một sự kiện có nhắc.")
                }
                .listRowBackground(Theme.surface)

                Section("Ghi chú") {
                    TextField("Ghi chú thêm", text: $note.notes, axis: .vertical)
                        .lineLimit(3...6)
                }
                .listRowBackground(Theme.surface)

                if !isNew {
                    Section {
                        Button("Xóa sự kiện", role: .destructive) {
                            store.delete(note.id)
                            dismiss()
                        }
                    }
                    .listRowBackground(Theme.surface)
                }
            }
            .chronosBackground()
            .onChange(of: note.calendarType) { old, new in
                var before = note
                before.calendarType = old
                note = before.rewritten(as: new, tzOffset: tzOffset)
            }
            .onChange(of: note.category) { _, category in
                if isNew {
                    note.isYearlyRecurring = category.repeatsYearlyByDefault
                }
            }
            .onChange(of: [note.year, note.month]) {
                if note.isLeapMonth && engine.leapMonth(ofLunarYear: note.year, tzOffset: tzOffset) != note.month {
                    note.isLeapMonth = false
                }
            }
            .navigationTitle(isNew ? "Sự kiện mới" : note.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Hủy") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Lưu", action: save).disabled(!canSave)
                }
            }
        }
    }

    @ViewBuilder
    private var lunarFields: some View {
        let leap = engine.leapMonth(ofLunarYear: note.year, tzOffset: tzOffset)
        Stepper("Năm \(String(note.year))", value: $note.year, in: EngineService.supportedYears)
        Picker("Tháng", selection: $note.month) {
            ForEach(1...12, id: \.self) { Text("Tháng \($0)").tag($0) }
        }
        Picker("Ngày", selection: $note.day) {
            ForEach(1...30, id: \.self) { Text("Ngày \($0)").tag($0) }
        }
        Toggle("Tháng nhuận", isOn: $note.isLeapMonth)
            .disabled(leap != note.month)
        Text(ProfileFormView.leapNote(year: note.year, month: note.month, leap: leap))
            .font(.caption)
            .foregroundStyle(Theme.inkSecondary)
    }

    private func reminder(_ title: String, offset: Int) -> some View {
        Toggle(title, isOn: Binding(
            get: { note.reminderOffsets.contains(offset) },
            set: { on in
                note.reminderOffsets.removeAll { $0 == offset }
                if on {
                    note.reminderOffsets.append(offset)
                    note.reminderOffsets.sort()
                }
            }
        ))
    }

    private var canSave: Bool {
        !note.title.trimmingCharacters(in: .whitespaces).isEmpty
            && note.occurrence(inYear: note.year, tzOffset: tzOffset) != nil
    }

    private func save() {
        note.title = note.title.trimmingCharacters(in: .whitespaces)
        let saved = note
        Task {
            if !saved.reminderOffsets.isEmpty {
                _ = await NotificationManager.requestAuthorization()
            }
            store.save(saved)
            dismiss()
        }
    }

    /// A `Date` in UTC standing for the solar day of the note, for `DatePicker`.
    private var solarDate: Binding<Date> {
        Binding(
            get: { Calendar.civil.date(from: DateComponents(year: note.year, month: note.month, day: note.day)) ?? .now },
            set: {
                let parts = Calendar.civil.dateComponents([.year, .month, .day], from: $0)
                note.year = parts.year ?? note.year
                note.month = parts.month ?? note.month
                note.day = parts.day ?? note.day
            }
        )
    }

    /// What the card under the date says: the day on the other calendar, the next time the
    /// note falls, and the customs a yearly lunar note follows.
    nonisolated static func preview(_ note: CalendarNote, today: SolarDate, tzOffset: Double) -> [String] {
        guard let first = note.occurrence(inYear: note.year, tzOffset: tzOffset) else {
            return ["Ngày này không có trên lịch."]
        }
        var lines: [String] = []
        switch note.calendarType {
        case .lunar:
            lines.append(String(format: "Dương lịch năm %d: %@, %02d/%02d/%04d", note.year, first.weekdayName, Int(first.day), Int(first.month), Int(first.year)))
        case .solar:
            if let lunar = try? solarToLunar(date: first, tzOffset: tzOffset) {
                lines.append("Âm lịch: \(lunar.day) tháng \(lunar.month)\(lunar.isLeapMonth ? " nhuận" : "") năm \(lunar.year)")
            }
        }
        if let next = note.nextOccurrenceSolarDate(onOrAfter: today, tzOffset: tzOffset) {
            let left = today.days(until: next)
            let countdown = left == 0 ? "hôm nay" : left == 1 ? "ngày mai" : "còn \(left) ngày"
            lines.append(String(format: "Lần tới: %@, %02d/%02d/%04d (%@)", next.weekdayName, Int(next.day), Int(next.month), Int(next.year), countdown))
        } else {
            lines.append("Sự kiện đã qua và không lặp lại.")
        }
        if note.calendarType == .lunar && note.isYearlyRecurring {
            if note.day == 30 {
                lines.append("Năm nào tháng \(note.month) thiếu (29 ngày) thì tính vào ngày 29.")
            }
            lines.append("Năm có tháng \(note.month) nhuận vẫn tính vào tháng \(note.month) thường.")
        }
        return lines
    }
}

private struct NotePreviewCard: View {
    let lines: [String]

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Label("Đối chiếu", systemImage: "arrow.left.arrow.right")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(Theme.inkSecondary)
            ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                Text(line)
                    .font(index == 0 ? .subheadline.weight(.semibold) : .caption)
                    .foregroundStyle(index == 0 ? Theme.cinnabar : Theme.ink)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Theme.paper, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .accessibilityElement(children: .combine)
    }
}
