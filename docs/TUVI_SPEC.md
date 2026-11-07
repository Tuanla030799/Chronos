# Đặc tả Tử Vi Đẩu Số (TUVI_SPEC)

Nguồn của `crates/astro-core/src/tuvi.rs` (Giai đoạn 4, Phần 4B), theo Nam Phái Việt Nam và *Tử Vi Đẩu Số Tân Biên* của Vân Đằng Thái Thứ Lang. Mọi bảng và công thức ở đây là nguồn chuẩn duy nhất; code chỉ chuyển chúng thành `const` hoặc `match`, không thêm quy tắc mới.

**Trạng thái:** T1–T5 chốt ở Task 4B.2, T8 và T9 chốt ở lần duyệt cuối của Giai đoạn 4. Không còn điểm nào mở, xem mục 8.2.

## 0. Nguồn và quy ước

### 0.1. Nguồn dữ liệu

| Mục | Nguồn | Trạng thái |
| :-- | :-- | :-- |
| 1.1–1.3 | Task 4B.1; *Tân Biên* tr. 5–7 | Khớp sách; chiều 12 cung đã chốt theo sách (T1) |
| 1.4 | Task 4B.1; sách không nêu riêng | Khớp ngầm: tạo lại đúng bảng Lập Cục tr. 7 |
| 1.5 | Task 4B.1 | Sách không nói về tháng nhuận |
| 2 | Task 4B.1; tr. 7–9 | Khớp sách |
| 3 | Task 4B.1; tr. 9–10 | Khớp sách |
| 4.1–4.8 | tr. 10–16 | Lấy từ sách; T2, T3 đã chốt theo bảng in |
| 4.9 | tr. 33–51, phân loại do dự án lập | Sách không có bảng Cát, Hung; chủ dự án đã duyệt (T9) |
| 5.1–5.4 | Task 4B.1; tr. 16–17, 20–23 | Khớp sách |
| 5.5 | Task 4B.2; tr. 31, tr. 52; Mệnh Chủ, Thân Chủ theo bảng chủ dự án duyệt, chuẩn *Tử Vi Đẩu Số Toàn Thư* | Đã duyệt (T8) |
| 6.1 | tr. 33–38, chương Đặc tính các sao | Lấy từ sách |
| 6.2 | Phụ lục tr. 272–275; V3 tính từ đặc tả này | Xem mục 7 |

**Bản sách.** Dùng bản trên Internet Archive, mã `TuViDdauSoTanBien-VDThaiThuLang-DV`. Bìa là Tín Đức Thư Xã, Sài Gòn, giấy phép 722/TXB ngày 13-5-57, còn ruột là bản đánh máy lại, ghi "Tạo ebook: nguyenbahuy, 3/11/2014". Số trang trong tài liệu này là số in ở chân trang của bản đó. Đây không phải bản in gốc và có lỗi đánh máy, xem mục 0.4. "Sách 1" đến "Sách 4" là bốn lá số ở Phụ lục.

**`lasotuvi`.** Thư viện Python của doanguyen (MIT, commit `ace8379`) chỉ để tham khảo, không lấy dữ liệu. Chú thích trong code cho biết nó an theo sách của cụ Thiên Lương, không theo *Tân Biên*. Không dùng nó làm chuẩn so kết quả, xem mục 7.3.

### 0.2. Chỉ số, ký hiệu và kiểu Rust

- Địa Chi từ 0 (Tý) đến 11 (Hợi), Thiên Can từ 0 (Giáp) đến 9 (Quý), trùng `Branch::index` và `Stem::index`.
- `mod` là phép lấy dư không âm, ứng với `rem_euclid` trong Rust.
- **Thuận** là tăng chỉ số Chi, cũng là chiều kim đồng hồ trên bản đồ 12 cung (tr. 5). **Nghịch** là giảm chỉ số Chi.
- `s` là Can năm, `y` là Chi năm, `m` là tháng hiệu lực (mục 1.5), `d` là ngày âm, `h` là Chi giờ, `c` là số Cục.
- `dir = +1` với Dương Nam, Âm Nữ; `dir = −1` với Âm Nam, Dương Nữ. Âm Dương lấy theo Can năm, `Stem::yin_yang`; lấy theo Chi năm cũng cùng kết quả (tr. 6).
- Mọi sao đặt theo vị trí cung, tức một `Branch`.

Dùng lại từ `astro-core`: `Stem`, `Branch`, `CanChi`, `NaYin`, `FiveElements`, `YinYang`, `Gender`, `LifeStage`, `KhongVong`, `khong_vong`, `LunarDate`, `RatHourConvention`, `solar_to_lunar`, `year_can_chi`, `to_true_solar_time`.

Kiểu mới:

| Kiểu | Nội dung |
| :-- | :-- |
| `TuViPalaceRole` | 12 cung chức năng, mục 1.3 |
| `TuViStar` | 105 sao: cột **Rust** ở các mục 2–4, không gồm Tứ Hóa |
| `FourTransformations` | `HoaLoc`, `HoaQuyen`, `HoaKhoa`, `HoaKy`: gắn vào sao gốc, không phải một sao riêng |
| `FiveElementsBureau` | `ThuyNhi` = 2, `MocTam` = 3, `KimTu` = 4, `ThoNgu` = 5, `HoaLuc` = 6 |
| `StarBrightness` | `Mieu`, `Vuong`, `Dac`, `Binh`, `Ham`; dùng qua `Option`, `None` là sao không xét độ sáng |
| `StarCategory`, `StarNature` | `ChinhTinh`, `PhuTinh`; `Cat`, `Trung`, `Hung`, mục 4.9 |
| `LeapMonthRule` | `SameAsCurrentMonth`, `SplitAt15th` (mặc định) |
| `TuViConfig` | `tz_offset`, `leap_month_rule`, `rat_hour`, `use_true_solar_time`, `longitude` |
| `TuViPolarity` | `DuongNam`, `AmNam`, `DuongNu`, `AmNu` |
| `YinYangHarmony`, `MenhCucRelation` | Mục 5.5 |
| `TuViPlacedStar` | Một sao trong cung: `star`, `brightness`, `transformation` |
| `TuViPalace` | Một cung trên địa bàn |
| `TuViChart` | Thiên Bàn và 12 cung, xếp từ Tý đến Hợi |

`TuViPalace` là bản ghi của một cung; tên `Palace` đã là Cửu Cung của `fengshui` và đã export ở gốc crate. UniFFI không hỗ trợ mảng cố định nên `palaces` là `Vec`.

Mỗi `TuViStar` nằm trong đúng một cung, ở `main_stars` nếu là Chính Tinh, ở `minor_stars` nếu không. Vòng Trường Sinh là 12 biến thể của `TuViStar` như các phụ tinh khác; `life_stage` của cung lặp lại sao đó dưới dạng `LifeStage` để dùng chung với Bát Tự.

Quan Phù (vòng Thái Tuế) và Quan Phủ (vòng Bác Sỹ) là hai sao khác nhau nhưng trùng chữ khi bỏ dấu, nên hai biến thể là `QuanPhuThaiTue` và `QuanPhuBacSy`. "Phúc Đức" vừa là tên sao vừa là tên cung, nhưng thuộc hai `enum` khác nhau.

### 0.3. Khóa tra cứu

Tử Vi dùng lịch âm, không dùng tiết khí như Bát Tự.

| Khóa | Định nghĩa | Dùng cho |
| :-- | :-- | :-- |
| **Can Chi năm** | `year_can_chi(LunarDate.year)`: đổi tại Tết, không đổi tại Lập Xuân | `s`, `y`, Âm Dương |
| **Tháng hiệu lực `m`** | Mục 1.5 | Mệnh, Thân, sao theo tháng |
| **Ngày `d`** | `LunarDate.day`, 1–30, không đổi bởi quy tắc tháng nhuận | Tử Vi, sao theo ngày |
| **Chi giờ `h`** | Bảng tr. 6, trùng `Branch::from_hour`: 23:00–00:59 là Tý, 01:00–02:59 là Sửu, …, 21:00–22:59 là Hợi | Mệnh, Thân, sao theo giờ |
| **Tuổi** | Tuổi mụ, `năm xem − năm sinh + 1`, như mục 4.1 của `vietnam_custom_rules.md` | Đại Hạn, Tiểu Hạn |

Sách không gọi tên tuổi mụ, nhưng bốn lá số đều tính như vậy: 1913 có 44 tuổi vào năm 1956, 1891 có 66 tuổi vào năm 1956, 1904 có 47 tuổi vào năm 1950, 1931 có 23 tuổi vào năm 1953.

Múi giờ theo `tz_offset`, mặc định UTC+7. Với `use_true_solar_time = true`, giờ sinh đổi sang giờ mặt trời thực tại `longitude` trước, rồi mới lấy ngày âm và Chi giờ, như `effective_time` của `four_pillars`. Thiếu `longitude` thì lấy kinh tuyến chuẩn `tz_offset × 15`, tức chỉ hiệu chỉnh phương trình thời gian. Với `false`, dùng giờ đồng hồ và bỏ qua `longitude`.

**Giờ Tý Sớm (23:00–23:59).** Theo `rat_hour`, mặc định `SplitAtMidnight` (T4). `SplitAtMidnight` lấy ngày âm của ngày đang xét. `NewDayAtRatHour` lấy trọn `LunarDate` của hôm sau, nên tháng và năm cũng đổi nếu hôm sau sang tháng hay năm mới. Cả hai đều có `h` là Tý.

### 0.4. Lỗi đánh máy của bản sách và cách xử lý

