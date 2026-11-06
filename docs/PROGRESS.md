# Tiến trình dự án — Viet Astro Engine (Chronos)

> File này được theo dõi trong git, cùng với cả thư mục `docs/`.
> Cập nhật lần cuối: **2026-11-06**. Giai đoạn 3 (Bát Tự & Trạch Nhật): **hoàn thành 100%**.
> Giai đoạn 4A (Phong Thủy Bát Trạch & Cửu Cung Phi Tinh): **hoàn thành**, F1–F5 đã chốt.

## 1. Trạng thái

| Hạng mục | Trạng thái | Ghi chú |
|---|---|---|
| Workspace `viet-astro-engine` (3 crate) | ✅ Xong | `astro-core`, `astro-ffi`, `astro-wasm` (Task 1) |
| Thiên văn: Julian Day, ΔT, Mặt Trời, điểm Sóc | ✅ Xong | Thuật toán Meeus đầy đủ |
| 24 tiết khí | ✅ Xong | Giải lặp hội tụ; lệch cỡ chục giây so với số liệu công bố |
| Lịch Âm – Dương, tháng nhuận | ✅ Xong | Quy tắc Hồ Ngọc Đức; `tz_offset` mặc định 7.0 |
| Can Chi năm / tháng / ngày / giờ | ✅ Xong | Công thức Hồ Ngọc Đức |
| Giờ mặt trời thực | ✅ Xong | Kinh độ + phương trình thời gian |
| Tứ Trụ (lập 4 trụ) | ✅ Xong | `bazi_pillars.rs`; cấu hình `SolarTerms`/`LunarNewYear`, mặc định `SplitAtMidnight` |
| Cầu nối Web (WASM + TypeScript) | ✅ Xong | Build và test trên Node đều chạy |
| Cầu nối iOS (UniFFI → Swift) | 🟡 Gần xong | Thiếu bước đóng gói `AstroCore.xcframework` (cần Xcode) |
| Đặc tả Bát Tự & Trạch Nhật | ✅ Xong | `docs/BAZI_ALMANAC_SPEC.md`; Q1–Q10 đã chốt, còn Q11, Q12 |
| Bát Tự — phân tích lá số | ✅ Xong | `bazi_analysis.rs`: Tàng Can, Thập Thần, Trường Sinh, Thần Sát Mệnh Lý, Đại Vận |
| Trạch Nhật, hạn tuổi | ✅ Xong | `almanac.rs` |
| Tử Vi Đẩu Số | ⏳ Chưa làm | Chờ bảng quy tắc qua `/ingest-rules` |
| Đặc tả Phong Thủy | ✅ Xong | `docs/FENGSHUI_SPEC.md`; F1–F5 đã chốt |
| Phong Thủy — Bát Trạch, Cửu Tinh | ✅ Xong | `fengshui.rs`: Mệnh Quái, Trạch Quái, Bát Biến, Vận, Niên/Nguyệt/Nhật/Thời Tinh, bàn Cửu Cung; có trên Swift và WASM |
| App iOS SwiftUI (Phase 5 – UI) | ⏳ Chưa làm | Cần Xcode |
| Web app | ⏳ Chưa làm | |
| Commit lên git | ✅ Xong | `332c75e`, `cb32eda` trên `origin/develop` (GitHub, qua SSH) |

## 2. Việc tiếp theo

- [ ] Cài Xcode → chạy `./build-ios.sh` → kéo `build/ios/AstroCore.xcframework` và
      `build/ios/AstroCore.swift` vào project Xcode, gọi thử `checkConnection`
- [ ] Đối chiếu Trực, Sao, Thần của các vector kiểm thử với lịch in (Q12)
- [ ] Đối chiếu Can Chi tháng nhuận (mục 4) với sách nguồn
- [x] Commit và push phần nền móng lên `origin/develop`
- [x] Chốt F1–F5 ở `docs/FENGSHUI_SPEC.md`
- [x] Viết module Phong Thủy trong `astro-core`, đưa lên Swift và WASM
- [ ] Đối chiếu 3 vector Phong Thủy với lịch in
- [ ] Đưa bảng quy tắc Tử Vi vào qua `/ingest-rules`
- [ ] Bắt đầu app iOS (SwiftUI) dùng XCFramework
- [x] Commit Giai đoạn 3 cùng `docs/`, `.gitignore`, `THIRD_PARTY_NOTICES.md`
- [x] Push các commit của Giai đoạn 3 lên `origin/develop`

