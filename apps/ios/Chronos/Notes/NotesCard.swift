import SwiftUI

/// Under the day sheet: the notes of the chosen day, a button to add one on that day, and what
/// is coming in the next 30 days with the days left.
struct NotesCard: View {
    let selected: SolarDate
    let today: SolarDate
    let engine: EngineService
    @Environment(NoteStore.self) private var store
    @State private var draft: NoteDraft?

    static let horizonDays = 30

    var body: some View {
        let tzOffset = engine.tzOffset
        let dayNotes = NoteCalendar.notes(store.notes, on: selected, tzOffset: tzOffset)
        let upcoming = NoteCalendar.upcoming(store.notes, from: today, days: Self.horizonDays, tzOffset: tzOffset)
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Ghi chú & Sự kiện", systemImage: "note.text")

            if dayNotes.isEmpty {
                Text("Ngày \(selected.day)/\(selected.month) chưa có ghi chú.")
                    .font(.subheadline).foregroundStyle(Theme.inkSecondary)
            } else {
                ForEach(dayNotes) { note in
                    row(note, trailing: nil)
                }
            }

            Button {
                draft = NoteDraft(note: .draft(on: selected, tzOffset: tzOffset), isNew: true)
            } label: {
                Label("Thêm sự kiện / ngày giỗ", systemImage: "plus")
                    .font(.subheadline.weight(.semibold))
                    .foregroundStyle(Theme.cinnabar)
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 10)
                    .background(Theme.cinnabar.opacity(0.1), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
            }
            .buttonStyle(.plain)

            Divider().overlay(Theme.rule)

            Text("SỰ KIỆN SẮP TỚI (\(Self.horizonDays) NGÀY TỚI)")
                .font(.caption2.weight(.semibold)).tracking(0.6)
                .foregroundStyle(Theme.inkSecondary)
            if upcoming.isEmpty {
                Text("Không có sự kiện nào trong \(Self.horizonDays) ngày tới.")
                    .font(.subheadline).foregroundStyle(Theme.inkSecondary)
            } else {
                ForEach(upcoming) { event in
                    row(event.note, trailing: event.countdown)
                }
            }
        }
        .card()
        .sheet(item: $draft) { draft in
            NoteFormView(note: draft.note, isNew: draft.isNew, engine: engine)
        }
    }

    /// "Giỗ Ông Nội · 15/10 Âm lịch", with the days left on the right; opens the note.
    private func row(_ note: CalendarNote, trailing: String?) -> some View {
        Button {
            draft = NoteDraft(note: note, isNew: false)
        } label: {
            HStack(alignment: .center, spacing: 10) {
                Image(systemName: note.category.systemImage)
                    .foregroundStyle(Theme.cinnabar)
                    .frame(width: 20)
                VStack(alignment: .leading, spacing: 2) {
                    Text(note.title)
                        .font(.subheadline.weight(.semibold)).foregroundStyle(Theme.ink)
                        .multilineTextAlignment(.leading)
                    Text("\(note.category.label) · \(note.dateLabel)")
                        .font(.caption).foregroundStyle(Theme.inkSecondary)
                    if !note.notes.isEmpty {
                        Text(note.notes).font(.caption).foregroundStyle(Theme.inkSecondary).lineLimit(2)
                            .multilineTextAlignment(.leading)
                    }
                }
                Spacer(minLength: 4)
                if let trailing {
                    Chip(text: trailing, color: Theme.gold)
                }
                Image(systemName: "chevron.right").font(.caption).foregroundStyle(Theme.inkSecondary)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("\(note.title) · \(note.dateLabel)\(trailing.map { " (\($0))" } ?? "")")
        .accessibilityHint("Mở để sửa")
    }
}
