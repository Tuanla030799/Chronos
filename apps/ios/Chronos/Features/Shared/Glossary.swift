import SwiftUI

/// A short explanation of the terms or marks of one screen, for a reader new to them.
struct GlossaryTopic: Identifiable, Hashable {
    struct Item: Hashable {
        let term: String
        let text: String
    }

    let title: String
    var intro: String?
    let items: [Item]

    var id: String { title }
}

/// Every help text of the app, in one place. They explain what the screen shows and name the
/// engine's conventions; they add no reading of a chart.
enum Glossary {
    // MARK: Lịch

    static let calendarMarks = GlossaryTopic(
        title: "Ký hiệu trên lịch",
        items: [
            .init(term: "Chấm vàng", text: "Ngày Hoàng Đạo: thần trực nhật của ngày thuộc nhóm tốt."),
            .init(term: "Chấm đỏ", text: "Ngày Hắc Đạo: thần trực nhật của ngày thuộc nhóm xấu."),
            .init(term: "Số nhỏ dưới ngày", text: "Ngày âm lịch. Mùng 1 ghi cả tháng, ví dụ 1/10; chữ n là tháng nhuận."),
            .init(term: "Dấu trang ở góc trên", text: "Ngày có ghi chú hoặc sự kiện của bạn, kể cả ngày giỗ lặp lại hằng năm."),
            .init(term: "Giờ Hoàng Đạo", text: "Sáu canh giờ tốt trong ngày, mỗi canh hai tiếng. Giờ Tý kéo từ 23h đến 1h."),
        ]
    )

    // MARK: Xem tuổi

    static let tamTaiNote = "Hạn ba năm liên tiếp, nên thận trọng khi làm việc lớn."
    static let kimLauNote = "Chủ yếu xem khi động thổ, xây nhà hoặc cưới hỏi."
    static let hoangOcNote = "Chủ yếu xem khi động thổ, xây nhà hoặc cưới hỏi."
    static let taiSuiNote = "Quan hệ xung, hợp giữa Địa Chi của tuổi và Địa Chi của năm đang xem."

    // MARK: Bát Tự

    static let baziBoundaryNote = "Tiết khí: năm đổi tại Lập Xuân, tháng đổi tại mỗi Tiết. Mùng 1 âm lịch: năm đổi tại Tết, tháng đổi theo tháng âm."
    static let ratHourNote = "Giờ Tý kéo từ 23h đến 1h. Chia nửa đêm: sinh lúc 23h vẫn thuộc ngày hôm đó (Tý sớm). Sang ngày từ 23h: sinh lúc 23h tính sang ngày hôm sau."

    static let baziTerms = GlossaryTopic(
        title: "Thuật ngữ Bát Tự",
        intro: "Bát Tự (Tứ Trụ) ghi năm, tháng, ngày, giờ sinh bằng bốn cặp Thiên Can và Địa Chi.",
        items: [
            .init(term: "Nhật Chủ", text: "Thiên Can của ngày sinh, đại diện cho chính mình. Các Can khác được đọc theo quan hệ với nó."),
            .init(term: "Thập Thần", text: "Tên quan hệ giữa một Thiên Can và Nhật Chủ, ví dụ Tỷ Kiên, Chính Quan."),
            .init(term: "Tàng Can", text: "Các Thiên Can ẩn trong một Địa Chi: Bản khí, rồi Trung khí và Dư khí nếu có."),
            .init(term: "Trường Sinh", text: "Mười hai giai đoạn của Nhật Chủ tại Địa Chi của từng trụ, từ Trường Sinh đến Dưỡng."),
            .init(term: "Nạp Âm", text: "Ngũ hành của cả cặp Can Chi. Nạp Âm của trụ năm thường gọi là bản mệnh."),
            .init(term: "Tuần Không Vong", text: "Hai Địa Chi trống của một tuần Giáp mười ngày, tính từ trụ Ngày và từ trụ Năm. Trụ có Địa Chi trùng được ghi Không."),
            .init(term: "Đại Vận", text: "Vận trình đổi mỗi mười năm, mỗi bước mang một cặp Can Chi tính từ trụ tháng."),
        ]
    )

    // MARK: Tử Vi

    static let leapMonthNote = "Dành cho người sinh tháng nhuận. Chia ở ngày 15: mùng 1 đến 15 tính theo tháng trước, từ 16 tính theo tháng sau. Như tháng trước: cả tháng nhuận tính như tháng mà nó lặp lại."

