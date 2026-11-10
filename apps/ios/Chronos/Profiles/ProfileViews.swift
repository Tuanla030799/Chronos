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
                ProfileEditorView(profile: profile, engine: engine)
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

/// Form for one profile. Saving asks the engine to resolve the birth date first, so an
/// impossible lunar date (a leap month the year does not have) is refused here.
struct ProfileEditorView: View {
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
                Section("Ngày giờ sinh") {
                    Picker("Lịch", selection: $profile.birth.kind) {
                        ForEach(BirthDate.Kind.allCases) { Text($0.label).tag($0) }
                    }
                    .pickerStyle(.segmented)
                    switch profile.birth.kind {
                    case .solar:
                        DatePicker("Ngày", selection: utcDate(\.solarDay), displayedComponents: .date)
                    case .lunar:
                        Stepper("Năm \(String(profile.birth.year))", value: $profile.birth.year, in: EngineService.supportedYears)
                        Picker("Tháng", selection: $profile.birth.month) {
                            ForEach(1...12, id: \.self) { Text("Tháng \($0)").tag($0) }
                        }
                        Picker("Ngày", selection: $profile.birth.day) {
                            ForEach(1...30, id: \.self) { Text("Ngày \($0)").tag($0) }
                        }
                        Toggle("Tháng nhuận", isOn: $profile.birth.isLeapMonth)
                    }
                    DatePicker("Giờ", selection: utcDate(\.clock), displayedComponents: .hourAndMinute)
                }
                Section {
                    Stepper(value: $profile.tzOffset, in: -12...14, step: 0.5) {
                        Text("Múi giờ GMT\(profile.tzOffset >= 0 ? "+" : "")\(profile.tzOffset.formatted())")
                    }
                    Toggle("Dùng giờ Mặt Trời thực", isOn: usesLongitude)
                    if let longitude = profile.longitude {
                        TextField("Kinh độ nơi sinh (độ Đông)", value: Binding(
                            get: { longitude },
                            set: { profile.longitude = $0 }
                        ), format: .number.precision(.fractionLength(0...4)))
                        .keyboardType(.decimalPad)
                    }
                } footer: {
                    Text("Giờ Mặt Trời thực chỉnh giờ sinh theo kinh độ nơi sinh và phương trình thời gian, dùng cho Bát Tự và Tử Vi.")
                }
            }
            .environment(\.timeZone, .gmt)
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