| Trang | Bản sách ghi | Đặc tả dùng | Căn cứ |
| :-- | :-- | :-- | :-- |
| 6 | Danh sách cung sau Mệnh bắt đầu từ Phúc Đức, thiếu Phụ Mẫu | Có Phụ Mẫu ngay sau Mệnh | Ví dụ Đại Hạn tr. 20; cả 4 lá số |
| 8 | Kim Tứ Cục, ô Mùi: 14, 21, 27. Số 21 lặp lại ở ô Thìn, không có số 24 | 14, 24, 27 | Công thức mục 2.2 |
| 10 | Vòng Bác Sỹ kết thúc bằng Quan Phù | Quan Phủ | Sách 2, 3, 4 viết Quan Phủ ở vị trí này |
| 10 | Vòng Tràng Sinh thiếu Suy, chỉ có 11 sao | Có Suy, sau Đế Vượng | Mục 3.79 của sách; cả 4 lá số |
| 11 | Hợi, Mão, Mùi: Hỏa Tinh khởi ở Dần | Dậu | Suy ngược từ Sách 2 và Sách 4 đều ra Dậu; xem T6 |
| 12 | Hồng Loan "kể là năm sinh" | Kể là năm Tý | Cùng cách viết với các sao theo Chi năm |
| 14 | Can Tân: Hóa Quyền là Thiên Lương | Thái Dương | Sách 2 và Sách 4 đều đặt Hóa Quyền cùng cung Thái Dương; xem T6 |
| 16 | Ví dụ "năm Đinh Tý" | Đinh Tỵ | Đinh không ghép với Tý |
| 17 | Ví dụ: Canh Ngọ an Triệt ở Thân–Dậu | Ngọ–Mùi | Chính bảng và sơ đồ tr. 17 |
| 20 | Ví dụ mục 9.4: Sửu xung chiếu Tý, tam hợp Dậu, Mùi | Xung chiếu Mùi, tam hợp Tỵ, Dậu, nhị hợp Tý | Bảng tam chiếu, xung chiếu tr. 18–19 |
| 272–275 | Nhãn Chi trong sơ đồ bị xoay | Đọc theo vị trí ô của mẫu tr. 5 | Dấu M, V, Đ, H; câu "Lưu Thái Tuế năm Dần tại cung Phu Quân" (tr. 274), "năm Tỵ tại cung Tật Ách" (tr. 275) |
| 275 | "Linh Linh"; Thiên Mã in cả ở Phụ Mẫu; Thiếu Dương in cả ở Tật Ách | Linh Tinh; bỏ hai ô in thừa | Hai sao này đã có ở đúng cung |

## 1. Địa bàn, Mệnh, Thân và Can 12 cung

### 1.1. Địa bàn

Vị trí và tên 12 cung không đổi (tr. 5):

| Tỵ (5) | Ngọ (6) | Mùi (7) | Thân (8) |
| :-: | :-: | :-: | :-: |
| **Thìn (4)** | | | **Dậu (9)** |
| **Mão (3)** | | | **Tuất (10)** |
| **Dần (2)** | **Sửu (1)** | **Tý (0)** | **Hợi (11)** |

### 1.2. Cung Mệnh và cung Thân

```
menh = (2 + (m − 1) − h) mod 12
than = (2 + (m − 1) + h) mod 12
```

Từ Dần là tháng Giêng đếm thuận đến tháng sinh, gọi đó là giờ Tý, rồi đếm nghịch (Mệnh) hoặc thuận (Thân) đến giờ sinh.

Thân chỉ có thể rơi vào Mệnh, Phúc Đức, Quan Lộc, Thiên Di, Tài Bạch hoặc Phu Thê (tr. 7). Công thức thỏa điều này trên cả 144 tổ hợp `(m, h)`.

### 1.3. 12 cung chức năng

Đi **thuận** từ Mệnh: `branch(k) = (menh + k) mod 12`.

| `k` | Cung | Rust |
| --: | :-- | :-- |
| 0 | Mệnh | `Menh` |
| 1 | Phụ Mẫu | `PhuMau` |
| 2 | Phúc Đức | `PhucDuc` |
| 3 | Điền Trạch | `DienTrach` |
| 4 | Quan Lộc | `QuanLoc` |
| 5 | Nô Bộc | `NoBoc` |
| 6 | Thiên Di | `ThienDi` |
| 7 | Tật Ách | `TatAch` |
| 8 | Tài Bạch | `TaiBach` |
| 9 | Tử Tức | `TuTuc` |
| 10 | Phu Thê | `PhuThe` |
| 11 | Huynh Đệ | `HuynhDe` |

Sách gọi Phu Thê là Thê Thiếp, hay Phu Quân với số đàn bà.

### 1.4. Can 12 cung (Ngũ Hổ Độn)

```
dan_stem    = ((s mod 5) × 2 + 2) mod 10
stem(b)     = (dan_stem + ((b − 2) mod 12)) mod 10
```

Can tăng dần từ Dần tới Sửu, nên Tý và Sửu nhận `+10`, `+11`: năm Giáp có Bính Dần … Ất Hợi, Bính Tý, Đinh Sửu. `dan_stem` bằng Can tháng Giêng của `month_can_chi`. Can và Chi của mỗi cung luôn cùng tính Âm Dương, nên `CanChi::new` không bao giờ trả `None`.

| Can năm | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi | Tý | Sửu |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Giáp, Kỷ | Bính | Đinh | Mậu | Kỷ | Canh | Tân | Nhâm | Quý | Giáp | Ất | Bính | Đinh |
| Ất, Canh | Mậu | Kỷ | Canh | Tân | Nhâm | Quý | Giáp | Ất | Bính | Đinh | Mậu | Kỷ |
| Bính, Tân | Canh | Tân | Nhâm | Quý | Giáp | Ất | Bính | Đinh | Mậu | Kỷ | Canh | Tân |
| Đinh, Nhâm | Nhâm | Quý | Giáp | Ất | Bính | Đinh | Mậu | Kỷ | Canh | Tân | Nhâm | Quý |
| Mậu, Quý | Giáp | Ất | Bính | Đinh | Mậu | Kỷ | Canh | Tân | Nhâm | Quý | Giáp | Ất |

### 1.5. Tháng hiệu lực và tháng nhuận

Với tháng thường, `m = LunarDate.month`. Với tháng nhuận (`is_leap_month`), theo `LeapMonthRule`:

| Quy tắc | `m` |
| :-- | :-- |
| `SameAsCurrentMonth` | `month` |
| `SplitAt15th` (mặc định) | `month` nếu `d ≤ 15`; `month mod 12 + 1` nếu `d ≥ 16` |

Quy tắc chỉ đổi `m`. Ngày `d` và Can Chi năm giữ nguyên. Từ năm 1600 đến 2400 ở UTC+7 không có tháng 12 nhuận, nên `mod 12` chỉ là phòng hờ cho đầu vào ngoài dải.

*Tân Biên* không nói về tháng nhuận: `LeapMonthRule` và mặc định `SplitAt15th` đến từ Task 4B.1.

## 2. Cục và 14 Chính Tinh

### 2.1. Ngũ Hành Cục

Lấy Can và Chi của cung Mệnh, tra Nạp Âm bằng `CanChi::na_yin().element()`:

| Hành | Cục | `c` | Trường Sinh khởi tại (mục 3.3) |
| :-- | :-- | --: | :-- |
| Thủy | Thủy Nhị Cục | 2 | Thân |
| Mộc | Mộc Tam Cục | 3 | Hợi |
| Kim | Kim Tứ Cục | 4 | Tỵ |
| Thổ | Thổ Ngũ Cục | 5 | Thân |
| Hỏa | Hỏa Lục Cục | 6 | Dần |

Kết quả theo Can năm và cung Mệnh, khớp 120/120 ô với bảng Lập Cục tr. 7:

| Can năm | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| Giáp, Kỷ | 2 | 2 | 6 | 6 | 3 | 3 | 5 | 5 | 4 | 4 | 6 | 6 |
| Ất, Canh | 6 | 6 | 5 | 5 | 4 | 4 | 3 | 3 | 2 | 2 | 5 | 5 |
| Bính, Tân | 5 | 5 | 3 | 3 | 2 | 2 | 4 | 4 | 6 | 6 | 3 | 3 |
| Đinh, Nhâm | 3 | 3 | 4 | 4 | 6 | 6 | 2 | 2 | 5 | 5 | 4 | 4 |
| Mậu, Quý | 4 | 4 | 2 | 2 | 5 | 5 | 6 | 6 | 3 | 3 | 2 | 2 |

Bản Mệnh là Nạp Âm của Can Chi năm, chỉ để hiển thị, không tham gia an sao.

### 2.2. Vị trí sao Tử Vi

Đầu vào `d` (1–30) và `c`:

```
nếu d mod c = 0:
    q   = d / c
    pos = (2 + q − 1) mod 12
ngược lại:
    r    = c − (d mod c)
    q    = (d + r) / c
    base = (2 + q − 1) mod 12
    pos  = (base − r) mod 12   nếu r lẻ
    pos  = (base + r) mod 12   nếu r chẵn
```

Kết quả, khớp 149/150 ô với bảng tr. 7–8; ô còn lại là lỗi đánh máy ở mục 0.4:

| Cục | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Thủy Nhị Cục | 22, 23 | 1, 24, 25 | 2, 3, 26, 27 | 4, 5, 28, 29 | 6, 7, 30 | 8, 9 | 10, 11 | 12, 13 | 14, 15 | 16, 17 | 18, 19 | 20, 21 |
| Mộc Tam Cục | 25 | 2, 28 | 3, 5 | 6, 8 | 1, 9, 11 | 4, 12, 14 | 7, 15, 17 | 10, 18, 20 | 13, 21, 23 | 16, 24, 26 | 19, 27, 29 | 22, 30 |
| Kim Tứ Cục | 5 | 3, 9 | 4, 7, 13 | 8, 11, 17 | 2, 12, 15, 21 | 6, 16, 19, 25 | 10, 20, 23, 29 | 14, 24, 27 | 18, 28 | 22 | 26 | 1, 30 |
| Thổ Ngũ Cục | 7 | 4, 12 | 5, 9, 17 | 10, 14, 22 | 3, 15, 19, 27 | 8, 20, 24 | 1, 13, 25, 29 | 6, 18, 30 | 11, 23 | 16, 28 | 21 | 2, 26 |
| Hỏa Lục Cục | 9, 19 | 5, 15, 25 | 6, 11, 21 | 12, 17, 27 | 4, 18, 23 | 10, 24, 29 | 2, 16, 30 | 8, 22 | 14, 28 | 1, 20 | 7, 26 | 3, 13 |

### 2.3. Vòng Tử Vi (6 sao)

| Sao | Rust | Vị trí |
| :-- | :-- | :-- |
| Tử Vi | `TuVi` | `tv` |
| Thiên Cơ | `ThienCo` | `tv − 1` |
| Thái Dương | `ThaiDuong` | `tv − 3` |
| Vũ Khúc | `VuKhuc` | `tv − 4` |
| Thiên Đồng | `ThienDong` | `tv − 5` |
| Liêm Trinh | `LiemTrinh` | `tv − 8` |

Sách mô tả chiều ngược lại: từ Tử Vi đi thuận, bỏ 3 cung an Liêm Trinh, bỏ 2 cung an Thiên Đồng, rồi Vũ Khúc, Thái Dương, bỏ 1 cung an Thiên Cơ. Hai cách cho cùng vị trí.

### 2.4. Vòng Thiên Phủ (8 sao)

`tp = (4 − tv) mod 12`, đối xứng với Tử Vi qua trục Dần–Thân.