    static let tuviSymbols = GlossaryTopic(
        title: "Chú giải ký hiệu",
        intro: "Chữ trong ngoặc sau tên sao là độ sáng của sao tại cung đó.",
        items: [
            .init(term: "(M) Miếu địa", text: "Rất tốt."),
            .init(term: "(V) Vượng địa", text: "Tốt."),
            .init(term: "(Đ) Đắc địa", text: "Khá."),
            .init(term: "(B) Bình hòa", text: "Trung bình."),
            .init(term: "(H) Hãm địa", text: "Bất lợi, thử thách."),
            .init(term: "<THÂN>", text: "Cung an Thân, ảnh hưởng mạnh từ trung vận."),
            .init(term: "ĐH", text: "Cung Đại hạn mười năm chứa tuổi của năm đang xem."),
            .init(term: "TH", text: "Cung Tiểu hạn của năm đang xem."),
            .init(term: "TUẦN, TRIỆT", text: "Tuần Trung Không Vong và Triệt Lộ Không Vong đóng tại cung."),
            .init(term: "Viền vàng, tam giác", text: "Hai cung Tam Hợp với cung đang chọn."),
            .init(term: "Viền đứt, đường đứt", text: "Cung Xung Chiếu, đối diện cung đang chọn."),
            .init(term: "Số ở góc trên", text: "Tuổi bắt đầu Đại hạn của cung."),
            .init(term: "Cát, Hung", text: "Số phụ tinh tốt và xấu trong cung."),
            .init(term: "Dòng cuối ô", text: "Vòng Trường Sinh của cung, và năm có Tiểu hạn đến cung."),
        ]
    )

    // MARK: Phong Thủy

    /// The two groups of trigrams, listed from the engine.
    static var kuaGroups: GlossaryTopic {
        func names(_ group: TrigramGroup) -> String {
            Trigram.allCases.filter { trigramGroup(value: $0) == group }.map { trigramName(value: $0) }.joined(separator: ", ")
        }
        return GlossaryTopic(
            title: "Đông Tứ Mệnh, Tây Tứ Mệnh",
            intro: "Mệnh Quái tính từ năm sinh âm lịch và giới tính. Tám quái chia hai nhóm.",
            items: [
                .init(term: "Đông Tứ Mệnh", text: "Gồm các quái \(names(.dongTu))."),
                .init(term: "Tây Tứ Mệnh", text: "Gồm các quái \(names(.tayTu))."),
                .init(term: "Hướng tốt", text: "Bốn hướng tốt của một người là hướng của bốn quái cùng nhóm với Mệnh Quái; bốn hướng của nhóm kia là hướng xấu."),
            ]
        )
    }

    static let nineStarPhases = GlossaryTopic(
        title: "Trạng thái của Cửu Tinh",
        intro: "Mỗi Vận dài hai mươi năm do một sao làm chủ. Trạng thái của một sao tính theo vị trí của nó so với sao chủ Vận.",
        items: [
            .init(term: "Đương vận", text: "Sao đang làm chủ Vận."),
            .init(term: "Sinh khí", text: "Sao kế tiếp sao chủ Vận."),
            .init(term: "Thoái khí", text: "Sao liền trước sao chủ Vận."),
            .init(term: "Thất vận", text: "Các sao còn lại. Sao thất vận được đọc theo nghĩa xấu của nó; ba trạng thái trên đọc theo nghĩa tốt."),
        ]
    )
}

/// The ⓘ button: opens the explanation of a topic.
struct InfoButton: View {
    let topic: GlossaryTopic
    /// Shown beside the icon when set.
    var label: String?
    @State private var showing = false

    var body: some View {
        Button {
            showing = true
        } label: {
            HStack(spacing: 4) {
                Image(systemName: "info.circle")
                if let label {
                    Text(label)
                }
            }
            .font(.caption.weight(.semibold))
            .foregroundStyle(Theme.cinnabar)
            .frame(minWidth: 28, minHeight: 28)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityLabel(label ?? "Giải thích: \(topic.title)")
        .sheet(isPresented: $showing) {
            GlossarySheet(topic: topic)
        }
    }
}

struct GlossarySheet: View {
    let topic: GlossaryTopic
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    if let intro = topic.intro {
                        Text(intro).font(.subheadline).foregroundStyle(Theme.inkSecondary)
                    }
                    ForEach(topic.items, id: \.self) { item in
                        VStack(alignment: .leading, spacing: 2) {
                            Text(item.term).font(.subheadline.weight(.bold)).fontDesign(.serif).foregroundStyle(Theme.ink)
                            Text(item.text).font(.subheadline).foregroundStyle(Theme.ink)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                        if item != topic.items.last {
                            Divider().overlay(Theme.rule)
                        }
                    }
                }
                .card()
                .padding(16)
            }
            .background(Theme.paper)
            .navigationTitle(topic.title)
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

/// One line of help under a setting or a figure.
struct HelpNote: View {
    let text: String

    var body: some View {
        Text(text)
            .font(.caption2)
            .foregroundStyle(Theme.inkSecondary)
            .fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}
