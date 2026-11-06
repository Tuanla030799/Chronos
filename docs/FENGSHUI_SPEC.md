# Đặc tả Phong Thủy (FENGSHUI_SPEC)

Tài liệu đầu vào để viết code Rust cho Bát Trạch và Cửu Cung Phi Tinh trong `astro-core` (Giai đoạn 4, Phần 4A). Mọi bảng và công thức ở đây là nguồn chuẩn duy nhất; code chỉ chuyển chúng thành `const` hoặc `match`, không thêm quy tắc mới.

## 0. Nguồn và quy ước

### 0.1. Nguồn dữ liệu

| Mục | Nguồn | Trạng thái |
| :-- | :-- | :-- |
| 1 | Task 4A.1 của chủ dự án, đối chiếu `NineStarUtil.POSITION`, `NineStarUtil.WU_XING`, `LunarUtil.POSITION_DESC` | Khớp nguồn |
| 2 | Task 4A.1 | Đã kiểm chứng, xem mục 2.3 |
| 3 | Bảng chuẩn *Bát Trạch Minh Cảnh* do chủ dự án cung cấp ở Task 4A.2; `lunar-ts` không có phần này | Đã duyệt, khớp 64/64 ô (F1) |
| 4.1–4.6 | `.ref/lunar-ts/src/lib/` (lunar-typescript 1.8.6 của 6tail, giấy phép MIT): `NineStar.ts`, `NineStarUtil.ts`, `Lunar.ts`, `LunarYear.ts` | Khớp nguồn, đã quét từng ngày, xem mục 5 |
| 4.7 | Task 4A.1, Task 4A.2; `lunar-ts` không có bàn Cửu Cung | Đã chốt (F3) |

`LunarUtil.ts` chỉ góp `POSITION_DESC` và `BASE_MONTH_ZHI_INDEX = 2`. Thông báo bản quyền MIT của dữ liệu 6tail nằm ở `THIRD_PARTY_NOTICES.md`.

### 0.2. Chỉ số, hàm phụ và kiểu Rust

- Chỉ số bắt đầu từ 0: `index = số Lạc Thư − 1`, trùng `NineStar.fromIndex` của nguồn. Số Lạc Thư 1–9 là giá trị hiển thị.
- `mod` là phép lấy dư không âm, ứng với `rem_euclid` trong Rust.
- `sao(x) = ((x − 1) mod 9) + 1` đưa số nguyên bất kỳ về 1–9, trong đó 0 thành 9.
- `Trigram` có 8 biến thể, vì Mệnh Quái không bao giờ là 5. `Palace` có 9 biến thể, thêm `TrungCung`, dùng cho bàn Cửu Cung. `Trigram` không có chỉ số riêng: thứ tự khai báo Khảm, Khôn, Chấn, Tốn, Càn, Đoài, Cấn, Ly là thứ tự hàng và cột của ma trận trong code.
- Càn và Cấn trùng chữ khi bỏ dấu, nên hai biến thể là `CanKim` và `CanTho`.
- Dùng lại từ `astro-core`: `Gender`, `Luck`, `FiveElements`, `Branch`, `SolarDate::to_jdn`, `SolarTerm`, `almanac::solar_month_branch`. `fengshui::Gender` là `bazi_analysis::Gender`, không khai báo lại.
- Cột **Rust** là tên biến thể trong `crates/astro-core/src/fengshui.rs`. Cột **Hán** giữ chữ giản thể của nguồn để đối chiếu; code hiển thị dùng cột Hán-Việt.

### 0.3. Khóa tra cứu

Mỗi phép tính dùng khóa riêng, nhầm khóa là sai kết quả. `S` là năm dương lịch của ngày đang xét.

| Khóa | Định nghĩa | Dùng cho |
| :-- | :-- | :-- |
| **Năm âm lịch** | `LunarDate.year` | Mệnh Quái; Niên Tinh, Nguyệt Tinh, Nguyên, Vận (`sect = 1`) |
| **Năm Lập Xuân theo ngày** | `S − 1` nếu ngày đang xét trước ngày Lập Xuân của năm `S`; ngược lại là `S`. Cả ngày giao tiết đã thuộc năm mới | Niên Tinh, Nguyệt Tinh, Nguyên, Vận (mặc định) |
| **Năm Lập Xuân theo thời khắc** | Như trên nhưng so đúng thời khắc giao tiết | Niên Tinh, Nguyệt Tinh, Nguyên, Vận (`sect = 3`) |
| **Chi tháng tiết khí theo ngày** | Mục 0.3 của `BAZI_ALMANAC_SPEC.md`, đã có `almanac::solar_month_branch` | Nguyệt Tinh (mặc định, `sect = 1`) |
| **Chi tháng tiết khí theo thời khắc** | Như trên nhưng so đúng thời khắc giao Tiết | Nguyệt Tinh (`sect = 3`) |
| **Bảng Chí theo năm dương** | Đông Chí của năm `S − 1`, Hạ Chí và Đông Chí của năm `S`, xét theo ngày địa phương | Nhật Tinh, Thời Tinh |
| **Chi ngày dân dụng** | `(JDN + 1) mod 12`, không đổi sau 23:00 | Thời Tinh |
| **Chi giờ** | 23:00–00:59 là Tý (0), 01:00–02:59 là Sửu (1), …, 21:00–22:59 là Hợi (11) | Thời Tinh |

Bảng Chí đi theo **năm dương lịch**, không theo năm âm lịch, vì `Lunar.fromSolar` tạo `LunarYear.fromYear(solar.getYear())` (`Lunar.ts:100`). `Lunar.fromYmdHms` cũng đổi về năm dương (`Lunar.ts:90`), nên hai cách tạo `Lunar` cho cùng kết quả: quét 73.353 ngày không lệch ngày nào.

