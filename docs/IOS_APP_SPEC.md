# Đặc tả ứng dụng iOS (IOS_APP_SPEC)

> Giai đoạn 5B. Task 5B.1 dựng dự án, tầng dịch vụ và khung điều hướng. Task 5B.2 chốt I1–I5, hoàn thiện tab Lịch và tab Xem tuổi. Các tab còn lại làm ở 5B.3–5B.5.

## 1. Dự án Xcode

| Mục | Giá trị |
| :-- | :-- |
| Vị trí | `apps/ios/Chronos.xcodeproj`, mã nguồn ở `apps/ios/Chronos/`, test ở `apps/ios/ChronosTests/` |
| Hệ điều hành | iOS 17.0 trở lên, chỉ iPhone, khóa dọc |
| Ngôn ngữ | Swift 6 (kiểm tra đồng thời đầy đủ), SwiftUI, Observation (`@Observable`) |
| Bundle ID | `com.leanhtuan.chronos`, test `com.leanhtuan.chronos.tests` (I2, đã duyệt) |
| Scheme | `Chronos` (dùng chung, có trong git), gồm cả target test |
| Simulator | Chỉ arm64: `EXCLUDED_ARCHS[sdk=iphonesimulator*] = x86_64`, khớp với `build-ios.sh` |

**Cách tạo project.** Máy không có XcodeGen hay Tuist, nên `project.pbxproj` viết tay ở định dạng `objectVersion = 77` với *thư mục đồng bộ* (`PBXFileSystemSynchronizedRootGroup`): mọi file trong `Chronos/` và `ChronosTests/` tự vào target, thêm file Swift mới không phải sửa project. File project chỉ còn bốn tham chiếu ra ngoài thư mục app:

| Tham chiếu | Đường dẫn | Sinh bởi |
| :-- | :-- | :-- |
| `AstroCore.xcframework` (Frameworks) | `../../build/ios/` | `./build-ios.sh` |
| `AstroCore.swift` (Sources) | `../../build/ios/` | `./build-ios.sh` |
| `chronos_content.sqlite` (Resources) | `../../build/content/` | `cargo run -p astro-data -- build --release` |
| `activities.json` (Resources) | `../../data/interpretations/vocab/` | Có sẵn trong git |

`build/` không vào git, nên chạy hai lệnh sinh trước lần build đầu. Thiếu file nào thì Xcode báo đúng tên file đó.

`AstroCore.swift` biên dịch thẳng vào module `Chronos`, không phải module riêng: các kiểu của engine (`Branch`, `TuViChart`…) dùng trực tiếp, không cần `import`. Vì thế tên kiểu của app không được trùng tên do UniFFI sinh ra (ví dụ `Palace` thuộc Phong Thủy).

`activities.json` đi kèm vì SQLite chỉ lưu mã hoạt động (`XuatHanh`) của `good_for`, `avoid`; nhãn tiếng Việt nằm ở từ vựng (mục 3.6 của `INTERPRETATION_DB_SPEC.md`).

```bash
./build-ios.sh
cargo run -p astro-data -- build --release
xcodebuild -project apps/ios/Chronos.xcodeproj -scheme Chronos \
  -destination 'platform=iOS Simulator,name=iPhone 17' test
```

**Xem trên simulator.** Xcode 27.2 trên máy này không có `Simulator.app` nên `open -a Simulator` báo không tìm thấy; cửa sổ thiết bị ảo nằm trong `DeviceHub.app` (`/Applications/Xcode-27.2.0.app/Contents/Applications/DeviceHub.app`). Chạy từ Xcode (⌘R) hoặc:

```bash
open /Applications/Xcode-27.2.0.app/Contents/Applications/DeviceHub.app
xcrun simctl launch booted com.leanhtuan.chronos -ChronosInitialTab 3   # mở thẳng tab Tử Vi
xcrun simctl io booted screenshot man-hinh.png
```

`-ChronosInitialTab N` (0 Lịch … 4 Phong Thủy) chỉ chọn tab lúc mở app; sau đó đổi tab bình thường.

## 2. Kiến trúc

```text
SwiftUI views (5 tab, hồ sơ)
   │  đọc Profile từ ProfileStore (@Observable, môi trường SwiftUI)
   │  nhận AppServices qua init
   ▼
EngineService ──── hàm của AstroCore (Rust qua UniFFI)
   │  Profile → SolarDateTime, LunarDate, TuViConfig, FourPillarsOptions
   ▼
ContentRepository ── khóa từ AstroCore ──► SQLite3 hệ thống, chỉ đọc, immutable=1
```