## 3. Nhật ký

### 2026-11-06 — Task 4A.2: `fengshui.rs` và cầu nối

- Đóng F1–F5: mục 3.2 khớp 64/64 ô với bảng *Bát Trạch Minh Cảnh* của chủ dự án; Trạch Quái theo Tọa;
  `generate_loshu_grid(center_star, forward)`; Vận cùng khóa Lập Xuân với Niên Tinh; giữ đúng `getDayNineStar`.
- Thêm `fengshui.rs` với `Trigram`, `Palace`, `CompassDirection`, `EightMansionsStar`, `NineStar`, `KuaInfo`,
  `LoShuGrid`. `Gender` dùng lại của `bazi_analysis`. Swift và WASM có thêm 49 hàm mỗi bên.
- `LoShuGrid` có 9 trường theo tên cung, vì UniFFI không hỗ trợ mảng cố định.
- `tz_offset = 8.0`: Nhật Tinh và Thời Tinh khớp `lunar-ts` trên mọi ngày 1600–2400; Niên, Nguyệt Tinh mặc định, Nguyên,
  Vận khớp trên 73.353 ngày 1900–2100. Ngoài dải đó Niên Tinh lệch 1 ngày, Nguyệt Tinh lệch 5 ngày, đều do Tiết sát nửa đêm.
- `tz_offset = 7.0`: khác nguồn đúng như bảng mục 0.3 của đặc tả. Đường nối Nhật Tinh là 13 lần, 241 ngày, thêm 1/1/2078.
- Bốn test khóa kết quả nguồn bằng mã băm FNV-1a; cách tạo lại ở mục 5 của đặc tả.
- Kiểm thử đột biến: vòng đầu 606 đột biến, 37 đột biến thật còn sống, chủ yếu do `tz_offset` ở UTC+7 chưa được ghim.
  Sau khi thêm test: 740 đột biến, 728 bị bắt, 12 còn lại không đổi hành vi nên không có test nào bắt được.
- Kiểm chứng: 201 test core, 18 test FFI, 23 test WASM đều qua; chương trình Swift thật chạy qua bindings sinh ra.
  Chưa đóng gói `AstroCore.xcframework` vì chưa có Xcode. Vector chưa đối chiếu với lịch in.

### 2026-11-05 — Task 4A.1: đặc tả Phong Thủy

- Tạo `docs/FENGSHUI_SPEC.md`: Bát Quái và Lạc Thư, Mệnh Quái, ma trận Bát Du Niên, 9 sao, Tam Nguyên Cửu Vận,
  Niên/Nguyệt/Nhật/Thời Tinh, đường bay Lạc Thư, 3 vector kiểm thử. Chưa viết code Rust.
- Công thức Mệnh Quái của Task 4A.1 đúng: `k` của Nam bằng Niên Tinh của `lunar-ts` trên 200 năm 1900–2099.
- Bảng tiết khí trong `Lunar` đi theo năm dương lịch, không theo năm âm lịch (`Lunar.ts:100`).
- Phát hiện lỗi trong nguồn, đặc tả đã tránh: `LunarYear.getNineStar` sai ngoặc, `LunarTime.getNineStar` thiếu nhánh
  `DONG_ZHI`. Nhật Tinh có 225 ngày nhảy sao tại 1/1 (F5).
- Kiểm chứng: quét 73.353 ngày 1900–2100, Niên, Nguyệt (cả 3 `sect`), Nhật, Thời Tinh đều không lệch nguồn; đọc ngược
  đặc tả, 158.647 phép kiểm đều qua; `astro-core` ở UTC+7 khớp 3 vector ở JDN, ngày âm, Can Chi, Chi tháng.
- Ma trận Bát Du Niên chưa có sách đối chiếu, chờ chủ dự án duyệt (F1). Vector chưa đối chiếu với lịch in.

### 2026-11-05 — Giai đoạn 3: Bát Tự và Trạch Nhật

- Chốt Q1–Q9 theo quyết định của chủ dự án; thêm mục 1.3 (quan hệ Can Chi), 2.5 (Thần Sát Mệnh Lý), 2.6 (Đại Vận)
  vào đặc tả. Bỏ `docs/` khỏi `.gitignore`; thêm `THIRD_PARTY_NOTICES.md`.
