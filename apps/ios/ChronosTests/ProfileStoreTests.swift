import Foundation
import Testing
@testable import Chronos

@MainActor
@Suite("ProfileStore")
struct ProfileStoreTests {
    private func withDefaults(_ body: (UserDefaults) throws -> Void) rethrows {
        let suite = "ChronosTests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suite)!
        defer { defaults.removePersistentDomain(forName: suite) }
        try body(defaults)
    }

    private func person(_ name: String, kind: BirthDate.Kind = .solar) -> Profile {
        Profile(
            name: name,
            gender: .female,
            birth: BirthDate(kind: kind, year: 1992, month: 3, day: 4, isLeapMonth: false, hour: 5, minute: 6),
            longitude: 105.85
        )
    }

    @Test func theFirstProfileBecomesTheSelection() {
        withDefaults { defaults in
            let store = ProfileStore(defaults: defaults)
            #expect(store.selected == nil)
            let lan = person("Lan")
            store.save(lan)
            store.save(person("Hoa"))
            #expect(store.selected == lan)
            #expect(store.profiles.count == 2)
        }
    }

    @Test func profilesAndSelectionSurviveARelaunch() {
        withDefaults { defaults in
            let store = ProfileStore(defaults: defaults)
            let lan = person("Lan", kind: .lunar)
            let hoa = person("Hoa")
            store.save(lan)
            store.save(hoa)
            store.select(hoa.id)

            let reopened = ProfileStore(defaults: defaults)
            #expect(reopened.profiles == [lan, hoa])
            #expect(reopened.selected == hoa)
        }
    }

    @Test func savingAnExistingIdReplacesIt() {
        withDefaults { defaults in
            let store = ProfileStore(defaults: defaults)
            var lan = person("Lan")
            store.save(lan)
            lan.name = "Lan Anh"
            store.save(lan)
            #expect(store.profiles.map(\.name) == ["Lan Anh"])
        }
    }

    @Test func deletingTheSelectionFallsBackToTheFirstProfile() {
        withDefaults { defaults in
            let store = ProfileStore(defaults: defaults)
            let lan = person("Lan")
            let hoa = person("Hoa")
            store.save(lan)
            store.save(hoa)
            store.delete([lan.id])
            #expect(store.selected == hoa)
            store.delete([hoa.id])
            #expect(store.selected == nil)
            #expect(ProfileStore(defaults: defaults).profiles.isEmpty)
        }
    }

    @Test func selectingAnUnknownIdChangesNothing() {
        withDefaults { defaults in
            let store = ProfileStore(defaults: defaults)
            let lan = person("Lan")
            store.save(lan)
            store.select(UUID())
            #expect(store.selected == lan)
        }
    }
}
