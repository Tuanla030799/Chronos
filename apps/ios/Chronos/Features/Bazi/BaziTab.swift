import SwiftUI

/// Bát Tự: the Four Pillars with Nạp Âm, Thập Thần and Trường Sinh, then their readings.
struct BaziTab: View {
    let services: AppServices
    @Environment(ProfileStore.self) private var store

    var body: some View {
        NavigationStack {
            Group {
                if let profile = store.selected {
                    switch Result(catching: { try reading(profile) }) {
                    case .success(let reading):
                        List {
                            Section("Tứ Trụ") {
                                PillarGrid(analysis: reading.analysis)
                            }
                            Section("Luận giải") {
                                ForEach(reading.entries) { EntryView(entry: $0, content: services.content) }
                            }
                        }
                        .chronosBackground()
                    case .failure(let error):
                        UnavailableView(title: "Không lập được Tứ Trụ", message: String(describing: error))
                    }
                } else {
                    NoProfileView()
                }
            }
            .navigationTitle("Bát Tự")
            .profileToolbar(services)
        }
    }

    private struct Reading {
        let analysis: BaziAnalysis
        let entries: [ContentEntry]
    }

    private func reading(_ profile: Profile) throws -> Reading {
        let analysis = try services.engine.bazi(for: profile)
        return Reading(analysis: analysis, entries: try services.content.bazi(analysis))
    }
}

/// Hour, Day, Month and Year from left to right, as a printed Bát Tự reads.
private struct PillarGrid: View {
    let analysis: BaziAnalysis

    var body: some View {
        let pillars = [("Giờ", analysis.hour), ("Ngày", analysis.day), ("Tháng", analysis.month), ("Năm", analysis.year)]
        Grid(horizontalSpacing: 8, verticalSpacing: 6) {
            GridRow {
                ForEach(pillars, id: \.0) { Text($0.0).font(.caption).foregroundStyle(Theme.inkSecondary) }
            }
            GridRow {
                ForEach(pillars, id: \.0) { pillar in
                    Text(pillar.1.tenGod.map { tenGodName(value: $0) } ?? "Nhật Chủ")
                        .font(.caption2)
                        .foregroundStyle(Theme.gold)
                }
            }
            GridRow {
                ForEach(pillars, id: \.0) { Text(stemName(stem: $0.1.canChi.stem)).font(.title3.weight(.semibold)) }
            }
            GridRow {
                ForEach(pillars, id: \.0) { Text(branchName(branch: $0.1.canChi.branch)).font(.title3.weight(.semibold)) }
            }
            GridRow {
                ForEach(pillars, id: \.0) { Text(naYinName(value: $0.1.naYin)).font(.caption2).multilineTextAlignment(.center) }
            }
            GridRow {
                ForEach(pillars, id: \.0) { Text(lifeStageName(value: $0.1.lifeStage)).font(.caption2) }
            }
        }
        .foregroundStyle(Theme.ink)
        .frame(maxWidth: .infinity)
    }
}
