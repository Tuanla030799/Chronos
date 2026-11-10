import SwiftUI

/// Bát Biến between two people: pick another profile, or type a lunar birth year and gender.
struct KuaCompatibilityCard: View {
    let services: AppServices
    let profile: Profile
    let kua: KuaInfo
    let open: (FengShuiSheet) -> Void

    private enum Source: String, CaseIterable, Identifiable {
        case profile = "Hồ sơ"
        case quick = "Nhập nhanh"

        var id: Self { self }
    }

    @Environment(ProfileStore.self) private var store
    @State private var chosenSource: Source?
    @State private var partnerID: Profile.ID?
    @State private var quickYear = 1992
    @State private var quickGender = ProfileGender.female

    private var others: [Profile] {
        store.profiles.filter { $0.id != profile.id }
    }

    private var source: Source {
        chosenSource ?? (others.isEmpty ? .quick : .profile)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            CardTitle(text: "Tương hợp Bát Biến", systemImage: "heart.circle")

            if !others.isEmpty {
                Picker("Đối tác", selection: Binding(get: { source }, set: { chosenSource = $0 })) {
                    ForEach(Source.allCases) { Text($0.rawValue).tag($0) }
                }
                .pickerStyle(.segmented)
            }

            switch source {
            case .profile:
                Picker("Người kia", selection: Binding(
                    get: { partnerID ?? others.first?.id },
                    set: { partnerID = $0 }
                )) {
                    ForEach(others) { Text($0.name).tag(Profile.ID?.some($0.id)) }
                }
                .tint(Theme.cinnabar)
            case .quick:
                Stepper(value: $quickYear, in: EngineService.supportedYears) {
                    Text("Năm sinh âm lịch \(String(quickYear))").foregroundStyle(Theme.ink)
                }
                Picker("Giới tính", selection: $quickGender) {
                    ForEach(ProfileGender.allCases) { Text($0.label).tag($0) }
                }
                .pickerStyle(.segmented)
            }

            if let partner = partnerKua {
                result(partner)
            }
        }
        .card()
    }

    private var partnerKua: KuaInfo? {
        switch source {
        case .profile:
            guard let partner = others.first(where: { $0.id == partnerID }) ?? others.first else { return nil }
            return try? services.engine.kua(for: partner)
        case .quick:
            return kuaInfo(birthLunarYear: Int32(quickYear), gender: quickGender.engineValue)
        }
    }

    @ViewBuilder
    private func result(_ partner: KuaInfo) -> some View {
        let compatibility = kuaCompatibility(first: kua, second: partner)
        let good = compatibility.luck == .cat
        let color = good ? Theme.gold : Theme.cinnabar

        Divider().overlay(Theme.rule)

        HStack(alignment: .center, spacing: 8) {
            person("Bạn", kua)
            VStack(spacing: 4) {
                Image(systemName: "arrow.left.and.right").foregroundStyle(Theme.inkSecondary)
                Text(eightMansionsStarName(value: compatibility.star))
                    .font(.headline).fontDesign(.serif).foregroundStyle(color)
                Chip(text: good ? "Cát" : "Hung", color: color)
            }
            .frame(maxWidth: .infinity)
            person("Đối tác", partner)
        }

        let entries = (try? services.content.kuaCompatibility(compatibility)) ?? []
        ForEach(entries) { entry in
            EntryView(entry: entry, content: services.content)
            Divider().overlay(Theme.rule)
        }
        Button("Xem sao \(eightMansionsStarName(value: compatibility.star)) cho hướng nhà, bếp, bàn làm việc") {
            open(.eightStar(compatibility.star, nil))
        }
        .font(.subheadline.weight(.semibold))
        .foregroundStyle(Theme.cinnabar)
    }

    private func person(_ title: String, _ kua: KuaInfo) -> some View {
        VStack(spacing: 2) {
            Text(title).font(.caption2).foregroundStyle(Theme.inkSecondary)
            Text(trigramName(value: kua.trigram))
                .font(.title2.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
            ElementBadge(element: trigramElement(value: kua.trigram))
            Text("\(trigramGroupName(value: kua.group)) Mệnh").font(.caption2).foregroundStyle(Theme.inkSecondary)
        }
        .frame(maxWidth: .infinity)
        .accessibilityElement(children: .combine)
    }
}