| Sao | Rust | Vị trí |
| :-- | :-- | :-- |
| Thiên Phủ | `ThienPhu` | `tp` |
| Thái Âm | `ThaiAm` | `tp + 1` |
| Tham Lang | `ThamLang` | `tp + 2` |
| Cự Môn | `CuMon` | `tp + 3` |
| Thiên Tướng | `ThienTuong` | `tp + 4` |
| Thiên Lương | `ThienLuong` | `tp + 5` |
| Thất Sát | `ThatSat` | `tp + 6` |
| Phá Quân | `PhaQuan` | `tp + 10` |

## 3. Ba vòng sao cố định

### 3.1. Vòng Thái Tuế (12 sao)

`Thái Tuế` tại `y`, luôn đi thuận, cả nam lẫn nữ: sao thứ `i` tại `(y + i) mod 12`.

| `i` | Sao | Rust | `i` | Sao | Rust |
| --: | :-- | :-- | --: | :-- | :-- |
| 0 | Thái Tuế | `ThaiTue` | 6 | Tuế Phá | `TuePha` |
| 1 | Thiếu Dương | `ThieuDuong` | 7 | Long Đức | `LongDuc` |
| 2 | Tang Môn | `TangMon` | 8 | Bạch Hổ | `BachHo` |
| 3 | Thiếu Âm | `ThieuAm` | 9 | Phúc Đức | `PhucDuc` |
| 4 | Quan Phù | `QuanPhuThaiTue` | 10 | Điếu Khách | `DieuKhach` |
| 5 | Tử Phù | `TuPhu` | 11 | Trực Phù | `TrucPhu` |

### 3.2. Lộc Tồn, Kình Dương, Đà La và vòng Bác Sỹ (12 sao)

| Can năm | Lộc Tồn | Kình Dương | Đà La |
| :-- | :-- | :-- | :-- |
| Giáp | Dần | Mão | Sửu |
| Ất | Mão | Thìn | Dần |
| Bính | Tỵ | Ngọ | Thìn |
| Đinh | Ngọ | Mùi | Tỵ |
| Mậu | Tỵ | Ngọ | Thìn |
| Kỷ | Ngọ | Mùi | Tỵ |
| Canh | Thân | Dậu | Mùi |
| Tân | Dậu | Tuất | Thân |
| Nhâm | Hợi | Tý | Tuất |
| Quý | Tý | Sửu | Hợi |

Kình Dương đứng trước Lộc Tồn một cung, Đà La đứng sau một cung, không đổi theo nam nữ (tr. 10).

`Bác Sỹ` cùng cung Lộc Tồn. Sao thứ `i` của vòng tại `(loc + dir × i) mod 12`.

| `i` | Sao | Rust | `i` | Sao | Rust |
| --: | :-- | :-- | --: | :-- | :-- |
| 0 | Bác Sỹ | `BacSy` | 6 | Phi Liêm | `PhiLiem` |
| 1 | Lực Sỹ | `LucSy` | 7 | Hỷ Thần | `HyThan` |
| 2 | Thanh Long | `ThanhLong` | 8 | Bệnh Phù | `BenhPhu` |
| 3 | Tiểu Hao | `TieuHao` | 9 | Đại Hao | `DaiHao` |
| 4 | Tướng Quân | `TuongQuan` | 10 | Phục Binh | `PhucBinh` |
| 5 | Tấu Thư | `TauThu` | 11 | Quan Phủ | `QuanPhuBacSy` |

`LocTon`, `KinhDuong`, `DaLa` là ba biến thể của `TuViStar`.

### 3.3. Vòng Trường Sinh (12 sao)

Điểm khởi theo bảng mục 2.1. Sao thứ `i` tại `(start + dir × i) mod 12`, theo đúng thứ tự `LifeStage::ALL`: Trường Sinh, Mộc Dục, Quan Đới, Lâm Quan, Đế Vượng, Suy, Bệnh, Tử, Mộ, Tuyệt, Thai, Dưỡng. Sách viết Tràng Sinh; đặc tả giữ Trường Sinh như `LifeStage`.

Biến thể của `TuViStar` trùng tên biến thể của `LifeStage`: `TruongSinh`, `MocDuc`, `QuanDoi`, `LamQuan`, `DeVuong`, `Suy`, `Benh`, `Tu`, `Mo`, `Tuyet`, `Thai`, `Duong`.

## 4. Phụ tinh và Tứ Hóa

### 4.1. Theo tháng sinh

| Sao | Rust | Quy tắc | Vị trí |
| :-- | :-- | :-- | :-- |
| Tả Phụ | `TaPhu` | Thìn là tháng Giêng, đếm thuận | `(4 + m − 1) mod 12` |
| Hữu Bật | `HuuBat` | Tuất là tháng Giêng, đếm nghịch | `(10 − (m − 1)) mod 12` |
| Thiên Hình | `ThienHinh` | Dậu là tháng Giêng, đếm thuận | `(9 + m − 1) mod 12` |
| Thiên Riêu | `ThienRieu` | Sửu là tháng Giêng, đếm thuận | `(1 + m − 1) mod 12` |
| Thiên Y | `ThienY` | Cùng cung Thiên Riêu | |
| Thiên Giải | `ThienGiai` | Thân là tháng Giêng, đếm thuận | `(8 + m − 1) mod 12` |
| Địa Giải | `DiaGiai` | Mùi là tháng Giêng, đếm thuận | `(7 + m − 1) mod 12` |

Thiên Giải mỗi tháng đi một cung, không nhảy cung (tr. 13, Sách 1–4). Task và các lá số viết Tả Phù, mục An Sao viết Tả Phụ; đó là cùng một sao.

| `m` | Tả Phụ | Hữu Bật | Thiên Hình | Thiên Riêu, Thiên Y | Thiên Giải | Địa Giải |
| --: | :-- | :-- | :-- | :-- | :-- | :-- |
| 1 | Thìn | Tuất | Dậu | Sửu | Thân | Mùi |
| 2 | Tỵ | Dậu | Tuất | Dần | Dậu | Thân |
| 3 | Ngọ | Thân | Hợi | Mão | Tuất | Dậu |
| 4 | Mùi | Mùi | Tý | Thìn | Hợi | Tuất |
| 5 | Thân | Ngọ | Sửu | Tỵ | Tý | Hợi |
| 6 | Dậu | Tỵ | Dần | Ngọ | Sửu | Tý |
| 7 | Tuất | Thìn | Mão | Mùi | Dần | Sửu |
| 8 | Hợi | Mão | Thìn | Thân | Mão | Dần |
| 9 | Tý | Dần | Tỵ | Dậu | Thìn | Mão |
| 10 | Sửu | Sửu | Ngọ | Tuất | Tỵ | Thìn |
| 11 | Dần | Tý | Mùi | Hợi | Ngọ | Tỵ |
| 12 | Mão | Hợi | Thân | Tý | Mùi | Ngọ |

### 4.2. Theo giờ sinh

| Sao | Rust | Quy tắc | Vị trí |
| :-- | :-- | :-- | :-- |
| Văn Xương | `VanXuong` | Tuất là giờ Tý, đếm nghịch | `(10 − h) mod 12` |
| Văn Khúc | `VanKhuc` | Thìn là giờ Tý, đếm thuận | `(4 + h) mod 12` |
| Địa Không | `DiaKhong` | Hợi là giờ Tý, đếm nghịch | `(11 − h) mod 12` |
| Địa Kiếp | `DiaKiep` | Hợi là giờ Tý, đếm thuận | `(11 + h) mod 12` |
| Thai Phụ | `ThaiPhu` | Ngọ là giờ Tý, đếm thuận | `(6 + h) mod 12` |
| Phong Cáo | `PhongCao` | Dần là giờ Tý, đếm thuận | `(2 + h) mod 12` |

| Giờ | Văn Xương | Văn Khúc | Địa Không | Địa Kiếp | Thai Phụ | Phong Cáo |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Tý | Tuất | Thìn | Hợi | Hợi | Ngọ | Dần |
| Sửu | Dậu | Tỵ | Tuất | Tý | Mùi | Mão |
| Dần | Thân | Ngọ | Dậu | Sửu | Thân | Thìn |
| Mão | Mùi | Mùi | Thân | Dần | Dậu | Tỵ |
| Thìn | Ngọ | Thân | Mùi | Mão | Tuất | Ngọ |
| Tỵ | Tỵ | Dậu | Ngọ | Thìn | Hợi | Mùi |
| Ngọ | Thìn | Tuất | Tỵ | Tỵ | Tý | Thân |
| Mùi | Mão | Hợi | Thìn | Ngọ | Sửu | Dậu |
| Thân | Dần | Tý | Mão | Mùi | Dần | Tuất |
| Dậu | Sửu | Sửu | Dần | Thân | Mão | Hợi |
| Tuất | Tý | Dần | Sửu | Dậu | Thìn | Tý |
| Hợi | Hợi | Mão | Tý | Tuất | Tỵ | Sửu |

### 4.3. Theo ngày sinh

Tính sau khi đã có Tả, Hữu, Xương, Khúc.

| Sao | Rust | Quy tắc | Vị trí |
| :-- | :-- | :-- | :-- |
| Tam Thai | `TamThai` | Từ Tả Phụ là mồng một, đếm thuận đến ngày sinh | `(ta_phu + d − 1) mod 12` |
| Bát Tọa | `BatToa` | Từ Hữu Bật là mồng một, đếm nghịch đến ngày sinh | `(huu_bat − (d − 1)) mod 12` |
| Ân Quang | `AnQuang` | Từ Văn Xương là mồng một, đếm thuận đến ngày sinh, lùi lại một cung | `(van_xuong + d − 2) mod 12` |
| Thiên Quý | `ThienQuy` | Từ Văn Khúc là mồng một, đếm nghịch đến ngày sinh, lùi lại một cung | `(van_khuc − d + 2) mod 12` |

"Lùi lại" là đi ngược chiều vừa đếm, nên với Thiên Quý là tiến một cung. Cả 4 lá số xác nhận cách đọc này. Ân Quang và Thiên Quý luôn đối xứng qua trục Sửu–Mùi: tổng hai chỉ số bằng 2 theo `mod 12`.

### 4.4. Hỏa Tinh và Linh Tinh

| Chi năm | Hỏa Tinh khởi | Linh Tinh khởi |
| :-- | :-- | :-- |
| Dần, Ngọ, Tuất | Sửu | Mão |
| Thân, Tý, Thìn | Dần | Tuất |
| Tỵ, Dậu, Sửu | Mão | Tuất |
| Hợi, Mão, Mùi | Dậu | Tuất |

Cung khởi là giờ Tý. Dương Nam, Âm Nữ: Hỏa đếm thuận, Linh đếm nghịch. Âm Nam, Dương Nữ: ngược lại.

```
hoa_tinh  = (hoa_start  + dir × h) mod 12     // HoaTinh
linh_tinh = (linh_start − dir × h) mod 12     // LinhTinh
```

