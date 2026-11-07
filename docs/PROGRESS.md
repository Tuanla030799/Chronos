# Tiến trình dự án — Viet Astro Engine (Chronos)

> File này được theo dõi trong git, cùng với cả thư mục `docs/`.
> Cập nhật lần cuối: **2026-11-07**. Giai đoạn 3 (Bát Tự & Trạch Nhật): **hoàn thành 100%**.
> Giai đoạn 4A (Phong Thủy Bát Trạch & Cửu Cung Phi Tinh): **hoàn thành**, F1–F5 đã chốt.
> Giai đoạn 4B (Tử Vi Đẩu Số): **hoàn thành**, T1–T5, T8, T9 đã chốt. **Giai đoạn 4 hoàn thành.**
> Giai đoạn 5A (Cơ sở dữ liệu luận giải): D1–D16 đã chốt. TV-01 đến TV-06, TV-15 đã duyệt (487/488 đoạn); TV-07 đến TV-11 đã nạp, chờ duyệt.

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
| Cầu nối iOS (UniFFI → Swift) | ✅ Xong | `build/ios/AstroCore.xcframework` đã đóng gói bằng Xcode 27.2 |
| Đặc tả Bát Tự & Trạch Nhật | ✅ Xong | `docs/BAZI_ALMANAC_SPEC.md`; Q1–Q10 đã chốt, còn Q11, Q12 |
| Bát Tự — phân tích lá số | ✅ Xong | `bazi_analysis.rs`: Tàng Can, Thập Thần, Trường Sinh, Thần Sát Mệnh Lý, Đại Vận |
| Trạch Nhật, hạn tuổi | ✅ Xong | `almanac.rs` |
| Đặc tả Tử Vi Đẩu Số | ✅ Xong | `docs/TUVI_SPEC.md`; T1–T5, T8, T9 đã chốt |
| Tử Vi Đẩu Số — lập lá số | ✅ Xong | `tuvi.rs`: 105 sao, Tứ Hóa, Miếu Hãm, Tuần, Triệt, Đại Hạn, Tiểu Hạn, Thiên Bàn; có trên Swift và WASM |
| Đặc tả Phong Thủy | ✅ Xong | `docs/FENGSHUI_SPEC.md`; F1–F5 đã chốt |
| Phong Thủy — Bát Trạch, Cửu Tinh | ✅ Xong | `fengshui.rs`: Mệnh Quái, Trạch Quái, Bát Biến, Vận, Niên/Nguyệt/Nhật/Thời Tinh, bàn Cửu Cung; có trên Swift và WASM |
| Đặc tả CSDL luận giải offline | ✅ Xong | `docs/INTERPRETATION_DB_SPEC.md`; D1–D12 đã chốt |
| Crate `astro-data` | ✅ Xong | `validate`, `build`, `stats`, `scan`, `review`; V1–V13; 80 test |
| Dữ liệu luận giải | 🟡 Đang làm | TV-01 đến TV-06, TV-15 đã duyệt (487/488 đoạn); TV-07 đến TV-11 `draft`, chờ duyệt; còn 8 batch Tử Vi và 11 batch khác |
| Module `content` (khóa và facts) | ✅ Xong | `content.rs`; có trên Swift và WASM |
| Độ sáng phụ tinh (D8) | ✅ Xong | 17 sao và Hóa Kỵ, theo tr. 39–43 |
| App iOS SwiftUI (Phase 5 – UI) | ⏳ Chưa làm | Đã có Xcode |
| Web app | ⏳ Chưa làm | |
| Commit lên git | ✅ Xong | `332c75e`, `cb32eda` trên `origin/develop` (GitHub, qua SSH) |

## 2. Việc tiếp theo