Giờ truyền vào là giờ đồng hồ. Muốn dùng giờ mặt trời thực thì đổi bằng `to_true_solar_time` rồi chỉ truyền cho `day_nine_star` và `hour_nine_star`. Niên Tinh, Nguyệt Tinh, Nguyên, Vận luôn theo giờ đồng hồ, như trụ năm và trụ tháng của `four_pillars`: giờ mặt trời thực làm lệch thời khắc so với giao tiết.

**Múi giờ.** Nguồn tính giao tiết theo giờ Bắc Kinh, dự án dùng `tz_offset`, mặc định UTC+7. Đây là khác biệt có chủ ý, như mục 0.3 của `BAZI_ALMANAC_SPEC.md`. Với `tz_offset = 8.0` và `LapXuanDay`, `fengshui.rs` khớp nguồn trên cả 73.353 ngày, xem mục 5. So `fengshui.rs` ở UTC+7 với nguồn trên 73.353 ngày đó, mọi khác biệt của `LapXuanDay` đều nằm ở:

| Đại lượng | Số ngày khác | Những ngày đó |
| :-- | :-- | :-- |
| Niên Tinh | 8 | Ngày trước ngày Lập Xuân của nguồn trong các năm 1910, 1943, 1976, 1980, 2009, 2013, 2046, 2079: ở Việt Nam, Lập Xuân rơi vào 23:xx hôm đó |
| Nguyệt Tinh | 102 | Các ngày có Tiết rơi vào 23:xx giờ Việt Nam, gồm 8 ngày Lập Xuân trên |
| Thời Tinh | 17 | Ngày Chí rơi vào 23:xx giờ Việt Nam: Hạ Chí 1924, 1928, 1957, 1986, 2015, 2044, 2077; Đông Chí 1914, 1947, 1951, 1980, 1984, 2017, 2050, 2054, 2083, 2087 |
| Nhật Tinh | 217 | Trong khoảng 22/05/2077 đến 31/12/2077: trong 17 ngày Chí trên, chỉ Hạ Chí 2077 đổi mốc Giáp Tý, từ 21/07/2077 về 22/05/2077. Cả 224 ngày của khoảng này khác chiều bay hoặc khác sao |

Thời khắc của `astro-core` lệch giờ nguồn lùi một tiếng tối đa 38 giây trong 1900–2100, và 33 giây với Lập Xuân và hai Chí. Mức lệch này chỉ đổi ngày một lần, ở Hạ Chí 1953: `astro-core` cho 00:00:01 ngày 22/06, giờ nguồn lùi một tiếng là 23:59:53 ngày 21/06. Vì giờ nguồn gốc của nó cũng thuộc ngày 22/06, năm 1953 không nằm trong bảng trên.

## 1. Hậu Thiên Bát Quái và Lạc Thư Cửu Cung

### 1.1. Bảng Cửu Cung

| Chỉ số | Số Lạc Thư | Hán | Hán-Việt | Rust | Hướng | Ngũ Hành | Nhóm |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 0 | 1 | 坎 | Khảm | `Kham` | Bắc | Thủy | Đông Tứ |
| 1 | 2 | 坤 | Khôn | `Khon` | Tây Nam | Thổ | Tây Tứ |
| 2 | 3 | 震 | Chấn | `Chan` | Đông | Mộc | Đông Tứ |
| 3 | 4 | 巽 | Tốn | `Ton` | Đông Nam | Mộc | Đông Tứ |
| 4 | 5 | 中 | Trung Cung | `TrungCung` | Trung tâm | Thổ | Không có |
| 5 | 6 | 乾 | Càn | `CanKim` | Tây Bắc | Kim | Tây Tứ |
| 6 | 7 | 兑 | Đoài | `Doai` | Tây | Kim | Tây Tứ |
| 7 | 8 | 艮 | Cấn | `CanTho` | Đông Bắc | Thổ | Tây Tứ |
| 8 | 9 | 离 | Ly | `Ly` | Nam | Hỏa | Đông Tứ |

`TrungCung` chỉ có trong `Palace`, không có trong `Trigram`.

### 1.2. Phân nhóm

| Nhóm | Rust | Quái |
| :-- | :-- | :-- |
| Đông Tứ Trạch / Đông Tứ Mệnh | `DongTu` | Khảm (1), Ly (9), Chấn (3), Tốn (4) |
| Tây Tứ Trạch / Tây Tứ Mệnh | `TayTu` | Càn (6), Khôn (2), Cấn (8), Đoài (7) |

### 1.3. Tám hướng

Kiểu Rust là `CompassDirection`. Thứ tự này là thứ tự cột của ma trận mục 3.2.

| Chỉ số | Hướng | Rust | Quái |
| :-- | :-- | :-- | :-- |
| 0 | Bắc | `Bac` | Khảm |
| 1 | Đông Bắc | `DongBac` | Cấn |
| 2 | Đông | `Dong` | Chấn |
| 3 | Đông Nam | `DongNam` | Tốn |
| 4 | Nam | `Nam` | Ly |
| 5 | Tây Nam | `TayNam` | Khôn |
| 6 | Tây | `Tay` | Đoài |
| 7 | Tây Bắc | `TayBac` | Càn |

Hướng đối (`opposite`) cách 4 chỉ số: Bắc với Nam, Đông Bắc với Tây Nam, Đông với Tây, Đông Nam với Tây Bắc.

## 2. Mệnh Quái

### 2.1. Công thức

Đầu vào: `birth_lunar_year: i32`, `gender: Gender`.

```text
r = birth_lunar_year.rem_euclid(9)
Nam: k = (11 - r).rem_euclid(9)
Nữ:  k = (r + 4).rem_euclid(9)
Nếu k = 0 thì k = 9
Nếu k = 5 thì Nam lấy 2 (Khôn), Nữ lấy 8 (Cấn)
```