- Đổi tên `sexagenary.rs` → `canchi.rs`, `four_pillars.rs` → `bazi_pillars.rs`, `CelestialStem` → `Stem`,
  `EarthlyBranch` → `Branch` (cả API Swift và TypeScript).
- Thêm `bazi_analysis.rs`, `almanac.rs`; cầu nối có thêm `analyzeBazi`, `almanacDay`, `ageRestrictions` và các hàm tên.
- Đại Vận theo `Yun.ts` với `sect = 2` (tính theo phút), không phải mặc định `sect = 1` của `EightChar.getYun`.
- Kiểm chứng: 135 test core, 6 test FFI, 7 test WASM đều qua, gồm 3 vector của đặc tả và test đối chiếu từng ô 15 Thần Sát
  với `DAY_SHEN_SHA`. Xuất bảng từ Rust rồi so với đặc tả và `vietnam_custom_rules.md`: 17/17 nhóm khớp.
- Chốt tên `enum` là `Stem`, `Branch` (đã sửa `CLAUDE.md`). Chủ dự án xác nhận 6 cặp Lục Phá (Q10), khớp code.
- Thêm `main_hidden_stem`, assert JDN cho 3 vector; WASM có thêm `mainHiddenStem`, `stemName`, `branchName`.
- **Giai đoạn 3 hoàn thành 100%.** Còn hai việc đối chiếu không chặn code: Q11 Can Chi tháng nhuận, Q12 vector với lịch in.

### 2026-11-05 — Task 3.0: đặc tả Bát Tự & Trạch Nhật

- Tạo `docs/BAZI_ALMANAC_SPEC.md` từ `.ref/lunar-ts` (Tàng Can, Nạp Âm, Trường Sinh, Thập Thần, 12 Trực,
  28 Sao, 12 Thần Hoàng/Hắc Đạo, 15 Thần Sát) và `docs/vietnam_custom_rules.md` (giữ nguyên).
- `LunarUtil` không có sẵn danh sách 15 Thần Sát: đã giải mã `DAY_SHEN_SHA` ra quy tắc theo tháng, kèm 15 ô
  ngoại lệ; quy tắc cộng ngoại lệ tái tạo đúng dữ liệu nguồn.
- 28 Sao quy được về công thức `(JDN + 11) mod 28`.
- Phát hiện lệch: Vãng Vong tháng 9, 10 khác nhau giữa hai nguồn (Q1); Phần D vẫn giữ nguyên.
- Kiểm chứng: đọc ngược đặc tả, 32/32 kiểm tra so với nguồn đều qua; 3 vector khớp `astro-core` ở JDN, ngày âm,
  Can Chi ngày, Tiết. Trực, Sao, Thần trong vector chưa đối chiếu với lịch in.

### 2026-11-03 — Task 2B: `solar_terms.rs` và `true_solar_time.rs`

- Đổi tên module: `solar_term.rs` → `solar_terms.rs`, `solar_time.rs` → `true_solar_time.rs`.
- Bộ giải tiết khí chuyển sang Newton-Raphson thật (đạo hàm sai phân trung tâm) trên
  `get_sun_longitude_deg`; hội tụ dưới 1 ms. Thêm `get_exact_solar_term_time(year, term, tz)`.
- Lập Xuân (GMT+7): 2025 → 21:10:23 ngày 3/2; 2026 → 03:02:02 ngày 4/2. So với số công bố lệch
  khoảng 10 giây, do ΔT ngoại suy chứ không do bộ giải.
- `true_solar_time.rs`: `equation_of_time_minutes`, `longitude_offset_minutes`,
  `to_true_solar_time(clock, kinh độ, kinh tuyến chuẩn)`, `VIETNAM_STANDARD_MERIDIAN = 105.0`,
  hằng số kinh độ 7 thành phố và `MAJOR_CITIES`.
- Sinh 22:55 ở TP.HCM: ngày 3/11 thành 23:17 (giờ Tý sớm), ngày 11/2 thành 22:47 (vẫn giờ Hợi).
- Swift và JS có thêm `getExactSolarTermTime`.
- Kiểm chứng: 84 test core, 5 test FFI, 6 test WASM đều qua.

### 2026-11-03 — Task 2A: module `astronomy` và `lunar_calendar`

- Tách module theo spec: `astronomy.rs` (kèm `astronomy/` gồm `delta_t`, `moon_phase`, `sun`,
  `vsop87_earth`) và `lunar_calendar.rs` (đổi tên từ `lunar.rs`).