- [x] Duyệt TV-01, TV-15; chốt D6a, D6b, D10, D11, D12
- [x] Task 5A.3: module `content`, V10, V13, bảng độ sáng phụ tinh
- [x] Chốt D13 theo sách; `rights` của *Tân Biên* là `paraphrased`
- [x] Skill `/ingest-content`, nạp TV-02 đến TV-06
- [x] Chốt D14–D16; thêm 6 fact mới (`birth_time_of_day`, `lunar_day_range`, `opposite_only`, `in_than`, `any_loc`, `element_relation`/`_all`)
- [x] Duyệt TV-02 đến TV-06 ở `build/content/review/` (kiểm định độc lập, 487/488 đoạn)
- [x] Nạp TV-07 đến TV-11
- [ ] Duyệt TV-07 đến TV-11 ở `build/content/review/`
- [ ] Nạp TV-12 đến TV-14, TV-16 đến TV-19
- [ ] Kéo `build/ios/AstroCore.xcframework` và `build/ios/AstroCore.swift` vào project Xcode, gọi thử `checkConnection`
- [ ] Đối chiếu Trực, Sao, Thần của các vector kiểm thử với lịch in (Q12)
- [ ] Đối chiếu Can Chi tháng nhuận (mục 4) với sách nguồn
- [x] Commit và push phần nền móng lên `origin/develop`
- [x] Chốt F1–F5 ở `docs/FENGSHUI_SPEC.md`
- [x] Viết module Phong Thủy trong `astro-core`, đưa lên Swift và WASM
- [ ] Đối chiếu 3 vector Phong Thủy với lịch in
- [x] Có bảng quy tắc Tử Vi: lấy từ *Tân Biên* vào `docs/TUVI_SPEC.md`
- [x] Chốt T1–T5 ở `docs/TUVI_SPEC.md`
- [x] Viết `tuvi.rs`, đưa lên Swift và WASM
- [x] Chốt T8 (Mệnh Chủ, Thân Chủ) và T9 (bảng Cát, Hung)
- [ ] Đối chiếu V3 (Dương Nam, tháng nhuận) với một lá số lập bằng nguồn khác
- [ ] Bắt đầu app iOS (SwiftUI) dùng XCFramework
- [x] Commit Giai đoạn 3 cùng `docs/`, `.gitignore`, `THIRD_PARTY_NOTICES.md`
- [x] Push các commit của Giai đoạn 3 lên `origin/develop`

## 3. Nhật ký

### 2026-11-08 — Task 5A.5: D14–D16, kiểm định TV-02 đến TV-06, nạp TV-07 đến TV-11

