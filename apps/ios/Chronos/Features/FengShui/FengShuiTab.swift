import SwiftUI

/// Phong Thủy: Mệnh Quái and the Bát Trạch chart of the person, the Nine Stars ruling now, and
/// the Bát Biến match with another profile.
struct FengShuiTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store
    @State private var partnerID: Profile.ID?

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    List {
                        switch Result(catching: { try services.engine.kua(for: profile) }) {
                        case .success(let kua):
                            kuaSection(kua)
                            partnerSection(profile: profile, kua: kua)
                        case .failure(let error):
                            Text(String(describing: error)).foregroundStyle(Theme.cinnabar)
                        }
                        nineStarSection
                    }
                    .chronosBackground()
                } else {
                    NoProfileView()
                }
            }
            .navigationTitle("Phong Thủy")
            .profileToolbar(services)
        }
    }

    @ViewBuilder
    private func kuaSection(_ kua: KuaInfo) -> some View {
        Section("Mệnh Quái") {
            if let entry = try? services.content.kua(kua) {
                EntryView(entry: entry, content: services.content)
            }
        }
        Section("Bát Trạch") {
            ForEach(eightMansionsChart(trigram: kua.trigram), id: \.direction) { item in
                let star = (try? services.content.eightStar(item.star)) ?? nil
                LabeledContent(compassDirectionName(value: item.direction)) {
                    Text(eightMansionsStarName(value: item.star))
                        .foregroundStyle(Theme.luck(star?.luck))
                }
            }
        }
    }

    @ViewBuilder
    private func partnerSection(profile: Profile, kua: KuaInfo) -> some View {
        let others = store.profiles.filter { $0.id != profile.id }
        if !others.isEmpty {
            Section("Tương hợp vợ chồng") {
                Picker("Người kia", selection: $partnerID) {
                    Text("Chọn hồ sơ").tag(Profile.ID?.none)
                    ForEach(others) { Text($0.name).tag(Profile.ID?.some($0.id)) }
                }
                if let partner = others.first(where: { $0.id == partnerID }),
                   let partnerKua = try? services.engine.kua(for: partner) {
                    let compatibility = kuaCompatibility(first: kua, second: partnerKua)
                    ForEach((try? services.content.kuaCompatibility(compatibility)) ?? []) {
                        EntryView(entry: $0, content: services.content)
                    }
                }
            }
        }
    }

    @ViewBuilder
    private var nineStarSection: some View {
        switch Result(catching: { try services.engine.nineStars(at: services.engine.now()) }) {
        case .success(let stars):
            Section("Cửu Tinh (\(nineStarName(value: stars.van)) đương vận)") {
                nineStarRow("Năm", stars.year, period: stars.van)
                nineStarRow("Tháng", stars.month, period: stars.van)
                nineStarRow("Ngày", stars.day.star, period: stars.van)
                nineStarRow("Giờ", stars.hour.star, period: stars.van)
            }
        case .failure(let error):
            Text(String(describing: error)).foregroundStyle(Theme.cinnabar)
        }
    }

    @ViewBuilder
    private func nineStarRow(_ title: String, _ star: NineStar, period: NineStar) -> some View {
        if let reading = try? services.content.nineStar(star, period: period) {
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text("\(title): \(reading.entry.title)").font(.headline).foregroundStyle(Theme.ink)
                    Spacer()
                    Text(reading.phaseName)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(reading.isAuspicious ? Theme.gold : Theme.cinnabar)
                }
                Text(reading.entry.summary).font(.subheadline)
                if let meaning = reading.meaning {
                    Text(meaning).font(.subheadline).foregroundStyle(Theme.inkSecondary)
                }
            }
            .foregroundStyle(Theme.ink)
        }
    }
}
