# Đặc tả ứng dụng iOS (IOS_APP_SPEC)

> Giai đoạn 5B, **hoàn thành**. Task 5B.1 dựng dự án, tầng dịch vụ và khung điều hướng. Task 5B.2 chốt I1–I5, hoàn thiện tab Lịch và tab Xem tuổi. Task 5B.3 làm tab Bát Tự và Phong Thủy, 5B.4 tab Tử Vi, 5B.5 Widget và rà soát tổng thể. Task 5B.6 nâng cấp trải nghiệm: bỏ hồ sơ mẫu, đối chiếu Âm–Dương trong form, chú thích thuật ngữ, ghi chú và nhắc ngày giỗ, Widget cỡ lớn.

## 1. Dự án Xcode

| Mục | Giá trị |
| :-- | :-- |
| Vị trí | `apps/ios/Chronos.xcodeproj`; app ở `apps/ios/Chronos/`, Widget ở `apps/ios/ChronosWidget/`, mã dùng chung ở `apps/ios/Shared/`, test ở `apps/ios/ChronosTests/` |
| Hệ điều hành | iOS 17.0 trở lên, chỉ iPhone, khóa dọc |
| Ngôn ngữ | Swift 6 (kiểm tra đồng thời đầy đủ), SwiftUI, Observation (`@Observable`) |
| Bundle ID | `com.leanhtuan.chronos`, Widget `com.leanhtuan.chronos.widget`, test `com.leanhtuan.chronos.tests` (I2, đã duyệt) |
| Scheme | `Chronos` (build app, Widget theo phụ thuộc, và target test) và `ChronosWidget` (chạy Widget trên SpringBoard, cỡ medium), cả hai dùng chung, có trong git |
| Simulator | Chỉ arm64: `EXCLUDED_ARCHS[sdk=iphonesimulator*] = x86_64`, khớp với `build-ios.sh` |

**Cách tạo project.** Máy không có XcodeGen hay Tuist, nên `project.pbxproj` viết tay ở định dạng `objectVersion = 77` với *thư mục đồng bộ* (`PBXFileSystemSynchronizedRootGroup`): mọi file trong thư mục tự vào target, thêm file Swift mới không phải sửa project.

| Thư mục | Thuộc target |
| :-- | :-- |
| `Chronos/` | App |
| `ChronosWidget/` | Widget; `Info.plist` (khai báo `com.apple.widgetkit-extension`) được loại khỏi Resources bằng một `PBXFileSystemSynchronizedBuildFileExceptionSet` |
| `Shared/` | App và Widget: `Theme` (bảng màu), tên thứ và khung giờ, `DaySnapshot` và timeline, `CalendarNote`, `NoteStorage` và quy tắc ngày lặp lại (`NoteCalendar`), các view Widget. Vì thuộc cả app nên `ChronosTests` kiểm được logic và giao diện Widget |
| `ChronosTests/` | Test |
| `Config/` (không đồng bộ) | `Chronos.entitlements`, `ChronosWidget.entitlements`: App Group `group.com.leanhtuan.chronos`. **Hiện không gắn vào project** (mục 1.1); giữ sẵn để bật lại |

App nhúng `ChronosWidget.appex` qua bước "Embed Foundation Extensions". File project còn bốn tham chiếu ra ngoài thư mục app:

| Tham chiếu | Đường dẫn | Sinh bởi |
| :-- | :-- | :-- |
| `AstroCore.xcframework` (Frameworks, app và Widget) | `../../build/ios/` | `./build-ios.sh` |
| `AstroCore.swift` (Sources, app và Widget) | `../../build/ios/` | `./build-ios.sh` |
| `chronos_content.sqlite` (Resources, chỉ app) | `../../build/content/` | `cargo run -p astro-data -- build --release` |
| `activities.json` (Resources, chỉ app) | `../../data/interpretations/vocab/` | Có sẵn trong git |

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

### 1.1. Chạy trên iPhone thật với Apple ID miễn phí (Personal Team)

Tài khoản miễn phí không ký được capability App Groups ("Personal development teams do not support the App Groups capability"), nên project **không** đặt `CODE_SIGN_ENTITLEMENTS`: cả `Chronos` và `ChronosWidget` ký không có quyền nào. Mở Xcode, chọn Team ở *Signing & Capabilities* của hai target (Bundle ID có thể phải đổi thành tên duy nhất của bạn, vì `com.leanhtuan.chronos` đã thuộc một Team khác nếu có người dùng nó), chọn iPhone rồi ⌘R. `AstroCore.xcframework` có đủ hai lát `ios-arm64` (thiết bị) và `ios-arm64-simulator`; bản build thiết bị `generic/platform=iOS` đã kiểm không lỗi, không cảnh báo, ra `arm64` iOS 17.

Thiếu App Group nghĩa là:

- Hồ sơ, ghi chú và nhắc 08:00 chạy bình thường (đều ở `UserDefaults.standard` của app; thông báo cục bộ không cần tài khoản trả phí).
- Widget vẫn hiện lịch ngày, Can Chi, giờ Hoàng Đạo, nhưng **không thấy ghi chú** của app.
- Bản cài bằng Personal Team hết hạn sau 7 ngày, phải cài lại từ Xcode; tối đa 3 app cùng lúc.