- **D14**: Phục Binh không bao giờ hội hợp Song Hao (lỗi vị trí sao của sách, như D10); sửa ở TV-03 `menh/than-cu/07`.
- **D15**: "Lộc" khi sách không ghi rõ Lộc Tồn hay Hóa Lộc dùng fact mới `any_loc`. "Song Lộc" (cần cả hai) không đổi.
- **D16**: thêm 6 trường `when` tính từ `TuViChart`: `birth_time_of_day`, `lunar_day_range`, `opposite_only`, `in_than`, `any_loc`, `element_relation`/`element_relation_all`, cùng fact tương ứng (`birth_time:`, `lunar_day:`, `oppose:`, `than_sit:`, `any_loc`, `element:<pair>:<relation>`). Thêm `TuViStar::element()` (Ngũ hành 14 Chính Tinh, lấy từ `profile.element` đã duyệt của TV-01), `ElementRelation`, `BirthTimeOfDay` vào `astro-core`. Không cần sửa `astro-ffi`, `astro-wasm`: không hàm nào trong hai cầu nối dùng tới các kiểu mới, `wasm-pack build` và test FFI vẫn qua không đổi.
- Áp 6 fact mới vào TV-03, TV-04, TV-06: 18 đoạn chuyển "Lộc" sang `any_loc`; các đoạn "cung Thân có X tọa thủ" sang `in_than`; "đồng cung hoặc xung chiếu" của Bạch Hổ, Tướng Quân thử chuyển sang `opposite_only` nhưng V10 (vector V4) bắt được chỗ sai — sao đó chỉ tới bằng tam hợp ở lá số mẫu, không phải xung chiếu — nên trả lại `meets_all` cho hai đoạn đó; "sinh ban ngày/đêm" của Thái Dương, Thái Âm sang `birth_time_of_day`; ngũ hành sinh khắc Bản Mệnh/Cung/Chính Tinh ở 4 đoạn `menh/sinh-khac/*`, `menh/tuyet-dia/03` sang `element_relation`.
- **Kiểm định độc lập TV-02 đến TV-06** (5 Agent độc lập, mỗi batch một Agent, đối chiếu từng mục với nguyên văn, theo cách đã làm với TV-15): TV-02 (97 mục, đọc gần hết, không lỗi), TV-03 (51 đoạn, đọc hết, không lỗi), TV-04 (110 đoạn, đọc hết, không lỗi), TV-05 (105 đoạn, đọc hết, 2 câu phú bị lược phát hiện và khôi phục), TV-06 (107 đoạn, đọc hết, 1 câu phú bị lược phát hiện và khôi phục). 487/488 đoạn chuyển `approved`; 1 đoạn (`menh/thai-duong/61`, TV-05) giữ `draft` vì "Binh" đọc thành Phục Binh không có căn cứ chữ rõ.
- Cập nhật fixture V10 (`fixtures/tuvi_palace.json`) theo dữ liệu mới: vector V4 thêm 2 đoạn `menh/sinh-khac/02`, `/03` (quan hệ sinh khắc mới đúng cho lá số mẫu).
- Commit riêng từng phần: hạ tầng fact mới, rồi TV-02, TV-03 (kèm fixture), TV-04, TV-05, TV-06.
- **Nạp song song TV-07 đến TV-11** bằng `/ingest-content` (5 Agent, mỗi batch một Agent): TV-07 Tham Lang, Cự Môn (119 đoạn); TV-08 Thiên Lương, Thất Sát, Phá Quân (103 đoạn); TV-09 Xương Khúc, Khôi Việt, Lộc Tồn, Tả Hữu (101 đoạn); TV-10 Kình Dương, Đà La, Hỏa Linh, Không Kiếp (92 đoạn); TV-11 Tứ Hóa, Lục Bại (92 đoạn). Tổng 507 đoạn mới, 0 gap, tất cả còn `draft`. Nhiều đoạn dùng cùng cách sửa của D14 cho các bộ sao không bao giờ hội hợp được (Cự Môn/Vũ Khúc, Đà La/Kình Dương, Cự Môn/Thiên Phủ…), và một số dòng sách bất khả thi về vị trí sao không nạp, theo D10.
- Cập nhật tiếp fixture V10: vector V4, V5 thêm các đoạn mới hợp lệ từ TV-09, TV-10, TV-11 (Lộc Tồn, Tả Hữu, Khôi Việt, Hỏa Linh, Hóa Lộc, Hóa Khoa…) nay đúng xuất hiện trên hai lá số mẫu.
- **Kiểm chứng:** `cargo test --workspace` (401 test core + astro-data + FFI + WASM, gồm V10 5 vector) qua hết; `validate` 141 mục, 995 đoạn, 18 gap, 0 lỗi; `scan` 0 đoạn không bao giờ hiện trên 510.432 lá số, cung Mệnh hiện trung bình 34,92 đoạn, 15,29 đoạn chắc chắn (10 ít nhất, 100 nhiều nhất, tính cả TV-07 đến TV-11 còn `draft`).
- Chưa kiểm định độc lập TV-07 đến TV-11; mục còn `draft`, chờ batch kiểm định tiếp theo (xem mục 1).

### 2026-11-08 — Task 5A.4: `/ingest-content` và TV-02 đến TV-06