- Thêm API theo spec: `jd_from_date`, `jd_to_date`, `get_new_moon_day` (đếm `k` từ Sóc 1/1/1900),
  `get_sun_longitude_deg`, `get_solar_term_index`; `solar_to_lunar(day, month, year, tz)`.
- Khác spec có chủ ý: `solar_to_lunar`/`lunar_to_solar` trả `Result` thay vì giá trị trần, vì đầu
  vào có thể không tồn tại (30/2, tháng nhuận không có).
- Hai ví dụ GMT+7/GMT+8 trong spec (Tết 1978, 23/01/2033) thực tế **không** lệch giữa VN và TQ;
  test dùng các ca lệch thật: Tết 1968, 1985, 2007, 2030; tháng nhuận 1984, 1985, 1987.
- Kiểm chứng: 80 test core, 5 test FFI, 6 test WASM đều qua.

### 2026-11-03 — Commit và push

- Cấu hình danh tính git toàn máy: Le Anh Tuan <tuanla0307@gmail.com>.
- Đổi `origin` sang SSH (`git@github.com:Tuanla030799/Chronos.git`).
- Commit `332c75e` (chore: cấu hình dự án) và `cb32eda` (feat: workspace và engine), rồi push
  nhánh `develop`. Đây là nhánh đầu tiên lên repo trống nên GitHub đặt `develop` làm nhánh mặc định.

### 2026-11-02 — Task 1: chỉnh workspace theo spec

- Gộp trình sinh bindings vào `astro-ffi` (`[[bin]] uniffi-bindgen`, bật bằng `--features bindgen`)
  nên workspace còn đúng 3 crate.
- Thêm hàm kiểm tra kết nối `check_connection(SolarDate)` ở core, xuất ra Swift (`checkConnection`)
  và JS (`checkConnection`); trả về ngày đã kiểm tra, JDN và phiên bản engine.
- `build-ios.sh`: chỉ build `aarch64-apple-ios` và `aarch64-apple-ios-sim`, kết quả
  `build/ios/AstroCore.xcframework` cùng `build/ios/AstroCore.swift`.
- Kiểm chứng: 71 test core, 5 test FFI, 6 test WASM đều qua; chương trình Swift gọi
  `checkConnection` chạy đúng cả nhánh lỗi. `build-ios.sh` đã chạy hết các bước với một
  `xcodebuild` giả: static lib đúng platform, header và modulemap đủ. Bước `xcodebuild` thật chưa
  chạy vì chưa có Xcode.
- Version kiểm tra lại, vẫn là bản mới nhất: uniffi 0.32.2, wasm-bindgen 0.2.130, serde 1.0.229.

### 2026-11-02 — Nền móng

- Cài Rust stable 1.99.1 qua rustup (kèm target `wasm32-unknown-unknown`, `aarch64-apple-ios`,
  `aarch64-apple-ios-sim`, `x86_64-apple-ios`) và `wasm-pack` qua Homebrew.
- Viết toàn bộ `astro-core`, hai cầu nối và `build-ios.sh`.
- Kiểm chứng: 69 test core, 4 test FFI và 5 test WASM (trên Node) đều qua; clippy `-D warnings`,
  rustfmt và rustdoc sạch. Đã biên dịch, chạy một chương trình Swift thật gọi vào Rust (macOS) và
  gọi package JS thật từ Node.
- Phát hiện: **Tết 2030 ở Việt Nam là 2/2/2030**, ở Trung Quốc là 3/2. Sóc lúc 16:07 UTC, tức
  23:07 giờ Hà Nội.
- Trên máy này static lib iOS (thiết bị + simulator) và Swift bindings sinh từ lib iOS đều build
  được; chỉ thiếu `xcodebuild` vì chưa cài Xcode.

## 4. Vấn đề mở — cần xác nhận

1. **Giờ Tý muộn (23:00–23:59): đã có căn cứ.** Ở cả hai quy ước, trụ giờ là giờ Tý của *ngày hôm
   sau*; chỉ trụ ngày khác nhau. `lunar-ts` xác nhận: `_computeTime` dùng `dayGanIndexExact`;
   `sect = 2` ứng với `SplitAtMidnight`, `sect = 1` ứng với `NewDayAtRatHour`.
2. **Can Chi tháng nhuận:** đang lấy theo tháng mà nó lặp lại, như `amlich.js` của Hồ Ngọc Đức.
   Cần đối chiếu.