**Bật lại App Group khi có tài khoản Apple Developer trả phí:** trong Developer Portal tạo App Group `group.com.leanhtuan.chronos` và bật nó cho cả hai App ID; trong `project.pbxproj` thêm `CODE_SIGN_ENTITLEMENTS = Config/Chronos.entitlements;` vào hai cấu hình (Debug, Release) của target `Chronos` và `CODE_SIGN_ENTITLEMENTS = Config/ChronosWidget.entitlements;` vào hai cấu hình của `ChronosWidget` (hoặc thêm capability *App Groups* trong Xcode, nó tự làm việc này). Không cần sửa mã: `NoteStorage.shared` thấy được container và chuyển sang suite của nhóm. Ghi chú đã lưu ở `UserDefaults.standard` không tự chuyển sang nhóm; chép chúng một lần nếu cần.

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

1. **App không tự ghép khóa và không lặp logic lá số.** Khóa luôn đến từ `almanacDayKeys`, `ageRestrictionKeys`, `baziKeys`, `kuaKey`, `eightStarKey`, `eightStarUseKey`, `kuaCompatibilityKeys`, `nineStarKey`, `tuviPalaceQuery`, cùng các khóa theo từng mục `bazi…Key` và `tuvi…Key`. Lọc đoạn Tử Vi dùng `gateMatches` của Rust (mục 9 của `INTERPRETATION_DB_SPEC.md`).
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
- Kiểm `PRAGMA application_id = 1128813134` và `user_version = 2` (D20: thêm cột `passage.correction`); sai thì báo `incompatibleFile`. Đọc bảng `meta` thành `ContentMetadata`.
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
- **Không còn hồ sơ mẫu (Task 5B.6, thay phần này của I1):** lần mở đầu tiên danh sách rỗng và không ghi gì xuống `UserDefaults`; bốn tab cần hồ sơ hiện `ProfileEmptyStateView`. Người đã có "Hồ sơ mẫu" lưu từ bản trước vẫn giữ nó cho đến khi tự xóa. `Profile.sample` chỉ còn là dữ liệu của bộ test.

## 3. Điều hướng

`MainTabView` gồm 5 tab, mỗi tab một `NavigationStack` có menu hồ sơ ở góc phải (chọn nhanh, mở "Quản lý hồ sơ").

| Tab | Đã có ở 5B.1 | Việc của bước sau |
| :-- | :-- | :-- |
| Lịch Vạn Niên (`CalendarTab`) | **5B.2 xong**, mục 3.1 | Widget lịch ngày (I3) |
| Xem tuổi (`AlmanacTab`) | **5B.2 xong**, mục 3.2 | Lọc ngày theo việc |
| Bát Tự (`BaziTab`) | **5B.3 xong**, mục 3.3 | Đại Vận chi tiết (Thập Thần của vận), Lưu Niên |
| Tử Vi (`TuViTab`) | **5B.4 xong**, mục 3.5 | Cách cục, sao lưu niên (khi có dữ liệu `tuvi.han_passage`) |
| Phong Thủy (`FengShuiTab`) | **5B.3 xong**, mục 3.4 | Trạch Quái theo hướng nhà |

Tab nào cần hồ sơ mà chưa có thì hiện `ProfileEmptyStateView` (mục 3.7). Lỗi engine hiện ngay trong tab, không làm app dừng.

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

### 3.3. Tab Bát Tự

- **Đầu trang:** hồ sơ đang đọc (đổi nhanh bằng nút hai mũi tên khi có từ hai hồ sơ), hai lựa chọn lập trụ giữ qua các lần mở app (`@AppStorage`): *năm, tháng đổi tại* Tiết khí (mặc định) hoặc mùng 1 âm lịch (`YearMonthBoundary`), và *giờ Tý* chia ở nửa đêm (mặc định) hoặc sang ngày từ 23h (`RatHourConvention`); thêm một nhãn cho biết hồ sơ lập theo giờ đồng hồ hay giờ Mặt Trời thực (kinh độ đặt trong hồ sơ).
- **Nhật Chủ và Bản mệnh Nạp Âm:** hai thẻ cạnh nhau, Can ngày và Nạp Âm của trụ năm, kèm Ngũ Hành và câu tóm tắt; chạm mở văn bản.
- **Bảng Tứ Trụ:** bốn cột Giờ, Ngày, Tháng, Năm; trụ Ngày được gạch chân Chu Sa. Các hàng: Thập Thần của Thiên Can (cột Ngày ghi "Nhật Chủ"), Thiên Can và Địa Chi chữ lớn có nhãn Ngũ Hành, Tàng Can (Bản khí, Trung khí, Dư khí theo vị trí trong mảng của nguồn, mỗi Can kèm Thập Thần), Trường Sinh, Nạp Âm, Thần Sát của từng trụ. Mọi ô có nền giấy là ô chạm được và mở văn bản trong sheet.
- **Không Vong, Khôi Canh:** hai dòng Không Vong tính từ trụ Ngày và trụ Năm theo mục 2.5C của `BAZI_ALMANAC_SPEC.md`, chạm mở văn bản; Khôi Canh hiện khi trụ Ngày là một trong bốn trụ của đặc tả. Nếu Địa Chi của một trụ trùng hai chi trống thì ô Địa Chi ghi "Không · Ngày" hoặc "Không · Năm". Đây là phép so sánh thẳng, không đi kèm luận đoán; trụ gốc không bị so với chính Không Vong của nó (đặc tả để phần này cho phần hiển thị).
- **Đại Vận:** tuổi, tháng, ngày khởi vận, chiều thuận hoặc nghịch, rồi 10 bước (bước 0 "Trước vận") trong một hàng cuộn ngang; bước có năm hiện tại tô viền Chu Sa, ghi "HIỆN TẠI" và tự cuộn vào giữa.
- **Luận giải:** mục lục theo nhóm (Nhật Chủ, Nạp Âm, Thập Thần, Trường Sinh, Thần Sát, Can Chi), mỗi dòng mở sheet. Văn bản đến từ `ContentRepository`; khóa của từng ô do AstroCore đưa ra (`baziDayMasterKey`, `baziTenGodKey`, `baziLifeStageKey`, `baziNaYinKey`, `baziShenShaKey`, `baziKhongVongKey`, `baziKhoiCanhKey`), app không tự ghép chuỗi. `baziKeys` chỉ liệt kê Thập Thần của Thiên Can, nên Thập Thần của Tàng Can được đọc thẳng từ kho theo khóa.
- **Ngũ Hành (Kim, Mộc, Thủy, Hỏa, Thổ):** màu và tên luôn đi cùng nhau, không dùng màu một mình. Ngũ Hành của Can, Chi, Nạp Âm do `stemElement`, `branchElement`, `naYinElement` của core trả về.