Công thức đồng dư này đúng cho mọi thế kỷ và là công thức dùng trong code. Cách tính tay theo `last_two = birth_lunar_year.rem_euclid(100)` chỉ là dạng khác của nó: gọi `s` là tổng chữ số của `last_two` rút về một chữ số, năm 19xx có Nam `10 − s`, Nữ `5 + s`; năm 20xx có Nam `9 − s`, Nữ `6 + s`, rồi rút về 1–9.

### 2.2. Bảng tra theo số dư

| `r` | Nam: `k` | Nam: Mệnh Quái | Nữ: `k` | Nữ: Mệnh Quái |
| :-- | :-- | :-- | :-- | :-- |
| 0 | 2 | Khôn (2) | 4 | Tốn (4) |
| 1 | 1 | Khảm (1) | 5 | Cấn (8) |
| 2 | 9 | Ly (9) | 6 | Càn (6) |
| 3 | 8 | Cấn (8) | 7 | Đoài (7) |
| 4 | 7 | Đoài (7) | 8 | Cấn (8) |
| 5 | 6 | Càn (6) | 9 | Ly (9) |
| 6 | 5 | Khôn (2) | 1 | Khảm (1) |
| 7 | 4 | Tốn (4) | 2 | Khôn (2) |
| 8 | 3 | Chấn (3) | 3 | Chấn (3) |

### 2.3. Kiểm chứng

| Mốc | `r` | Nam | Nữ |
| :-- | :-- | :-- | :-- |
| 1990 Canh Ngọ | 1 | Khảm (1) | `k` = 5, thành Cấn (8) |
| 2000 Canh Thìn | 2 | Ly (9) | Càn (6) |

`lunar-ts` không có API Mệnh Quái, nên kiểm chứng gián tiếp trên 200 năm 1900–2099, không lệch năm nào:

- `k` của Nam bằng Niên Tinh cùng năm trong `lunar-ts`, lấy tại ngày 1/7.
- `k` của cả Nam và Nữ bằng kết quả cách tính tay theo tổng chữ số.

Mệnh Quái dùng năm âm lịch, Niên Tinh dùng năm Lập Xuân. Hai khóa này khác nhau trong khoảng giữa Tết và Lập Xuân, nên ở đó `k` của Nam có thể khác Niên Tinh. Vector 3 ở mục 5 là một ví dụ.

## 3. Bát Du Niên

Mục này không có trong `lunar-ts`. Ma trận 3.2 đã đối chiếu từng ô với bảng chuẩn *Bát Trạch Minh Cảnh* do chủ dự án cung cấp ở Task 4A.2: khớp 64/64 ô (F1).

### 3.1. Tám sao

| Chỉ số | Hán | Hán-Việt | Rust | Sao đối ứng | Cát/Hung | Hào đổi |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 0 | 生气 | Sinh Khí | `SinhKhi` | Tham Lang | Cát | Trên |
| 1 | 天医 | Thiên Y | `ThienY` | Cự Môn | Cát | Giữa, dưới |
| 2 | 延年 | Diên Niên (Phước Đức) | `DienNien` | Vũ Khúc | Cát | Cả ba |
| 3 | 伏位 | Phục Vị | `PhucVi` | Tả Phù | Cát | Không đổi |
| 4 | 绝命 | Tuyệt Mệnh | `TuyetMenh` | Phá Quân | Hung | Giữa |
| 5 | 五鬼 | Ngũ Quỷ | `NguQuy` | Liêm Trinh | Hung | Trên, giữa |
| 6 | 六煞 | Lục Sát | `LucSat` | Văn Khúc | Hung | Trên, dưới |
| 7 | 祸害 | Họa Hại | `HoaHai` | Lộc Tồn | Hung | Dưới |

Tên kiểu Rust là `EightMansionsStar`; cột **Sao đối ứng** là `corresponding_star_name`.

### 3.2. Ma trận 8×8

Hàng là Mệnh Quái hoặc Trạch Quái, cột là hướng, cũng là quái đích ở mục 1.3. Trong code, `A.star_towards(B)` tra hàng `A`, cột `B`; `A.star_at(hướng)` tra theo hướng.

| Quái | Bắc | Đông Bắc | Đông | Đông Nam | Nam | Tây Nam | Tây | Tây Bắc |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 1 Khảm | Phục Vị | Ngũ Quỷ | Thiên Y | Sinh Khí | Diên Niên | Tuyệt Mệnh | Họa Hại | Lục Sát |
| 2 Khôn | Tuyệt Mệnh | Sinh Khí | Họa Hại | Ngũ Quỷ | Lục Sát | Phục Vị | Thiên Y | Diên Niên |
| 3 Chấn | Thiên Y | Lục Sát | Phục Vị | Diên Niên | Sinh Khí | Họa Hại | Tuyệt Mệnh | Ngũ Quỷ |
| 4 Tốn | Sinh Khí | Tuyệt Mệnh | Diên Niên | Phục Vị | Thiên Y | Ngũ Quỷ | Lục Sát | Họa Hại |
| 6 Càn | Lục Sát | Thiên Y | Ngũ Quỷ | Họa Hại | Tuyệt Mệnh | Diên Niên | Sinh Khí | Phục Vị |
| 7 Đoài | Họa Hại | Diên Niên | Tuyệt Mệnh | Lục Sát | Ngũ Quỷ | Thiên Y | Phục Vị | Sinh Khí |
| 8 Cấn | Ngũ Quỷ | Phục Vị | Lục Sát | Tuyệt Mệnh | Họa Hại | Sinh Khí | Diên Niên | Thiên Y |
| 9 Ly | Diên Niên | Họa Hại | Sinh Khí | Thiên Y | Phục Vị | Lục Sát | Ngũ Quỷ | Tuyệt Mệnh |

Hằng `EIGHT_MANSIONS` trong code chép bảng của chủ dự án, cột theo thứ tự `Trigram::ALL`, khác thứ tự hướng ở trên. Test `matrix_matches_section_3_2_by_direction` so đủ 64 ô giữa hai cách xếp.

### 3.3. Quy tắc kiểm chứng

