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

/// One Tử Vi passage: the verse of the book framed, the condition a reader must judge, the
/// modern text, and the note where the data departs from the printed book.
struct PassageView: View {
    let passage: Passage

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SensitiveText(sensitivity: passage.sensitivity) {
                VStack(alignment: .leading, spacing: 8) {
                    if let verse = passage.verse {
                        Text(verse)
                            .font(.subheadline).italic().fontDesign(.serif)
                            .foregroundStyle(Theme.cinnabar)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(Theme.cinnabar.opacity(0.06), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                            .overlay(alignment: .leading) {
                                Rectangle().fill(Theme.cinnabar.opacity(0.6)).frame(width: 2)
                            }
                    }
                    if let condition = passage.conditionText {
                        Label("Nếu " + condition, systemImage: "questionmark.circle")
                            .font(.caption)
                            .foregroundStyle(Theme.inkSecondary)
                    }
                    ForEach(Array(passage.body.enumerated()), id: \.offset) { _, paragraph in
                        Text(paragraph).font(.subheadline).foregroundStyle(Theme.ink)
                    }
                }
            }
            if let correction = passage.correction {
                CorrectionNote(text: correction)
            }
            Text(passage.citation)
                .font(.caption2)
                .foregroundStyle(Theme.inkSecondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

/// A note that the text differs from the printed book, folded to one line until tapped.
struct CorrectionNote: View {
    let text: String
    @State private var expanded = false

    var body: some View {
        Button {
            withAnimation(.snappy) { expanded.toggle() }
        } label: {
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Image(systemName: "pencil.and.list.clipboard")
                VStack(alignment: .leading, spacing: 2) {
                    Text("Đính chính so với bản in").font(.caption.weight(.semibold))
                    Text(text)
                        .font(.caption)
                        .lineLimit(expanded ? nil : 1)
                        .multilineTextAlignment(.leading)
                }
                Spacer(minLength: 0)
                Image(systemName: expanded ? "chevron.up" : "chevron.down").font(.caption2)
            }
            .foregroundStyle(Theme.gold)
            .padding(8)
            .background(Theme.gold.opacity(0.08), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityHint(expanded ? "Thu gọn" : "Mở đầy đủ")
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