### 3.4. Tab Phong Thủy

- **Mệnh Quái:** quái, chữ Hán, Ngũ Hành, quái số, Đông Tứ hoặc Tây Tứ Mệnh, hướng của quái, năm âm sinh, câu tóm tắt và "Đọc đầy đủ".
- **Bát Trạch, tám hướng:** bốn hướng tốt (Sinh Khí, Thiên Y, Diên Niên, Phục Vị) và bốn hướng xấu (Tuyệt Mệnh, Ngũ Quỷ, Lục Sát, Họa Hại) lấy từ `eightMansionsChart`, chia theo `eightMansionsStarLuck`. Chạm một hướng mở sheet: ý nghĩa chung của sao rồi bốn ứng dụng (Hướng nhà, Bếp, Bàn làm việc, Kết hôn) của batch FS-02; bếp tại Sinh Khí hiện Hung nhờ `extra.verdict`.
- **Tương hợp Bát Biến:** chọn hồ sơ khác, hoặc nhập nhanh năm sinh **âm lịch** và giới tính của đối tác (chọn "Nhập nhanh" tự bật khi chưa có hồ sơ thứ hai). Hiện hai quái, sao Bát Biến giữa hai người, Cát hoặc Hung, văn bản của sao và của hôn nhân, và nút mở bốn ứng dụng của sao đó.
- **Cửu Cung Phi Tinh:** chọn thời điểm (mặc định bây giờ, GMT+7), hiện Vận và Nguyên, bốn ô Năm, Tháng, Ngày, Giờ (sao ở giữa và trạng thái vận) dùng để chọn bàn, rồi bàn 3×3 `LoShuBoard`. Bố cục Nam ở trên như mục 4.7 của `FENGSHUI_SPEC.md`; `LoShuLayout` giữ thứ tự cung và có test đối chiếu với `loShuRows`. Năm, Tháng bay thuận; Ngày, Giờ theo chiều của chuỗi sao (F3). Mỗi ô: hướng, số sao (màu Ngũ Hành), tên sao, nhãn Đương vận, Sinh khí, Thoái khí hoặc Thất vận. Chạm một ô mở sheet có `auspicious_meaning` hoặc `inauspicious_meaning` tùy trạng thái, rồi văn bản chung của sao. Không cần hồ sơ để dùng bàn này.

### 3.5. Tab Tử Vi

- **Đầu trang:** hồ sơ đang xem (đổi nhanh), năm xem âm lịch (mặc định năm nay, có Can Chi và tuổi mụ), hai lựa chọn giữ qua các lần mở app: *tháng nhuận* chia ở ngày 15 (mặc định) hoặc cả tháng như tháng trước (`LeapMonthRule`), và *giờ Tý* chia ở nửa đêm (mặc định) hoặc sang ngày từ 23h (`RatHourConvention`). Hai lựa chọn này riêng với tab Bát Tự.
- **Bàn cờ 4×4:** Tỵ, Ngọ, Mùi, Thân ở hàng trên; Thìn, Mão bên trái; Dậu, Tuất bên phải; Dần, Sửu, Tý, Hợi ở hàng dưới (`TuViBoardLayout`). Thiên Bàn 2×2 ở giữa: họ tên, Âm Dương Nam Nữ, ngày dương và âm, Can Chi năm, tháng, ngày, giờ, Bản mệnh, Cục, Cục sinh khắc Mệnh, thuận hay nghịch lý, Mệnh chủ, Thân chủ.
- **Mỗi cung:** tên cung (Mệnh màu Chu Sa), Can Chi của cung kèm `<THÂN>`, tuổi khởi Đại hạn; Chính Tinh với ký hiệu độ sáng và màu Ngũ Hành của sao (`tuviStarElement`), hoặc "Vô Chính Diệu"; các Tứ Hóa trong cung; số phụ tinh Cát và Hung; nhãn TUẦN, TRIỆT, ĐH, TH (Đại hạn, Tiểu hạn của năm xem, `tuviDaiHanPalace`, `tuviTieuHanPalace`); Trường Sinh và năm có Tiểu hạn đến cung ("năm Thìn").
- **Chọn cung:** mặc định Mệnh. Cung chọn viền Chu Sa, hai cung Tam Hợp viền Kim, cung Xung Chiếu viền đứt Chu Sa, nối bằng tam giác Kim và đường đứt Chu Sa vẽ bằng `Canvas` sau các ô (chỉ lộ ra ở khe và qua Thiên Bàn). Tam Hợp và Xung Chiếu do core trả (`branchTrine`, `branchClash`).
- **Chi tiết cung** (ngay dưới bàn cờ, không phải sheet): Can Chi, Đại hạn, Trường Sinh, Thân cư, Tuần, Triệt, Đại hạn và Tiểu hạn của năm xem; tên các cung Tam Hợp và Xung Chiếu. Sao chia bốn nhóm: Chính tinh, phụ tinh Cát, phụ tinh Hung và sát, phụ tinh Trung tính (theo `tuviStarNature`, bảng T9). Nhận định chung `tuvi.palace` khi có. Luận giải Primary theo section; câu phú đóng khung nghiêng; ghi chú "Đính chính so với bản in" thu gọn một dòng (D20). Đoạn More thu dưới "Xem thêm các trường hợp phối hợp sao" kèm số đoạn (D12). Đoạn gắn Death hoặc Sexuality vẫn ẩn sau một lần chạm (D9).
- **Sheet sao:** loại sao, Cát Hung, Ngũ Hành, độ sáng tại cung, Tứ Hóa, nhãn và ngũ hành trong `extra.profile`, rồi văn bản `tuvi.star`, `tuvi.star_brightness`, `tuvi.transformation`. Khóa do core trả (`tuviStarKey`, `tuviStarBrightnessKey`, `tuviTransformationKey`, `tuviPalaceKey`).
- Mọi sheet của tab dùng nền giấy Dó hoặc Đêm Huyền Không; tab không còn `List` mặc định.