Mỗi quái viết thành ba hào đọc từ dưới lên, 1 là hào dương: Càn `111`, Đoài `110`, Ly `101`, Chấn `100`, Tốn `011`, Khảm `010`, Cấn `001`, Khôn `000`. Sao của một ô do các hào khác nhau giữa quái ở hàng và quái của hướng quyết định, theo cột **Hào đổi** ở mục 3.1.

Test của `astro-core` khẳng định các tính chất sau trên ma trận:

1. Mỗi ô khớp quy tắc đổi hào.
2. Ma trận đối xứng: sao của quái A nhìn về quái B bằng sao của B nhìn về A.
3. Mỗi hàng có đủ 8 sao, không trùng.
4. Bốn sao Cát luôn nằm ở bốn hướng cùng nhóm với quái ở hàng, bốn sao Hung ở nhóm còn lại.

### 3.4. Cách dùng

- **Mệnh Quái:** hàng là Mệnh Quái của người, tra sao theo từng hướng bằng `Trigram::star_at`, hoặc cả tám hướng bằng `Trigram::chart`.
- **Trạch Quái:** hàng là quái của hướng **Tọa**, ví dụ nhà tọa Bắc hướng Nam là Khảm trạch (F2). Hàm `house_trigram(sitting)` nhận hướng Tọa. Nếu chỉ biết hướng nhà nhìn ra thì `house_trigram_from_facing(facing)` lấy `Tọa = facing.opposite()`. Có sẵn quái của hướng Tọa thì tra thẳng bằng `Trigram::star_at`.

### 3.5. Tương hợp Bát Biến giữa hai người

Theo Task 4A.2. Bát Biến là tên khác của tám sao Bát Du Niên khi so hai quái. Hàm `kua_compatibility(first, second)` dùng lại ma trận 3.2, không có quy tắc nào khác và không chấm điểm: lấy Mệnh Quái của hai người làm hàng và cột, sao ở ô đó là kết quả, Cát hay Hung theo mục 3.1.

Ma trận đối xứng nên thứ tự hai người không đổi kết quả. Hai người cùng Mệnh Quái cho Phục Vị. Hai người cùng nhóm ở mục 1.2 luôn cho sao Cát, khác nhóm luôn cho sao Hung.

## 4. Huyền Không Cửu Tinh

### 4.1. Chín sao

| Chỉ số | Số | Hán | Hán-Việt | Rust | Ngũ Hành | Màu | Cung gốc | Huyền Không | Cát/Hung |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 0 | 1 | 一白水 | Nhất Bạch Thủy | `NhatBach` | Thủy | Trắng | Khảm | Tham Lang | Cát |
| 1 | 2 | 二黑土 | Nhị Hắc Thổ | `NhiHac` | Thổ | Đen | Khôn | Cự Môn | Hung |
| 2 | 3 | 三碧木 | Tam Bích Mộc | `TamBich` | Mộc | Xanh biếc | Chấn | Lộc Tồn | Hung |
| 3 | 4 | 四绿木 | Tứ Lục Mộc | `TuLuc` | Mộc | Xanh lục | Tốn | Văn Khúc | Cát |
| 4 | 5 | 五黄土 | Ngũ Hoàng Thổ | `NguHoang` | Thổ | Vàng | Trung Cung | Liêm Trinh | Hung |
| 5 | 6 | 六白金 | Lục Bạch Kim | `LucBach` | Kim | Trắng | Càn | Vũ Khúc | Cát |
| 6 | 7 | 七赤金 | Thất Xích Kim | `ThatXich` | Kim | Đỏ | Đoài | Phá Quân | Hung |
| 7 | 8 | 八白土 | Bát Bạch Thổ | `BatBach` | Thổ | Trắng | Cấn | Tả Phù | Cát |
| 8 | 9 | 九紫火 | Cửu Tử Hỏa | `CuuTu` | Hỏa | Tím | Ly | Hữu Bật | Cát |

Nguồn: `NineStarUtil.NUMBER`, `COLOR`, `WU_XING`, `POSITION`, `LUCK_XUAN_KONG` và `NineStar.NAME_XUAN_KONG`. Cột Cát/Hung là giá trị cố định của `LUCK_XUAN_KONG`, không đổi theo Vận.

### 4.2. Tam Nguyên Cửu Vận

Nguồn: `LunarYear.getYuan`, `LunarYear.getYun` (`LunarYear.ts:310-316`), với `YUAN = ['下', '上', '中']`, `YUN = ['七', '八', '九', '一', '二', '三', '四', '五', '六']`:

```text
Nguyên = YUAN[floor((năm + 2696) / 60) mod 3]
Vận    = YUN[floor((năm + 2696) / 20) mod 9]
```

Dạng rút gọn, tương đương trên 1600–2400:

```text
van    = ((năm - 1864).div_euclid(20)).rem_euclid(9) + 1
nguyen = ((năm - 1864).div_euclid(60)).rem_euclid(3)     // 0 Thượng, 1 Trung, 2 Hạ
```

| Nguyên | Rust | Vận | Sao chủ vận | Năm |
| :-- | :-- | :-- | :-- | :-- |
| Thượng Nguyên | `ThuongNguyen` | 1 | Nhất Bạch | 1864–1883 |
| Thượng Nguyên | `ThuongNguyen` | 2 | Nhị Hắc | 1884–1903 |
| Thượng Nguyên | `ThuongNguyen` | 3 | Tam Bích | 1904–1923 |
| Trung Nguyên | `TrungNguyen` | 4 | Tứ Lục | 1924–1943 |
| Trung Nguyên | `TrungNguyen` | 5 | Ngũ Hoàng | 1944–1963 |
| Trung Nguyên | `TrungNguyen` | 6 | Lục Bạch | 1964–1983 |
| Hạ Nguyên | `HaNguyen` | 7 | Thất Xích | 1984–2003 |
| Hạ Nguyên | `HaNguyen` | 8 | Bát Bạch | 2004–2023 |
| Hạ Nguyên | `HaNguyen` | 9 | Cửu Tử | 2024–2043 |