Quy tắc:

1. **App không tự ghép khóa và không lặp logic lá số.** Khóa luôn đến từ `almanacDayKeys`, `ageRestrictionKeys`, `baziKeys`, `kuaKey`, `eightStarKey`, `eightStarUseKey`, `kuaCompatibilityKeys`, `nineStarKey`, `tuviPalaceQuery`. Lọc đoạn Tử Vi dùng `gateMatches` của Rust (mục 9 của `INTERPRETATION_DB_SPEC.md`).
2. **GMT+7 mặc định.** `EngineService.vietnamOffset = 7`; mỗi hồ sơ có `tzOffset` riêng.
3. **Không văn bản tự sinh.** Mọi câu luận giải lấy từ SQLite; app chỉ thêm nhãn giao diện.

### 2.1. `EngineService`

Struct `Sendable`, bọc hàm tự do của AstroCore:

| Hàm | Việc |
| :-- | :-- |
| `today(now:)`, `now(_:)` | Ngày giờ dân sự theo `tzOffset`, không theo múi giờ máy |
| `day`, `almanac`, `lunar`, `nineStars(at:)` | Lịch, Trạch Nhật, Cửu Tinh |
| `birth(of:)` | Hồ sơ → `SolarDateTime`; ngày âm đi qua `lunarToSolar` theo `tzOffset` của hồ sơ |
| `lunarBirthDate(of:)` | Ngày âm của lúc sinh; dùng cho năm âm của Mệnh Quái, hạn tuổi |
| `bazi(for:)`, `tuviChart(for:)` | Có kinh độ thì bật giờ Mặt Trời thực, không có thì dùng giờ đồng hồ |
| `ageRestrictions(for:targetLunarYear:)`, `kua(for:)` | Theo năm âm lịch sinh |

Ngày sinh không hợp lệ (tháng nhuận năm không có, 25 giờ) ném `AstroError` của engine; form hồ sơ gọi `birth(of:)` trước khi lưu.

### 2.2. `ContentRepository`

- Mở `chronos_content.sqlite` trong bundle bằng URI `file:…?immutable=1`, cờ `SQLITE_OPEN_READONLY | SQLITE_OPEN_URI`, đúng mục 2.5 của `INTERPRETATION_DB_SPEC.md`.
- Kiểm `PRAGMA application_id = 1128813134` và `user_version = 1`; sai thì báo `incompatibleFile`. Đọc bảng `meta` thành `ContentMetadata`.
- Statement chuẩn bị một lần, giữ lại (`SQLITE_PREPARE_PERSISTENT`); mọi truy cập qua một khóa nên dùng chung giữa các luồng được.
- Bỏ mục `draft = 1` trừ khi `includeDrafts`.

| Hàm | Trả về |
| :-- | :-- |
| `entry(_:)`, `entries(_:)` | `ContentEntry`; khóa là `gap` thì bỏ qua |
| `almanacDay`, `ageRestrictions`, `bazi`, `kuaCompatibility` | `[ContentEntry]` theo đúng thứ tự khóa |
| `kua`, `eightStar`, `eightStarUse` | `ContentEntry?` |
| `nineStar(_:period:)` | `NineStarReading`: mục, trạng thái (`nineStarPhase`), và văn bản Cát hay Hung đúng trạng thái |
| `tuviPalace(chart:branch:)` | `PalaceReading`: mục của `keys`, rồi các đoạn |
| `passages(for:)` | Đoạn của một `PalaceQuery`, theo thứ tự sách |

`PalaceReading.primary` và `.more` tách theo `tier` (D12), nhóm theo `section` theo thứ tự xuất hiện. `ContentEntry.effectiveLuck` lấy `extra.verdict` trước `luck`, để bếp đặt ở sao tốt hiện là Hung (FS-02). Cột `luck` có cả `Trung`, nên repository dùng `ContentLuck` ba giá trị thay vì `Luck` hai giá trị của engine.

**D9.** `Sensitivity` đọc mặt nạ bit của build (`Death` bit 0 … `FamilyLoss` bit 4). Đoạn hoặc mục có `Death`, `Sexuality` ẩn sau một lần chạm, kèm câu "Đây là tư liệu tham khảo văn hóa, không phải lời khuyên y tế hay pháp lý."

### 2.3. `ProfileStore`