- Chốt D13 theo tr. 39; `rights` của *Tân Biên* thành `paraphrased` (V11 chặn `quote`). Commit Task 5A.3 (`1db0145`, `bce6a0c`, `be5aa6d`) và hạ tầng nạp (`8d05853`).
- Skill `/ingest-content` và lệnh `astro-data expand`: viết dạng gọn một dòng một mục rồi bung ra JSON. Thêm fact `ban_menh`, `polarity`, `harmony` cho chương Mệnh; `profile.dau` thành tùy chọn.
- Nạp song song TV-02 (97 mục, 16 gap), TV-03 (50 đoạn), TV-04 (110), TV-05 (105), TV-06 (107); 70 `correction`, 154 `condition_text`. Dạng gọn bằng 54% JSON.
- Tích hợp: thêm fact `is_menh` vì neo Mệnh áp cho cung Thân làm đoạn về chính cung Mệnh hiện sai; câu phú phải nguyên văn, khôi phục 45 câu bị lược; thêm `NamMenh`; sửa một chỗ đọc ngược nghĩa ở Thái Dương (tr. 68 "cô" là "vô", theo tr. 78). Không có cặp sao lặp giữa các batch.
- Kiểm chứng: 280 test core, 85 test `astro-data`, 26 FFI đều qua; `clippy`, `rustfmt` sạch; validate 0 lỗi; 488/488 đoạn xuất hiện trên 510.432 lá số. Cung Mệnh hiện trung bình 18,1 đoạn, 5,8 đoạn chắc chắn. Chưa kiểm định độc lập.

### 2026-11-07 — Task 5A.3: D8, module `content`, V13

- Chốt D6a, D6b, D10, D11, D12; duyệt TV-01, TV-15; commit pilot (`44c7e27`, `a0de711`, `2913d13`).
- D8: `TuViStar::brightness_at` có thêm 17 phụ tinh, `FourTransformations::brightness_at` cho Hóa Kỵ, theo tr. 39–43. Bảng trong Task 5A.3 khác sách ở Hỏa Linh, Xương Khúc; dùng theo sách, ghi D13. Đại Hao, Tiểu Hao, Thiên Mã, Thiên Hình không có `Ham` vì sách không gọi hãm.
- `NineStar::phase_in`, `NineStarPhase` theo D6.
- `content.rs`: khóa cho ngày, hạn tuổi, Bát Tự, Bát Trạch, Cửu Tinh; `tuvi_palace_query`, `Gate`, `gate_matches`. Swift, WASM có thêm 14 hàm mỗi bên.
- Đọc lại sách: quy tắc mượn sao xung chiếu chỉ có ở 8 cung, không phải 12. Cung vô Chính diệu luôn đối diện đúng 2 Chính Tinh (24/24 trạng thái).
- `astro-data`: V13 quét lá số bằng `content`, dừng khi thấy đủ; V10 fixture 5 vector, đối chiếu tay V2, V3, V4, V5 với lời sách; `tier` cho D12; test mọi khóa của `content` đều có trong registry.
- TV-15: 14 đoạn chuyển `condition_text` sang `brightness`. Một cung Quan Lộc còn trung bình 6,3 đoạn, 2,6 đoạn chắc chắn.
- Kiểm chứng: 279 test core, 80 test `astro-data`, 26 test FFI, 30 test WASM đều qua; `clippy -D warnings`, `rustfmt` sạch; `wasm-pack build`, `./build-ios.sh` thành công; chương trình Swift thật chạy qua bindings, kể cả nhánh lỗi. 116/116 đoạn xuất hiện trên 510.432 lá số.

### 2026-11-07 — Task 5A.2: `astro-data` và hai batch thử