### 3.6. Widget (`ChronosWidget`)

Một Widget `ChronosDay`, `StaticConfiguration`, bốn cỡ, nền giấy Dó hoặc Đêm Huyền Không qua `containerBackground`. Chỉ dùng `AstroCore.xcframework` và `AstroCore.swift` (I3): không SQLite, không `activities.json`, không Nên/Kỵ. Không cần hồ sơ. Giờ Việt Nam GMT+7 như toàn app.

| Cỡ | Nội dung |
| :-- | :-- |
| `systemSmall` | Thứ, ngày dương cỡ lớn (Chủ Nhật màu Chu Sa), tháng và năm, ngày âm, Can Chi ngày, nhãn Hoàng Đạo hoặc Hắc Đạo có chấm |
| `systemMedium` | Bên trái như tờ lịch nhỏ; bên phải Can Chi năm và tháng, Can Chi ngày kèm Hoàng/Hắc Đạo, Tiết Khí, Trực, Sao (Nhị Thập Bát Tú), sáu giờ Hoàng Đạo; giờ đang chạy hoặc kế tiếp in đậm màu Kim. Có sự kiện thì thêm một dòng: sự kiện hôm nay, nếu không là sự kiện gần nhất trong 30 ngày ("Còn 9 ngày: Giỗ Ông Nội") |
| `systemLarge` | Nửa trên là tờ lịch ngày đầy đủ như cỡ vừa. Nửa dưới "Ghi chú & Sự kiện": tối đa 5 dòng, mỗi dòng có tên, loại và ngày ("Ngày giỗ · 15/10 Âm lịch"), số ngày còn lại ("Hôm nay", "Ngày mai", "Còn 9 ngày"); thừa thì ghi "và N sự kiện khác"; không có thì ghi rõ |
| `accessoryRectangular` (màn hình khóa) | Ngày dương và ngày âm, Can Chi ngày và Hoàng/Hắc Đạo. Dòng thứ ba ưu tiên sự kiện của hôm nay hoặc ngày mai ("Ngày mai: Giỗ Ông Nội"); không có thì là giờ Hoàng Đạo đang chạy ("Đang giờ Tỵ 09–11") hoặc kế tiếp ("Giờ tới Thân 15–17", thêm "(mai)" nếu sang ngày sau). Không đặt màu: hệ thống tự phủ màu màn hình khóa |

- **Giờ trong ngày.** Ngày đổi lúc nửa đêm, như mặc định `SplitAtMidnight` của engine, nên một ngày dân sự có 13 khung: Tý 00–01, Sửu 01–03 … Hợi 21–23, Tý 23–24; cả hai khung Tý dùng Hoàng Đạo của chính ngày đó (`HourSlot`). Giờ Hoàng Đạo kế tiếp là khung đang chạy nếu nó Hoàng Đạo, nếu không là khung Hoàng Đạo đầu tiên còn lại trong ngày, hết thì lấy khung đầu tiên của ngày mai (`DayMoment.upcoming`).
- **Làm mới.** `DayTimeline` cho một mục ngay lúc này, một mục ở mỗi ranh giới canh giờ còn lại (01, 03 … 23 giờ) và một mục lúc 00:00 hôm sau, chính sách `.atEnd`: Widget đổi ngày đúng nửa đêm và giờ Hoàng Đạo luôn cập nhật.
- **Ghi chú trên Widget.** Widget đọc `NoteStorage` trong `UserDefaults` của App Group `group.com.leanhtuan.chronos` mỗi lần lập timeline, rồi tính sự kiện 30 ngày tới cho từng mục (nên sang ngày mới thì "Còn N ngày" tự giảm). App gọi `WidgetCenter.shared.reloadAllTimelines()` mỗi khi ghi chú đổi. Đây là dữ liệu của người dùng, không phải SQLite luận giải: I3 vẫn giữ nguyên.
- **Kiểm tra giao diện.** Không có công cụ chạm để thêm Widget lên màn hình chính simulator, nên `DayWidgetTests` dựng các view (gồm cỡ lớn có và không có sự kiện) bằng `ImageRenderer` ở chế độ sáng và tối (đã xem ảnh khi làm 5B.5 và 5B.6), và `pluginkit` xác nhận simulator đã đăng ký `com.leanhtuan.chronos.widget`.

