import Foundation
import Observation
import WidgetKit

/// The saved notes and events. Stored in the App Group so the widget reads them; every change
/// reloads the widget and reschedules the reminders.
@MainActor
@Observable
final class NoteStore {
    private(set) var notes: [CalendarNote]

    @ObservationIgnored private let defaults: UserDefaults
    @ObservationIgnored private let onChange: @MainActor ([CalendarNote]) -> Void

    init(
        defaults: UserDefaults = NoteStorage.shared,
        onChange: @escaping @MainActor ([CalendarNote]) -> Void = NoteStore.broadcast
    ) {
        self.defaults = defaults
        self.onChange = onChange
        notes = NoteStorage.load(from: defaults)
    }

    /// Adds `note`, or replaces the one with its id.
    func save(_ note: CalendarNote) {
        if let index = notes.firstIndex(where: { $0.id == note.id }) {
            notes[index] = note
        } else {
            notes.append(note)
        }
        persist()
    }

    func delete(_ id: CalendarNote.ID) {
        notes.removeAll { $0.id == id }
        persist()
    }

    private func persist() {
        NoteStorage.save(notes, to: defaults)
        onChange(notes)
    }

    /// Tells the widget and the notification center that the notes changed.
    static func broadcast(_ notes: [CalendarNote]) {
        WidgetCenter.shared.reloadAllTimelines()
        Task { await NotificationManager.sync(notes) }
    }
}
