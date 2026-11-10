import SwiftUI

extension View {
    /// A paper card with a thin rule border.
    func card(padding: CGFloat = 16) -> some View {
        self.padding(padding)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Theme.surface, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
            .overlay(RoundedRectangle(cornerRadius: 16, style: .continuous).strokeBorder(Theme.rule, lineWidth: 1))
    }
}

/// Small heading above a card.
struct CardTitle: View {
    let text: String
    var systemImage: String?

    var body: some View {
        Label {
            Text(text.uppercased())
        } icon: {
            if let systemImage {
                Image(systemName: systemImage)
            }
        }
        .font(.caption.weight(.semibold))
        .tracking(0.6)
        .foregroundStyle(Theme.inkSecondary)
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// A rounded label, tinted by its color.
struct Chip: View {
    let text: String
    var color: Color = Theme.inkSecondary

    var body: some View {
        Text(text)
            .font(.caption.weight(.medium))
            .foregroundStyle(color)
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .background(color.opacity(0.12), in: Capsule())
    }
}

/// Name and birth of the person a tab reads, with a menu to switch to another profile.
struct ProfileHeaderRow: View {
    let profile: Profile
    @Environment(ProfileStore.self) private var store

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "person.crop.circle.fill")
                .font(.largeTitle)
                .foregroundStyle(Theme.cinnabar)
            VStack(alignment: .leading, spacing: 2) {
                Text(profile.name).font(.headline).foregroundStyle(Theme.ink)
                Text(profile.summary).font(.caption).foregroundStyle(Theme.inkSecondary)
            }
            Spacer(minLength: 0)
            if store.profiles.count > 1 {
                Menu {
                    ForEach(store.profiles) { other in
                        Button {
                            store.select(other.id)
                        } label: {
                            if other.id == profile.id {
                                Label(other.name, systemImage: "checkmark")
                            } else {
                                Text(other.name)
                            }
                        }
                    }
                } label: {
                    Label("Đổi hồ sơ", systemImage: "arrow.left.arrow.right")
                        .labelStyle(.iconOnly)
                        .font(.body.weight(.semibold))
                        .frame(width: 36, height: 36)
                        .background(Theme.paper, in: Circle())
                }
                .accessibilityLabel("Đổi hồ sơ")
            }
        }
    }
}

/// The Ngũ Hành of a Can, Chi or Nạp Âm: its name in its color, never the color alone.
struct ElementBadge: View {
    let element: FiveElements

    var body: some View {
        Text(fiveElementsName(value: element))
            .font(.system(size: 10, weight: .semibold))
            .foregroundStyle(Theme.element(element))
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(Theme.element(element).opacity(0.12), in: Capsule())
    }
}

/// Lays children out in rows, wrapping to the next row when one is full.
struct FlowLayout: Layout {
    var spacing: CGFloat = 6

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let rows = rows(width: proposal.width ?? .infinity, subviews: subviews)
        let height = rows.map(\.height).reduce(0, +) + spacing * CGFloat(max(rows.count - 1, 0))
        let width = rows.map(\.width).max() ?? 0
        return CGSize(width: proposal.width ?? width, height: height)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        var y = bounds.minY
        for row in rows(width: bounds.width, subviews: subviews) {
            var x = bounds.minX
            for index in row.indices {
                let size = subviews[index].sizeThatFits(.unspecified)
                subviews[index].place(at: CGPoint(x: x, y: y), proposal: ProposedViewSize(size))
                x += size.width + spacing
            }
            y += row.height + spacing
        }
    }

    private struct Row {
        var indices: [Int] = []
        var width: CGFloat = 0
        var height: CGFloat = 0
    }

    private func rows(width: CGFloat, subviews: Subviews) -> [Row] {
        var rows: [Row] = []
        var current = Row()
        for index in subviews.indices {
            let size = subviews[index].sizeThatFits(.unspecified)
            let needed = current.indices.isEmpty ? size.width : current.width + spacing + size.width
            if needed > width, !current.indices.isEmpty {
                rows.append(current)
                current = Row()
            }
            current.width = current.indices.isEmpty ? size.width : current.width + spacing + size.width
            current.height = max(current.height, size.height)
            current.indices.append(index)
        }
        if !current.indices.isEmpty {
            rows.append(current)
        }
        return rows
    }
}

/// The full text of one entry, opened from a row.
struct EntrySheet: View {
    let entry: ContentEntry
    let content: ContentRepository
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                EntryView(entry: entry, content: content)
                    .card()
                    .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle(entry.title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Đóng") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium, .large])
    }
}

/// One tappable row: a luck dot, the title and the one-line summary.
struct EntryRow: View {
    let entry: ContentEntry
    var luck: ContentLuck?
    let open: () -> Void

    var body: some View {
        Button(action: open) {
            HStack(alignment: .top, spacing: 12) {
                Circle()
                    .fill(Theme.luck(luck ?? entry.effectiveLuck))
                    .frame(width: 8, height: 8)
                    .padding(.top, 6)
                VStack(alignment: .leading, spacing: 2) {
                    Text(entry.title).font(.subheadline.weight(.semibold)).foregroundStyle(Theme.ink)
                    Text(entry.summary).font(.caption).foregroundStyle(Theme.inkSecondary).lineLimit(2)
                        .multilineTextAlignment(.leading)
                }
                Spacer(minLength: 0)
                Image(systemName: "chevron.right").font(.caption).foregroundStyle(Theme.inkSecondary)
                    .padding(.top, 4)
            }
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}
