# Đặc tả ứng dụng iOS (IOS_APP_SPEC)

> Giai đoạn 5B. Task 5B.1 dựng dự án, tầng dịch vụ và khung điều hướng. Giao diện chi tiết từng tab làm ở 5B.2–5B.5.

## 1. Dự án Xcode

| Mục | Giá trị |
| :-- | :-- |
| Vị trí | `apps/ios/Chronos.xcodeproj`, mã nguồn ở `apps/ios/Chronos/`, test ở `apps/ios/ChronosTests/` |
| Hệ điều hành | iOS 17.0 trở lên, chỉ iPhone, khóa dọc |
| Ngôn ngữ | Swift 6 (kiểm tra đồng thời đầy đủ), SwiftUI, Observation (`@Observable`) |
| Bundle ID | `com.leanhtuan.chronos`, test `com.leanhtuan.chronos.tests` (I2) |
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

## 3. Điều hướng

`MainTabView` gồm 5 tab, mỗi tab một `NavigationStack` có menu hồ sơ ở góc phải (chọn nhanh, mở "Quản lý hồ sơ").

| Tab | Đã có ở 5B.1 | Việc của bước sau |
| :-- | :-- | :-- |
| Lịch Vạn Niên (`CalendarTab`) | Chọn ngày; ngày âm, Can Chi năm/tháng/ngày, tiết khí, Giờ Hoàng Đạo, Trực/Sao/Thần/ngày kỵ kèm Nên/Kỵ | 5B.2: lưới tháng, ô ngày, Widget |
| Xem tuổi (`AlmanacTab`) | Tuổi mụ, Tam Tai, Kim Lâu, Hoang Ốc theo năm âm chọn | 5B.3: lọc ngày theo việc, xung tuổi (I5) |
| Bát Tự (`BaziTab`) | Bàn Tứ Trụ (Thập Thần, Can, Chi, Nạp Âm, Trường Sinh), các mục luận giải | 5B.3: Tàng Can, Đại Vận, Thần Sát theo trụ |
| Tử Vi (`TuViTab`) | Địa Bàn 12 cung quanh Thiên Bàn; chạm cung mở sheet: sao, đoạn Primary theo section, "Xem thêm" cho đoạn More, mục tra cứu | 5B.4: vẽ đủ phụ tinh, Đại Hạn, Tiểu Hạn |
| Phong Thủy (`FengShuiTab`) | Mệnh Quái, 8 hướng Bát Trạch, tương hợp với hồ sơ khác, Cửu Tinh năm/tháng/ngày/giờ theo trạng thái vận | 5B.5: bàn Cửu Cung, Trạch Quái theo hướng nhà |

Tab nào cần hồ sơ mà chưa có thì hiện hướng dẫn thêm hồ sơ. Lỗi engine hiện ngay trong tab, không làm app dừng.

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

`ChronosTests` (Swift Testing, chạy trong app trên simulator), 27 ca:

- **Đối chiếu với Rust.** Lá số 30/11/1931 âm lịch, giờ Dần, Nam: danh sách `stable_id` của 5 cung (Mệnh, Quan Lộc, Điền Trạch, Nô Bộc, Thiên Di) phải trùng đúng thứ tự với `scan::select` của `astro-data`, hàm mà V13 dùng. Quan Lộc của lá số này vừa mượn Chính Tinh cung xung chiếu, vừa là cung an Thân, nên phủ cả hai nhánh của `tuviPalaceQuery`.
- **Độ phủ.** Mọi ngày năm 2026 có văn bản cho mọi khóa Trạch Nhật; mọi hạn tuổi của người sinh 1940–2015 năm 2026; 8 quái, 8 sao × 4 ứng dụng, 9 sao × 9 vận; Bát Tự chỉ thiếu khóa `bazi.can_chi` (D18).
- **EngineService.** Ngày dân sự GMT+7 qua nửa đêm UTC; mùng 1 Tết Giáp Thìn ra 10/02/2024; sinh trước Tết thuộc năm âm trước; tháng nhuận không có bị từ chối; hồ sơ âm lịch ra cùng lá số với `tuviChartFromLunar`; kinh độ bật giờ Mặt Trời thực.
- **ProfileStore.** Lưu, mở lại, thay, xóa, chọn, trên một `UserDefaults` suite riêng.
- **Tốc độ** (Debug, simulator): 1.000 lần đọc mục mất 21 ms, khoảng 21 µs mỗi lần; đọc đủ 12 cung một lá số mất 56 ms.

## 6. Quyết định và việc mở

| Mã | Nội dung | Trạng thái |
| :-- | :-- | :-- |
| I1 | Hồ sơ lưu ở `UserDefaults` thay vì SwiftData: dữ liệu nhỏ, một mảng; Widget đọc được qua App Group suite mà không cần chung model container | Đã chọn ở 5B.1, chủ dự án xem lại |
| I2 | Bundle ID `com.leanhtuan.chronos` đặt theo tên trong git; cần đổi nếu tài khoản Apple Developer dùng tiền tố khác | Chờ xác nhận |
| I3 | Widget và App Group (mục 2.5 của `INTERPRETATION_DB_SPEC.md`): SQLite thuộc cả hai target hay đặt ở App Group | Chốt ở 5B.2 |
| I4 | Simulator chỉ arm64; Mac Intel cần thêm target `x86_64-apple-ios` vào `build-ios.sh` | Giữ như Task 1 |
| I5 | "Xung tuổi theo năm" chưa có hàm engine, khóa hay dữ liệu luận giải. Cần quy tắc từ nguồn đã duyệt trước khi làm, không tự đặt | Chờ nguồn, trước 5B.3 |