- `@MainActor @Observable`, đặt vào môi trường SwiftUI ở `ChronosApp`; mọi tab đọc `store.selected`.
- `Profile`: họ tên, giới tính, `BirthDate` (dương hoặc âm lịch, tháng nhuận, giờ phút), `tzOffset`, `longitude` (nil = giờ đồng hồ).
- Lưu JSON ở `UserDefaults` khóa `chronos.profiles.v1` (I1). Hồ sơ đầu tiên tự được chọn; xóa hồ sơ đang chọn thì chọn hồ sơ đầu còn lại.
- **Hồ sơ mẫu (I1):** lần mở app đầu tiên, khi khóa trên chưa từng có, lưu `Profile.sample`: "Hồ sơ mẫu", Nam, 15/05/1990 09:30 dương lịch, GMT+7. Đã lưu gì rồi, kể cả danh sách rỗng sau khi người dùng xóa hết, thì không thêm lại.

## 3. Điều hướng

`MainTabView` gồm 5 tab, mỗi tab một `NavigationStack` có menu hồ sơ ở góc phải (chọn nhanh, mở "Quản lý hồ sơ").

| Tab | Đã có ở 5B.1 | Việc của bước sau |
| :-- | :-- | :-- |
| Lịch Vạn Niên (`CalendarTab`) | **5B.2 xong**, mục 3.1 | Widget lịch ngày (I3) |
| Xem tuổi (`AlmanacTab`) | **5B.2 xong**, mục 3.2 | Lọc ngày theo việc |
| Bát Tự (`BaziTab`) | Bàn Tứ Trụ (Thập Thần, Can, Chi, Nạp Âm, Trường Sinh), các mục luận giải | 5B.3: Tàng Can, Đại Vận, Thần Sát theo trụ |
| Tử Vi (`TuViTab`) | Địa Bàn 12 cung quanh Thiên Bàn; chạm cung mở sheet: sao, đoạn Primary theo section, "Xem thêm" cho đoạn More, mục tra cứu | 5B.4: vẽ đủ phụ tinh, Đại Hạn, Tiểu Hạn |
| Phong Thủy (`FengShuiTab`) | Mệnh Quái, 8 hướng Bát Trạch, tương hợp với hồ sơ khác, Cửu Tinh năm/tháng/ngày/giờ theo trạng thái vận | 5B.5: bàn Cửu Cung, Trạch Quái theo hướng nhà |

Tab nào cần hồ sơ mà chưa có thì hiện hướng dẫn thêm hồ sơ. Lỗi engine hiện ngay trong tab, không làm app dừng.

### 3.1. Tab Lịch

- **Lưới tháng:** tuần bắt đầu thứ Hai, Chủ Nhật màu Chu Sa. Mỗi ô có ngày dương, ngày âm nhỏ bên dưới (mùng 1 ghi cả tháng, `n` cho tháng nhuận), chấm Kim là ngày Hoàng Đạo, chấm Chu Sa là Hắc Đạo (`pathGodIsHoangDao`). Hôm nay có viền, ngày chọn tô đặc. Đổi tháng bằng hai mũi tên hoặc vuốt ngang, có hiệu ứng trượt; giữ nguyên ngày trong tháng nếu tháng mới có ngày đó. Nút "Hôm nay" hiện khi đã chọn ngày khác.
- **Tờ lịch ngày:** ngày dương cỡ lớn kiểu chữ có chân, thứ, ngày âm và năm Can Chi, ba cột Can Chi Năm/Tháng/Ngày, nhãn Hoàng Đạo hoặc Hắc Đạo kèm tên thần trực nhật, tiết khí (ghi "giao tiết" ngày bắt đầu tiết).
- **Giờ Hoàng Đạo:** đủ 12 canh giờ với khung giờ (Tý 23–01 …), giờ Hoàng Đạo có biểu tượng mặt trời và nền Kim.
- **Việc nên làm, kiêng kỵ:** theo từng nguồn (Trực, Tú, thần, ngày kỵ) như dữ liệu ghi, nhãn lấy từ `activities.json`. Không gộp hay tự loại trừ giữa các nguồn: khi Trực nói nên mà một thần nói kỵ, cả hai cùng hiện.
- **Trực, Nhị Thập Bát Tú, Thần trực nhật, Thần Sát, Ngày kỵ dân gian:** mỗi mục một dòng có chấm Cát/Hung và câu tóm tắt; chạm mở sheet đọc đủ văn bản từ `ContentRepository`.

### 3.2. Tab Xem tuổi