### 3.7. Hồ sơ

- Danh sách và form hồ sơ dùng nền giấy, hàng nền `surface` (gắn `listRowBackground` vào từng hàng hoặc section, vì gắn ở `List` không có tác dụng).
- Đổi giữa Dương lịch và Âm lịch trong form **quy đổi** ngày sinh bằng engine (`EngineService.birth(_:rewrittenAs:tzOffset:)`): 15/05/1990 dương thành 21/4/1990 âm và ngược lại, tháng nhuận giữ đúng. Ngày không tồn tại trên lịch cũ thì giữ nguyên số để form báo lỗi khi lưu.
- Mọi tab đọc `ProfileStore.selected` (`@Observable`), nên thêm, sửa, xóa hồ sơ hiện ngay ở cả bốn tab cần hồ sơ; có test cho việc báo thay đổi và cho hồ sơ âm lịch tháng nhuận.
- **Trạng thái trống (`ProfileEmptyStateView`):** biểu tượng, "Chưa có hồ sơ", một câu về việc tab làm được gì, và nút Chu Sa "Thêm hồ sơ đương số" mở thẳng form hồ sơ mới. Tab Xem tuổi, Bát Tự, Tử Vi hiện nó toàn màn hình; tab Phong Thủy hiện nó trong một thẻ, bên dưới vẫn có bàn Cửu Cung (không cần hồ sơ).
- **Thẻ đối chiếu (`ProfileFormView`, `EngineService.birthPreview`):** ngay dưới ngày giờ sinh, vẽ lại mỗi khi một trường đổi. Đang nhập dương lịch thì hiện "Âm lịch: 21 tháng 4 năm 1990"; đang nhập âm lịch thì hiện "Dương lịch: Thứ Ba, 15/05/1990". Dòng dưới là Can Chi: năm, tháng, ngày theo âm lịch (`dayInfo`), giờ theo đồng hồ với Tý chia ở nửa đêm (`fourPillars`). Có kinh độ thì thêm "Giờ Mặt Trời thực: 09:29 ngày 31/03" (`trueSolarTime`). Ngày không tồn tại thì thẻ báo lỗi của engine.
- **Tháng nhuận:** form hỏi `lunarYear` của AstroCore. Công tắc chỉ bật được ở đúng tháng mà năm đó nhuận; đổi năm hoặc tháng thì tự tắt (`EngineService.fitted`), và ngày không vượt số ngày của tháng (29 hoặc 30). Dòng ghi chú nói rõ "Năm 2023 nhuận tháng 2, không nhuận tháng 5" hoặc "Năm 2024 âm lịch không có tháng nhuận".
- **Nơi sinh:** khi bật giờ Mặt Trời thực, menu chọn nhanh Hà Nội 105,85; TP. Hồ Chí Minh 106,66; Đà Nẵng 108,20; Hải Phòng 106,68; Cần Thơ 105,78 (độ Đông), hoặc nhập tay. Chú thích giải thích kinh tuyến 105° của giờ Việt Nam và khi nào giờ Mặt Trời thực làm đổi lá số.

### 3.8. Ghi chú, sự kiện và nhắc ngày giỗ

- **`CalendarNote`** (`Shared/`): `id`, `title`, `notes`, `calendarType` (`.lunar` hoặc `.solar`), `day`, `month`, `year`, `isLeapMonth`, `isYearlyRecurring`, `category` (Ngày giỗ, Sinh nhật, Lễ cúng, Việc quan trọng, Ghi chú), `reminderOffsets` (mặc định `[0, 1, 3]`). Lưu JSON ở khóa `chronos.notes.v1` của `UserDefaults` App Group; `NoteStore` (`@Observable`) là nơi app đọc và ghi. Không có entitlement App Group thì `NoteStorage.shared` dùng `UserDefaults.standard`: app vẫn chạy đủ (hồ sơ, ghi chú, nhắc 08:00 đều không cần App Group), chỉ Widget không thấy ghi chú vì hai tiến trình không chung vùng lưu. Bản dự phòng hỏi `containerURL(forSecurityApplicationGroupIdentifier:)` trước khi mở suite, nên máy không có quyền cũng không bị ghi lỗi `CFPrefsPlistSource`. Test dùng suite riêng.
- **Ngày lặp lại (`CalendarNote.occurrence(inYear:)`, `nextOccurrenceSolarDate`):**
  - Ghi chú âm lịch hằng năm đổi sang dương lịch bằng `lunarToSolar` cho từng năm âm; lần tới là của năm nay nếu chưa qua, nếu qua rồi là của năm sau.
  - **Tháng thiếu:** ngày 30 rơi vào tháng chỉ có 29 ngày thì tính vào ngày 29 (số ngày lấy từ `lunarYear`).
  - **Tháng nhuận:** năm có tháng nhuận trùng tháng sự kiện vẫn tính tháng thường. Chỉ chính năm của sự kiện mới đọc `isLeapMonth`.
  - Ghi chú dương lịch hằng năm vào 29/2 tính ngày 28/2 ở năm không nhuận. Đây là lựa chọn của bản cài đặt, theo cùng tinh thần với tháng thiếu.
  - Ghi chú hằng năm bắt đầu từ `year` của nó, không hiện ở các năm trước đó. Ghi chú không lặp chỉ có ở đúng ngày của nó.