Chu kỳ lặp 180 năm: 2044 trở lại Thượng Nguyên Vận 1. Nguồn chỉ nhận một số năm, không quy định ngày đổi Vận. Dự án dùng đúng năm `Y` của Niên Tinh ở mục 4.3 (F4), nên Vận đổi cùng lúc với Niên Tinh, mặc định tại ngày Lập Xuân.

### 4.3. Niên Tinh

Nguồn: `Lunar.getYearNineStar` và `_getYearNineStar` (`Lunar.ts:1225-1250`):

```text
yuan   = floor((Y + 2696) / 60) mod 3
offset = (62 + yuan * 3 - chỉ_số_Giáp_Tý_từ_1) mod 9, 0 thành 9
index  = offset - 1
```

Rút gọn được thành:

```text
Niên Tinh = sao(11 - Y)
```

`Y` chọn theo `sect`:

| `sect` nguồn | `NineStarBoundary` | `Y` | Chi tháng cho Nguyệt Tinh |
| :-- | :-- | :-- | :-- |
| 1 | `LunarNewYear` | Năm âm lịch | Theo ngày |
| 2 | `LapXuanDay` | Năm Lập Xuân theo ngày | Theo ngày |
| 3 | `LapXuanExact` | Năm Lập Xuân theo thời khắc | Theo thời khắc |

Mặc định của nguồn và của dự án là `sect = 2`, tức **Niên Tinh đổi tại ngày Lập Xuân**, không đổi tại Tết.

`LunarNewYear` lấy năm từ `solar_to_lunar`, nên các ngày trước Tết 1600 ngoài phạm vi. `LapXuanExact` so thời khắc không làm tròn, như `four_pillars`.

Không dùng `LunarYear.getNineStar` (`LunarYear.ts:318-326`). Hàm này đặt sai ngoặc nên `yuan` là số lẻ, trả về chỉ số không nguyên, sai 198 trên 201 năm trong 1900–2100.

### 4.4. Nguyệt Tinh

Nguồn: `Lunar.getMonthNineStar` (`Lunar.ts:1252-1270`):

```text
n = 27 - (chi_năm mod 3) * 3
Nếu chi_tháng < 2 (Tý, Sửu) thì n = n - 3
index = (n - chi_tháng) mod 9
```

`chi_năm`, `chi_tháng` đều tính từ Tý = 0. Rút gọn được thành:

```text
start = 8 nếu chi_năm mod 3 = 0; 5 nếu = 1; 2 nếu = 2
k     = (chi_tháng - 2) mod 12        // tháng Dần là 0, tháng Sửu là 11
Nguyệt Tinh = sao(start - k)
```

Bước `n − 3` của nguồn chỉ là để hai tháng Tý, Sửu nối tiếp chuỗi đếm lùi. Với `sect` 2 và 3, tháng Tý, Sửu vẫn thuộc năm đã mở từ tháng Dần trước đó, nên `chi_năm` không đổi. Với `sect = 1`, `chi_năm` đổi ngay tại Tết, nên tháng Sửu sau Tết lấy Chi của năm âm mới.

| Chi năm | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi | Tý | Sửu |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Tý, Ngọ, Mão, Dậu | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 9 | 8 | 7 | 6 |
| Thìn, Tuất, Sửu, Mùi | 5 | 4 | 3 | 2 | 1 | 9 | 8 | 7 | 6 | 5 | 4 | 3 |
| Dần, Thân, Tỵ, Hợi | 2 | 1 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 9 |

Không dùng `LunarMonth.getNineStar`. Hàm đó đi theo tháng âm lịch và năm âm lịch, không theo tháng tiết khí.

### 4.5. Nhật Tinh

Nguồn: `Lunar.getDayNineStar` (`Lunar.ts:1277-1303`). Mọi phép so sánh và trừ đều theo ngày, dùng JDN.

**Mốc Giáp Tý gần nhất.** Với `C` là JDN của ngày Chí:

```text
i = (C + 49) mod 60          // chỉ số Can Chi ngày, Giáp Tý = 0
mốc(C) = C + 60 - i  nếu i > 29
mốc(C) = C - i       nếu i <= 29
```

Khi `i = 30`, hai phía cách đều 30 ngày, nguồn chọn phía sau.

**Tính sao.** Với `D` là JDN của ngày đang xét, `S` là năm dương lịch của ngày đó:

```text
A = mốc(Đông Chí năm S − 1)
N = mốc(Hạ Chí năm S)
B = mốc(Đông Chí năm S)
```

| Nhánh | Điều kiện, xét theo thứ tự | `offset` | Chiều |
| :-- | :-- | :-- | :-- |
| 1 | `A ≤ D < N` | `(D − A) mod 9` | Thuận |
| 2 | `N ≤ D < B` | `8 − ((D − N) mod 9)` | Nghịch |
| 3 | `D ≥ B` | `(D − B) mod 9` | Thuận |
| 4 | `D < A` | `(8 + (A − D)) mod 9` | Nghịch |

Nhật Tinh là `offset + 1`: từ mốc gần Đông Chí đếm thuận khởi Nhất Bạch, từ mốc gần Hạ Chí đếm nghịch khởi Cửu Tử. Trên 73.353 ngày của mục 5, nhánh 1 đến 4 lần lượt dùng ở 33.832, 36.167, 2.667 và 687 ngày ở UTC+8, ở 33.772, 36.227, 2.667 và 687 ngày ở UTC+7.

**Đường nối.** Khoảng cách giữa hai mốc liên tiếp thường là 180 ngày, chia hết cho 9, nên chuỗi sao nối liền. Trong 1900–2100 có 18 khoảng dài 240 ngày, không chia hết cho 9, nên chuỗi nhảy. Các số dưới đây là của UTC+8, tức của nguồn:

