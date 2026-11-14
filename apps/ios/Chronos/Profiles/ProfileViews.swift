import SwiftUI

/// All saved profiles: pick one, add, edit or delete.
struct ProfileListView: View {
    let engine: EngineService
    @Environment(ProfileStore.self) private var store
    @Environment(\.dismiss) private var dismiss
    @State private var editing: Profile?

    var body: some View {
        NavigationStack {
            List {
                ForEach(store.profiles) { profile in
                    HStack {
                        Button {
                            store.select(profile.id)
                        } label: {
                            VStack(alignment: .leading) {
                                Text(profile.name).foregroundStyle(Theme.ink)
                                Text(profile.summary).font(.caption).foregroundStyle(Theme.inkSecondary)
                            }
                        }
                        Spacer()
                        if profile.id == store.selectedID {
                            Image(systemName: "checkmark").foregroundStyle(Theme.cinnabar)
                        }
                        Button("Sửa", systemImage: "pencil") { editing = profile }
                            .labelStyle(.iconOnly)
                            .buttonStyle(.borderless)
                    }
                    .listRowBackground(Theme.surface)
                }
                .onDelete { offsets in
                    store.delete(Set(offsets.map { store.profiles[$0].id }))
                }
            }
            .chronosBackground()
            .overlay {
                if store.profiles.isEmpty {
                    UnavailableView(title: "Chưa có hồ sơ", message: "Thêm hồ sơ đầu tiên bằng nút +.", systemImage: "person.crop.circle.badge.plus")
                }
            }
            .navigationTitle("Hồ sơ")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Xong") { dismiss() }
                }
                ToolbarItem(placement: .topBarLeading) {
                    Button("Thêm", systemImage: "plus") { editing = Self.newProfile(engine: engine) }
                }
            }
            .sheet(item: $editing) { profile in
                ProfileFormView(profile: profile, engine: engine)
            }
        }
    }

    static func newProfile(engine: EngineService) -> Profile {
        let today = engine.today()
        return Profile(
            name: "",
            gender: .male,
            birth: BirthDate(kind: .solar, year: Int(today.year) - 30, month: 1, day: 1, hour: 12, minute: 0)
        )
    }
}