- **Tab Lịch:** ô ngày có ghi chú mang dấu trang nhỏ ở góc trên. Dưới tờ lịch ngày là thẻ "Ghi chú & Sự kiện": ghi chú của ngày đang chọn, nút "Thêm sự kiện / ngày giỗ" (form mở sẵn ngày đang xem, viết theo âm lịch, loại Ngày giỗ, lặp hằng năm), rồi "Sự kiện sắp tới (30 ngày tới)" tính từ hôm nay, mỗi dòng có nhãn "Hôm nay", "Ngày mai" hoặc "Còn N ngày". Chạm một dòng để sửa hoặc xóa.
- **Form sự kiện (`NoteFormView`):** tên, loại, lịch (đổi lịch thì quy đổi ngày), ngày, tháng nhuận (cùng kiểm tra như form hồ sơ), lặp hằng năm, ba công tắc nhắc, ghi chú. Thẻ đối chiếu ghi ngày trên lịch kia, lần tới và còn bao nhiêu ngày, cùng quy tắc tháng thiếu và tháng nhuận khi áp dụng.
- **Thông báo (`NotificationManager`, `ReminderPlan`):** xin quyền khi người dùng lưu một sự kiện có nhắc. `ReminderPlan` tính các thông báo cục bộ lúc 08:00 **giờ Việt Nam (GMT+7)** vào ngày diễn ra và trước 1, 3 ngày, cho mọi lần xảy ra trong 400 ngày tới, bỏ giờ đã qua, giữ tối đa 60 cái sớm nhất (iOS cho 64). `NotificationManager.sync` xóa các thông báo cũ của ghi chú rồi đặt lại theo kế hoạch, chạy mỗi khi thêm, sửa, xóa ghi chú và mỗi lần mở app.

### 3.9. Chú thích cho người mới (`Glossary`)

Mọi lời giải thích nằm ở `Features/Shared/Glossary.swift`. Chúng nói màn hình đang hiện gì và engine dùng quy ước nào, không luận đoán lá số. Ba dạng: nút `ⓘ` (`InfoButton`) mở sheet nền giấy, nút `ⓘ` ở tiêu đề thẻ (`CardTitle(info:)`), và dòng ghi chú ngắn (`HelpNote`).

| Tab | Chú thích |
| :-- | :-- |
| Lịch | `ⓘ` cạnh chú giải lưới tháng: chấm Hoàng Đạo và Hắc Đạo, số ngày âm, dấu trang của ghi chú, giờ Hoàng Đạo |
| Xem tuổi | Một dòng dưới mỗi ô: Tam Tai, Kim Lâu, Hoang Ốc, Thái Tuế |
| Bát Tự | Dòng giải thích ranh giới năm, tháng và quy ước giờ Tý; phụ đề của Nhật Chủ và Bản mệnh; `ⓘ` ở thẻ Tứ Trụ và Đại Vận: Nhật Chủ, Thập Thần, Tàng Can, Trường Sinh, Nạp Âm, Tuần Không Vong, Đại Vận |
| Tử Vi | Nút "Chú giải ký hiệu" trên bàn cờ: năm mức độ sáng (M, V, Đ, B, H), `<THÂN>`, ĐH, TH, TUẦN, TRIỆT, viền Tam Hợp và Xung Chiếu, các số trong ô; dòng giải thích quy tắc tháng nhuận và giờ Tý |
| Phong Thủy | `ⓘ` ở thẻ Mệnh Quái: Đông Tứ Mệnh và Tây Tứ Mệnh (danh sách quái lấy từ engine); `ⓘ` ở thẻ Cửu Cung: Đương vận, Sinh khí, Thoái khí, Thất vận |

Lời chú giải độ sáng sao (Rất tốt, Tốt, Khá, Trung bình, Bất lợi), `<THÂN>` và bốn dòng ở tab Xem tuổi do chủ dự án đưa ra ở Task 5B.6. Các câu còn lại mô tả đúng cách engine tính, và có test giữ cho chúng khớp: năm ký hiệu độ sáng trùng `starBrightnessMark`; bốn hướng tốt thuộc các quái cùng nhóm; chỉ Thất vận đọc theo nghĩa xấu.


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

Ngũ Hành có năm màu riêng, chỉ dùng cho Can, Chi, Nạp Âm, quái và sao, luôn kèm tên:

| Token | Sáng | Tối |
| :-- | :-- | :-- |
| `kim` | `#565E6B` | `#B9C2CF` |
| `moc` | `#2E6B3B` | `#73C48A` |
| `thuy` | `#1F5F8B` | `#6FB1E6` |
| `hoa` | `#9C4116` | `#F28A5E` |
| `tho` | `#7A5430` | `#CFA97E` |

Cả năm đạt tối thiểu 4,5:1 trên `paper` và `surface`, cả khi làm chữ trên nền pha 12% của chính nó (thấp nhất: `hoa` sáng 4,71:1 trên giấy). `hoa` sáng phải tối từ `#A8481A` xuống `#9C4116` vì bản đầu chỉ đạt 4,20:1.