- 6 khoảng từ mốc Đông Chí sang mốc Hạ Chí: sao nhảy tại `N`. Các khoảng này kết thúc vào 1917, 1974, 1997, 2020, 2077, 2100.
- 12 khoảng từ mốc Hạ Chí sang mốc Đông Chí: `A` rơi vào tháng 1. Vì bảng Chí đổi vào 1/1, sao nhảy tại **1/1**, chứ không tại `A`. Từ 1/1 đến ngày trước `A`, nhánh 4 đếm lùi về Nhất Bạch.

Ví dụ: 31/12/2008 là 4, 1/1/2009 là 9, 18/1/2009 là 1, 19/1/2009 là 1. Nếu đếm liền từ `N` thì 1/1/2009 sẽ là 3.

Tổng cộng 225 ngày khác với cách đếm liền. Dự án giữ đúng nguồn (F5), và rustdoc của `day_nine_star` ghi lại đặc điểm này.

Ở UTC+7, mốc của Hạ Chí 2077 về 22/05/2077 như mục 0.3, nên số khoảng là 5 và 13, số ngày là 241. Cửa sổ thứ 13 ngoài bảng dưới: năm 2078, `A` là 17/01/2078, 16 ngày, sao 31/12/2077 là 2, sao 1/1/2078 là 7.

| Năm `S` | `A` | Số ngày từ 1/1 đến trước `A` | Sao ngày 31/12 trước đó | Sao ngày 1/1 |
| :-- | :-- | :-- | :-- | :-- |
| 1906 | 20/01/1906 | 19 | 5 | 1 |
| 1929 | 19/01/1929 | 18 | 4 | 9 |
| 1940 | 22/01/1940 | 21 | 7 | 3 |
| 1952 | 19/01/1952 | 18 | 4 | 9 |
| 1963 | 21/01/1963 | 20 | 6 | 2 |
| 1986 | 20/01/1986 | 19 | 5 | 1 |
| 2009 | 19/01/2009 | 18 | 4 | 9 |
| 2032 | 19/01/2032 | 18 | 4 | 9 |
| 2043 | 21/01/2043 | 20 | 6 | 2 |
| 2055 | 18/01/2055 | 17 | 3 | 8 |
| 2066 | 20/01/2066 | 19 | 5 | 1 |
| 2089 | 19/01/2089 | 18 | 4 | 9 |

### 4.6. Thời Tinh

Nguồn: `Lunar.getTimeNineStar` (`Lunar.ts:1305-1317`).

```text
Thuận nếu ngày ≥ Đông Chí năm S hoặc ngày < Hạ Chí năm S; ngược lại là Nghịch
start = [0, 3, 6][chi_ngày mod 3] nếu Thuận; [8, 5, 2][chi_ngày mod 3] nếu Nghịch
index = (start + chi_giờ) mod 9   nếu Thuận
index = (start - chi_giờ) mod 9   nếu Nghịch
```

Ngày Đông Chí và ngày Hạ Chí tính cả ngày: nguyên ngày Đông Chí đã Thuận, nguyên ngày Hạ Chí đã Nghịch.

| Chiều | Chi ngày | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Thuận | Tý, Ngọ, Mão, Dậu | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 1 | 2 | 3 |
| Thuận | Thìn, Tuất, Sửu, Mùi | 4 | 5 | 6 | 7 | 8 | 9 | 1 | 2 | 3 | 4 | 5 | 6 |
| Thuận | Dần, Thân, Tỵ, Hợi | 7 | 8 | 9 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
| Nghịch | Tý, Ngọ, Mão, Dậu | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 9 | 8 | 7 |
| Nghịch | Thìn, Tuất, Sửu, Mùi | 6 | 5 | 4 | 3 | 2 | 1 | 9 | 8 | 7 | 6 | 5 | 4 |
| Nghịch | Dần, Thân, Tỵ, Hợi | 3 | 2 | 1 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 |

Ba điểm dễ nhầm:

1. **Thời Tinh và Nhật Tinh đổi chiều ở hai mốc khác nhau.** Thời Tinh đổi đúng ngày Chí, Nhật Tinh đổi tại mốc Giáp Tý, cách ngày Chí tới 30 ngày. Vector 2 rơi vào khoảng lệch này.
2. **Giờ Tý Sớm giữ Chi ngày dân dụng.** Lúc 23:xx, `chi_giờ` là Tý nhưng `chi_ngày` vẫn của ngày đang xét, không sang ngày hôm sau như Can giờ. Ở vector 2, dùng Chi ngày hôm sau sẽ ra 9 thay vì 3.
3. **Không dùng `LunarTime.getNineStar`.** Hàm đó thiếu nhánh `DONG_ZHI`, nên từ Đông Chí đến 31/12 hằng năm nó ra Nghịch. Nó lệch `Lunar.getTimeNineStar` ở 2.045 ngày trong 1900–2100.

### 4.7. Đường bay Lạc Thư

Nguồn không có bàn Cửu Cung; `NineStarUtil.POSITION` chỉ cho cung gốc của từng sao.

**Thứ tự đường bay** theo số Lạc Thư của cung: 5 → 6 → 7 → 8 → 9 → 1 → 2 → 3 → 4, tức Trung Cung → Càn → Đoài → Cấn → Ly → Khảm → Khôn → Chấn → Tốn.

**Công thức.** Với `c` là sao nhập Trung Cung, `p` là số Lạc Thư của cung:

```text
Bay thuận (forward = true, +1):   sao tại cung p = sao(c + (p - 5))
Bay nghịch (forward = false, -1): sao tại cung p = sao(c - (p - 5))
```

Tức là đi dọc đường bay, mỗi cung sao tăng 1 khi bay thuận, giảm 1 khi bay nghịch.

**Bố cục 3×3**, Nam ở trên:

| Tốn (Đông Nam) | Ly (Nam) | Khôn (Tây Nam) |
| :-- | :-- | :-- |
| **Chấn (Đông)** | **Trung Cung** | **Đoài (Tây)** |
| **Cấn (Đông Bắc)** | **Khảm (Bắc)** | **Càn (Tây Bắc)** |

