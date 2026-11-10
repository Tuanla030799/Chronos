import Foundation
import Observation

/// The saved profiles and the one every tab reads. Stored as JSON in `UserDefaults`, which a
/// Widget can later share through an App Group suite.
@MainActor
@Observable
final class ProfileStore {
    static let storageKey = "chronos.profiles.v1"

    private(set) var profiles: [Profile]
    private(set) var selectedID: Profile.ID?

    @ObservationIgnored private let defaults: UserDefaults

    private struct Stored: Codable {
        var profiles: [Profile]
        var selectedID: Profile.ID?
    }

    /// `seed` is saved on the very first launch only, so every tab has a chart to show. Once
    /// anything is stored, even an empty list after the user deleted everything, it is not
    /// added again.
    init(defaults: UserDefaults = .standard, seed: Profile? = .sample) {
        self.defaults = defaults
        guard let data = defaults.data(forKey: Self.storageKey) else {
            profiles = seed.map { [$0] } ?? []
            selectedID = seed?.id
            if seed != nil {
                persist()
            }
            return
        }
        let stored = try? JSONDecoder().decode(Stored.self, from: data)
        profiles = stored?.profiles ?? []
        selectedID = stored?.selectedID ?? stored?.profiles.first?.id
    }

    var selected: Profile? {
        profiles.first { $0.id == selectedID }
    }

    /// Adds `profile`, or replaces the one with its id. A first profile becomes the selection.
    func save(_ profile: Profile) {
        if let index = profiles.firstIndex(where: { $0.id == profile.id }) {
            profiles[index] = profile
        } else {
            profiles.append(profile)
        }
        if selected == nil {
            selectedID = profile.id
        }
        persist()
    }

    func delete(_ ids: Set<Profile.ID>) {
        profiles.removeAll { ids.contains($0.id) }
        if selected == nil {
            selectedID = profiles.first?.id
        }
        persist()
    }

    func select(_ id: Profile.ID) {
        guard profiles.contains(where: { $0.id == id }) else { return }
        selectedID = id
        persist()
    }

    private func persist() {
        let stored = Stored(profiles: profiles, selectedID: selectedID)
        guard let data = try? JSONEncoder().encode(stored) else { return }
        defaults.set(data, forKey: Self.storageKey)
    }
}