3. **Quy ước giờ Tý mặc định: đã chốt** là `SplitAtMidnight`. Dự án gọi 23:00–23:59 là giờ Tý Sớm.
4. **Phong Thủy F1–F5: đã chốt** ở Task 4A.2, xem mục 6.1 của `docs/FENGSHUI_SPEC.md`. Còn lại việc đối chiếu vector với
   lịch in, không chặn code.

## 5. Quyết định kỹ thuật

- **Độ chính xác:** dùng thuật toán Meeus đầy đủ (VSOP87 theo Phụ lục III; Sóc theo chương 49 với
  25 số hạng tuần hoàn và 14 đối số hành tinh) thay cho chuỗi rút gọn trong bài của Hồ Ngọc Đức.
  Khi Sóc rơi trong vài phút quanh nửa đêm, kết quả có thể khác `amlich.js`; bản này chính xác hơn.
- **ΔT:** đa thức Espenak–Meeus (2006); sau 2005 là ngoại suy, khoảng +5 giây vào năm 2025.
- **Phạm vi:** năm 1600–2400, lịch Gregory proleptic.
- **Tứ Trụ:** trụ năm và trụ tháng so *thời khắc tuyệt đối* với tiết khí nên không phụ thuộc múi
  giờ hay giờ mặt trời thực; trụ ngày và trụ giờ dùng giờ địa phương (giờ đồng hồ, hoặc giờ mặt
  trời thực khi có kinh độ).
- **Đặt tên:** enum dùng tiếng Việt không dấu; lưu ý `EarthlyBranch::Ty` là Tý, `EarthlyBranch::Ti`
  là Tỵ.
- **Cầu nối:** UniFFI dùng `#[uniffi::remote]` nên Swift nhận đúng struct/enum của core, không có
  lớp DTO. WASM nhận và trả object JS (camelCase) qua serde; lỗi được ném ra dưới dạng `Error` của JS.
- **Chất lượng mã:** Rust edition 2024, `#![forbid(unsafe_code)]`; `astro-core` không có dependency
  bắt buộc (serde là feature tùy chọn).
- **Phong Thủy:** Mệnh Quái theo năm âm lịch; Niên Tinh, Nguyên, Vận đổi tại ngày Lập Xuân. Giờ mặt trời thực chỉ dùng cho
  Nhật Tinh và Thời Tinh, như trụ năm, trụ tháng của Tứ Trụ. `boundary` của Swift là `Optional` vì UniFFI không có giá trị
  mặc định kiểu enum; WASM dùng `unchecked_optional_param_type` để `nineStars(time)` qua được kiểm tra TypeScript.
- **Tên workspace:** workspace ảo của Cargo không có trường `name`, nên tên `viet-astro-engine`
  được ghi ở comment đầu `Cargo.toml`; thư mục repo vẫn là `Chronos`.
- **iOS:** simulator chỉ build cho Apple Silicon (`aarch64-apple-ios-sim`) theo Task 1. Muốn chạy
  simulator trên Mac Intel thì thêm target `x86_64-apple-ios` và gộp hai lib simulator bằng `lipo`.

## 6. Bản đồ mã nguồn