**Ví dụ.** Các dòng dưới đây viết theo bố cục trên, từ trái sang phải, từ trên xuống dưới:

| Trường hợp | Hàng trên | Hàng giữa | Hàng dưới |
| :-- | :-- | :-- | :-- |
| `c` = 5, bay thuận (Lạc Thư gốc) | 4 9 2 | 3 5 7 | 8 1 6 |
| `c` = 5, bay nghịch | 6 1 8 | 7 5 3 | 2 9 4 |
| `c` = 3, bay thuận (Niên Tinh 2024) | 2 7 9 | 1 3 5 | 6 8 4 |
| `c` = 3, bay nghịch | 4 8 6 | 5 3 1 | 9 7 2 |

**Chiều bay của từng bàn (F3).** Hàm lập bàn là `generate_loshu_grid(center_star: NineStar, forward: bool) -> LoShuGrid`:

| Bàn | `forward` |
| :-- | :-- |
| Năm, Tháng | `true` |
| Ngày | Chiều của nhánh Nhật Tinh ở mục 4.5 |
| Giờ | Chiều Thời Tinh ở mục 4.6 |
| Sơn Tinh, Hướng Tinh | Nơi gọi tự truyền; quy tắc chọn chiều ngoài phạm vi tài liệu này |

`NineStars` trả sao Ngày, sao Giờ kèm chiều bay trong `FlyingStar`. UniFFI không có mảng cố định, nên `LoShuGrid` có 9 trường theo tên cung; `star_at` đọc theo cung, `rows` đọc theo bố cục 3×3.

## 5. Vector kiểm thử

Can Chi, ngày âm, Nguyên, Vận và bốn Cửu Tinh chạy trực tiếp từ `.ref/lunar-ts`. Nguồn không có Mệnh Quái, số nhánh và bàn Cửu Cung, nên các giá trị đó tính theo mục 2, 4.5 và 4.7. Cả ba vector nằm xa các ngày giao tiết bị lệch múi giờ ở mục 0.3, và `astro-core` ở UTC+7 cho cùng JDN, ngày âm, Can Chi ngày, Can Chi năm âm, Chi tháng. Các vector **chưa đối chiếu với lịch in**.

**Đầu vào và khóa tra cứu:**

| # | Dương lịch | JDN | Âm lịch | Năm âm | Năm Lập Xuân | Chi tháng | Can Chi ngày | Chi giờ | Mục đích |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 1 | 20/05/1990 10:30 | 2448032 | 26/4/1990 | Canh Ngọ | Canh Ngọ | Tỵ | Ất Dậu | Tỵ | Ca thường, Nữ gặp quy tắc số 5 |
| 2 | 25/06/2000 23:30 | 2451721 | 24/5/2000 | Canh Thìn | Canh Thìn | Ngọ | Giáp Dần | Tý | Giờ Tý Sớm; đã qua Hạ Chí nhưng chưa tới mốc Giáp Tý |
| 3 | 02/02/2022 08:00 | 2459613 | 2/1/2022 | Nhâm Dần | Tân Sửu | Sửu | Bính Tuất | Thìn | Sau Tết, trước Lập Xuân; Nam gặp quy tắc số 5 |

**Kết quả mong đợi:**

| # | Mệnh Quái Nam | Mệnh Quái Nữ | Nguyên, Vận | Niên Tinh | Nguyệt Tinh | Nhật Tinh | Thời Tinh |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 1 | Khảm (1) | `k` = 5, thành Cấn (8) | Hạ Nguyên, Vận 7 | 1 | 5 | 7 (nhánh 1, Thuận) | 6 (Thuận) |
| 2 | Ly (9) | Càn (6) | Hạ Nguyên, Vận 7 | 9 | 1 | 9 (nhánh 1, Thuận) | 3 (Nghịch) |
| 3 | `k` = 5, thành Khôn (2) | Khảm (1) | Hạ Nguyên, Vận 8 | 6 | 3 | 5 (nhánh 1, Thuận) | 8 (Thuận) |

Nguyên và Vận của ba vector giống nhau dù lấy theo năm âm hay năm Lập Xuân.

**Bàn Niên Tinh bay thuận của ba vector**, viết theo bố cục mục 4.7:

| # | `c` | Hàng trên | Hàng giữa | Hàng dưới |
| :-- | :-- | :-- | :-- | :-- |
| 1 | 1 | 9 5 7 | 8 1 3 | 4 6 2 |
| 2 | 9 | 8 4 6 | 7 9 2 | 3 5 1 |
| 3 | 6 | 5 1 3 | 4 6 8 | 9 2 7 |

Thêm 3 mốc Niên Tinh từ `NineStarTest.ts` của nguồn: dương lịch 19/02/1985 là 6, âm lịch 1/1/2022 là 6, âm lịch 1/1/2033 là 4.

**Phạm vi đã quét.** So `fengshui.rs` thật với `lunar-ts` ở `tz_offset = 8.0`, trên mọi ngày của dải:

| Đại lượng | Dải | Lấy mẫu mỗi ngày | Sai lệch |
| :-- | :-- | :-- | :-- |
| Nhật Tinh | 1600–2400 | 3 giờ | 0 |
| Thời Tinh | 1600–2400 | 3 giờ; 13 giờ trong 01/02/1900–01/12/2100 | 0 |
| Niên Tinh, Nguyệt Tinh, `LapXuanDay` | 01/02/1900–01/12/2100 | 13 giờ | 0 |
| Niên Tinh, `LapXuanDay` | Ngoài dải trên, tới 1600 và 2400 | 3 giờ | 1 ngày: 03/02/1741 |
| Nguyệt Tinh, `LapXuanDay` | Ngoài dải trên, tới 1600 và 2400 | 3 giờ | 5 ngày: 04/04/1654, 07/07/1718, 03/02/1741, 07/09/2233, 05/06/2295 |
| Nguyên, Vận | 1600–2400 | 1 | 0 |