- Chốt D1–D9, commit đặc tả (`febd3ea`). D4 cho phép `canonical_summary` ngoài Tử Vi; D5 đổi `remedies` thành `advice`; D6 bỏ `nine_star_phase`, thêm 2 trường cho `fengshui.nine_star`.
- `astro-core`: thêm `ALL` cho 7 enum; tách `chinh_tinh_branches` khỏi `place_stars` để validator không chép lại công thức.
- Crate `astro-data`: 24 collection, 460 khóa, kiểm V1–V9, V11, V12; xuất SQLite và JSON Web, đọc ngược để so. `tools/extract_pdf_pages.swift` dựng `.ref/sources/tan_bien/pages.txt`.
- Đọc nguyên văn làm schema đổi: gộp cặp sao, neo toàn cung, `correction`, `not_with`, `with_transformations`, `transformations_all`, `menh_all`, `Neither`.
- TV-01: 42 mục, 2 `gap`. TV-15: 116 đoạn, 1 mục cung, đủ 14/14 Chính Tinh. Tất cả còn `draft`. Build: SQLite 94 KB, JSON Web 72 KB, gzip 10 KB; toàn bộ Tử Vi ước 1,1 MB.
- Validator bắt 2 lỗi trong dữ liệu vừa sinh. Đọc đối chiếu phát hiện 4 lỗi in của sách: 8.5 "Tý, Ngọ" là Tỵ, Ngọ; 8.6 "tại Thìn" là Thân; 8.8 "Nguyệt đồng cung" là Nhật; 8.9 "Thần" là Thân.
- Kiểm định độc lập 160 mục: 0 bỏ sót, 0 đổi nghĩa nặng, 4 lỗi nặng về điều kiện, 28 lỗi nhẹ, 8/8 đính chính được đồng ý. Đã sửa hết nhưng chưa kiểm định lại, riêng 8.43 chưa nạp (D10). Lỗi tập trung ở phụ tinh, bộ sao: 4/38, so với 0/79 ở Chính Tinh.
- Quét 510.432 lá số: 116/116 đoạn đều xuất hiện. Một cung Quan Lộc hiện 8,6 đoạn, chỉ 2,3 đoạn không kèm "Nếu". Cung có Sát tinh hiện thêm 12 dòng "Nếu…" (D12).
- JSON đầu ra gấp đôi ước tính token, nên viết bằng dạng gọn rồi bung ra.
- Kiểm chứng: 254 test core, 71 test `astro-data`, 21 test FFI, 28 test WASM đều qua; `clippy -D warnings`, `rustfmt` sạch; `astro-ffi` biên dịch cho cả hai target iOS. Không có API mới cho Swift, WASM nên chưa build lại XCFramework. Bộ quét lá số mới là mã dùng một lần, chưa vào repo.

### 2026-11-07 — Task 5A.1: đặc tả CSDL luận giải

- Tạo `docs/INTERPRETATION_DB_SPEC.md`. Chưa sinh dữ liệu, chưa viết code, chờ duyệt D1–D9.
- Đề xuất Phương án A: JSON trong `data/interpretations/` là nguồn duy nhất, crate build-time `astro-data` ra SQLite (iOS) và JSON (Web). Rust chỉ giữ khóa và logic chọn mục. Web chưa cần IndexedDB.
- Đề bài khác code ở 5 chỗ: `VietnamTaboo` có 9 biến thể, `NaYin` 30 chứ không phải 60, `BaziStar` 10 cộng Không Vong và Khôi Canh, `StarBrightness` có thêm `Binh`, chưa có trạng thái đương vận, thất vận. 7 `enum` thiếu `ALL`; 34 tên biến thể trùng giữa các `enum`.
- Đo *Tân Biên*: 11 cung ngoài Mệnh có 1.241 đoạn nhưng chỉ 11 đoạn (0,9%) nêu Miếu, Vượng, Đắc, Hãm, Bình; cung Mệnh là 71%. Sách viết theo Chi và sao đồng cung, nên đề xuất đoạn luận có điều kiện thay khối 14 × 12 × độ sáng (672 ô, phần lớn không có nguồn). Nô Bộc không chia theo Chính Tinh.
- PDF của *Tân Biên* có lớp văn bản số dấu đúng, tốt hơn hẳn `_djvu.txt`; đọc bằng PDFKit. Tr. 33 định nghĩa đơn thủ, hội hợp, sáng sủa, mờ ám; riêng "nhiều" không có ngưỡng. Tr. 20 có thêm một lỗi đánh máy về xung chiếu của cung Sửu.
- Tử Vi đủ nguồn, chia 19 batch. Bát Tự, Phong Thủy, 12 Trực, 12 Thần, 15 Thần Sát chưa có văn bản nguồn; cách hóa giải hạn không có nguồn nào.