### 4.5. Theo Can năm

| Can năm | Thiên Khôi | Thiên Việt | Thiên Quan | Thiên Phúc | Lưu Hà | Thiên Trù | Quốc Ấn | Đường Phù |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Giáp | Sửu | Mùi | Mùi | Dậu | Dậu | Tỵ | Tuất | Mùi |
| Ất | Tý | Thân | Thìn | Thân | Tuất | Ngọ | Hợi | Thân |
| Bính | Hợi | Dậu | Tỵ | Tý | Mùi | Tý | Sửu | Tuất |
| Đinh | Hợi | Dậu | Dần | Hợi | Thìn | Tỵ | Dần | Hợi |
| Mậu | Sửu | Mùi | Mão | Mão | Tỵ | Ngọ | Sửu | Tuất |
| Kỷ | Tý | Thân | Dậu | Dần | Ngọ | Thân | Dần | Hợi |
| Canh | Ngọ | Dần | Hợi | Ngọ | Thân | Dần | Thìn | Sửu |
| Tân | Ngọ | Dần | Dậu | Tỵ | Mão | Ngọ | Tỵ | Dần |
| Nhâm | Mão | Tỵ | Tuất | Ngọ | Hợi | Dậu | Mùi | Thìn |
| Quý | Mão | Tỵ | Ngọ | Tỵ | Dần | Tuất | Thân | Tỵ |

`ThienKhoi`, `ThienViet`, `ThienQuan`, `ThienPhuc`, `LuuHa`, `ThienTru`, `QuocAn`, `DuongPhu`. Quốc Ấn là cung thứ 9 đếm thuận từ Lộc Tồn, `(loc + 8) mod 12`. Đường Phù là cung thứ 8 đếm nghịch, `(loc − 7) mod 12`.

Lưu Hà của Can Tân ở Mão theo bảng tr. 16 (T3). Hai lá số năm Tân của sách, Sách 2 và Sách 4, in nhầm sang Thìn.

### 4.6. Theo Chi năm

| Sao | Rust | Quy tắc |
| :-- | :-- | :-- |
| Long Trì | `LongTri` | Thìn là năm Tý, đếm thuận: `(4 + y) mod 12` |
| Phượng Các | `PhuongCac` | Tuất là năm Tý, đếm nghịch: `(10 − y) mod 12` |
| Giải Thần | `GiaiThan` | Cùng cung Phượng Các |
| Hồng Loan | `HongLoan` | Mão là năm Tý, đếm nghịch: `(3 − y) mod 12` |
| Thiên Hỷ | `ThienHy` | Đối cung Hồng Loan: `(9 − y) mod 12` |
| Thiên Không | `ThienKhong` | Trước Thái Tuế một cung: `(y + 1) mod 12` |
| Thiên Khốc | `ThienKhoc` | Ngọ là năm Tý, đếm nghịch: `(6 − y) mod 12` |
| Thiên Hư | `ThienHu` | Ngọ là năm Tý, đếm thuận: `(6 + y) mod 12`, luôn cùng cung Tuế Phá |
| Cô Thần, Quả Tú | `CoThan`, `QuaTu` | Theo Tam Hội, bảng dưới |
| Đào Hoa, Thiên Mã, Kiếp Sát, Hoa Cái | `DaoHoa`, `ThienMa`, `KiepSat`, `HoaCai` | Theo Tam Hợp, bảng dưới |
| Phá Toái | `PhaToai` | Tý, Ngọ, Mão, Dậu ở Tỵ; Dần, Thân, Tỵ, Hợi ở Dậu; Thìn, Tuất, Sửu, Mùi ở Sửu |

| Chi năm | Cô Thần | Quả Tú |
| :-- | :-- | :-- |
| Hợi, Tý, Sửu | Dần | Tuất |
| Dần, Mão, Thìn | Tỵ | Sửu |
| Tỵ, Ngọ, Mùi | Thân | Thìn |
| Thân, Dậu, Tuất | Hợi | Mùi |

| Chi năm | Đào Hoa | Thiên Mã | Kiếp Sát | Hoa Cái |
| :-- | :-- | :-- | :-- | :-- |
| Thân, Tý, Thìn | Dậu | Dần | Tỵ | Thìn |
| Dần, Ngọ, Tuất | Mão | Thân | Hợi | Tuất |
| Tỵ, Dậu, Sửu | Ngọ | Hợi | Dần | Sửu |
| Hợi, Mão, Mùi | Tý | Tỵ | Thân | Mùi |

Ma trận đầy đủ:

| Chi năm | Long Trì | Phượng Các, Giải Thần | Hồng Loan | Thiên Hỷ | Cô Thần | Quả Tú | Đào Hoa | Thiên Mã | Kiếp Sát | Hoa Cái | Phá Toái | Thiên Không | Thiên Khốc | Thiên Hư |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Tý | Thìn | Tuất | Mão | Dậu | Dần | Tuất | Dậu | Dần | Tỵ | Thìn | Tỵ | Sửu | Ngọ | Ngọ |
| Sửu | Tỵ | Dậu | Dần | Thân | Dần | Tuất | Ngọ | Hợi | Dần | Sửu | Sửu | Dần | Tỵ | Mùi |
| Dần | Ngọ | Thân | Sửu | Mùi | Tỵ | Sửu | Mão | Thân | Hợi | Tuất | Dậu | Mão | Thìn | Thân |
| Mão | Mùi | Mùi | Tý | Ngọ | Tỵ | Sửu | Tý | Tỵ | Thân | Mùi | Tỵ | Thìn | Mão | Dậu |
| Thìn | Thân | Ngọ | Hợi | Tỵ | Tỵ | Sửu | Dậu | Dần | Tỵ | Thìn | Sửu | Tỵ | Dần | Tuất |
| Tỵ | Dậu | Tỵ | Tuất | Thìn | Thân | Thìn | Ngọ | Hợi | Dần | Sửu | Dậu | Ngọ | Sửu | Hợi |
| Ngọ | Tuất | Thìn | Dậu | Mão | Thân | Thìn | Mão | Thân | Hợi | Tuất | Tỵ | Mùi | Tý | Tý |
| Mùi | Hợi | Mão | Thân | Dần | Thân | Thìn | Tý | Tỵ | Thân | Mùi | Sửu | Thân | Hợi | Sửu |
| Thân | Tý | Dần | Mùi | Sửu | Hợi | Mùi | Dậu | Dần | Tỵ | Thìn | Dậu | Dậu | Tuất | Dần |
| Dậu | Sửu | Sửu | Ngọ | Tý | Hợi | Mùi | Ngọ | Hợi | Dần | Sửu | Tỵ | Tuất | Dậu | Mão |
| Tuất | Dần | Tý | Tỵ | Hợi | Hợi | Mùi | Mão | Thân | Hợi | Tuất | Sửu | Hợi | Thân | Thìn |
| Hợi | Mão | Hợi | Thìn | Tuất | Dần | Tuất | Tý | Tỵ | Thân | Mùi | Dậu | Tý | Mùi | Tỵ |

### 4.7. Tứ Hóa

An vào đúng cung của sao gốc, nên phải tính sau khi có Xương, Khúc, Tả, Hữu. `HoaLoc`, `HoaQuyen`, `HoaKhoa`, `HoaKy`.

| Can | Hóa Lộc | Hóa Quyền | Hóa Khoa | Hóa Kỵ | Căn cứ |
| :-- | :-- | :-- | :-- | :-- | :-- |
| Giáp | Liêm Trinh | Phá Quân | Vũ Khúc | Thái Dương | Tr. 14, Sách 3 |
| Ất | Thiên Cơ | Thiên Lương | Tử Vi | Thái Âm | Tr. 14 |
| Bính | Thiên Đồng | Thiên Cơ | Văn Xương | Liêm Trinh | Tr. 14 |
| Đinh | Thái Âm | Thiên Đồng | Thiên Cơ | Cự Môn | Tr. 14 |
| Mậu | Tham Lang | Thái Âm | Hữu Bật | Thiên Cơ | Tr. 14 |
| Kỷ | Vũ Khúc | Tham Lang | Thiên Lương | Văn Khúc | Tr. 14 |
| Canh | Thái Dương | Vũ Khúc | Thái Âm | Thiên Đồng | Tr. 14, Task 4B.1 |
| Tân | Cự Môn | Thái Dương | Văn Khúc | Văn Xương | Sách 2, Sách 4 (tr. 14 in sai, mục 0.4) |
| Nhâm | Thiên Lương | Tử Vi | Tả Phụ | Vũ Khúc | Tr. 14 (T2) |
| Quý | Phá Quân | Cự Môn | Thái Âm | Tham Lang | Tr. 14, Sách 1 |

Biến thể:

- **Can Canh.** `lasotuvi`, theo sách cụ Thiên Lương, ghi Nhật, Vũ, Đồng, Âm: đổi chỗ Hóa Khoa và Hóa Kỵ. *Tân Biên* và Task 4B.1 ghi Nhật, Vũ, Âm, Đồng.
- **Can Nhâm.** Một số dị bản đọc chệch Tả Phụ thành Thiên Phủ; `lasotuvi` cũng ghi Thiên Phủ. Đặc tả theo tr. 14 (T2).

### 4.8. Mười sao bổ sung (T5)

Task 4B.1 chưa liệt kê, Task 4B.2 đưa vào `TuViStar`. Cả 4 lá số đều in các sao này, và quy tắc đã khớp.

| Sao | Rust | Quy tắc | Trang |
| :-- | :-- | :-- | :-- |
| Thiên Đức | `ThienDuc` | `(9 + y) mod 12` | 12 |
| Nguyệt Đức | `NguyetDuc` | `(5 + y) mod 12` | 12 |
| Thiên Tài | `ThienTai` | `(menh + y) mod 12` | 13 |
| Thiên Thọ | `ThienTho` | `(than + y) mod 12` | 13 |
| Thiên Thương | `ThienThuong` | Luôn ở cung Nô Bộc | 13 |
| Thiên Sứ | `ThienSu` | Luôn ở cung Tật Ách | 13 |
| Thiên La | `ThienLa` | Luôn ở Thìn | 13 |
| Địa Võng | `DiaVong` | Luôn ở Tuất | 13 |
| Lưu Niên Văn Tinh | `LuuNienVanTinh` | Giáp Tỵ, Ất Ngọ, Bính Thân, Đinh Dậu, Mậu Thân, Kỷ Dậu, Canh Hợi, Tân Tý, Nhâm Dần, Quý Mão | 16 |
| Đẩu Quân | `DauQuan` | Từ Thái Tuế là tháng Giêng đếm nghịch đến tháng sinh, gọi đó là giờ Tý, đếm thuận đến giờ sinh: `(y − (m − 1) + h) mod 12` | 16 |