Năm ngày lệch đều do một Tiết rơi trong vòng một phút quanh nửa đêm và hai engine đặt khác ngày, không do logic Cửu Tinh. Khi cho logic của `fengshui.rs` dùng bảng tiết khí và ngày Tết của `lunar-ts`, cả 8 đại lượng khớp nguồn trên mọi ngày 1600–2400 và tại mọi Tiết.

Hai ranh giới còn lại có thêm khác biệt do đầu vào, không do logic Cửu Tinh:

- `LapXuanExact`: khớp nguồn từ 45 giây trở lên quanh mỗi Tiết; trong khoảng lệch giữa hai engine, tối đa 38 giây trong 1900–2100, kết quả có thể khác.
- `LunarNewYear`: đi theo `solar_to_lunar`, nên khác nguồn ở những ngày hai lịch âm khác nhau. Trong 73.353 ngày, ở UTC+8 chỉ khác ngày 03/02/1916: nguồn đặt Tết 1916 vào 03/02, `solar_to_lunar` vào 04/02. Trước 1929, lịch âm của nguồn dựa vào bảng lịch sử.

**Mã băm trong test.** Test của `fengshui.rs` khóa kết quả nguồn bằng FNV-1a 64 bit (khởi đầu `0xcbf29ce484222325`, hệ số `0x100000001b3`), mỗi số sao 1–9 là một bước, theo thứ tự thời gian. Muốn tạo lại, dùng lunar-typescript 1.8.6 với `Solar.fromYmdHms(y, m, d, h, mi, 0).getLunar()`:

| Test | Lấy mẫu | Hàm nguồn |
| :-- | :-- | :-- |
| `day_star_matches_the_source_on_every_day` | Mỗi ngày 01/02/1900–01/12/2100, lúc 12:00 | `getDayNineStar()` |
| `hour_star_matches_the_source_on_every_day` | Cùng các ngày đó, lúc `h:30` với `h` là 0, 1, 3, 5, …, 21 | `getTimeNineStar()` |
| `hour_star_matches_the_source_around_every_solstice` | Năm 1901–2099, Hạ Chí rồi Đông Chí, ngày trước rồi ngày đó, lúc 12:00 | `getTimeNineStar()` |
| `year_and_month_stars_match_the_source_around_every_tiet` | Năm 1901–2099, 12 Tiết theo thứ tự `SolarTerm::ALL` từ Lập Xuân tới Tiểu Hàn, ngày trước rồi ngày đó, lúc 12:00 | `getYearNineStar()` rồi `getMonthNineStar()` |

Với `LapXuanDay`, Niên Tinh và Nguyệt Tinh chỉ đổi vào ngày giao Tiết, nên phép khóa ở dòng cuối phủ được toàn dải.

## 6. Quyết định và vấn đề còn mở

### 6.1. Đã chốt

| Vấn đề | Quyết định | Căn cứ |
| :-- | :-- | :-- |
| Năm tính Mệnh Quái | Năm âm lịch | Task 4A.1 |
| Niên Tinh, Nguyệt Tinh | Mặc định `sect = 2`: Niên Tinh đổi tại ngày Lập Xuân, Nguyệt Tinh đổi tại ngày mỗi Tiết | Mặc định của nguồn |
| Nguồn Niên Tinh | `Lunar.getYearNineStar`, không dùng `LunarYear.getNineStar` | Hàm sau có lỗi ngoặc |
| Nguồn Nguyệt Tinh | `Lunar.getMonthNineStar`, không dùng `LunarMonth.getNineStar` | Task 4A.1 quy định theo Chi tháng |
| Nguồn Thời Tinh | `Lunar.getTimeNineStar`, không dùng `LunarTime.getNineStar` | Hàm sau thiếu nhánh `DONG_ZHI` |
| Giờ Tý Sớm với Thời Tinh | Giữ Chi ngày dân dụng | `getDayZhiIndex` của nguồn |
| Giờ mặt trời thực | Chỉ dùng cho Nhật Tinh, Thời Tinh; các sao còn lại theo giờ đồng hồ | Tiền lệ `four_pillars` |
| Thời khắc của `LapXuanExact` | So không làm tròn. Nguồn so đến giây, nhưng hai engine đã lệch nhau tới 38 giây | Tiền lệ `four_pillars` |
| Múi giờ | Xét Chí và Tiết theo `tz_offset`, chấp nhận khác nguồn ở các ngày liệt kê tại mục 0.3 | `CLAUDE.md`, `BAZI_ALMANAC_SPEC.md` mục 0.3 |
| F1: Ma trận Bát Du Niên | Đóng: mục 3.2 khớp 64/64 ô với bảng chuẩn *Bát Trạch Minh Cảnh* | Task 4A.2 |
| F2: Trạch Quái | Tính theo Tọa; có hàm suy từ hướng nhà bằng `opposite` | Task 4A.2 |
| F3: Chiều bay | `generate_loshu_grid(center_star, forward)`: Năm, Tháng `true`; Ngày, Giờ theo chiều của chuỗi sao | Task 4A.2 |
| F4: Vận | Cùng khóa năm với Niên Tinh, mặc định Lập Xuân theo ngày | Task 4A.2 |
| F5: Đường nối Nhật Tinh | Giữ đúng `Lunar.getDayNineStar`, ghi trong rustdoc | Task 4A.2 |

### 6.2. Còn mở

F1–F5 đã đóng ở Task 4A.2, không còn câu hỏi nào chặn code. Việc còn lại không chặn code: đối chiếu các vector mục 5 với lịch in.

Ngoài phạm vi tài liệu này: bàn Huyền Không theo Vận (Sơn Tinh, Hướng Tinh), và các tên sao theo Bắc Đẩu, Kỳ Môn, Thái Ất trong `NineStar.ts`.