### 2026-11-06 — Chốt T8, T9, đóng Giai đoạn 4

- T8: chủ dự án duyệt bảng Mệnh Chủ, Thân Chủ chuẩn *Tử Vi Đẩu Số Toàn Thư*. Mệnh Chủ khớp cả 12 ô; Thân Chủ của Ngọ đổi từ
  Linh Tinh sang Hỏa Tinh, 11 ô còn lại khớp.
- T9: duyệt bảng Cát, Hung; Thất Sát đổi từ `Cat` sang `Hung`, Phá Quân vốn là `Hung`.

### 2026-11-06 — Task 4B.2: `tuvi.rs` và cầu nối

- Đóng T1–T5: 12 cung đi thuận; Hóa Khoa Can Nhâm là Tả Phụ; Lưu Hà Can Tân ở Mão; dùng `RatHourConvention`; thêm 10 sao.
- Thêm `tuvi.rs`: `TuViChart`, `TuViPalace`, `TuViPlacedStar`, `TuViStar` (105 sao), `FourTransformations`,
  `FiveElementsBureau`, `StarBrightness`, `TuViConfig`, `tuvi_chart`, `tuvi_chart_from_lunar`. Swift có thêm 23 hàm, WASM 22.
- Task 4B.2 cần ba nội dung đặc tả chưa có. Thuận lý và Cục sinh khắc có trong sách (tr. 31). **Mệnh Chủ, Thân Chủ** sách
  không có (T8). **Cát, Hung** sách không có bảng, dự án lập theo nhãn và mô tả (T9).
- Tứ Hóa là `FourTransformations` gắn vào sao gốc, không phải `TuViStar`, nên enum có 105 sao, cộng 4 Hóa là 109 vị trí.
- `use_true_solar_time` cần kinh độ: thêm `longitude`, thiếu thì lấy kinh tuyến chuẩn của múi giờ.
- Phụ tinh luôn có `brightness` là `None`: sách có ghi Đắc địa cho vài phụ tinh, nhưng đặc tả chỉ phủ 14 Chính Tinh.
- Kiểm chứng: 248 test core, 21 test FFI, 28 test WASM đều qua; `clippy -D warnings`, `rustfmt` sạch. Test so từng sao với
  4 lá số in trong sách, chỉ trừ đúng các lỗi in đã ghi. `tuvi.rs` khớp mô hình Python độc lập trên 47.808 lá số.
  67 đột biến viết tay: vòng đầu sót 8, sau khi thêm test toàn bảng thì bắt hết. Chương trình Swift thật chạy qua bindings
  sinh ra. Static lib iOS build được; chưa đóng gói `AstroCore.xcframework` vì chưa có Xcode.

### 2026-11-06 — Task 4B.1: đặc tả Tử Vi Đẩu Số

- Tạo `docs/TUVI_SPEC.md`: 12 cung, Cục, 14 Chính Tinh, 3 vòng sao, phụ tinh, Tứ Hóa, Tuần, Triệt, Đại Hạn, Tiểu Hạn,
  bảng Miếu Hãm, 5 vector (3 theo yêu cầu, thêm 2 từ sách). Chưa viết code Rust.
- Nguồn chính là *Tân Biên* bản Internet Archive, đọc từ ảnh trang. Đây là bản đánh máy lại, có lỗi: mục 0.4 của đặc tả
  liệt kê 11 chỗ. Bốn lá số ở Phụ lục dùng làm đối chứng.