/// Form for one profile. The card under the birth fields shows the same birth on the other
/// calendar as the fields change. Saving asks the engine to resolve the birth date first, so
/// an impossible date is refused here.
struct ProfileFormView: View {
    @State var profile: Profile
    let engine: EngineService
    @Environment(ProfileStore.self) private var store
    @Environment(\.dismiss) private var dismiss
    @State private var error: String?

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Họ tên", text: $profile.name)
                    Picker("Giới tính", selection: $profile.gender) {
                        ForEach(ProfileGender.allCases) { Text($0.label).tag($0) }
                    }
                    .pickerStyle(.segmented)
                }
                .listRowBackground(Theme.surface)

                Section {
                    Picker("Lịch", selection: $profile.birth.kind) {
                        ForEach(BirthDate.Kind.allCases) { Text($0.label).tag($0) }
                    }
                    .pickerStyle(.segmented)
                    switch profile.birth.kind {
                    case .solar:
                        DatePicker("Ngày", selection: utcDate(\.solarDay), displayedComponents: .date)
                    case .lunar:
                        lunarFields
                    }
                    DatePicker("Giờ", selection: utcDate(\.clock), displayedComponents: .hourAndMinute)
                    BirthPreviewCard(preview: Result { try engine.birthPreview(for: profile) })
                } header: {
                    Text("Ngày giờ sinh")
                }
                .listRowBackground(Theme.surface)

                Section {
                    Stepper(value: $profile.tzOffset, in: -12...14, step: 0.5) {
                        Text("Múi giờ GMT\(profile.tzOffset >= 0 ? "+" : "")\(profile.tzOffset.formatted())")
                    }
                    Toggle("Dùng giờ Mặt Trời thực", isOn: usesLongitude)
                    if let longitude = profile.longitude {
                        Menu {
                            ForEach(Birthplace.common) { place in
                                Button("\(place.name) (\(place.longitude.formatted())°Đ)") { profile.longitude = place.longitude }
                            }
                        } label: {
                            LabeledContent("Nơi sinh") {
                                Text(Birthplace.common.first { $0.longitude == longitude }?.name ?? "Chọn tỉnh, thành")
                            }
                        }
                        TextField("Kinh độ nơi sinh (độ Đông)", value: Binding(
                            get: { longitude },
                            set: { profile.longitude = $0 }
                        ), format: .number.precision(.fractionLength(0...4)))
                        .keyboardType(.decimalPad)
                    }
                } footer: {
                    Text("Đồng hồ Việt Nam chạy theo kinh tuyến 105° Đông. Giờ Mặt Trời thực chỉnh giờ sinh theo kinh độ nơi sinh và phương trình thời gian, nên có thể lệch giờ đồng hồ; nó chỉ làm đổi lá số khi giờ sinh sát ranh giới hai canh giờ. Dùng cho Bát Tự và Tử Vi. Chọn nhanh một tỉnh, thành hoặc nhập kinh độ.")
                }
                .listRowBackground(Theme.surface)
            }
            .chronosBackground()
            .environment(\.timeZone, .gmt)
            .onChange(of: profile.birth.kind) { old, new in
                var before = profile.birth
                before.kind = old
                profile.birth = engine.birth(before, rewrittenAs: new, tzOffset: profile.tzOffset)
            }
            .onChange(of: [profile.birth.year, profile.birth.month]) {
                profile.birth = engine.fitted(profile.birth, tzOffset: profile.tzOffset)
            }
            .navigationTitle(profile.name.isEmpty ? "Hồ sơ mới" : profile.name)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Hủy") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Lưu", action: save)
                        .disabled(profile.name.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
            .alert("Ngày sinh không hợp lệ", isPresented: Binding(get: { error != nil }, set: { if !$0 { error = nil } })) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(error ?? "")
            }
        }
    }

    /// Year, month, day and the leap switch of a lunar birth. The switch works only in the
    /// month the year repeats; the days stop at the last day of the month.
    @ViewBuilder
    private var lunarFields: some View {
        let birth = profile.birth
        let leap = engine.leapMonth(ofLunarYear: birth.year, tzOffset: profile.tzOffset)
        let length = engine.lunarMonthLength(year: birth.year, month: birth.month, isLeap: birth.isLeapMonth, tzOffset: profile.tzOffset) ?? 30
        Stepper("Năm \(String(birth.year))", value: $profile.birth.year, in: EngineService.supportedYears)
        Picker("Tháng", selection: $profile.birth.month) {
            ForEach(1...12, id: \.self) { Text("Tháng \($0)").tag($0) }
        }
        Picker("Ngày", selection: $profile.birth.day) {
            ForEach(1...length, id: \.self) { Text("Ngày \($0)").tag($0) }
        }
        Toggle("Tháng nhuận", isOn: $profile.birth.isLeapMonth)
            .disabled(leap != birth.month)
        Text(Self.leapNote(year: birth.year, month: birth.month, leap: leap))
            .font(.caption)
            .foregroundStyle(Theme.inkSecondary)
    }

    /// Why the leap switch is on offer or not.
    nonisolated static func leapNote(year: Int, month: Int, leap: Int?) -> String {
        guard let leap else { return "Năm \(year) âm lịch không có tháng nhuận." }
        if leap == month {
            return "Năm \(year) nhuận tháng \(leap): bật nếu sinh vào tháng \(leap) nhuận."
        }
        return "Năm \(year) nhuận tháng \(leap), không nhuận tháng \(month)."
    }

    private var usesLongitude: Binding<Bool> {
        Binding(
            get: { profile.longitude != nil },
            set: { profile.longitude = $0 ? (profile.longitude ?? Profile.standardMeridian) : nil }
        )
    }

    private func save() {
        profile.name = profile.name.trimmingCharacters(in: .whitespaces)
        do {
            _ = try engine.birth(of: profile)
            store.save(profile)
            dismiss()
        } catch {
            self.error = String(describing: error)
        }
    }

    /// A `Date` in UTC standing for part of the birth fields, for `DatePicker`.
    private func utcDate(_ part: WritableKeyPath<BirthDate, DateComponents>) -> Binding<Date> {
        Binding(
            get: { BirthDate.utc.date(from: profile.birth[keyPath: part]) ?? .now },
            set: { profile.birth[keyPath: part] = BirthDate.utc.dateComponents([.year, .month, .day, .hour, .minute], from: $0) }
        )
    }
}

