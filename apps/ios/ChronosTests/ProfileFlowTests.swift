import Foundation
import Observation
import Testing
@testable import Chronos

/// Set from an observation callback, read after it.
private final class Flag: @unchecked Sendable {
    var raised = false
}

@MainActor
@Suite("Profiles across the tabs")
struct ProfileFlowTests {
    let services: AppServices

    init() throws {
        services = try AppServices.bundled()
    }

    private func withStore(_ body: (ProfileStore) throws -> Void) rethrows {
        let suite = "ChronosTests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suite)!
        defer { defaults.removePersistentDomain(forName: suite) }
        let store = ProfileStore(defaults: defaults)
        store.save(.sample)
        try body(store)
    }

    /// 10th day of the leap 2nd month of Quý Mão (2023).
    private static let leapBirth = BirthDate(kind: .lunar, year: 2023, month: 2, day: 10, isLeapMonth: true, hour: 14, minute: 0)

    /// What the four tabs that read a person compute for `profile`.
    private func readings(_ profile: Profile) throws -> (age: AgeReading, bazi: BaziReading, tuvi: TuViChart, kua: KuaInfo) {
        (
            try AgeReading(profile: profile, year: 2026, services: services),
            try BaziReading(profile: profile, settings: BaziSettings(), services: services),
            try services.engine.tuviChart(for: profile),
            try services.engine.kua(for: profile)
        )
    }

    @Test func editingTheSelectedProfileNotifiesTheTabsAtOnce() throws {
        try withStore { store in
            var edited = try #require(store.selected)
            let flag = Flag()
            withObservationTracking {
                _ = store.selected
            } onChange: {
                flag.raised = true
            }
            edited.birth = Self.leapBirth
            store.save(edited)
            #expect(flag.raised)
            #expect(store.selected?.birth == Self.leapBirth)
        }
    }

    @Test func everyTabReadsAnEditedProfileOnALeapLunarMonth() throws {
        try withStore { store in
            let before = try readings(try #require(store.selected))
            var edited = try #require(store.selected)
            edited.birth = Self.leapBirth
            store.save(edited)
            let after = try readings(try #require(store.selected))

            #expect(after.age.birthYear == 2023)
            #expect(after.age.birthYear != before.age.birthYear)
            #expect(after.bazi.analysis.pillars != before.bazi.analysis.pillars)
            #expect(after.tuvi.lunar == LunarDate(year: 2023, month: 2, day: 10, isLeapMonth: true))
            #expect(after.kua.birthLunarYear == 2023)
        }
    }

    @Test func deletingTheSelectionMovesTheTabsToTheNextProfile() throws {
        try withStore { store in
            let sample = try #require(store.selected)
            let other = Profile(name: "Lan", gender: .female, birth: Self.leapBirth)
            store.save(other)
            store.delete([sample.id])
            #expect(store.selected == other)
            #expect(try readings(other).kua.gender == .female)
        }
    }

    @Test func switchingCalendarsKeepsTheBirthday() {
        let engine = services.engine
        let solar = Profile.sample.birth
        let lunar = engine.birth(solar, rewrittenAs: .lunar, tzOffset: 7)
        #expect(lunar.kind == .lunar)
        #expect((lunar.year, lunar.month, lunar.day, lunar.isLeapMonth) == (1990, 4, 21, false))
        #expect((lunar.hour, lunar.minute) == (solar.hour, solar.minute))
        #expect(engine.birth(lunar, rewrittenAs: .solar, tzOffset: 7) == solar)

        let leapAsSolar = engine.birth(Self.leapBirth, rewrittenAs: .solar, tzOffset: 7)
        #expect(leapAsSolar.kind == .solar && !leapAsSolar.isLeapMonth)
        #expect(engine.birth(leapAsSolar, rewrittenAs: .lunar, tzOffset: 7) == Self.leapBirth)
    }

    @Test func aLunarDateThatDoesNotExistKeepsItsNumbers() {
        // 2024 has no leap 2nd month; the editor refuses this on saving.
        let missing = BirthDate(kind: .lunar, year: 2024, month: 2, day: 10, isLeapMonth: true, hour: 8, minute: 0)
        let rewritten = services.engine.birth(missing, rewrittenAs: .solar, tzOffset: 7)
        #expect(rewritten.kind == .solar)
        #expect((rewritten.year, rewritten.month, rewritten.day) == (2024, 2, 10))
    }
}