### 4.9. Phân loại sao

`TuViStar::category`: 14 sao ở mục 2.3, 2.4 là `ChinhTinh` (Chính diệu, tr. 31), còn lại là `PhuTinh`.

`TuViStar::nature` là phân loại `Cat`, `Trung`, `Hung`. *Tân Biên* không có bảng này: dự án lập theo quy tắc dưới đây, từ chương Đặc tính các sao (tr. 33–51), và chủ dự án đã duyệt (T9). Đây là tính chất gốc của sao, không xét Miếu Hãm và không xét sao đi cùng.

1. Tiêu đề mục có nhãn Sát tinh, Bại tinh hay Hung tinh: `Hung`.
2. Có nhãn Tù tinh hay Ám tinh, kể cả đi kèm nhãn tốt: `Trung`.
3. Có nhãn khác (Đế, Phúc, Tài, Quý, Thiện, Quyền, Dũng, Phú, Văn, Trợ, Phù): `Cat`.
4. Không có nhãn thì xét các dòng mô tả chung, không xét dòng "gặp sao X": toàn lợi là `Cat`, toàn hại là `Hung`, có cả hai hoặc ghi "gặp tốt thì tốt, gặp xấu thì xấu" là `Trung`.

| Tính chất | Theo nhãn (quy tắc 1–3) | Theo mô tả (quy tắc 4) |
| :-- | :-- | :-- |
| `Cat` | Tử Vi, Thiên Cơ, Thái Dương, Vũ Khúc, Thiên Đồng, Thiên Phủ, Thái Âm, Thiên Tướng, Lộc Tồn, Tả Phụ, Hữu Bật, Văn Xương, Văn Khúc, Thiên Khôi, Thiên Việt | Thiếu Dương, Thiếu Âm, Long Đức, Phúc Đức, Bác Sỹ, Thanh Long, Tấu Thư, Hỷ Thần, Trường Sinh, Đế Vượng, Dưỡng, Thiên Y, Thiên Giải, Địa Giải, Tam Thai, Bát Tọa, Ân Quang, Thiên Quý, Thiên Quan, Thiên Phúc, Thiên Trù, Quốc Ấn, Đường Phù, Long Trì, Phượng Các, Giải Thần, Thiên Hỷ, Thiên Mã, Hoa Cái, Thiên Đức, Nguyệt Đức, Thiên Thọ, Lưu Niên Văn Tinh |
| `Trung` | Liêm Trinh, Cự Môn, Thiên Lương | Thái Tuế, Quan Phù, Lực Sỹ, Tướng Quân, Phi Liêm, Phục Binh, Quan Đới, Lâm Quan, Tử, Mộ, Tuyệt, Thai, Thiên Hình, Thiên Riêu, Thai Phụ, Phong Cáo, Hồng Loan, Cô Thần, Quả Tú, Đào Hoa, Thiên Tài, Đẩu Quân |
| `Hung` | Tham Lang, Thất Sát, Phá Quân, Tang Môn, Bạch Hổ, Kình Dương, Đà La, Tiểu Hao, Đại Hao, Địa Không, Địa Kiếp, Hỏa Tinh, Linh Tinh, Thiên Khốc, Thiên Hư | Tử Phù, Tuế Phá, Điếu Khách, Trực Phù, Bệnh Phù, Quan Phủ, Mộc Dục, Suy, Bệnh, Lưu Hà, Kiếp Sát, Phá Toái, Thiên Không, Thiên Thương, Thiên Sứ, Thiên La, Địa Võng |

Ngoại lệ duy nhất là Thất Sát: sách chỉ gắn nhãn Quyền tinh, Dũng tinh, nhưng chủ dự án chốt `Hung`, vì bộ Sát, Phá, Tham thuộc nhóm Sát tinh trong Tử Vi truyền thống dù đắc địa làm võ cách. Phá Quân và Tham Lang là `Hung` theo quy tắc 1. Tứ Hóa xét theo quy tắc 4: Hóa Lộc, Hóa Khoa là `Cat`, Hóa Quyền là `Trung`, Hóa Kỵ là `Hung`.

## 5. Tuần, Triệt, Đại Hạn và Tiểu Hạn

### 5.1. Tuần Trung Không Vong

Án ngữ giữa 2 cung trống của tuần Giáp chứa năm sinh: `khong_vong(can_chi_nam)` của `bazi_analysis`, đã có từ Giai đoạn 3. Khớp bảng tr. 17 trên cả 60 năm.

| Năm sinh trong khoảng | Cung an Tuần |
| :-- | :-- |
| Giáp Tý – Quý Dậu | Tuất – Hợi |
| Giáp Tuất – Quý Mùi | Thân – Dậu |
| Giáp Thân – Quý Tỵ | Ngọ – Mùi |
| Giáp Ngọ – Quý Mão | Thìn – Tỵ |
| Giáp Thìn – Quý Sửu | Dần – Mão |
| Giáp Dần – Quý Hợi | Tý – Sửu |

### 5.2. Triệt Lộ Không Vong

| Can năm | Cung an Triệt |
| :-- | :-- |
| Giáp, Kỷ | Thân – Dậu |
| Ất, Canh | Ngọ – Mùi |
| Bính, Tân | Thìn – Tỵ |
| Đinh, Nhâm | Dần – Mão |
| Mậu, Quý | Tý – Sửu |

Tuần và Triệt nằm giữa hai cung, nên trả về một cặp `Branch`, không phải một `TuViStar`.

### 5.3. Đại Hạn 10 năm

Ghi số Cục ở cung Mệnh, rồi đi theo `dir`, mỗi cung cộng 10:

```
branch(k) = (menh + dir × k) mod 12
tuổi bắt đầu của cung thứ k = c + 10 × k        (k = 0..11)
```

Mỗi cung phụ trách 10 năm từ tuổi bắt đầu. Dưới tuổi `c` chưa vào Đại Hạn nào.

Sách còn cách thứ hai, không ghi số Cục ở Mệnh mà bắt đầu từ cung kế tiếp, và nói người ta hay dùng cách thứ nhất "vì nó chính xác hơn" (tr. 21). Bốn lá số đều dùng cách thứ nhất. Đặc tả chỉ cài cách thứ nhất.

### 5.4. Tiểu Hạn 1 năm

| Chi năm sinh | Cung khởi (1 tuổi) |
| :-- | :-- |
| Dần, Ngọ, Tuất | Thìn |
| Thân, Tý, Thìn | Tuất |
| Tỵ, Dậu, Sửu | Mùi |
| Hợi, Mão, Mùi | Sửu |

```
t = +1 với Nam, −1 với Nữ          // không xét Âm Dương
branch(tuổi) = (start + t × (tuổi − 1)) mod 12
```

Sách ghi cách tương đương: viết Chi năm sinh vào cung khởi rồi lần lượt viết các Chi tiếp theo (tr. 22–23), nên năm có Chi `x` ứng với `(start + t × (x − y)) mod 12`.

Mỗi `TuViPalace` ghi Chi năm có Tiểu Hạn ở cung đó, cùng tuổi đầu tiên ứng với cung đó, từ 1 đến 12.

### 5.5. Thiên Bàn

**Can Chi.** Năm và tháng theo `LunarDate` hiệu lực: `year_can_chi`, `month_can_chi` với tháng in trên lịch, không dùng `m`. Ngày và giờ lấy từ `four_pillars` khi lập từ giờ dương lịch. Lập từ ngày âm thì ngày theo `lunar_to_solar`, giờ theo `hour_can_chi` của chính ngày đó.

**Âm Dương thuận lý, nghịch lý** (`YinYangHarmony`). So Âm Dương của năm sinh với cung Mệnh (tr. 31, tr. 52): cùng tính là `ThuanLy`, khác tính là `NghichLy`. Cung có chỉ số Chi chẵn là Dương. `lasotuvi` so giới tính với cung Mệnh; đặc tả không theo.

**Bản Mệnh và Cục** (`MenhCucRelation`). So hành Nạp Âm của năm sinh với hành của Cục (tr. 31):

| Quan hệ | Biến thể | Sách nhận định |
| :-- | :-- | :-- |
| Bản Mệnh sinh Cục | `MenhSinhCuc` | Tốt nhất |
| Cục sinh Bản Mệnh | `CucSinhMenh` | Tốt, nhưng kém Bản Mệnh sinh Cục |
| Bản Mệnh khắc Cục | `MenhKhacCuc` | Rất xấu |
| Cục khắc Bản Mệnh | `CucKhacMenh` | Không nêu riêng |
| Cùng hành | `BinhHoa` | Không nêu |

Sinh, khắc theo tr. 24: Kim sinh Thủy, Thủy sinh Mộc, Mộc sinh Hỏa, Hỏa sinh Thổ, Thổ sinh Kim; Kim khắc Mộc, Mộc khắc Thổ, Thổ khắc Thủy, Thủy khắc Hỏa, Hỏa khắc Kim.

**Mệnh Chủ, Thân Chủ.** *Tân Biên* không có hai sao này. Bảng dưới do chủ dự án duyệt, chuẩn *Tử Vi Đẩu Số Toàn Thư* (T8). Mệnh Chủ theo Chi cung Mệnh, Thân Chủ theo Chi năm:

| Chi | Mệnh Chủ | Thân Chủ |
| :-- | :-- | :-- |
| Tý | Tham Lang | Hỏa Tinh |
| Sửu | Cự Môn | Thiên Tướng |
| Dần | Lộc Tồn | Thiên Lương |
| Mão | Văn Khúc | Thiên Đồng |
| Thìn | Liêm Trinh | Văn Xương |
| Tỵ | Vũ Khúc | Thiên Cơ |
| Ngọ | Phá Quân | Hỏa Tinh |
| Mùi | Vũ Khúc | Thiên Tướng |
| Thân | Liêm Trinh | Thiên Lương |
| Dậu | Văn Khúc | Thiên Đồng |
| Tuất | Lộc Tồn | Văn Xương |
| Hợi | Cự Môn | Thiên Cơ |

Ngoài phạm vi tài liệu này: Lưu Đại Hạn, Lưu Nguyệt Hạn, Lưu Nhật Hạn, Lưu Thời Hạn (tr. 21–24), Định hướng chiếu (tr. 17–20).

## 6. Miếu Hãm và vector kiểm thử

### 6.1. Bảng Miếu, Vượng, Đắc, Bình, Hãm

Lấy từ danh sách của từng sao ở tr. 33–38. M là `Mieu`, V là `Vuong`, Đ là `Dac`, B là `Binh`, H là `Ham`. Mỗi hàng phủ đủ 12 cung.