Cát dùng Kim, Hung dùng Chu Sa, Bình dùng `inkSecondary`. Mọi màu chữ đạt WCAG AA (4,5:1) cho chữ nhỏ trên nền của nó. Kim sáng phải tối xuống `#85601E`: màu vàng đồng `#B8892E` chỉ đạt 2,69:1 trên giấy.

| Cặp | Tỷ lệ |
| :-- | :-- |
| `ink` trên `paper`, sáng | 14,67:1 |
| `inkSecondary` trên `paper`, sáng / tối | 6,58:1 / 7,10:1 |
| `cinnabar` trên `paper` sáng / `surface` tối | 5,06:1 / 4,79:1 |
| `gold` trên `paper` / `surface`, sáng | 4,85:1 / 5,28:1 |
| `gold` trên `paper`, tối | 9,02:1 |

## 5. Kiểm thử

`ChronosTests` (Swift Testing, chạy trong app trên simulator), 116 ca:

- **Đối chiếu với Rust.** Lá số 30/11/1931 âm lịch, giờ Dần, Nam: danh sách `stable_id` của 5 cung (Mệnh, Quan Lộc, Điền Trạch, Nô Bộc, Thiên Di) phải trùng đúng thứ tự với `scan::select` của `astro-data`, hàm mà V13 dùng. Quan Lộc của lá số này vừa mượn Chính Tinh cung xung chiếu, vừa là cung an Thân, nên phủ cả hai nhánh của `tuviPalaceQuery`.
- **Độ phủ.** Mọi ngày năm 2026 có văn bản cho mọi khóa Trạch Nhật; mọi hạn tuổi của người sinh 1940–2015 năm 2026; 8 quái, 8 sao × 4 ứng dụng, 9 sao × 9 vận; Bát Tự chỉ thiếu khóa `bazi.can_chi` (D18).
- **EngineService.** Ngày dân sự GMT+7 qua nửa đêm UTC; mùng 1 Tết Giáp Thìn ra 10/02/2024; sinh trước Tết thuộc năm âm trước; tháng nhuận không có bị từ chối; hồ sơ âm lịch ra cùng lá số với `tuviChartFromLunar`; kinh độ bật giờ Mặt Trời thực.
- **ProfileStore.** Lưu, mở lại, thay, xóa, chọn, trên một `UserDefaults` suite riêng; lần mở đầu tiên rỗng và không ghi gì.
- **Lưới tháng, Thái Tuế.** Tháng 11/2026 có 6 ô trống trước Chủ Nhật mùng 1; mỗi ô tháng 2/2024 khớp `almanacDay` (Tết là "1/1"); cộng trừ tháng qua năm; hồ sơ mẫu năm 2026 là Trực và Hình Thái Tuế; Thái Tuế dùng năm âm sinh.
- **Bát Tự, Phong Thủy.** Hồ sơ mẫu ra Canh Ngọ, Tân Tỵ, Canh Thìn, Tân Tỵ; ranh giới năm giữa Tết và Lập Xuân đổi trụ năm (Kỷ Tỵ hoặc Canh Ngọ); quy ước giờ Tý đổi trụ ngày lúc 23h:30; mọi Can, Thập Thần, Trường Sinh, Nạp Âm, Thần Sát của bảng có văn bản; Thập Thần của Tàng Can được tìm thấy dù `baziKeys` không liệt kê; bước Đại Vận của năm 2026 là Giáp Thân (28–37 tuổi); đủ 8 sao × 4 ứng dụng; bếp tại Sinh Khí là Hung; mỗi quái có 4 hướng tốt, 4 xấu; `LoShuLayout` trùng `loShuRows` với cả 9 sao giữa, hai chiều bay; chiều bay của bốn bàn; ô bàn đọc đúng văn bản của trạng thái vận; tương hợp ra đúng hai khóa; năm màu Ngũ Hành khác nhau ở cả hai chế độ.
- **Tử Vi.** Bàn cờ đặt đủ 12 chi trên viền đúng thứ tự; mỗi cung có đúng 2 cung Tam Hợp và 1 cung Xung Chiếu, đối xứng; mỗi sao rơi vào đúng một nhóm; Tứ Hóa của tuổi Canh nằm ở Thái Dương, Vũ Khúc, Thái Âm, Thiên Đồng; nhãn sao có ký hiệu độ sáng; mọi sao của lá số mẫu có văn bản, chỉ Chính Tinh có Ngũ Hành; năm 2026 là 37 tuổi, Tiểu hạn ở Thìn (tuổi Ngọ khởi tại Thìn); năm trước khi sinh không có hạn; quy tắc tháng nhuận đổi tháng tính và cung Mệnh của người sinh 20 tháng 2 nhuận 2023; cung Mệnh đọc được nhận định chung, đoạn Primary và đính chính; cả 12 cung đọc được.
- **Widget.** 13 khung giờ liền nhau, Tý ở hai đầu; giờ đồng hồ rơi đúng khung; nhãn khung giờ giống tab Lịch; `DaySnapshot` khớp engine; ngày đọc theo giờ Việt Nam (23:30 UTC là 06:30 hôm sau); giờ Hoàng Đạo kế tiếp là khung đang chạy, khung sau trong ngày, hoặc khung đầu của ngày mai; timeline đổi ở mỗi canh giờ và lúc nửa đêm, mục nửa đêm là ngày mới; ba view dựng đúng kích thước ở chế độ sáng và tối.
- **Hồ sơ qua các tab.** Sửa hồ sơ đang chọn báo thay đổi ngay; bốn tab đọc được hồ sơ âm lịch 10 tháng 2 nhuận Quý Mão; xóa hồ sơ đang chọn chuyển sang hồ sơ kế tiếp; đổi loại lịch giữ đúng ngày sinh, kể cả tháng nhuận; ngày âm không tồn tại giữ nguyên số.
- **Form hồ sơ.** Thẻ đối chiếu của 15/05/1990 09:30 ra "Âm lịch: 21 tháng 4 năm 1990" và "Năm Canh Ngọ, Tháng Tân Tỵ, Ngày Canh Thìn, Giờ Tân Tỵ"; nhập âm lịch ra "Dương lịch: Thứ Ba, 15/05/1990" cùng Can Chi; đổi ngày hoặc giờ thì thẻ đổi theo; tháng nhuận được ghi tên; kinh độ thêm giờ Mặt Trời thực; ngày không tồn tại không có thẻ; tháng nhuận của 2023, 2024, 2025; lịch âm được chỉnh cho vừa năm; năm nơi sinh đúng kinh độ đã cho.
- **Ghi chú và nhắc.** Ngày giỗ 15/10 mỗi năm; ngày 30 vào tháng thiếu thành ngày 29 (và engine tự nó từ chối ngày 30 đó); năm nhuận tháng 2 giữ tháng thường; sự kiện tạo ngay trong tháng nhuận; ghi chú không lặp; 29/2 dương lịch; lần tới năm nay hoặc năm sau, kể cả đầu tháng 1 dương còn thuộc năm âm cũ; danh sách 30 ngày xếp gần trước; ngày và tháng tìm ra ghi chú; nháp trên một ngày; `NoteStore` lưu, thay, xóa, báo thay đổi; kế hoạch nhắc 08:00 trước 3, 1, 0 ngày, bỏ giờ đã qua, giới hạn 60, lần xảy ra kế tiếp của ghi chú hằng năm.
- **Sự kiện trên Widget.** Thứ tự và đếm ngược của sự kiện; màn hình khóa ưu tiên sự kiện hôm nay hoặc ngày mai; ghi chú app lưu thì Widget đọc lại được qua cùng suite; cỡ lớn dựng đúng kích thước.
- **Chú thích.** Hai nhóm quái lấy từ engine; hướng tốt thuộc nhóm của quái; trạng thái Cửu Tinh; năm ký hiệu độ sáng.
- **Tốc độ** (Debug, simulator): 1.000 lần đọc mục mất 21 ms, khoảng 21 µs mỗi lần; đọc đủ 12 cung một lá số mất 56 ms.