| Đường dẫn | Nội dung |
|---|---|
| `crates/astro-core/src/engine.rs` | `ENGINE_VERSION`, hàm kiểm tra kết nối `check_connection` |
| `crates/astro-core/src/time.rs` | `SolarDate`, `SolarDateTime`, JDN ↔ ngày, JD theo `tz_offset` |
| `crates/astro-core/src/astronomy.rs` | API thiên văn theo Task 2A: JDN, Sóc, kinh độ Mặt Trời, chỉ số Trung khí |
| `crates/astro-core/src/astronomy/delta_t.rs` | ΔT = TT − UT |
| `crates/astro-core/src/astronomy/sun.rs`, `vsop87_earth.rs` | Kinh độ biểu kiến Mặt Trời, chương động, phương trình thời gian |
| `crates/astro-core/src/astronomy/moon_phase.rs` | Điểm Sóc thật |
| `crates/astro-core/src/solar_terms.rs` | `enum SolarTerm`, thời khắc giao tiết (Newton-Raphson) |
| `crates/astro-core/src/lunar_calendar.rs` | Đổi Âm ↔ Dương, tháng nhuận |
| `crates/astro-core/src/canchi.rs` | `Stem`, `Branch`, `FiveElements`, `YinYang`, `CanChi`, `NaYin`, Xung, Hợp, Hình, Hại, Phá |
| `crates/astro-core/src/true_solar_time.rs` | Giờ mặt trời thực, kinh độ các thành phố |
| `crates/astro-core/src/bazi_pillars.rs` | Tứ Trụ, quy ước giờ Tý, mốc chuyển năm/tháng |
| `crates/astro-core/src/bazi_analysis.rs` | Tàng Can, Thập Thần, Trường Sinh, Thần Sát Mệnh Lý, Đại Vận |
| `crates/astro-core/src/almanac.rs` | 12 Trực, 28 Sao, 12 Thần, 15 Thần Sát, ngày kỵ Việt Nam, hạn tuổi |
| `crates/astro-core/src/fengshui.rs` | Bát Trạch, Mệnh Quái, Trạch Quái, Bát Biến, Tam Nguyên Cửu Vận, Cửu Tinh, bàn Cửu Cung |
| `crates/astro-core/src/day_info.rs` | Dữ liệu cho một ô lịch |
| `crates/astro-ffi/` | Cầu nối UniFFI (Swift module `AstroCore`) |
| `crates/astro-ffi/src/bin/uniffi-bindgen.rs` | Trình sinh bindings, cùng phiên bản `uniffi` với thư viện |
| `crates/astro-wasm/` | Cầu nối WebAssembly và khai báo TypeScript |
| `build-ios.sh` | Build static lib iOS, sinh Swift bindings, đóng gói `AstroCore.xcframework` |

## 7. API công khai

| Chức năng | Rust (`astro_core`) | Swift (`AstroCore`) | JS / TS (`astro-wasm`) |
|---|---|---|---|
| Kiểm tra kết nối | `check_connection(date)` | `checkConnection(date:)` | `checkConnection(date)` |
| Dương → Âm | `solar_to_lunar(day, month, year, tz_offset)` | `solarToLunar(date:tzOffset:)` | `solarToLunar(date, tz_offset?)` |
| Âm → Dương | `lunar_to_solar(date, tz_offset)` | `lunarToSolar(date:tzOffset:)` | `lunarToSolar(date, tz_offset?)` |
| Các tháng của một năm âm | `lunar_year(year, tz_offset)` | `lunarYear(year:tzOffset:)` | `lunarYear(year, tz_offset?)` |
| 24 tiết khí trong năm | `solar_terms_in_year(year, tz_offset)` | `solarTermsInYear(year:tzOffset:)` | `solarTermsInYear(year, tz_offset?)` |
| Dữ liệu một ngày | `day_info(date, tz_offset)` | `dayInfo(date:tzOffset:)` | `dayInfo(date, tz_offset?)` |
| Tứ Trụ | `four_pillars(birth, &options)` | `fourPillars(birth:options:)` | `fourPillars(birth, options)` |
| Phân tích lá số | `analyze_bazi(birth, &options, gender)` | `analyzeBazi(birth:options:gender:)` | `analyzeBazi(birth, options, gender)` |
| Xem ngày | `almanac_day(date, tz_offset)` | `almanacDay(date:tzOffset:)` | `almanacDay(date, tz_offset?)` |
| Hạn tuổi | `age_restrictions(birth_year, target_year)` | `ageRestrictions(birthLunarYear:targetLunarYear:)` | `ageRestrictions(birth_lunar_year, target_lunar_year)` |
| Mệnh Quái | `kua_info(birth_lunar_year, gender)` | `kuaInfo(birthLunarYear:gender:)` | `kuaInfo(birth_lunar_year, gender)` |
| Trạch Quái | `house_trigram(sitting)`, `house_trigram_from_facing(facing)` | `houseTrigram(sitting:)`, `houseTrigramFromFacing(facing:)` | `houseTrigram(sitting)`, `houseTrigramFromFacing(facing)` |
| Tra sao Bát Trạch | `Trigram::star_towards(other)`, `star_at(direction)`, `chart()` | `starTowards(trigram:other:)`, `starAt(trigram:direction:)`, `eightMansionsChart(trigram:)` | `starTowards(trigram, other)`, `starAt(trigram, direction)`, `eightMansionsChart(trigram)` |
| Tương hợp Bát Biến | `kua_compatibility(first, second)` | `kuaCompatibility(first:second:)` | `kuaCompatibility(first, second)` |
| Cửu Tinh của một thời điểm | `nine_stars(time, tz_offset, boundary)` | `nineStars(time:tzOffset:boundary:)` | `nineStars(time, tz_offset?, boundary?)` |
| Từng sao | `year_nine_star`, `month_nine_star`, `day_nine_star`, `hour_nine_star` | `yearNineStar`, `monthNineStar`, `dayNineStar`, `hourNineStar` | `yearNineStar`, `monthNineStar`, `dayNineStar`, `hourNineStar` |
| Theo số năm | `star_of_year(year)`, `van_of_year(year)`, `nguyen_of_year(year)` | `starOfYear(year:)`, `vanOfYear(year:)`, `nguyenOfYear(year:)` | `starOfYear(year)`, `vanOfYear(year)`, `nguyenOfYear(year)` |
| Bàn Cửu Cung | `generate_loshu_grid(center_star, forward)`, `LoShuGrid::star_at`, `rows` | `generateLoshuGrid(centerStar:forward:)`, `loShuStarAt(grid:palace:)`, `loShuRows(grid:)` | `generateLoshuGrid(center_star, forward?)`, `loShuStarAt(grid, palace)`, `loShuRows(grid)` |
| Thời khắc giao tiết | `get_exact_solar_term_time(year, term, tz_offset)` | `getExactSolarTermTime(year:term:tzOffset:)` | `getExactSolarTermTime(year, term, tz_offset?)` |
| Giờ mặt trời thực | `to_true_solar_time(clock, longitude, standard_meridian)` | `trueSolarTime(clock:longitude:tzOffset:)` | `trueSolarTime(clock, longitude, tz_offset?)` |
| Tên hiển thị | `CanChi::name_vi()`, `name_han()`, `SolarTerm::name_vi()`… | `canChiName`, `canChiHanName`, `stemName`, `branchName`, `solarTermName`, `trigramName`, `nineStarName`… | `canChiName`, `canChiHanName`, `solarTermName`, `trigramName`, `nineStarName`… |