- Khác Task 4B.1: 12 cung đi thuận, không đi nghịch (T1); tr. 14 ghi Hóa Khoa Can Nhâm là Tả Phụ, không phải Thiên Phủ (T2).
- Bảng in trái với lá số của chính sách: Hóa Quyền Can Tân, Hỏa Tinh nhóm Hợi-Mão-Mùi (đã sửa theo lá số);
  Lưu Hà Can Tân (T3, giữ theo bảng).
- `lasotuvi` theo sách cụ Thiên Lương, khác *Tân Biên* ở 5 quy tắc và thiếu Miếu Hãm của Thiên Phủ: không dùng làm chuẩn.
  Chỉ đọc bảng dữ liệu của nó, không chạy code.
- Kiểm chứng: 4 lá số khớp 434/436 vị trí sao; bảng Tử Vi khớp 149/150 ô, bảng Lập Cục 120/120, Tuần 60/60, 20 ví dụ trong
  sách đều khớp; đọc ngược đặc tả, 791 phép kiểm đều qua.
- Chưa kiểm được: sách không có lá số Dương Nam, Âm Nữ, nên chiều thuận chỉ dựa vào lời văn; Tứ Hóa chỉ có đối chứng
  cho Can Giáp, Tân, Quý.

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
5. **Tử Vi T1–T5, T8, T9: đã chốt**, xem mục 8.2 của `docs/TUVI_SPEC.md`. Mệnh Chủ, Thân Chủ và bảng Cát, Hung không đến từ
   *Tân Biên* nhưng đã được chủ dự án duyệt.
6. **CSDL luận giải D1–D16: đã chốt**, xem mục 12 của `docs/INTERPRETATION_DB_SPEC.md`.

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
- **Tử Vi:** năm đổi tại Tết, tuổi là tuổi mụ, giờ mặt trời thực chỉ dùng khi bật `use_true_solar_time`. Vòng Trường Sinh là
  12 biến thể của `TuViStar`, `life_stage` của cung lặp lại sao đó. `TuViPalace` là bản ghi một cung; `Palace` đã thuộc Phong Thủy.
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
| `crates/astro-core/src/tuvi.rs` | Lá số Tử Vi: 12 cung, Cục, 105 sao, Tứ Hóa, Miếu Hãm, Tuần, Triệt, Đại Hạn, Tiểu Hạn, Thiên Bàn |
| `crates/astro-core/src/content.rs` | Khóa luận giải, truy vấn cung, cổng lọc đoạn |
| `crates/astro-core/src/day_info.rs` | Dữ liệu cho một ô lịch |
| `crates/astro-ffi/` | Cầu nối UniFFI (Swift module `AstroCore`) |
| `crates/astro-ffi/src/bin/uniffi-bindgen.rs` | Trình sinh bindings, cùng phiên bản `uniffi` với thư viện |
| `crates/astro-wasm/` | Cầu nối WebAssembly và khai báo TypeScript |
| `crates/astro-data/` | Công cụ build-time: kiểm và biên dịch dữ liệu luận giải |
| `data/interpretations/` | Nguồn chuẩn của dữ liệu luận giải |
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
| Lá số Tử Vi | `tuvi_chart(birth, gender, &config)`, `tuvi_chart_from_lunar(lunar, hour, gender, &config)` | `tuviChart(birth:gender:config:)`, `tuviChartFromLunar(lunar:hour:gender:config:)` | `tuviChart(birth, gender, options?)`, `tuviChartFromLunar(lunar, hour, gender, options?)` |
| Hạn theo tuổi | `TuViChart::dai_han_palace(age)`, `tieu_han_palace(age)` | `tuviDaiHanPalace(chart:age:)`, `tuviTieuHanPalace(chart:age:)` | `tuviDaiHanPalace(chart, age)`, `tuviTieuHanPalace(chart, age)` |
| Khóa luận giải | `almanac_day_keys`, `age_restriction_keys`, `bazi_keys`, `kua_key`, `kua_compatibility_keys`, `eight_star_key`, `eight_star_use_key`, `nine_star_key` | Cùng tên camelCase | Cùng tên camelCase |
| Chọn đoạn Tử Vi | `tuvi_palace_query(&chart, branch)`, `gate_matches(&gates, &facts)` | `tuviPalaceQuery(chart:branch:)`, `gateMatches(gates:facts:)` | `tuviPalaceQuery(chart, branch)`, `gateMatches(gates, facts)` |
| Trạng thái Cửu Tinh | `NineStar::phase_in(period)`, `NineStarPhase::name_vi`, `is_auspicious` | `nineStarPhase(star:period:)`, `nineStarPhaseName`, `nineStarPhaseIsAuspicious` | Cùng tên |
| Thuộc tính sao | `TuViStar::name_vi`, `category`, `nature`, `brightness_at`; `FourTransformations::star_for` | `tuviStarName`, `tuviStarCategory`, `tuviStarNature`, `tuviStarBrightness`, `fourTransformationStar` | Cùng tên với Swift |
| Thời khắc giao tiết | `get_exact_solar_term_time(year, term, tz_offset)` | `getExactSolarTermTime(year:term:tzOffset:)` | `getExactSolarTermTime(year, term, tz_offset?)` |
| Giờ mặt trời thực | `to_true_solar_time(clock, longitude, standard_meridian)` | `trueSolarTime(clock:longitude:tzOffset:)` | `trueSolarTime(clock, longitude, tz_offset?)` |
| Tên hiển thị | `CanChi::name_vi()`, `name_han()`, `SolarTerm::name_vi()`… | `canChiName`, `canChiHanName`, `stemName`, `branchName`, `solarTermName`, `trigramName`, `nineStarName`… | `canChiName`, `canChiHanName`, `solarTermName`, `trigramName`, `nineStarName`… |