- Chọn năm âm lịch bằng hai mũi tên (mặc định năm nay, có nút "Năm nay"), hiện Can Chi của năm. Hồ sơ chọn ở menu góc phải.
- Thẻ người xem: chữ đầu của chi năm sinh, họ tên, tuổi Can Chi theo năm âm sinh, tuổi mụ.
- Bốn ô tổng quan: Tam Tai, Kim Lâu, Hoang Ốc, Thái Tuế. Phạm hạn thì chữ Chu Sa; "không phạm" để màu trung tính, không ghi là Cát.
- **Vòng Hoang Ốc:** 6 cung trên một vòng tròn theo thứ tự 1–6, màu theo `hoangOcLuck`, cung của tuổi năm đó tô đặc; giữa vòng là tuổi mụ. Dưới 10 tuổi thì không vẽ vòng.
- **Thái Tuế (I5):** chi năm sinh và chi của năm, các quan hệ dạng nhãn Kim (Hợp) hoặc Chu Sa (còn lại).
- **Lời giải:** các mục `almanac.restriction`, `tam_tai`, `kim_lau`, `hoang_oc` của hạn đang có, từ SQLite.

## 4. Bảng màu

Phong cách Á Đông đương đại: ban ngày giấy Dó và mực Nho, ban đêm "Đêm Huyền Không", điểm nhấn Chu Sa và Kim. Định nghĩa ở `Design/Theme.swift` bằng `UIColor` động, đổi theo chế độ sáng/tối của hệ thống.

| Token | Sáng | Tối | Dùng cho |
| :-- | :-- | :-- | :-- |
| `paper` | `#F4ECDC` giấy Dó | `#0E1220` đêm Huyền Không | Nền màn hình |
| `surface` | `#FBF6EA` | `#171C2E` | Ô cung, thẻ |
| `ink` | `#1F1A17` mực Nho | `#EDE6D6` | Chữ chính |
| `inkSecondary` | `#5B5148` | `#A69F92` | Chữ phụ, trích nguồn |
| `rule` | `#D9CCB4` | `#2A3047` | Đường kẻ Địa Bàn |
| `cinnabar` | `#B23A2B` Chu Sa | `#E0604C` | Tint, tên cung, Hung, Kỵ |
| `gold` | `#85601E` Kim | `#D9AE5A` | Cát, Nên, Thập Thần |

Cát dùng Kim, Hung dùng Chu Sa, Bình dùng `inkSecondary`. Mọi màu chữ đạt WCAG AA (4,5:1) cho chữ nhỏ trên nền của nó. Kim sáng phải tối xuống `#85601E`: màu vàng đồng `#B8892E` chỉ đạt 2,69:1 trên giấy.

| Cặp | Tỷ lệ |
| :-- | :-- |
| `ink` trên `paper`, sáng | 14,67:1 |
| `inkSecondary` trên `paper`, sáng / tối | 6,58:1 / 7,10:1 |
| `cinnabar` trên `paper` sáng / `surface` tối | 5,06:1 / 4,79:1 |
| `gold` trên `paper` / `surface`, sáng | 4,85:1 / 5,28:1 |
| `gold` trên `paper`, tối | 9,02:1 |

## 5. Kiểm thử

`ChronosTests` (Swift Testing, chạy trong app trên simulator), 34 ca:

- **Đối chiếu với Rust.** Lá số 30/11/1931 âm lịch, giờ Dần, Nam: danh sách `stable_id` của 5 cung (Mệnh, Quan Lộc, Điền Trạch, Nô Bộc, Thiên Di) phải trùng đúng thứ tự với `scan::select` của `astro-data`, hàm mà V13 dùng. Quan Lộc của lá số này vừa mượn Chính Tinh cung xung chiếu, vừa là cung an Thân, nên phủ cả hai nhánh của `tuviPalaceQuery`.
- **Độ phủ.** Mọi ngày năm 2026 có văn bản cho mọi khóa Trạch Nhật; mọi hạn tuổi của người sinh 1940–2015 năm 2026; 8 quái, 8 sao × 4 ứng dụng, 9 sao × 9 vận; Bát Tự chỉ thiếu khóa `bazi.can_chi` (D18).
- **EngineService.** Ngày dân sự GMT+7 qua nửa đêm UTC; mùng 1 Tết Giáp Thìn ra 10/02/2024; sinh trước Tết thuộc năm âm trước; tháng nhuận không có bị từ chối; hồ sơ âm lịch ra cùng lá số với `tuviChartFromLunar`; kinh độ bật giờ Mặt Trời thực.
- **ProfileStore.** Lưu, mở lại, thay, xóa, chọn, trên một `UserDefaults` suite riêng; hồ sơ mẫu chỉ được thêm ở lần đầu.
- **Lưới tháng, Thái Tuế.** Tháng 11/2026 có 6 ô trống trước Chủ Nhật mùng 1; mỗi ô tháng 2/2024 khớp `almanacDay` (Tết là "1/1"); cộng trừ tháng qua năm; hồ sơ mẫu năm 2026 là Trực và Hình Thái Tuế; Thái Tuế dùng năm âm sinh.
- **Tốc độ** (Debug, simulator): 1.000 lần đọc mục mất 21 ms, khoảng 21 µs mỗi lần; đọc đủ 12 cung một lá số mất 56 ms.