/// The same birth on the other calendar, with its Can Chi, redrawn as the fields change.
private struct BirthPreviewCard: View {
    let preview: Result<BirthPreview, Error>

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Label("Đối chiếu", systemImage: "arrow.left.arrow.right")
                .font(.caption2.weight(.semibold))
                .foregroundStyle(Theme.inkSecondary)
            switch preview {
            case .success(let preview):
                Text(preview.counterpart)
                    .font(.subheadline.weight(.semibold)).fontDesign(.serif)
                    .foregroundStyle(Theme.cinnabar)
                Text(preview.canChi).font(.caption).foregroundStyle(Theme.ink)
                if let solarTime = preview.solarTime {
                    Text(solarTime).font(.caption).foregroundStyle(Theme.gold)
                }
            case .failure(let error):
                Text("Ngày này không có trên lịch: \(String(describing: error))")
                    .font(.caption).foregroundStyle(Theme.cinnabar)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(Theme.paper, in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        .accessibilityElement(children: .combine)
    }
}

/// A birthplace to pick the longitude from.
struct Birthplace: Identifiable, Hashable {
    let name: String
    /// Degrees east.
    let longitude: Double

    var id: String { name }

    static let common = [
        Birthplace(name: "Hà Nội", longitude: 105.85),
        Birthplace(name: "TP. Hồ Chí Minh", longitude: 106.66),
        Birthplace(name: "Đà Nẵng", longitude: 108.20),
        Birthplace(name: "Hải Phòng", longitude: 106.68),
        Birthplace(name: "Cần Thơ", longitude: 105.78),
    ]
}

/// Shown by a tab that reads a person while none is saved: what the tab does, and a button
/// that opens the form for a first profile.
struct ProfileEmptyStateView: View {
    let engine: EngineService
    let systemImage: String
    let message: String
    @State private var adding: Profile?

    var body: some View {
        VStack(spacing: 14) {
            Image(systemName: systemImage)
                .font(.system(size: 44, weight: .light))
                .foregroundStyle(Theme.gold)
                .frame(width: 92, height: 92)
                .background(Theme.surface, in: Circle())
                .overlay(Circle().strokeBorder(Theme.rule, lineWidth: 1))
            Text("Chưa có hồ sơ")
                .font(.title3.weight(.bold)).fontDesign(.serif)
                .foregroundStyle(Theme.ink)
            Text(message)
                .font(.subheadline)
                .foregroundStyle(Theme.inkSecondary)
                .multilineTextAlignment(.center)
            Button {
                adding = ProfileListView.newProfile(engine: engine)
            } label: {
                Label("Thêm hồ sơ đương số", systemImage: "plus")
                    .font(.headline)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
            }
            .buttonStyle(.borderedProminent)
            .tint(Theme.cinnabar)
            .padding(.top, 4)
        }
        .padding(24)
        .frame(maxWidth: .infinity)
        .sheet(item: $adding) { profile in
            ProfileFormView(profile: profile, engine: engine)
        }
    }
}

extension Profile {
    /// Meridian of GMT+7: the longitude whose solar time the clock keeps.
    static let standardMeridian = 105.0
}

extension BirthDate {
    static let utc: Calendar = {
        var calendar = Calendar(identifier: .gregorian)
        calendar.timeZone = .gmt
        return calendar
    }()

    /// The solar date, for a date picker.
    var solarDay: DateComponents {
        get { DateComponents(year: year, month: month, day: day) }
        set {
            year = newValue.year ?? year
            month = newValue.month ?? month
            day = newValue.day ?? day
        }
    }

    /// The clock time, on a fixed day, for a time picker.
    var clock: DateComponents {
        get { DateComponents(year: 2000, month: 1, day: 1, hour: hour, minute: minute) }
        set {
            hour = newValue.hour ?? hour
            minute = newValue.minute ?? minute
        }
    }
}