`tz_offset` mặc định là 7.0 ở Swift và JS; `boundary` mặc định `LapXuanDay`, `forward` mặc định `true`. Rust còn có các hàm cấp thấp như
`sun::apparent_longitude`, `sun::equation_of_time`, `moon_phase::new_moon_jd_ut`,
`solar_term::solar_term_jd_ut`, `canchi::{year,month,day,hour}_can_chi`.

## 8. Kiểm chứng

```bash
cargo test --workspace                                  # 288 test core + 70 astro-data + 17 repository_data + 26 FFI
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
wasm-pack build crates/astro-wasm --target web
wasm-pack test --node crates/astro-wasm                 # 30 test
./build-ios.sh                                          # cần Xcode
cargo run -p astro-data -- validate                     # kiểm dữ liệu luận giải
cargo run -p astro-data -- build                        # build/content/
cargo run --release -p astro-data -- scan               # V13 và tải từng cung
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
- Tử Vi: bảng Lập Cục, Tử Vi, Tuần, Triệt, Tứ Hóa, Miếu Hãm của *Tân Biên*; 20 ví dụ trong sách; 4 lá số in ở Phụ lục; 5 vector của đặc tả

## 9. Môi trường

| Thành phần | Phiên bản / trạng thái |
|---|---|
| macOS | 27.0.1, Apple Silicon (arm64) |
| Rust | 1.99.1 stable (rustup; PATH nạp từ `~/.zshenv`, `~/.profile`); target theo `rust-toolchain.toml`: `wasm32-unknown-unknown`, `aarch64-apple-ios`, `aarch64-apple-ios-sim` |
| wasm-pack | 0.15.1 (Homebrew) |
| Node.js | 22.23.2 |
| Swift | 6.4 (Command Line Tools) |
| Xcode | 27.2 |
| Crate chính | uniffi 0.32.2 · wasm-bindgen 0.2.130 · serde-wasm-bindgen 0.6.5 · serde 1.0.229 · rusqlite 0.41.1 · sha2 0.11.1 |