| Sao | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| Tử Vi | B | Đ | M | B | V | M | M | Đ | M | B | V | B |
| Thiên Cơ | Đ | Đ | H | M | M | V | Đ | Đ | V | M | M | H |
| Thái Dương | H | Đ | V | V | V | M | M | Đ | H | H | H | H |
| Vũ Khúc | V | M | V | Đ | M | H | V | M | V | Đ | M | H |
| Thiên Đồng | V | H | M | Đ | H | Đ | H | H | M | H | H | Đ |
| Liêm Trinh | V | Đ | V | H | M | H | V | Đ | V | H | M | H |
| Thiên Phủ | M | B | M | B | V | Đ | M | Đ | M | B | V | Đ |
| Thái Âm | V | Đ | H | H | H | H | H | Đ | V | M | M | M |
| Tham Lang | H | M | Đ | H | V | H | H | M | Đ | H | V | H |
| Cự Môn | V | H | V | M | H | H | V | H | Đ | M | H | Đ |
| Thiên Tướng | V | Đ | M | H | V | Đ | V | Đ | M | H | V | Đ |
| Thiên Lương | V | Đ | V | V | M | H | M | Đ | V | H | M | H |
| Thất Sát | M | Đ | M | H | H | V | M | Đ | M | H | H | V |
| Phá Quân | M | V | H | H | Đ | H | M | V | H | H | Đ | H |

Sách 2 in Vũ Khúc ở Tý là M, Sách 3 in Phá Quân ở Mùi là Đ. Bảng trên theo danh sách tr. 35 và tr. 38, xem T7. Hai ô này ở V1, V2 ghi theo bảng.

#### Phụ tinh và Hóa Kỵ

Lấy từ tr. 39–43, D8 và D13 của `docs/INTERPRETATION_DB_SPEC.md`: chủ dự án chốt theo sách, kể cả Hỏa, Linh và Xương, Khúc. Sách chỉ liệt kê Đắc địa, không liệt kê Hãm địa. Lá số in không đánh dấu độ sáng phụ tinh.

| Sao | Rust | Đắc địa | Còn lại |
| :-- | :-- | :-- | :-- |
| Kình Dương, Đà La | `KinhDuong`, `DaLa` | Thìn, Tuất, Sửu, Mùi | Hãm |
| Hỏa Tinh, Linh Tinh | `HoaTinh`, `LinhTinh` | Dần, Mão, Thìn, Tỵ, Ngọ | Hãm |
| Văn Xương, Văn Khúc | `VanXuong`, `VanKhuc` | Thìn, Tuất, Sửu, Mùi, Tỵ, Hợi | Hãm |
| Địa Không, Địa Kiếp | `DiaKhong`, `DiaKiep` | Dần, Thân, Tỵ, Hợi | Hãm |
| Tang Môn, Bạch Hổ | `TangMon`, `BachHo` | Dần, Thân, Mão, Dậu | Hãm |
| Thiên Khốc, Thiên Hư | `ThienKhoc`, `ThienHu` | Tý, Ngọ, Mão, Dậu, Sửu, Mùi | Hãm |
| Thiên Riêu | `ThienRieu` | Dần, Mão, Dậu, Tuất | Hãm |
| Đại Hao, Tiểu Hao, Thiên Hình | `DaiHao`, `TieuHao`, `ThienHinh` | Dần, Thân, Mão, Dậu | `None` |
| Thiên Mã | `ThienMa` | Tỵ, Dần | `None` |
| Hóa Kỵ | `FourTransformations::HoaKy` | Thìn, Tuất, Sửu, Mùi | Hãm |

"Còn lại" là `Ham` khi sách có dùng chữ hãm địa cho sao đó, `None` khi sách không dùng. Sao khác không có độ sáng.

### 6.2. Vector kiểm thử

Các vector chứa đủ 105 sao và 4 Hóa. Vòng Trường Sinh và Tứ Hóa nằm cùng cột **Sao khác**. Đầu vào của V1, V2, V4, V5 là ngày âm in trong sách; ngày dương chỉ để tham khảo, đổi bằng `lunar_to_solar` ở UTC+7.

**V1 — Sách 2 (tr. 273).** Nam, âm lịch tháng 6, ngày 6 năm Tân Mão (1891), giờ Mão; dương lịch 11/07/1891. Âm Nam. Vector này khóa hai lỗi in của Can Tân và nhóm Hợi-Mão-Mùi. Lưu Hà ghi ở Mão theo bảng tr. 16; lá số in nhầm ở Thìn (T3).

- Mệnh Thìn, Thân Tuất (Thiên Di), Thủy Nhị Cục, Bản Mệnh Tùng Bách Mộc, `dir = −1`
- Tuần Ngọ–Mùi, Triệt Thìn–Tỵ, Tiểu Hạn 1 tuổi ở Sửu, `t = +1`

| Chi | Can Chi cung | Cung | Đại Hạn | Chính tinh | Sao khác |
| :-- | :-- | :-- | --: | :-- | :-- |
| Tý | Canh Tý | Tài Bạch | 42 | Vũ Khúc (V), Thiên Phủ (M) | Phúc Đức, Đại Hao, Mộ, Địa Giải, Bát Tọa, Hồng Loan, Đào Hoa, Thiên Đức, Lưu Niên Văn Tinh |
| Sửu | Tân Sửu | Tử Tức | 32 | Thái Dương (Đ), Thái Âm (Đ) | Điếu Khách, Bệnh Phù, Tử, Thiên Giải, Linh Tinh, Quả Tú, Hóa Quyền, Thiên Thọ, Đẩu Quân |
| Dần | Canh Dần | Phu Thê | 22 | Tham Lang (Đ) | Trực Phù, Hỷ Thần, Bệnh, Thiên Hình, Địa Kiếp, Tam Thai, Thiên Việt, Đường Phù |
| Mão | Tân Mão | Huynh Đệ | 12 | Thiên Cơ (M), Cự Môn (M) | Thái Tuế, Phi Liêm, Suy, Thiên Quý, Lưu Hà, Thiên Khốc, Hóa Lộc |
| Thìn | Nhâm Thìn | Mệnh | 2 | Tử Vi (V), Thiên Tướng (V) | Thiếu Dương, Tấu Thư, Đế Vượng, Thiên Không, Thiên La |
| Tỵ | Quý Tỵ | Phụ Mẫu | 112 | Thiên Lương (H) | Tang Môn, Tướng Quân, Lâm Quan, Hữu Bật, Phong Cáo, Thiên Phúc, Quốc Ấn, Cô Thần, Thiên Mã, Phá Toái |
| Ngọ | Giáp Ngọ | Phúc Đức | 102 | Thất Sát (M) | Thiếu Âm, Tiểu Hao, Quan Đới, Thiên Riêu, Thiên Y, Hỏa Tinh, Thiên Khôi, Thiên Trù, Thiên Hỷ |
| Mùi | Ất Mùi | Điền Trạch | 92 | — | Quan Phù, Thanh Long, Mộc Dục, Văn Xương, Văn Khúc, Long Trì, Phượng Các, Giải Thần, Hoa Cái, Hóa Khoa, Hóa Kỵ, Thiên Tài |
| Thân | Bính Thân | Quan Lộc | 82 | Liêm Trinh (V) | Tử Phù, Đà La, Lực Sỹ, Trường Sinh, Địa Không, Kiếp Sát, Nguyệt Đức |
| Dậu | Đinh Dậu | Nô Bộc | 72 | — | Tuế Phá, Lộc Tồn, Bác Sỹ, Dưỡng, Tả Phụ, Thai Phụ, Thiên Quan, Thiên Hư, Thiên Thương |
| Tuất | Mậu Tuất | Thiên Di | 62 | Phá Quân (Đ) | Long Đức, Kình Dương, Quan Phủ, Thai, Địa Võng |
| Hợi | Kỷ Hợi | Tật Ách | 52 | Thiên Đồng (Đ) | Bạch Hổ, Phục Binh, Tuyệt, Ân Quang, Thiên Sứ |

**V2 — Sách 3 (tr. 274).** Nữ, âm lịch tháng 9, ngày 10 năm Giáp Thìn (1904), giờ Ngọ; dương lịch 18/10/1904. Dương Nữ, Thân cư Mệnh. Tiểu Hạn 47 tuổi ở Tý.

- Mệnh Thìn, Thân Thìn (Mệnh), Mộc Tam Cục, Bản Mệnh Phú Đăng Hỏa, `dir = −1`
- Tuần Dần–Mão, Triệt Thân–Dậu, Tiểu Hạn 1 tuổi ở Tuất, `t = −1`

| Chi | Can Chi cung | Cung | Đại Hạn | Chính tinh | Sao khác |
| :-- | :-- | :-- | --: | :-- | :-- |
| Tý | Bính Tý | Tài Bạch | 43 | Cự Môn (V) | Bạch Hổ, Thanh Long, Dưỡng, Tả Phụ, Thai Phụ, Ân Quang |
| Sửu | Đinh Sửu | Tử Tức | 33 | Thiên Tướng (Đ) | Phúc Đức, Đà La, Lực Sỹ, Thai, Thiên Khôi, Quả Tú, Phá Toái, Thiên Đức |
| Dần | Bính Dần | Phu Thê | 23 | Thiên Đồng (M), Thiên Lương (V) | Điếu Khách, Lộc Tồn, Bác Sỹ, Tuyệt, Hữu Bật, Thiên Quý, Thiên Mã, Thiên Khốc, Đẩu Quân |
| Mão | Đinh Mão | Huynh Đệ | 13 | Vũ Khúc (Đ), Thất Sát (H) | Trực Phù, Kình Dương, Quan Phủ, Mộ, Địa Giải, Hóa Khoa |
| Thìn | Mậu Thìn | Mệnh | 3 | Thái Dương (V) | Thái Tuế, Phục Binh, Tử, Thiên Giải, Văn Xương, Linh Tinh, Hoa Cái, Hóa Kỵ, Thiên La |
| Tỵ | Kỷ Tỵ | Phụ Mẫu | 113 | — | Thiếu Dương, Đại Hao, Bệnh, Thiên Hình, Địa Không, Địa Kiếp, Bát Tọa, Thiên Trù, Thiên Hỷ, Cô Thần, Kiếp Sát, Thiên Không, Lưu Niên Văn Tinh |
| Ngọ | Canh Ngọ | Phúc Đức | 103 | Thiên Cơ (Đ) | Tang Môn, Bệnh Phù, Suy, Phượng Các, Giải Thần |
| Mùi | Tân Mùi | Điền Trạch | 93 | Tử Vi (Đ), Phá Quân (V) | Thiếu Âm, Hỷ Thần, Đế Vượng, Thiên Việt, Thiên Quan, Đường Phù, Hóa Quyền |
| Thân | Nhâm Thân | Quan Lộc | 83 | — | Quan Phù, Phi Liêm, Lâm Quan, Phong Cáo, Hỏa Tinh, Long Trì, Thiên Tài, Thiên Thọ |
| Dậu | Quý Dậu | Nô Bộc | 73 | Thiên Phủ (B) | Tử Phù, Tấu Thư, Quan Đới, Thiên Riêu, Thiên Y, Tam Thai, Thiên Phúc, Lưu Hà, Đào Hoa, Nguyệt Đức, Thiên Thương |
| Tuất | Giáp Tuất | Thiên Di | 63 | Thái Âm (M) | Tuế Phá, Tướng Quân, Mộc Dục, Văn Khúc, Quốc Ấn, Thiên Hư, Địa Võng |
| Hợi | Ất Hợi | Tật Ách | 53 | Liêm Trinh (H), Tham Lang (H) | Long Đức, Tiểu Hao, Trường Sinh, Hồng Loan, Hóa Lộc, Thiên Sứ |

