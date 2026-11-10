import SwiftUI

/// One interpretation entry: title, luck, summary, body and the fields of `extra`.
struct EntryView: View {
    let entry: ContentEntry
    let content: ContentRepository

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(entry.title)
                    .font(.headline)
                    .foregroundStyle(Theme.ink)
                Spacer()
                if let luck = entry.effectiveLuck {
                    Text(luck.label)
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(Theme.luck(luck))
                }
            }
            SensitiveText(sensitivity: entry.sensitivity) {
                VStack(alignment: .leading, spacing: 6) {
                    Text(entry.summary)
                    ForEach(Array(entry.body.enumerated()), id: \.offset) { _, paragraph in
                        Text(paragraph)
                    }
                    if let image = entry.extra.image {
                        Text(image).italic()
                    }
                    ForEach(Array((entry.extra.traits + entry.extra.fortune).enumerated()), id: \.offset) { _, line in
                        Text("• " + line)
                    }
                    if let verse = entry.extra.verse {
                        Text(verse).italic().foregroundStyle(Theme.inkSecondary)
                    }
                    activities("Nên", entry.extra.goodFor, color: Theme.gold)
                    activities("Kỵ", entry.extra.avoid, color: Theme.cinnabar)
                }
                .font(.subheadline)
                .foregroundStyle(Theme.ink)
            }
            Text(entry.citation)
                .font(.caption2)
                .foregroundStyle(Theme.inkSecondary)
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private func activities(_ title: String, _ items: [ActivityRef], color: Color) -> some View {
        if !items.isEmpty {
            let labels = items.map { content.activityLabel($0.activity) }.joined(separator: ", ")
            Text("\(Text(title + ":").bold().foregroundStyle(color)) \(labels)")
        }
    }
}

/// One Tử Vi passage, with its verse and the condition a reader must judge.
struct PassageView: View {
    let passage: Passage

    var body: some View {
        SensitiveText(sensitivity: passage.sensitivity) {
            VStack(alignment: .leading, spacing: 6) {
                if let verse = passage.verse {
                    Text(verse).italic().foregroundStyle(Theme.cinnabar)
                }
                if let condition = passage.conditionText {
                    Text("Nếu " + condition)
                        .font(.caption)
                        .foregroundStyle(Theme.inkSecondary)
                }
                ForEach(Array(passage.body.enumerated()), id: \.offset) { _, paragraph in
                    Text(paragraph)
                }
            }
            .font(.subheadline)
            .foregroundStyle(Theme.ink)
        }
        Text(passage.citation)
            .font(.caption2)
            .foregroundStyle(Theme.inkSecondary)
    }
}

/// Decision D9: text tagged Death or Sexuality stays folded until the reader taps it.
struct SensitiveText<Content: View>: View {
    let sensitivity: Sensitivity
    @ViewBuilder let content: Content
    @State private var revealed = false

    var body: some View {
        if sensitivity.isHiddenByDefault && !revealed {
            Button {
                revealed = true
            } label: {
                VStack(alignment: .leading, spacing: 4) {
                    Label("Nội dung nhạy cảm, chạm để xem", systemImage: "eye.slash")
                        .font(.subheadline.weight(.medium))
                    Text("Đây là tư liệu tham khảo văn hóa, không phải lời khuyên y tế hay pháp lý.")
                        .font(.caption)
                }
                .foregroundStyle(Theme.inkSecondary)
            }
            .buttonStyle(.plain)
        } else {
            content
        }
    }
}

/// A message when a tab cannot draw: no profile yet, or an engine error.
struct UnavailableView: View {
    let title: String
    let message: String
    var systemImage = "exclamationmark.triangle"

    var body: some View {
        ContentUnavailableView(title, systemImage: systemImage, description: Text(message))
            .foregroundStyle(Theme.inkSecondary)
    }
}