`tz_offset` mặc định là 7.0 ở Swift và JS; `boundary` mặc định `LapXuanDay`, `forward` mặc định `true`. Rust còn có các hàm cấp thấp như
`sun::apparent_longitude`, `sun::equation_of_time`, `moon_phase::new_moon_jd_ut`,
`solar_term::solar_term_jd_ut`, `canchi::{year,month,day,hour}_can_chi`.

## 8. Kiểm chứng

```bash
cargo test --workspace                                  # 201 test core + 18 test FFI
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
wasm-pack build crates/astro-wasm --target web
wasm-pack test --node crates/astro-wasm                 # 23 test
./build-ios.sh                                          # cần Xcode
```

Dữ liệu đối chiếu trong test:

- Ví dụ mẫu của Meeus: 7.a, 22.a, 25.b, 28.a, 49.a
- Xuân/thu phân và hạ/đông chí 2000, 2024, 2025 (USNO), sai lệch dưới 1,5 phút
- Lập Xuân 2023–2025 (lịch Trung Quốc, đổi sang UTC)
- 8 thời điểm Sóc quan sát được (1968–2025)
- Tết Việt Nam 1985–2030; các năm nhuận và không nhuận 2001–2028
- Tết lệch giữa VN và TQ: 1968, 1985, 2007, 2030; tháng 11 nhuận năm 2033 (TQ)
- Quét nhất quán mọi tháng âm từ 1900 đến 2100
- Ma trận Bát Trạch: bảng *Bát Trạch Minh Cảnh* của chủ dự án, quy tắc đổi hào, tính đối xứng
- Cửu Tinh: 3 vector của đặc tả, các ca ở UTC+7 trong mục 0.3, 4 mã băm kết quả `lunar-ts`

## 9. Môi trường

| Thành phần | Phiên bản / trạng thái |
|---|---|
| macOS | 27.0.1, Apple Silicon (arm64) |
| Rust | 1.99.1 stable (rustup; PATH nạp từ `~/.zshenv`, `~/.profile`); target theo `rust-toolchain.toml`: `wasm32-unknown-unknown`, `aarch64-apple-ios`, `aarch64-apple-ios-sim` |
| wasm-pack | 0.15.1 (Homebrew) |
| Node.js | 22.23.2 |
| Swift | 6.4 (Command Line Tools) |
| Xcode | **Chưa cài** |
| Crate chính | uniffi 0.32.2 · wasm-bindgen 0.2.130 · serde-wasm-bindgen 0.6.5 · serde 1.0.229 |