**V3 — Dương Nam, tháng nhuận.** Nam, dương lịch 10/06/2020 lúc 10:30, UTC+7, không truyền kinh độ. Âm lịch ngày 19 tháng 4 nhuận năm Canh Tý, giờ Tỵ. Sách không có lá số Dương Nam, nên đây là vector duy nhất đi chiều thuận, và được tính từ đặc tả, không có nguồn ngoài đối chứng. Đại Hạn trùng ví dụ tr. 20: Hỏa Lục Cục ghi 6 ở Mệnh, 16 ở Phụ Mẫu, 26 ở Phúc Đức.

Với `SplitAt15th`, `d = 19` nên `m = 5`:

- Mệnh Sửu, Thân Hợi (Phu Thê), Hỏa Lục Cục, Bản Mệnh Bích Thượng Thổ, `dir = +1`
- Tuần Thìn–Tỵ, Triệt Ngọ–Mùi, Tiểu Hạn 1 tuổi ở Tuất, `t = +1`

| Chi | Can Chi cung | Cung | Đại Hạn | Chính tinh | Sao khác |
| :-- | :-- | :-- | --: | :-- | :-- |
| Tý | Mậu Tý | Huynh Đệ | 116 | Tử Vi (B) | Thái Tuế, Tướng Quân, Thai, Thiên Giải, Bát Tọa |
| Sửu | Kỷ Sửu | Mệnh | 6 | — | Thiếu Dương, Tấu Thư, Dưỡng, Thiên Hình, Đường Phù, Thiên Không, Thiên Tài, Đẩu Quân |
| Dần | Mậu Dần | Phụ Mẫu | 16 | Phá Quân (H) | Tang Môn, Phi Liêm, Trường Sinh, Tam Thai, Thiên Việt, Thiên Trù, Cô Thần, Thiên Mã |
| Mão | Kỷ Mão | Phúc Đức | 26 | — | Thiếu Âm, Hỷ Thần, Mộc Dục, Hồng Loan |
| Thìn | Canh Thìn | Điền Trạch | 36 | Liêm Trinh (M), Thiên Phủ (V) | Quan Phù, Bệnh Phù, Quan Đới, Địa Kiếp, Thiên Quý, Quốc Ấn, Long Trì, Hoa Cái, Thiên La |
| Tỵ | Tân Tỵ | Quan Lộc | 46 | Thái Âm (H) | Tử Phù, Đại Hao, Lâm Quan, Thiên Riêu, Thiên Y, Văn Xương, Linh Tinh, Kiếp Sát, Phá Toái, Hóa Khoa, Nguyệt Đức |
| Ngọ | Nhâm Ngọ | Nô Bộc | 56 | Tham Lang (H) | Tuế Phá, Phục Binh, Đế Vượng, Hữu Bật, Địa Không, Thiên Khôi, Thiên Phúc, Thiên Khốc, Thiên Hư, Thiên Thương |
| Mùi | Quý Mùi | Thiên Di | 66 | Thiên Đồng (H), Cự Môn (H) | Long Đức, Đà La, Quan Phủ, Suy, Phong Cáo, Hỏa Tinh, Hóa Kỵ |
| Thân | Giáp Thân | Tật Ách | 76 | Vũ Khúc (V), Thiên Tướng (M) | Bạch Hổ, Lộc Tồn, Bác Sỹ, Bệnh, Tả Phụ, Lưu Hà, Hóa Quyền, Thiên Sứ |
| Dậu | Ất Dậu | Tài Bạch | 86 | Thái Dương (H), Thiên Lương (H) | Phúc Đức, Kình Dương, Lực Sỹ, Tử, Văn Khúc, Thiên Hỷ, Đào Hoa, Hóa Lộc, Thiên Đức |
| Tuất | Bính Tuất | Tử Tức | 96 | Thất Sát (H) | Điếu Khách, Thanh Long, Mộ, Ân Quang, Phượng Các, Giải Thần, Quả Tú, Địa Võng |
| Hợi | Đinh Hợi | Phu Thê | 106 | Thiên Cơ (H) | Trực Phù, Tiểu Hao, Tuyệt, Địa Giải, Thai Phụ, Thiên Quan, Thiên Thọ, Lưu Niên Văn Tinh |

Với `SameAsCurrentMonth`, `m = 4`: Mệnh Tý, Thân Tuất, Hỏa Lục Cục, Tử Vi ở Tý, Thiên Phủ ở Thìn.

**V4 — Sách 1 (tr. 272).** Nam, âm lịch tháng 9, ngày 23 năm Quý Sửu (1913), giờ Dần; dương lịch 22/10/1913. Âm Nam. Vector phụ.

- Mệnh Thân, Thân Tý (Quan Lộc), Mộc Tam Cục, Bản Mệnh Tang Đố Mộc, `dir = −1`
- Tuần Dần–Mão, Triệt Tý–Sửu, Tiểu Hạn 1 tuổi ở Mùi, `t = +1`

| Chi | Can Chi cung | Cung | Đại Hạn | Chính tinh | Sao khác |
| :-- | :-- | :-- | --: | :-- | :-- |
| Tý | Giáp Tý | Quan Lộc | 83 | Liêm Trinh (V), Thiên Tướng (V) | Trực Phù, Lộc Tồn, Bác Sỹ, Dưỡng, Tả Phụ, Linh Tinh |
| Sửu | Ất Sửu | Nô Bộc | 73 | Thiên Lương (Đ) | Thái Tuế, Kình Dương, Quan Phủ, Thai, Địa Kiếp, Hỏa Tinh, Hoa Cái, Phá Toái, Thiên Thọ, Thiên Thương |
| Dần | Giáp Dần | Thiên Di | 63 | Thất Sát (M) | Thiếu Dương, Phục Binh, Tuyệt, Hữu Bật, Lưu Hà, Hồng Loan, Cô Thần, Kiếp Sát, Thiên Không |
| Mão | Ất Mão | Tật Ách | 53 | Thiên Đồng (Đ) | Tang Môn, Đại Hao, Mộ, Địa Giải, Thiên Khôi, Thiên Sứ, Lưu Niên Văn Tinh |
| Thìn | Bính Thìn | Tài Bạch | 43 | Vũ Khúc (M) | Thiếu Âm, Bệnh Phù, Tử, Thiên Giải, Phong Cáo, Bát Tọa, Thiên La |
| Tỵ | Đinh Tỵ | Tử Tức | 33 | Thái Dương (M) | Quan Phù, Hỷ Thần, Bệnh, Thiên Hình, Ân Quang, Thiên Việt, Thiên Phúc, Đường Phù, Long Trì, Thiên Khốc |
| Ngọ | Mậu Ngọ | Phu Thê | 23 | Phá Quân (M) | Tử Phù, Phi Liêm, Suy, Văn Khúc, Thiên Quan, Đào Hoa, Hóa Lộc, Nguyệt Đức |
| Mùi | Kỷ Mùi | Huynh Đệ | 13 | Thiên Cơ (Đ) | Tuế Phá, Tấu Thư, Đế Vượng, Thiên Hư, Đẩu Quân |
| Thân | Canh Thân | Mệnh | 3 | Tử Vi (M), Thiên Phủ (M) | Long Đức, Tướng Quân, Lâm Quan, Văn Xương, Thai Phụ, Quốc Ấn, Thiên Hỷ |
| Dậu | Tân Dậu | Phụ Mẫu | 113 | Thái Âm (M) | Bạch Hổ, Tiểu Hao, Quan Đới, Thiên Riêu, Thiên Y, Địa Không, Thiên Quý, Phượng Các, Giải Thần, Hóa Khoa, Thiên Tài |
| Tuất | Nhâm Tuất | Phúc Đức | 103 | Tham Lang (V) | Phúc Đức, Thanh Long, Mộc Dục, Tam Thai, Thiên Trù, Quả Tú, Hóa Kỵ, Thiên Đức, Địa Võng |
| Hợi | Quý Hợi | Điền Trạch | 93 | Cự Môn (Đ) | Điếu Khách, Đà La, Lực Sỹ, Trường Sinh, Thiên Mã, Hóa Quyền |

**V5 — Sách 4 (tr. 275).** Nam, âm lịch tháng 11, ngày 30 năm Tân Mùi (1931), giờ Dần; dương lịch 07/01/1932. Âm Nam, ngày 30 nằm ở năm dương kế tiếp. Vector phụ. Lưu Hà ghi ở Mão theo bảng tr. 16; lá số in nhầm ở Thìn (T3). Tiểu Hạn 23 tuổi ở Hợi, đúng lời sách: "có Tuần án ngữ, Triệt xung chiếu".

- Mệnh Tuất, Thân Dần (Quan Lộc), Mộc Tam Cục, Bản Mệnh Lộ Bàng Thổ, `dir = −1`
- Tuần Tuất–Hợi, Triệt Thìn–Tỵ, Tiểu Hạn 1 tuổi ở Sửu, `t = +1`