## 6. Quyết định và việc mở

| Mã | Nội dung | Trạng thái |
| :-- | :-- | :-- |
| I1 | Hồ sơ lưu JSON ở `UserDefaults`, không dùng SwiftData. Lần mở đầu tiên có sẵn hồ sơ mẫu 15/05/1990 09:30, Nam (mục 2.3) | **Đã duyệt** 2026-11-11 |
| I2 | Bundle ID `com.leanhtuan.chronos` | **Đã duyệt** 2026-11-11 |
| I3 | Widget lịch ngày chỉ dùng `AstroCore.xcframework` và `activities.json` (thuộc cả target App và Widget). **Không** nhúng `chronos_content.sqlite` vào Widget, để giữ bộ nhớ Widget dưới giới hạn của iOS. Hệ quả: Widget hiện được những gì engine tính (ngày âm, Can Chi, tiết khí, Trực, Tú, thần trực nhật, Hoàng/Hắc Đạo, giờ Hoàng Đạo, Thần Sát, ngày kỵ). Danh sách Nên/Kỵ (`good_for`, `avoid`) chỉ nằm trong SQLite, nên `activities.json` (chỉ là nhãn) chưa giúp Widget hiện Nên/Kỵ; cần chốt thêm khi dựng Widget | **Đã duyệt** 2026-11-11; làm khi dựng Widget |
| I4 | Simulator chỉ arm64; Mac Intel cần thêm target `x86_64-apple-ios` vào `build-ios.sh` | Giữ như Task 1 |
| I5 | Xung tuổi theo năm = quan hệ của chi năm sinh với Thái Tuế (chi của năm đang xét), tính bằng phép Địa Chi có sẵn trong core, không thêm văn bản vào SQLite. Mục 6.1 | **Đã duyệt** 2026-11-11; làm ở 5B.2 |

### 6.1. Thái Tuế (I5)

`astro_core::almanac::tai_sui(birth_lunar_year, target_lunar_year)` trả `TaiSuiReading { birth_branch, year_branch, relations }`; Swift `taiSui(birthLunarYear:targetLunarYear:)`, JS `taiSui(...)`, kèm `taiSuiName`, `taiSuiLuck`. Năm sinh là năm âm lịch (sinh trước Tết thuộc năm trước), như Tam Tai.

| `TaiSui` | Tên | Điều kiện (chi sinh `b`, chi năm `y`) | Cát/Hung |
| :-- | :-- | :-- | :-- |
| `Truc` | Trực Thái Tuế (năm tuổi) | `b = y` | Hung |
| `Xung` | Xung Thái Tuế | `b.clash() = y` (Lục Xung) | Hung |
| `Hai` | Hại Thái Tuế | `b.harm() = y` (Lục Hại) | Hung |
| `Hinh` | Hình Thái Tuế | `b.punishment() = y` hoặc `y.punishment() = b`, gồm tự hình Thìn, Ngọ, Dậu, Hợi | Hung |
| `Pha` | Phá Thái Tuế | `b.destruction() = y` (Lục Phá) | Hung |
| `LucHop` | Hợp Thái Tuế (Lục Hợp) | `b.six_combination() = y` | Cát |
| `TamHop` | Hợp Thái Tuế (Tam Hợp) | cùng nhóm Tam Hợp, `b ≠ y` | Cát |

Nhiều quan hệ có thể cùng đúng và đều được trả về, theo thứ tự bảng: Dần gặp năm Hợi là Phá và Lục Hợp; Tỵ gặp năm Thân là Hình, Phá, Lục Hợp; người tuổi Ngọ năm Ngọ là Trực và Hình (Ngọ tự hình). App hiện tất cả, không tự chọn một. Cát/Hung theo chỉ định của chủ dự án: chỉ hai loại Hợp là Cát.