## 6. Quyết định và việc mở

| Mã | Nội dung | Trạng thái |
| :-- | :-- | :-- |
| I1 | Hồ sơ lưu JSON ở `UserDefaults`, không dùng SwiftData. Hồ sơ mẫu của lần mở đầu tiên **đã bỏ ở Task 5B.6**: danh sách bắt đầu rỗng, các tab hiện `ProfileEmptyStateView` (mục 2.3, 3.7) | **Đã duyệt** 2026-11-11; phần hồ sơ mẫu thay đổi 2026-11-14 theo yêu cầu của chủ dự án |
| I2 | Bundle ID `com.leanhtuan.chronos` | **Đã duyệt** 2026-11-11 |
| I3 | Widget lịch ngày chỉ dùng `AstroCore.xcframework` và `AstroCore.swift`. **Không** nhúng `chronos_content.sqlite` (và không cần `activities.json`), để giữ bộ nhớ Widget dưới giới hạn của iOS. Widget hiện những gì engine tính: ngày dương và âm, Can Chi, tiết khí, Hoàng/Hắc Đạo, Trực, Sao, giờ Hoàng Đạo; **không** hiện Nên/Kỵ (chúng chỉ nằm trong SQLite) | **Đã duyệt** 2026-11-11, chốt phạm vi ở 5B.3; **đã làm** ở 5B.5 (mục 3.6) |
| I4 | Simulator chỉ arm64; Mac Intel cần thêm target `x86_64-apple-ios` vào `build-ios.sh` | Giữ như Task 1 |
| I5 | Xung tuổi theo năm = quan hệ của chi năm sinh với Thái Tuế (chi của năm đang xét), tính bằng phép Địa Chi có sẵn trong core, không thêm văn bản vào SQLite. Mục 6.1 | **Đã duyệt** 2026-11-11; làm ở 5B.2 |
| I6 | Bốn quy ước của Task 5B.6: (1) thông báo nhắc bắn lúc 08:00 giờ Việt Nam (GMT+7) dù máy ở múi giờ khác (mục 3.8); (2) ghi chú dương lịch hằng năm vào 29/2 tính ngày 28/2 ở năm không nhuận; (3) ghi chú hằng năm chỉ có từ năm của nó trở đi; (4) thẻ đối chiếu của form hồ sơ lấy Can Chi năm, tháng, ngày theo âm lịch và giờ theo đồng hồ (Tý chia nửa đêm), nên có thể khác trụ năm của tab Bát Tự với người sinh giữa Tết và Lập Xuân (mục 3.7) | **Đã duyệt** 2026-11-14 |
| I7 | Bỏ `CODE_SIGN_ENTITLEMENTS` (App Groups) khỏi project để ký được bằng Apple ID miễn phí; `NoteStorage` tự dùng `UserDefaults.standard` khi không có quyền, Widget không thấy ghi chú cho đến khi bật lại (mục 1.1) | Task 5B.7, theo yêu cầu của chủ dự án 2026-11-14 |

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