| Chi | Can Chi cung | Cung | Đại Hạn | Chính tinh | Sao khác |
| :-- | :-- | :-- | --: | :-- | :-- |
| Tý | Canh Tý | Phúc Đức | 103 | — | Tử Phù, Đại Hao, Dưỡng, Hữu Bật, Ân Quang, Linh Tinh, Đào Hoa, Nguyệt Đức, Lưu Niên Văn Tinh |
| Sửu | Tân Sửu | Điền Trạch | 93 | — | Tuế Phá, Bệnh Phù, Thai, Địa Kiếp, Phá Toái, Thiên Hư |
| Dần | Canh Dần | Quan Lộc | 83 | — | Long Đức, Hỷ Thần, Tuyệt, Tả Phụ, Thiên Quý, Thiên Việt, Đường Phù, Thiên Hỷ |
| Mão | Tân Mão | Nô Bộc | 73 | Liêm Trinh (H), Phá Quân (H) | Bạch Hổ, Phi Liêm, Mộ, Lưu Hà, Phượng Các, Giải Thần, Thiên Thương |
| Thìn | Nhâm Thìn | Thiên Di | 63 | — | Phúc Đức, Tấu Thư, Tử, Phong Cáo, Quả Tú, Thiên Đức, Thiên La |
| Tỵ | Quý Tỵ | Tật Ách | 53 | Thiên Phủ (Đ) | Điếu Khách, Tướng Quân, Bệnh, Địa Giải, Thiên Phúc, Quốc Ấn, Thiên Mã, Thiên Tài, Thiên Sứ |
| Ngọ | Giáp Ngọ | Tài Bạch | 43 | Thiên Đồng (H), Thái Âm (H) | Trực Phù, Tiểu Hao, Suy, Thiên Giải, Văn Khúc, Thiên Khôi, Thiên Trù, Hóa Khoa |
| Mùi | Ất Mùi | Tử Tức | 33 | Vũ Khúc (M), Tham Lang (M) | Thái Tuế, Thanh Long, Đế Vượng, Thiên Hình, Tam Thai, Bát Tọa, Hỏa Tinh, Hoa Cái |
| Thân | Bính Thân | Phu Thê | 23 | Thái Dương (H), Cự Môn (Đ) | Thiếu Dương, Đà La, Lực Sỹ, Lâm Quan, Văn Xương, Thai Phụ, Hồng Loan, Cô Thần, Kiếp Sát, Thiên Không, Hóa Lộc, Hóa Quyền, Hóa Kỵ |
| Dậu | Đinh Dậu | Huynh Đệ | 13 | Thiên Tướng (H) | Tang Môn, Lộc Tồn, Bác Sỹ, Quan Đới, Địa Không, Thiên Quan, Thiên Thọ |
| Tuất | Mậu Tuất | Mệnh | 3 | Thiên Cơ (M), Thiên Lương (M) | Thiếu Âm, Kình Dương, Quan Phủ, Mộc Dục, Địa Võng |
| Hợi | Kỷ Hợi | Phụ Mẫu | 113 | Tử Vi (B), Thất Sát (V) | Quan Phù, Phục Binh, Trường Sinh, Thiên Riêu, Thiên Y, Long Trì, Thiên Khốc, Đẩu Quân |

## 7. Kiểm chứng

### 7.1. Đã kiểm

Mô hình Python độc lập của đặc tả, chép dữ liệu từ ảnh trang sách. Bảng ở mục 1.4, 2.1, 2.2, 3.2, 4.1, 4.2, 4.5, 4.6, 6.1 và các vector do mô hình này sinh ra, không gõ tay. Nạp Âm và Tuần lấy từ `astro-core`.

| Phép kiểm | Kết quả |
| :-- | :-- |
| Bảng Lập Cục tr. 7 với Nạp Âm | 120/120 ô khớp |
| Bảng Tử Vi tr. 7–8 với thuật toán mục 2.2 | 149/150 ô khớp; ô còn lại là lỗi đánh máy |
| Tuần tr. 17 với `khong_vong` | 60/60 năm khớp |
| 4 lá số, 109 sao mỗi lá | 434/436 vị trí khớp; 2 ô lệch đều là Lưu Hà của Can Tân, lỗi in của lá số (T3). Quan Phù và Quan Phủ so gộp, vì Sách 1 viết hai sao giống nhau |
| 4 lá số: Mệnh, Thân, Cục, Bản Mệnh, Đại Hạn, Tuần, Triệt | Khớp hết |
| 4 lá số: dấu Miếu Hãm của 56 Chính Tinh | 54/56 khớp (T7) |
| 20 ví dụ in trong tr. 9–23 | Khớp hết |
| Tính chất sách nêu: Thân chỉ ở 6 cung, Thiên Hư cùng Tuế Phá, Thai Phụ và Phong Cáo cách Văn Khúc 2 cung | Khớp trên mọi đầu vào |
| Tiểu Hạn | Khớp lời bình Sách 3 (47 tuổi) và Sách 4 (23 tuổi) |
| `tuvi.rs` với mô hình Python, 47.808 lá số: cả 60 Can Chi năm, 1.405 lá số tháng nhuận, đủ 109 sao, Mệnh, Thân, Cục, Tuần, Triệt, Đại Hạn, Tiểu Hạn, Can cung | 0 lệch |
| Test của `tuvi.rs` với 4 lá số in trong sách | Khớp từng sao, chỉ trừ đúng các lỗi in ở mục 0.4 và T7 |
| 67 đột biến viết tay trên `tuvi.rs` | Test bắt hết 67 |

### 7.2. Chưa kiểm được

- Chỉ có một bản sách, lại là bản đánh máy lại. Chưa đối chiếu bản in gốc.
- Sách không có lá số Dương Nam hay Âm Nữ. Chiều thuận của vòng Bác Sỹ, Trường Sinh, Hỏa, Linh, Đại Hạn chỉ dựa vào lời văn tr. 10, tr. 20, không có lá số đối chứng.
- Tứ Hóa chỉ có lá số đối chứng cho Can Giáp, Tân, Quý. Bảy Can còn lại chỉ dựa vào bảng tr. 14, mà bảng đó đã có một ô sai.
- Bốn lá số chỉ thuộc 3 Can, nên Khôi, Việt, Thiên Quan, Thiên Phúc, Lưu Hà, Thiên Trù và Lộc Tồn của 7 Can còn lại cũng chỉ dựa vào bảng. Riêng Thiên Quan, Thiên Phúc, Lưu Hà, Thiên Trù trùng `lasotuvi` trên cả 10 Can.
- Chưa kiểm V3 bằng phần mềm lập lá số khác.

### 7.3. Khác biệt của `lasotuvi` so với *Tân Biên*

Đọc tĩnh bảng dữ liệu trong mã nguồn, không chạy code.

| Nội dung | `lasotuvi` | *Tân Biên* |
| :-- | :-- | :-- |
| Khôi, Việt | Đinh Dậu–Hợi, Mậu Mùi–Sửu, Canh Mùi–Sửu, Nhâm Tỵ–Mão | Đinh Hợi–Dậu, Mậu Sửu–Mùi, Canh Ngọ–Dần, Nhâm Mão–Tỵ |
| Hỏa, Linh nhóm Tỵ, Dậu, Sửu | Hỏa Tuất, Linh Mão | Hỏa Mão, Linh Tuất |
| Phá Toái | Tý, Ngọ, Mão, Dậu ở Dậu; Dần, Thân, Tỵ, Hợi ở Tỵ | Đảo lại |
| Thiên Giải | Nhảy cung, `(8 + 2 × (m − 1)) mod 12` | Mỗi tháng một cung |
| Tứ Hóa Can Canh | Nhật, Vũ, Đồng, Âm | Nhật, Vũ, Âm, Đồng |
| Tứ Hóa Can Nhâm | Hóa Khoa ở Thiên Phủ | Hóa Khoa ở Tả Phụ |
| Âm Dương thuận lý | Giới tính với cung Mệnh | Năm sinh với cung Mệnh |
| Mệnh Chủ, Thân Chủ | Tra bảng Chi bằng chỉ số Can năm | Không có |
| Miếu Hãm | Thiếu cả hàng Thiên Phủ; 13 hàng còn lại trùng | Đủ 14 hàng |

## 8. Quyết định và vấn đề còn mở

### 8.1. Đã chốt theo nguồn

| Vấn đề | Quyết định | Căn cứ |
| :-- | :-- | :-- |
| Năm và tháng | Năm đổi tại Tết, tháng theo lịch âm | Lập thành theo tuổi và tháng âm, tr. 5–7 |
| Tuổi | Tuổi mụ | 4 lá số |
| Thứ tự vòng Thái Tuế, Bác Sỹ, Trường Sinh | Như mục 3 | Tr. 9–10, sửa lỗi ở mục 0.4 |
| Thiên Quý | `(van_khuc − d + 2) mod 12` | 4 lá số |
| Thiên Giải | Mỗi tháng một cung | Tr. 13, 4 lá số |
| Đại Hạn | Cách thứ nhất, ghi số Cục ở Mệnh | Task 4B.1, tr. 21 |
| Tiểu Hạn | Nam thuận, Nữ nghịch, không xét Âm Dương | Tr. 22, Task 4B.1 |
| Giờ mặt trời thực | Chỉ dùng khi truyền kinh độ | Tiền lệ `four_pillars` |
| Tên hiển thị | Tả Phụ, Quan Phủ, Trường Sinh | Sách dùng cả Tả Phù và Tả Phụ; `LifeStage` đã dùng Trường Sinh |

### 8.2. Chốt ở Task 4B.2 và lần duyệt cuối

| ID | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| **T1** | Chiều 12 cung | Thuận, `(menh + k) mod 12`, theo tr. 6 và 4 lá số |
| **T2** | Hóa Khoa Can Nhâm | Tả Phụ, theo tr. 14. Rustdoc ghi rằng một số dị bản đọc chệch thành Thiên Phủ |
| **T3** | Lưu Hà Can Tân | Mão, theo tr. 16. Rustdoc ghi rằng 2 lá số năm Tân in nhầm sang Thìn |
| **T4** | Giờ Tý Sớm | Dùng `RatHourConvention`, mặc định `SplitAtMidnight` |
| **T5** | 10 sao ở mục 4.8 | Đưa vào `TuViStar` |
| **T8** | Mệnh Chủ, Thân Chủ | Bảng mục 5.5, chủ dự án duyệt, chuẩn *Tử Vi Đẩu Số Toàn Thư*. Tý và Ngọ đều là Hỏa Tinh |
| **T9** | Cát, Hung | Bảng mục 4.9; Thất Sát là `Hung`, Phá Quân giữ `Hung` |

### 8.3. Đã xử lý theo sách

| ID | Vấn đề | Cách xử lý |
| :-- | :-- | :-- |
| **T6** | Bảng in tr. 11 (Hỏa Tinh nhóm Hợi, Mão, Mùi) và tr. 14 (Hóa Quyền Can Tân) trái với lá số của chính sách | Theo lá số: Dậu và Thái Dương, chủ dự án đã xác nhận ở Task 4B.2. Mỗi điểm có 2 lá số độc lập đối chứng, và `lasotuvi` cũng cho cùng kết quả |
| **T7** | Sách 2 in Vũ Khúc ở Tý là M, Sách 3 in Phá Quân ở Mùi là Đ, trái với danh sách tr. 35, tr. 38 | Theo danh sách, vì đó là nơi sách định nghĩa. `lasotuvi` cũng ghi V ở cả hai ô |
