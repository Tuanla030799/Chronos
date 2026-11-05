# Đặc tả Bát Tự & Trạch Nhật (BAZI_ALMANAC_SPEC)

Tài liệu đầu vào để viết code Rust cho phần Bát Tự và Trạch Nhật trong `astro-core`. Mọi bảng ở đây là dữ liệu tra cứu; code chỉ chuyển các bảng này thành `const` hoặc `match`, không thêm quy tắc mới.

## 0. Nguồn và quy ước

### 0.1. Nguồn dữ liệu

| Phần | Nguồn | Vai trò |
| :-- | :-- | :-- |
| A, B, C | `.ref/lunar-ts/src/lib/LunarUtil.ts` (lunar-typescript 1.8.6 của 6tail, giấy phép MIT), cùng logic tra trong `Lunar.ts`, `LunarTime.ts`, `EightChar.ts` | Dữ liệu Bát Tự và Trạch Nhật chung |
| D | `docs/vietnam_custom_rules.md` | Quy tắc dân gian Việt Nam, giữ nguyên, không sửa |
| Mục 2.5, Lục Phá ở mục 1.3 | Bảng do chủ dự án cung cấp | Thần Sát Mệnh Lý của lá số, chuẩn bất biến |

Khi hai nguồn mâu thuẫn, **Phần D được ưu tiên**. Mâu thuẫn được ghi ở mục 6, không sửa ngầm. Thông báo bản quyền MIT của dữ liệu 6tail nằm ở `THIRD_PARTY_NOTICES.md`.

### 0.2. Chỉ số và kiểu Rust

- Chỉ số bắt đầu từ 0, trùng `Stem::index()` và `Branch::index()` trong `astro-core`: Giáp = 0, Tý = 0. Mảng `GAN`, `ZHI` của nguồn bắt đầu từ 1 (phần tử 0 rỗng), đã quy đổi.
- Cột **Rust** là tên biến thể đề xuất cho `enum`.
- Cột **Hán** giữ nguyên chữ giản thể trong nguồn để đối chiếu. Code hiển thị dùng cột Hán-Việt.

### 0.3. Khóa tra cứu

Bảng nào dùng khóa nào là khác nhau, nhầm khóa là sai kết quả:

| Khóa | Định nghĩa | Dùng cho |
| :-- | :-- | :-- |
| **Chi tháng tiết khí theo ngày** | Chi của tháng tính theo Tiết: tháng Dần mở từ Lập Xuân, tháng Mão từ Kinh Trập… Tính theo **ngày**: cả ngày giao tiết đã thuộc tháng mới (`monthZhiIndex` của nguồn) | 12 Trực, Hoàng/Hắc Đạo của ngày, 15 Thần Sát |
| **Chi tháng tiết khí theo thời khắc** | Như trên nhưng so đúng thời khắc giao tiết (`monthZhiIndexExact`). `astro-core` đã làm vậy trong `four_pillars` | Trụ tháng, Trường Sinh của trụ tháng |
| **Tháng âm lịch** | `LunarDate.month`, 1–12 | Toàn bộ Phần D |
| **Chi ngày** | `(JDN + 1) mod 12`, theo ngày dân dụng | 12 Trực, 28 Sao, Thần Sát, Phần D |
| **Chi ngày sau 23:00** | Từ 23:00 lấy Chi của ngày hôm sau (`getDayZhiExact`) | Hoàng/Hắc Đạo của giờ |

Nguồn tính thời khắc giao tiết theo giờ Bắc Kinh (`ShouXingUtil` cộng `ONE_THIRD`, tức 8 giờ). Dự án này dùng `tz_offset`, mặc định UTC+7, theo `CLAUDE.md`. Vì vậy khi Tiết rơi vào 00:00–00:59 giờ Bắc Kinh, tức 23:00–23:59 hôm trước ở Việt Nam, ngày đổi tháng ở Việt Nam sớm hơn nguồn một ngày. Đây là khác biệt có chủ ý. Để lấy Chi tháng theo ngày, xét Tiết tại nửa đêm cuối ngày địa phương, giống cách `DayInfo.solar_term` đang làm.

Quy ước giờ Tý: dự án gọi 23:00–23:59 là **giờ Tý Sớm** (sách Hán gọi 晚子時, 夜子時) và 00:00–00:59 là giờ Tý Muộn. Từ 23:00, **Can giờ luôn tính theo ngày hôm sau** (`_computeTime` dùng `dayGanIndexExact`). Trụ ngày thì tùy `sect`: mặc định của dự án là `RatHourConvention::SplitAtMidnight` (giữ Can Chi ngày hôm đó, ứng với `sect = 2`, cũng là mặc định của `EightChar`); `NewDayAtRatHour` sang ngày mới, ứng với `sect = 1`.

## 1. Phần A: Bảng nền

### 1.1. Thiên Can

| Chỉ số | Hán | Hán-Việt | Rust | Ngũ Hành | Âm Dương |
| :-- | :-- | :-- | :-- | :-- | :-- |
| 0 | 甲 | Giáp | `Giap` | Mộc | Dương |
| 1 | 乙 | Ất | `At` | Mộc | Âm |
| 2 | 丙 | Bính | `Binh` | Hỏa | Dương |
| 3 | 丁 | Đinh | `Dinh` | Hỏa | Âm |
| 4 | 戊 | Mậu | `Mau` | Thổ | Dương |
| 5 | 己 | Kỷ | `Ky` | Thổ | Âm |
| 6 | 庚 | Canh | `Canh` | Kim | Dương |
| 7 | 辛 | Tân | `Tan` | Kim | Âm |
| 8 | 壬 | Nhâm | `Nham` | Thủy | Dương |
| 9 | 癸 | Quý | `Quy` | Thủy | Âm |

### 1.2. Địa Chi

| Chỉ số | Hán | Hán-Việt | Rust | Ngũ Hành |
| :-- | :-- | :-- | :-- | :-- |
| 0 | 子 | Tý | `Ty` | Thủy |
| 1 | 丑 | Sửu | `Suu` | Thổ |
| 2 | 寅 | Dần | `Dan` | Mộc |
| 3 | 卯 | Mão | `Mao` | Mộc |
| 4 | 辰 | Thìn | `Thin` | Thổ |
| 5 | 巳 | Tỵ | `Ti` | Hỏa |
| 6 | 午 | Ngọ | `Ngo` | Hỏa |
| 7 | 未 | Mùi | `Mui` | Thổ |
| 8 | 申 | Thân | `Than` | Kim |
| 9 | 酉 | Dậu | `Dau` | Kim |
| 10 | 戌 | Tuất | `Tuat` | Thổ |
| 11 | 亥 | Hợi | `Hoi` | Thủy |

Lưu ý trong Rust: `Ty` là Tý, `Ti` là Tỵ.

### 1.3. Quan hệ Can Chi

| Quan hệ | Các cặp | Nguồn |
| :-- | :-- | :-- |
| Ngũ Hợp Can | Giáp–Kỷ, Ất–Canh, Bính–Tân, Đinh–Nhâm, Mậu–Quý | `HE_GAN_5` |
| Xung Can | Giáp–Canh, Ất–Tân, Bính–Nhâm, Đinh–Quý; Mậu, Kỷ không có | `CHONG_GAN_4` |
| Lục Hợp | Tý–Sửu, Dần–Hợi, Mão–Tuất, Thìn–Dậu, Tỵ–Thân, Ngọ–Mùi | `HE_ZHI_6` |
| Lục Xung | Tý–Ngọ, Sửu–Mùi, Dần–Thân, Mão–Dậu, Thìn–Tuất, Tỵ–Hợi | `CHONG` |
| Tam Hợp | Thân–Tý–Thìn, Dần–Ngọ–Tuất, Tỵ–Dậu–Sửu, Hợi–Mão–Mùi | Mục 2.5 B và 4.5 |
| Lục Hại | Tý–Mùi, Sửu–Ngọ, Dần–Tỵ, Mão–Thìn, Thân–Hợi, Dậu–Tuất | Giải mã từ `月害` trong `DAY_SHEN_SHA` |
| Hình | Dần→Tỵ→Thân→Dần; Sửu→Tuất→Mùi→Sửu; Tý↔Mão; Thìn, Ngọ, Dậu, Hợi tự hình | Giải mã từ `月刑` trong `DAY_SHEN_SHA` |
| Lục Phá | Tý–Dậu, Ngọ–Mão, Thìn–Sửu, Tuất–Mùi, Dần–Hợi, Tỵ–Thân | Chủ dự án xác nhận |

## 2. Phần B: Bát Tự

### 2.1. Tàng Can

Nguồn: `ZHI_HIDE_GAN`. Nguồn chỉ lưu mảng theo thứ tự, không ghi nhãn; nhãn Bản khí, Trung khí, Dư khí được gán theo vị trí 0, 1, 2.

| Địa Chi | Bản khí | Trung khí | Dư khí |
| :-- | :-- | :-- | :-- |
| Tý | Quý | — | — |
| Sửu | Kỷ | Quý | Tân |
| Dần | Giáp | Bính | Mậu |
| Mão | Ất | — | — |
| Thìn | Mậu | Ất | Quý |
| Tỵ | Bính | Canh | Mậu |
| Ngọ | Đinh | Kỷ | — |
| Mùi | Kỷ | Đinh | Ất |
| Thân | Canh | Nhâm | Mậu |
| Dậu | Tân | — | — |
| Tuất | Mậu | Tân | Đinh |
| Hợi | Nhâm | Giáp | — |

### 2.2. 60 Hoa Giáp Nạp Âm

Nguồn: `NAYIN`. Mỗi cặp Can Chi liên tiếp chung một Nạp Âm. Chỉ số là `CanChi::cycle_index()`.

| Chỉ số | Can Chi | Hán | Nạp Âm | Ngũ Hành | Rust |
| :-- | :-- | :-- | :-- | :-- | :-- |
| 0, 1 | Giáp Tý, Ất Sửu | 海中金 | Hải Trung Kim | Kim | `HaiTrungKim` |
| 2, 3 | Bính Dần, Đinh Mão | 炉中火 | Lư Trung Hỏa | Hỏa | `LuTrungHoa` |
| 4, 5 | Mậu Thìn, Kỷ Tỵ | 大林木 | Đại Lâm Mộc | Mộc | `DaiLamMoc` |
| 6, 7 | Canh Ngọ, Tân Mùi | 路旁土 | Lộ Bàng Thổ | Thổ | `LoBangTho` |
| 8, 9 | Nhâm Thân, Quý Dậu | 剑锋金 | Kiếm Phong Kim | Kim | `KiemPhongKim` |
| 10, 11 | Giáp Tuất, Ất Hợi | 山头火 | Sơn Đầu Hỏa | Hỏa | `SonDauHoa` |
| 12, 13 | Bính Tý, Đinh Sửu | 涧下水 | Giản Hạ Thủy | Thủy | `GianHaThuy` |
| 14, 15 | Mậu Dần, Kỷ Mão | 城头土 | Thành Đầu Thổ | Thổ | `ThanhDauTho` |
| 16, 17 | Canh Thìn, Tân Tỵ | 白蜡金 | Bạch Lạp Kim | Kim | `BachLapKim` |
| 18, 19 | Nhâm Ngọ, Quý Mùi | 杨柳木 | Dương Liễu Mộc | Mộc | `DuongLieuMoc` |
| 20, 21 | Giáp Thân, Ất Dậu | 泉中水 | Tuyền Trung Thủy | Thủy | `TuyenTrungThuy` |
| 22, 23 | Bính Tuất, Đinh Hợi | 屋上土 | Ốc Thượng Thổ | Thổ | `OcThuongTho` |
| 24, 25 | Mậu Tý, Kỷ Sửu | 霹雳火 | Tích Lịch Hỏa | Hỏa | `TichLichHoa` |
| 26, 27 | Canh Dần, Tân Mão | 松柏木 | Tùng Bách Mộc | Mộc | `TungBachMoc` |
| 28, 29 | Nhâm Thìn, Quý Tỵ | 长流水 | Trường Lưu Thủy | Thủy | `TruongLuuThuy` |
| 30, 31 | Giáp Ngọ, Ất Mùi | 沙中金 | Sa Trung Kim | Kim | `SaTrungKim` |
| 32, 33 | Bính Thân, Đinh Dậu | 山下火 | Sơn Hạ Hỏa | Hỏa | `SonHaHoa` |
| 34, 35 | Mậu Tuất, Kỷ Hợi | 平地木 | Bình Địa Mộc | Mộc | `BinhDiaMoc` |
| 36, 37 | Canh Tý, Tân Sửu | 壁上土 | Bích Thượng Thổ | Thổ | `BichThuongTho` |
| 38, 39 | Nhâm Dần, Quý Mão | 金箔金 | Kim Bạch Kim | Kim | `KimBachKim` |
| 40, 41 | Giáp Thìn, Ất Tỵ | 覆灯火 | Phú Đăng Hỏa | Hỏa | `PhuDangHoa` |
| 42, 43 | Bính Ngọ, Đinh Mùi | 天河水 | Thiên Hà Thủy | Thủy | `ThienHaThuy` |
| 44, 45 | Mậu Thân, Kỷ Dậu | 大驿土 | Đại Dịch Thổ | Thổ | `DaiDichTho` |
| 46, 47 | Canh Tuất, Tân Hợi | 钗钏金 | Thoa Xuyến Kim | Kim | `ThoaXuyenKim` |
| 48, 49 | Nhâm Tý, Quý Sửu | 桑柘木 | Tang Đố Mộc | Mộc | `TangDoMoc` |
| 50, 51 | Giáp Dần, Ất Mão | 大溪水 | Đại Khê Thủy | Thủy | `DaiKheThuy` |
| 52, 53 | Bính Thìn, Đinh Tỵ | 沙中土 | Sa Trung Thổ | Thổ | `SaTrungTho` |
| 54, 55 | Mậu Ngọ, Kỷ Mùi | 天上火 | Thiên Thượng Hỏa | Hỏa | `ThienThuongHoa` |
| 56, 57 | Canh Thân, Tân Dậu | 石榴木 | Thạch Lựu Mộc | Mộc | `ThachLuuMoc` |
| 58, 59 | Nhâm Tuất, Quý Hợi | 大海水 | Đại Hải Thủy | Thủy | `DaiHaiThuy` |

Tên trong lịch Việt Nam có vài cách đọc. Bảng trên dùng cách phổ biến; `覆灯火` còn đọc Phúc Đăng Hỏa, `炉中火` còn đọc Lô Trung Hỏa, `金箔金` đọc sát chữ là Kim Bạc Kim, `桑柘木` còn đọc Tang Chá Mộc, `大驿土` nhiều lịch Việt ghi Đại Trạch Thổ.

### 2.3. Vòng Trường Sinh

Nguồn: `CHANG_SHENG`, `CHANG_SHENG_OFFSET`, `EightChar.getDiShi`. Thứ tự vòng:

| Chỉ số | Hán | Hán-Việt | Rust |
| :-- | :-- | :-- | :-- |
| 0 | 长生 | Trường Sinh | `TruongSinh` |
| 1 | 沐浴 | Mộc Dục | `MocDuc` |
| 2 | 冠带 | Quan Đới | `QuanDoi` |
| 3 | 临官 | Lâm Quan | `LamQuan` |
| 4 | 帝旺 | Đế Vượng | `DeVuong` |
| 5 | 衰 | Suy | `Suy` |
| 6 | 病 | Bệnh | `Benh` |
| 7 | 死 | Tử | `Tu` |
| 8 | 墓 | Mộ | `Mo` |
| 9 | 绝 | Tuyệt | `Tuyet` |
| 10 | 胎 | Thai | `Thai` |
| 11 | 养 | Dưỡng | `Duong` |

Công thức: `index = offset + (Can dương ? chi : -chi)`, rồi đưa về 0–11. Can dương thuận, Can âm nghịch. Mậu theo Bính, Kỷ theo Đinh.

| Can | offset | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Giáp | 1 | Mộc Dục | Quan Đới | Lâm Quan | Đế Vượng | Suy | Bệnh | Tử | Mộ | Tuyệt | Thai | Dưỡng | Trường Sinh |
| Ất | 6 | Bệnh | Suy | Đế Vượng | Lâm Quan | Quan Đới | Mộc Dục | Trường Sinh | Dưỡng | Thai | Tuyệt | Mộ | Tử |
| Bính | 10 | Thai | Dưỡng | Trường Sinh | Mộc Dục | Quan Đới | Lâm Quan | Đế Vượng | Suy | Bệnh | Tử | Mộ | Tuyệt |
| Đinh | 9 | Tuyệt | Mộ | Tử | Bệnh | Suy | Đế Vượng | Lâm Quan | Quan Đới | Mộc Dục | Trường Sinh | Dưỡng | Thai |
| Mậu | 10 | Thai | Dưỡng | Trường Sinh | Mộc Dục | Quan Đới | Lâm Quan | Đế Vượng | Suy | Bệnh | Tử | Mộ | Tuyệt |
| Kỷ | 9 | Tuyệt | Mộ | Tử | Bệnh | Suy | Đế Vượng | Lâm Quan | Quan Đới | Mộc Dục | Trường Sinh | Dưỡng | Thai |
| Canh | 7 | Tử | Mộ | Tuyệt | Thai | Dưỡng | Trường Sinh | Mộc Dục | Quan Đới | Lâm Quan | Đế Vượng | Suy | Bệnh |
| Tân | 0 | Trường Sinh | Dưỡng | Thai | Tuyệt | Mộ | Tử | Bệnh | Suy | Đế Vượng | Lâm Quan | Quan Đới | Mộc Dục |
| Nhâm | 4 | Đế Vượng | Suy | Bệnh | Tử | Mộ | Tuyệt | Thai | Dưỡng | Trường Sinh | Mộc Dục | Quan Đới | Lâm Quan |
| Quý | 3 | Lâm Quan | Quan Đới | Mộc Dục | Trường Sinh | Dưỡng | Thai | Tuyệt | Mộ | Tử | Bệnh | Suy | Đế Vượng |

### 2.4. Thập Thần

Nguồn: `SHI_SHEN`, khóa là Can ngày ghép Can kia. Với Tàng Can, tra từng Can tàng (`EightChar.getYearShiShenZhi` và các hàm cùng loại). Trụ ngày của chính Can ngày ghi là Nhật Chủ.

| Hán | Hán-Việt | Rust |
| :-- | :-- | :-- |
| 比肩 | Tỷ Kiên | `TyKien` |
| 劫财 | Kiếp Tài | `KiepTai` |
| 食神 | Thực Thần | `ThucThan` |
| 伤官 | Thương Quan | `ThuongQuan` |
| 偏财 | Thiên Tài | `ThienTai` |
| 正财 | Chính Tài | `ChinhTai` |
| 七杀 | Thất Sát | `ThatSat` |
| 正官 | Chính Quan | `ChinhQuan` |
| 偏印 | Thiên Ấn | `ThienAn` |
| 正印 | Chính Ấn | `ChinhAn` |

Hàng là Can ngày (Nhật Chủ), cột là Can được xét:

| Can ngày | Giáp | Ất | Bính | Đinh | Mậu | Kỷ | Canh | Tân | Nhâm | Quý |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Giáp | Tỷ Kiên | Kiếp Tài | Thực Thần | Thương Quan | Thiên Tài | Chính Tài | Thất Sát | Chính Quan | Thiên Ấn | Chính Ấn |
| Ất | Kiếp Tài | Tỷ Kiên | Thương Quan | Thực Thần | Chính Tài | Thiên Tài | Chính Quan | Thất Sát | Chính Ấn | Thiên Ấn |
| Bính | Thiên Ấn | Chính Ấn | Tỷ Kiên | Kiếp Tài | Thực Thần | Thương Quan | Thiên Tài | Chính Tài | Thất Sát | Chính Quan |
| Đinh | Chính Ấn | Thiên Ấn | Kiếp Tài | Tỷ Kiên | Thương Quan | Thực Thần | Chính Tài | Thiên Tài | Chính Quan | Thất Sát |
| Mậu | Thất Sát | Chính Quan | Thiên Ấn | Chính Ấn | Tỷ Kiên | Kiếp Tài | Thực Thần | Thương Quan | Thiên Tài | Chính Tài |
| Kỷ | Chính Quan | Thất Sát | Chính Ấn | Thiên Ấn | Kiếp Tài | Tỷ Kiên | Thương Quan | Thực Thần | Chính Tài | Thiên Tài |
| Canh | Thiên Tài | Chính Tài | Thất Sát | Chính Quan | Thiên Ấn | Chính Ấn | Tỷ Kiên | Kiếp Tài | Thực Thần | Thương Quan |
| Tân | Chính Tài | Thiên Tài | Chính Quan | Thất Sát | Chính Ấn | Thiên Ấn | Kiếp Tài | Tỷ Kiên | Thương Quan | Thực Thần |
| Nhâm | Thực Thần | Thương Quan | Thiên Tài | Chính Tài | Thất Sát | Chính Quan | Thiên Ấn | Chính Ấn | Tỷ Kiên | Kiếp Tài |
| Quý | Thương Quan | Thực Thần | Chính Tài | Thiên Tài | Chính Quan | Thất Sát | Chính Ấn | Thiên Ấn | Kiếp Tài | Tỷ Kiên |

### 2.5. Thần Sát Mệnh Lý (Bazi Shen Sha)

Bảng do chủ dự án cung cấp, là chuẩn bất biến, không suy diễn ngoài bảng này. Mục này dùng cho lá số Bát Tự, khác với 15 Thần Sát Trạch Nhật ở mục 3.4.

#### A. Các sao tra theo Thiên Can (Xét Can Ngày - Nhật Chủ, hoặc Can Năm)

| Thiên Can | Thiên Ất Quý Nhân | Văn Xương | Lộc Tồn | Kình Dương | Đà La |
| :---: | :---: | :---: | :---: | :---: | :---: |
| **Giáp** | Sửu, Mùi | Tỵ | Dần | Mão | Sửu |
| **Ất** | Tý, Thân | Ngọ | Mão | Thìn | Dần |
| **Bính** | Hợi, Dậu | Thân | Tỵ | Ngọ | Thìn |
| **Đinh** | Hợi, Dậu | Dậu | Ngọ | Mùi | Tỵ |
| **Mậu** | Sửu, Mùi | Thân | Tỵ | Ngọ | Thìn |
| **Kỷ** | Tý, Thân | Dậu | Ngọ | Mùi | Tỵ |
| **Canh** | Sửu, Mùi | Hợi | Thân | Dậu | Mùi |
| **Tân** | Dần, Ngọ | Tý | Dậu | Tuất | Thân |
| **Nhâm** | Mão, Tỵ | Dần | Hợi | Tý | Tuất |
| **Quý** | Mão, Tỵ | Mão | Tý | Sửu | Hợi |

#### B. Các sao tra theo Tam Hợp Địa Chi (Xét Chi Năm hoặc Chi Ngày)

| Nhóm Tam Hợp Chi | Dịch Mã | Đào Hoa (Hàm Trì) | Hoa Cái | Tướng Tinh | Kiếp Sát |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Thân, Tý, Thìn** | Dần | Dậu | Thìn | Tý | Tỵ |
| **Dần, Ngọ, Tuất** | Thân | Mão | Tuất | Ngọ | Hợi |
| **Tỵ, Dậu, Sửu** | Hợi | Ngọ | Sửu | Dậu | Dần |
| **Hợi, Mão, Mùi** | Tỵ | Tý | Mùi | Mão | Thân |

#### C. Tuần Không (Không Vong) và Khôi Canh

- **Không Vong (2 Địa Chi trống trong mỗi Tuần Giáp)**: Tính từ trụ Ngày hoặc trụ Năm, tìm chỉ số `diff = (branch_index as i8 - stem_index as i8).rem_euclid(12)`:
  - `diff == 0` (Giáp Tý): Tuất, Hợi
  - `diff == 10` (Giáp Tuất): Thân, Dậu
  - `diff == 8` (Giáp Thân): Ngọ, Mùi
  - `diff == 6` (Giáp Ngọ): Thìn, Tỵ
  - `diff == 4` (Giáp Thìn): Dần, Mão
  - `diff == 2` (Giáp Dần): Tý, Sửu
- **Khôi Canh Quý Nhân**: Trụ Ngày thuộc 1 trong 4 trụ: `Canh Thìn`, `Canh Tuất`, `Nhâm Thìn`, `Mậu Tuất`.

Lá số báo mọi trụ có Địa Chi khớp, kèm trụ gốc đã dùng để tra (Can Ngày, Can Năm, Chi Ngày, Chi Năm). Bảng không nói có loại trừ chính trụ gốc hay không, nên code không lọc; phần hiển thị tự chọn.

### 2.6. Đại Vận

Nguồn: `Yun.ts`, `DaYun.ts`.

| Bước | Quy tắc |
| :-- | :-- |
| Chiều | Thuận khi (Can năm dương và nam) hoặc (Can năm âm và nữ); còn lại nghịch |
| Khoảng cách | Thuận: từ lúc sinh đến Tiết kế tiếp. Nghịch: từ Tiết liền trước đến lúc sinh. Chỉ xét 12 Tiết, không xét Trung khí |
| Quy đổi (`sect = 2`) | Tính số phút tròn, rồi: 4320 phút = 1 năm, 360 phút = 1 tháng, 12 phút = 1 ngày, 1 phút = 2 giờ |
| Thời điểm khởi vận | Cộng lần lượt năm, tháng, ngày, giờ vào ngày giờ sinh; ngày vượt quá độ dài tháng thì lấy ngày cuối tháng |
| Vận số 0 | Giai đoạn trước khởi vận, không có Can Chi: từ năm sinh, tuổi 1, đến hết năm trước năm khởi vận |
| Vận số n ≥ 1 | Can Chi = Trụ tháng tiến (thuận) hoặc lùi (nghịch) n bước trong 60 Hoa Giáp; kéo dài 10 năm từ `năm khởi vận + (n - 1) × 10`; tuổi đầu vận = `năm đầu vận - năm sinh + 1` |

Nguồn có hai cách quy đổi. Dự án dùng `sect = 2` vì tính theo phút; `sect = 1` (mặc định của `EightChar.getYun`) tính theo canh giờ. Khi Trụ tính ở chế độ `LunarNewYear`, Đại Vận dùng Can năm và Trụ tháng của chính lá số đó; khoảng cách vẫn tính đến Tiết thiên văn.

## 3. Phần C: Trạch Nhật

### 3.1. 12 Trực

Nguồn: `ZHI_XING`, `Lunar.getZhiXing`. Công thức: `index = (Chi ngày - Chi tháng tiết khí theo ngày) mod 12`.

| Chỉ số | Hán | Hán-Việt | Rust |
| :-- | :-- | :-- | :-- |
| 0 | 建 | Kiến | `Kien` |
| 1 | 除 | Trừ | `Tru` |
| 2 | 满 | Mãn | `Man` |
| 3 | 平 | Bình | `Binh` |
| 4 | 定 | Định | `Dinh` |
| 5 | 执 | Chấp | `Chap` |
| 6 | 破 | Phá | `Pha` |
| 7 | 危 | Nguy | `Nguy` |
| 8 | 成 | Thành | `Thanh` |
| 9 | 收 | Thu | `Thu` |
| 10 | 开 | Khai | `Khai` |
| 11 | 闭 | Bế | `Be` |

Hàng là Chi tháng, cột là Chi ngày:

| Tháng \ Ngày | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Dần | Khai | Bế | Kiến | Trừ | Mãn | Bình | Định | Chấp | Phá | Nguy | Thành | Thu |
| Mão | Thu | Khai | Bế | Kiến | Trừ | Mãn | Bình | Định | Chấp | Phá | Nguy | Thành |
| Thìn | Thành | Thu | Khai | Bế | Kiến | Trừ | Mãn | Bình | Định | Chấp | Phá | Nguy |
| Tỵ | Nguy | Thành | Thu | Khai | Bế | Kiến | Trừ | Mãn | Bình | Định | Chấp | Phá |
| Ngọ | Phá | Nguy | Thành | Thu | Khai | Bế | Kiến | Trừ | Mãn | Bình | Định | Chấp |
| Mùi | Chấp | Phá | Nguy | Thành | Thu | Khai | Bế | Kiến | Trừ | Mãn | Bình | Định |
| Thân | Định | Chấp | Phá | Nguy | Thành | Thu | Khai | Bế | Kiến | Trừ | Mãn | Bình |
| Dậu | Bình | Định | Chấp | Phá | Nguy | Thành | Thu | Khai | Bế | Kiến | Trừ | Mãn |
| Tuất | Mãn | Bình | Định | Chấp | Phá | Nguy | Thành | Thu | Khai | Bế | Kiến | Trừ |
| Hợi | Trừ | Mãn | Bình | Định | Chấp | Phá | Nguy | Thành | Thu | Khai | Bế | Kiến |
| Tý | Kiến | Trừ | Mãn | Bình | Định | Chấp | Phá | Nguy | Thành | Thu | Khai | Bế |
| Sửu | Bế | Kiến | Trừ | Mãn | Bình | Định | Chấp | Phá | Nguy | Thành | Thu | Khai |

### 3.2. 28 Sao

Nguồn: `XIU`, `XIU_LUCK`, `ZHENG`, `ANIMAL`, `GONG`, `SHOU`. Nguồn tra sao theo Chi ngày và thứ trong tuần (84 ô). Bảng đó tương đương công thức sau, đã kiểm trên toàn bộ chu kỳ:

`index = (JDN + 11) mod 28`

| Chỉ số | Hán | Sao | Tên đầy đủ | Thất Chính | Con vật (Hán) | Phương | Tứ Tượng | Cát Hung | Rust |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 0 | 角 | Giác | Giác Mộc Giao | Mộc | Giao (蛟) | Đông | Thanh Long | Cát | `GiacMocGiao` |
| 1 | 亢 | Cang | Cang Kim Long | Kim | Long (龙) | Đông | Thanh Long | Hung | `CangKimLong` |
| 2 | 氐 | Đê | Đê Thổ Lạc | Thổ | Lạc (貉) | Đông | Thanh Long | Hung | `DeThoLac` |
| 3 | 房 | Phòng | Phòng Nhật Thố | Nhật | Thố (兔) | Đông | Thanh Long | Cát | `PhongNhatTho` |
| 4 | 心 | Tâm | Tâm Nguyệt Hồ | Nguyệt | Hồ (狐) | Đông | Thanh Long | Hung | `TamNguyetHo` |
| 5 | 尾 | Vĩ | Vĩ Hỏa Hổ | Hỏa | Hổ (虎) | Đông | Thanh Long | Cát | `ViHoaHo` |
| 6 | 箕 | Cơ | Cơ Thủy Báo | Thủy | Báo (豹) | Đông | Thanh Long | Cát | `CoThuyBao` |
| 7 | 斗 | Đẩu | Đẩu Mộc Giải | Mộc | Giải (獬) | Bắc | Huyền Vũ | Cát | `DauMocGiai` |
| 8 | 牛 | Ngưu | Ngưu Kim Ngưu | Kim | Ngưu (牛) | Bắc | Huyền Vũ | Hung | `NguuKimNguu` |
| 9 | 女 | Nữ | Nữ Thổ Bức | Thổ | Bức (蝠) | Bắc | Huyền Vũ | Hung | `NuThoBuc` |
| 10 | 虚 | Hư | Hư Nhật Thử | Nhật | Thử (鼠) | Bắc | Huyền Vũ | Hung | `HuNhatThu` |
| 11 | 危 | Nguy | Nguy Nguyệt Yến | Nguyệt | Yến (燕) | Bắc | Huyền Vũ | Hung | `NguyNguyetYen` |
| 12 | 室 | Thất | Thất Hỏa Trư | Hỏa | Trư (猪) | Bắc | Huyền Vũ | Cát | `ThatHoaTru` |
| 13 | 壁 | Bích | Bích Thủy Du | Thủy | Du (獝) | Bắc | Huyền Vũ | Cát | `BichThuyDu` |
| 14 | 奎 | Khuê | Khuê Mộc Lang | Mộc | Lang (狼) | Tây | Bạch Hổ | Hung | `KhueMocLang` |
| 15 | 娄 | Lâu | Lâu Kim Cẩu | Kim | Cẩu (狗) | Tây | Bạch Hổ | Cát | `LauKimCau` |
| 16 | 胃 | Vị | Vị Thổ Trĩ | Thổ | Trĩ (彘) | Tây | Bạch Hổ | Cát | `ViThoTri` |
| 17 | 昴 | Mão | Mão Nhật Kê | Nhật | Kê (鸡) | Tây | Bạch Hổ | Hung | `MaoNhatKe` |
| 18 | 毕 | Tất | Tất Nguyệt Ô | Nguyệt | Ô (乌) | Tây | Bạch Hổ | Cát | `TatNguyetO` |
| 19 | 觜 | Chủy | Chủy Hỏa Hầu | Hỏa | Hầu (猴) | Tây | Bạch Hổ | Hung | `ChuyHoaHau` |
| 20 | 参 | Sâm | Sâm Thủy Viên | Thủy | Viên (猿) | Tây | Bạch Hổ | Cát | `SamThuyVien` |
| 21 | 井 | Tỉnh | Tỉnh Mộc Hãn | Mộc | Hãn (犴) | Nam | Chu Tước | Cát | `TinhMocHan` |
| 22 | 鬼 | Quỷ | Quỷ Kim Dương | Kim | Dương (羊) | Nam | Chu Tước | Hung | `QuyKimDuong` |
| 23 | 柳 | Liễu | Liễu Thổ Chương | Thổ | Chương (獐) | Nam | Chu Tước | Hung | `LieuThoChuong` |
| 24 | 星 | Tinh | Tinh Nhật Mã | Nhật | Mã (马) | Nam | Chu Tước | Hung | `TinhNhatMa` |
| 25 | 张 | Trương | Trương Nguyệt Lộc | Nguyệt | Lộc (鹿) | Nam | Chu Tước | Cát | `TruongNguyetLoc` |
| 26 | 翼 | Dực | Dực Hỏa Xà | Hỏa | Xà (蛇) | Nam | Chu Tước | Hung | `DucHoaXa` |
| 27 | 轸 | Chẩn | Chẩn Thủy Dẫn | Thủy | Dẫn (蚓) | Nam | Chu Tước | Cát | `ChanThuyDan` |

Tên `enum` dùng tên đầy đủ vì Vĩ (尾) và Vị (胃) trùng nhau khi bỏ dấu. Hai con vật có chữ trong nguồn khác với tên quen dùng ở Việt Nam, xem mục 6.

### 3.3. 12 Thần Hoàng Đạo, Hắc Đạo

Nguồn: `TIAN_SHEN`, `ZHI_TIAN_SHEN_OFFSET`, `TIAN_SHEN_TYPE`, `TIAN_SHEN_TYPE_LUCK`.

| Chỉ số | Hán | Hán-Việt | Loại | Cát Hung | Rust |
| :-- | :-- | :-- | :-- | :-- | :-- |
| 0 | 青龙 | Thanh Long | Hoàng Đạo | Cát | `ThanhLong` |
| 1 | 明堂 | Minh Đường | Hoàng Đạo | Cát | `MinhDuong` |
| 2 | 天刑 | Thiên Hình | Hắc Đạo | Hung | `ThienHinh` |
| 3 | 朱雀 | Chu Tước | Hắc Đạo | Hung | `ChuTuoc` |
| 4 | 金匮 | Kim Quỹ | Hoàng Đạo | Cát | `KimQuy` |
| 5 | 天德 | Thiên Đức | Hoàng Đạo | Cát | `ThienDuc` |
| 6 | 白虎 | Bạch Hổ | Hắc Đạo | Hung | `BachHo` |
| 7 | 玉堂 | Ngọc Đường | Hoàng Đạo | Cát | `NgocDuong` |
| 8 | 天牢 | Thiên Lao | Hắc Đạo | Hung | `ThienLao` |
| 9 | 玄武 | Huyền Vũ | Hắc Đạo | Hung | `HuyenVu` |
| 10 | 司命 | Tư Mệnh | Hoàng Đạo | Cát | `TuMenh` |
| 11 | 勾陈 | Câu Trần | Hắc Đạo | Hung | `CauTran` |

Công thức: `index = (Chi đích + offset[Chi khóa]) mod 12`. Một bảng dùng cho hai việc:

- **Giờ:** Chi khóa là Chi ngày sau 23:00, Chi đích là Chi giờ (`LunarTime.getTianShen`).
- **Ngày:** Chi khóa là Chi tháng tiết khí theo ngày, Chi đích là Chi ngày (`Lunar.getDayTianShen`).

Hàng là Chi khóa, cột là Chi đích, chữ đậm là Hoàng Đạo:

| Khóa | offset | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Tý | 4 | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước |
| Sửu | 2 | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** |
| Dần | 0 | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần |
| Mão | 10 | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ |
| Thìn | 8 | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** |
| Tỵ | 6 | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** |
| Ngọ | 4 | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước |
| Mùi | 2 | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** |
| Thân | 0 | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần |
| Dậu | 10 | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ |
| Tuất | 8 | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** | Bạch Hổ | **Ngọc Đường** |
| Hợi | 6 | Bạch Hổ | **Ngọc Đường** | Thiên Lao | Huyền Vũ | **Tư Mệnh** | Câu Trần | **Thanh Long** | **Minh Đường** | Thiên Hình | Chu Tước | **Kim Quỹ** | **Thiên Đức** |

12 giờ Hoàng Đạo theo Chi ngày (6 giờ tốt mỗi ngày):

| Chi ngày | Giờ Hoàng Đạo |
| :-- | :-- |
| Tý | Tý, Sửu, Mão, Ngọ, Thân, Dậu |
| Sửu | Dần, Mão, Tỵ, Thân, Tuất, Hợi |
| Dần | Tý, Sửu, Thìn, Tỵ, Mùi, Tuất |
| Mão | Tý, Dần, Mão, Ngọ, Mùi, Dậu |
| Thìn | Dần, Thìn, Tỵ, Thân, Dậu, Hợi |
| Tỵ | Sửu, Thìn, Ngọ, Mùi, Tuất, Hợi |
| Ngọ | Tý, Sửu, Mão, Ngọ, Thân, Dậu |
| Mùi | Dần, Mão, Tỵ, Thân, Tuất, Hợi |
| Thân | Tý, Sửu, Thìn, Tỵ, Mùi, Tuất |
| Dậu | Tý, Dần, Mão, Ngọ, Mùi, Dậu |
| Tuất | Dần, Thìn, Tỵ, Thân, Dậu, Hợi |
| Hợi | Sửu, Thìn, Ngọ, Mùi, Tuất, Hợi |

Trong nguồn `天德` ở bảng này có vị trí trùng khớp với `宝光` (Bảo Quang) trong `SHEN_SHA`, nên lịch Việt ghi Bảo Quang cũng là thần này. `玄武` ở đây là `元武` trong `SHEN_SHA`. Đây không phải cát thần Thiên Đức ở mục 3.4.

### 3.4. 15 Thần Sát cốt lõi

Nguồn: `SHEN_SHA` (151 tên, phần tử đầu `无` nghĩa là không có) và `DAY_SHEN_SHA` (mã hex theo tháng tiết khí và Can Chi ngày, 720 ô). Nguồn **không có sẵn** danh sách 15 thần và **không ghi quy tắc**: bảng dưới được giải mã từ dữ liệu. Danh sách 15 thần đã được chủ dự án duyệt.

Cột tháng là Chi tháng tiết khí theo ngày. Ô là điều kiện của ngày: một Chi, một Can, hoặc một Can Chi cụ thể tùy cột Tiêu chí; `—` là tháng đó không có.

| Thần | Hán | Loại | Tiêu chí | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi | Tý | Sửu | Rust |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| Thiên Đức | 天德 | Cát | Can ngày | Đinh | — | Nhâm | Tân | — | Giáp | Quý | — | Bính | Ất | — | Canh | `ThienDuc` |
| Nguyệt Đức | 月德 | Cát | Can ngày | Bính | Giáp | Nhâm | Canh | Bính | Giáp | Nhâm | Canh | Bính | Giáp | Nhâm | Canh | `NguyetDuc` |
| Thiên Xá | 天赦 | Cát | Can Chi ngày | Mậu Dần | Mậu Dần | Mậu Dần | Giáp Ngọ | Giáp Ngọ | Giáp Ngọ | Mậu Thân | Mậu Thân | Mậu Thân | Giáp Tý | Giáp Tý | Giáp Tý | `ThienXa` |
| Thiên Nguyện | 天愿 | Cát | Can Chi ngày | Ất Hợi | Giáp Tuất | Ất Dậu | Bính Thân | Đinh Mùi | Mậu Ngọ | Kỷ Tỵ | Canh Thìn | Tân Mão | Nhâm Dần | Quý Sửu | Giáp Tý | `ThienNguyen` |
| Sinh Khí | 生气 | Cát | Chi ngày | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | Tuất | Hợi | `SinhKhi` |
| Thiên Hỷ | 天喜 | Cát | Chi ngày | Tuất | Hợi | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | Thân | Dậu | `ThienHy` |
| Tam Hợp | 三合 | Cát | Chi ngày | Ngọ, Tuất | Mùi, Hợi | Tý, Thân | Sửu, Dậu | Dần, Tuất | Mão, Hợi | Tý, Thìn | Sửu, Tỵ | Dần, Ngọ | Mão, Mùi | Thìn, Thân | Tỵ, Dậu | `TamHop` |
| Dịch Mã | 驿马 | Cát | Chi ngày | Thân | Tỵ | Dần | Hợi | Thân | Tỵ | Dần | Hợi | Thân | Tỵ | Dần | Hợi | `DichMa` |
| Nguyệt Phá | 月破 | Hung | Chi ngày | Thân | Dậu | Tuất | Hợi | Tý | Sửu | Dần | Mão | Thìn | Tỵ | Ngọ | Mùi | `NguyetPha` |
| Thiên Cương | 天罡 | Hung | Chi ngày | Tỵ | Tý | Mùi | Dần | Dậu | Thìn | Hợi | Ngọ | Sửu | Thân | Mão | Tuất | `ThienCuong` |
| Hà Khôi | 河魁 | Hung | Chi ngày | Hợi | Ngọ | Sửu | Thân | Mão | Tuất | Tỵ | Tý | Mùi | Dần | Dậu | Thìn | `HaKhoi` |
| Nguyệt Sát | 月煞 | Hung | Chi ngày | Sửu | Tuất | Mùi | Thìn | Sửu | Tuất | Mùi | Thìn | Sửu | Tuất | Mùi | Thìn | `NguyetSat` |
| Kiếp Sát | 劫煞 | Hung | Chi ngày | Hợi | Thân | Tỵ | Dần | Hợi | Thân | Tỵ | Dần | Hợi | Thân | Tỵ | Dần | `KiepSat` |
| Tai Sát | 灾煞 | Hung | Chi ngày | Tý | Dậu | Ngọ | Mão | Tý | Dậu | Ngọ | Mão | Tý | Dậu | Ngọ | Mão | `TaiSat` |
| Nguyệt Hình | 月刑 | Hung | Chi ngày | Tỵ | Tý | Thìn | Thân | Ngọ | Sửu | Dần | Dậu | Mùi | Hợi | Mão | Tuất | `NguyetHinh` |

Ngoại lệ trong dữ liệu nguồn (15 ô). **Thêm** là có thần dù không khớp quy tắc; **Bớt** là khớp quy tắc nhưng không có thần. Bảng trên cộng các ngoại lệ này tái tạo đúng từng ô của `DAY_SHEN_SHA` cho 15 thần:

| Thần | Tháng | Can Chi ngày | Loại |
| :-- | :-- | :-- | :-- |
| Thiên Đức | Thìn | Nhâm Dần | Bớt |
| Thiên Đức | Thân | Nhâm Ngọ | Thêm |
| Thiên Đức | Hợi | Kỷ Mão | Thêm |
| Nguyệt Đức | Ngọ | Mậu Ngọ | Thêm |
| Nguyệt Đức | Tuất | Canh Tuất | Thêm |
| Nguyệt Đức | Hợi | Ất Hợi | Thêm |
| Nguyệt Đức | Hợi | Kỷ Dậu | Thêm |
| Nguyệt Đức | Sửu | Bính Dần | Thêm |
| Sinh Khí | Sửu | Giáp Thân | Thêm |
| Hà Khôi | Hợi | Kỷ Sửu | Thêm |
| Kiếp Sát | Mùi | Quý Dậu | Thêm |
| Tai Sát | Mùi | Quý Dậu | Bớt |
| Tai Sát | Thân | Mậu Ngọ | Bớt |
| Nguyệt Hình | Dậu | Kỷ Dậu | Bớt |
| Nguyệt Hình | Tý | Đinh Mão | Bớt |

Quyết định: **giữ nguyên** các ngoại lệ để kết quả khớp 100% với dữ liệu nguồn.

## 4. Phần D: Quy tắc dân gian Việt Nam

Nguồn: `docs/vietnam_custom_rules.md`, giữ nguyên toàn bộ giá trị. Các bảng ở 4.2, 4.3, 4.5 được chép nguyên văn; phần liệt kê ở 4.1, 4.6, 4.7 được trình bày lại thành bảng, không đổi nội dung. Mọi tháng ở phần này là **tháng âm lịch**.

### 4.1. Ngày kỵ cố định theo ngày âm lịch

Xét theo `lunar_day` (1 đến 30), áp dụng giống nhau cho mọi tháng trong năm (kể cả tháng nhuận).

| Ngày kỵ | lunar_day | Ghi chú | Rust |
| :-- | :-- | :-- | :-- |
| Tam Nương Sát | [3, 7, 13, 18, 22, 27] | Kỵ khai trương, xuất hành, cưới hỏi, động thổ, sửa chữa | `TamNuongSat` |
| Nguyệt Kỵ | [5, 14, 23] | "Mùng năm, mười bốn, hai ba - Đi chơi cũng lỗ huống là đi buôn" | `NguyetKy` |
| Nguyệt Tận | Ngày cuối cùng của mỗi tháng Âm lịch (ngày 29 đối với tháng thiếu, hoặc ngày 30 đối với tháng đủ) | Lấy độ dài từ `LunarMonth.length` | `NguyetTan` |

### 4.2. Ngày Dương Công Kỵ Nhật (13 ngày đại kỵ trong năm)

Xét theo cặp (lunar_month, lunar_day). Quyết định của chủ dự án: áp dụng cả cho tháng nhuận, bất kể `is_leap_month`.

| Tháng Âm Lịch (lunar_month) | Ngày Dương Công Kỵ (lunar_day) |
| :-------------------------: | :----------------------------: |
|              1              |               13               |
|              2              |               11               |
|              3              |               9                |
|              4              |               7                |
|              5              |               5                |
|              6              |               3                |
|              7              |             8, 29              |
|              8              |               27               |
|              9              |               25               |
|             10              |               23               |
|             11              |               21               |
|             12              |               19               |

### 4.3. Ngày kỵ theo Địa Chi của ngày

Xét dựa trên lunar_month (Tháng Âm lịch từ 1 đến 12; tháng nhuận tính như tháng chính) đối chiếu với Địa Chi của ngày (day_branch).

| Tháng Âm | Thụ Tử (Đại kỵ mọi việc) | Sát Chủ Dương (Kỵ xây dựng, cưới gả, nhậm chức) | Sát Chủ Âm (Kỵ mai táng, tu sửa mộ phần) | Vãng Vong / Lục Bại (Kỵ xuất hành, cầu tài, ký kết) | Thiên Địa Chuyển Sát (Chính Đông/Nam/Tây/Bắc) |
| :------: | :----------------------: | :---------------------------------------------: | :--------------------------------------: | :-------------------------------------------------: | :-------------------------------------------: |
|    1     |           Tuất           |                       Tý                        |                    Tỵ                    |                         Dần                         |                      Mão                      |
|    2     |           Thìn           |                       Sửu                       |                    Tý                    |                         Tỵ                          |                      Mão                      |
|    3     |           Hợi            |                       Sửu                       |                   Mùi                    |                        Thân                         |                      Mão                      |
|    4     |            Tỵ            |                      Tuất                       |                   Mão                    |                         Hợi                         |                      Ngọ                      |
|    5     |            Tý            |                      Thìn                       |                   Thân                   |                         Mão                         |                      Ngọ                      |
|    6     |           Ngọ            |                      Thìn                       |                   Tuất                   |                         Ngọ                         |                      Ngọ                      |
|    7     |           Sửu            |                       Sửu                       |                   Hợi                    |                         Dậu                         |                      Dậu                      |
|    8     |           Mùi            |                      Thìn                       |                   Sửu                    |                         Tý                          |                      Dậu                      |
|    9     |           Dần            |                       Sửu                       |                   Ngọ                    |                         Mùi                         |                      Dậu                      |
|    10    |           Thân           |                      Thìn                       |                   Dậu                    |                        Thìn                         |                      Tý                       |
|    11    |           Mão            |                       Mùi                       |                   Dần                    |                        Tuất                         |                      Tý                       |
|    12    |           Dậu            |                      Thìn                       |                   Thìn                   |                         Sửu                         |                      Tý                       |

### 4.4. Tuổi Mụ

`lunar_age = target_lunar_year - birth_lunar_year + 1`

Ví dụ: Sinh năm 1990 (Canh Ngọ), xem năm 2026 (Bính Ngọ) -> `lunar_age = 2026 - 1990 + 1 = 37 tuổi mụ`.

### 4.5. Hạn Tam Tai

Xét theo Địa Chi năm sinh birth_branch và Địa Chi năm xem target_branch. Tam Tai kéo dài 3 năm liên tiếp theo nhóm Tam Hợp cục của Địa Chi năm sinh.

| Nhóm Tam Hợp Tuổi (birth_branch) | 3 Năm Phạm Tam Tai (target_branch) | Chi Tiết Năm Nhập / Cư / Xuất                                                 |
| :------------------------------- | :--------------------------------- | :---------------------------------------------------------------------------- |
| Thân, Tý, Thìn                   | Dần, Mão, Thìn                     | Dần (Năm đầu - Nhập), Mão (Năm giữa - Cư, nặng nhất), Thìn (Năm cuối - Xuất)  |
| Dần, Ngọ, Tuất                   | Thân, Dậu, Tuất                    | Thân (Năm đầu - Nhập), Dậu (Năm giữa - Cư, nặng nhất), Tuất (Năm cuối - Xuất) |
| Tỵ, Dậu, Sửu                     | Hợi, Tý, Sửu                       | Hợi (Năm đầu - Nhập), Tý (Năm giữa - Cư, nặng nhất), Sửu (Năm cuối - Xuất)    |
| Hợi, Mão, Mùi                    | Tỵ, Ngọ, Mùi                       | Tỵ (Năm đầu - Nhập), Ngọ (Năm giữa - Cư, nặng nhất), Mùi (Năm cuối - Xuất)    |

### 4.6. Hạn Kim Lâu

Công thức Cửu Cung chia 9: lấy tuổi mụ chia cho 9 lấy phần dư, `rem = lunar_age % 9`.

| rem | Kết quả | Ý nghĩa | Rust |
| :-- | :-- | :-- | :-- |
| `rem == 1` | Kim Lâu Thân | Kỵ bản thân người đứng tuổi làm nhà / cưới hỏi | `KimLauThan` |
| `rem == 3` | Kim Lâu Thê | Gây họa cho người phối ngẫu / vợ chồng | `KimLauThe` |
| `rem == 6` | Kim Lâu Tử | Gây họa cho con cái trong nhà | `KimLauTu` |
| `rem == 8` | Kim Lâu Súc | Hại vật nuôi, ảnh hưởng kinh tế gia đình | `KimLauSuc` |
| `rem` thuộc `[0, 2, 4, 5, 7]` | Không phạm Kim Lâu | An toàn | `None` |

### 4.7. Hạn Hoang Ốc

Khởi tính 6 cung theo bàn tay. Áp dụng cho `lunar_age >= 10`.

- `tens = lunar_age / 10` (chữ số hàng chục, chia lấy phần nguyên), `units = lunar_age % 10` (chữ số hàng đơn vị).
- `hoang_oc_index = ((tens + units - 1) % 6) + 1`, nhận giá trị từ 1 đến 6.

| hoang_oc_index | Cung | Cát Hung | Ý nghĩa | Rust |
| :-- | :-- | :-- | :-- | :-- |
| 1 | Nhất Cát | Tốt | Làm nhà an cư lập nghiệp, mọi việc hanh thông | `NhatCat` |
| 2 | Nhì Nghi | Tốt | Làm nhà hưng vượng, giàu có | `NhiNghi` |
| 3 | Tam Địa Sát | Xấu | Phạm hạn Hoang Ốc, gia chủ dễ mắc bệnh tật | `TamDiaSat` |
| 4 | Tứ Tấn Tài | Tốt | Làm nhà phúc lộc vẹn toàn | `TuTanTai` |
| 5 | Ngũ Thọ Tử | Xấu | Phạm hạn Hoang Ốc nặng, gia đạo chia rẽ, sinh ly tử biệt | `NguThoTu` |
| 6 | Lục Hoang Ốc | Xấu | Phạm hạn Hoang Ốc, làm ăn khó thành, nghèo túng | `LucHoangOc` |

Ví dụ kiểm tra công thức:

| Tuổi | tens | units | Phép tính | Kết quả |
| :-- | :-- | :-- | :-- | :-- |
| 10 | 1 | 0 | `((1 + 0 - 1) % 6) + 1 = 1` | Nhất Cát |
| 20 | 2 | 0 | `((2 + 0 - 1) % 6) + 1 = 2` | Nhì Nghi |
| 37 | 3 | 7 | `((3 + 7 - 1) % 6) + 1 = (9 % 6) + 1 = 4` | Tứ Tấn Tài |

## 5. Vector kiểm thử

Tính từ các bảng trong tài liệu này, dùng làm test đầu tiên cho code Rust. Các vector **chưa được đối chiếu với lịch in**.

| Dương lịch | JDN | Âm lịch | Can Chi ngày | Nạp Âm ngày | Chi tháng | Trực | Sao | Thần của ngày | Kỵ Việt Nam |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| 10/02/2024 | 2460351 | 1/1 | Giáp Thìn | Phú Đăng Hỏa | Dần | Mãn | Đê Thổ Lạc | Kim Quỹ (Hoàng Đạo) | — |
| 16/02/2024 | 2460357 | 7/1 | Canh Tuất | Thoa Xuyến Kim | Dần | Thành | Ngưu Kim Ngưu | Tư Mệnh (Hoàng Đạo) | Tam Nương Sát, Thụ Tử |
| 25/07/2025 | 2460882 | 1/6 nhuận | Ất Mùi | Sa Trung Kim | Mùi | Kiến | Cang Kim Long | Huyền Vũ (Hắc Đạo) | — |

## 6. Quyết định và vấn đề còn mở

### 6.1. Đã chốt

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| Q1 | Vãng Vong tháng 9, 10 lệch giữa hai nguồn | Giữ `vietnam_custom_rules.md` (tháng 9 = Mùi, tháng 10 = Thìn), khớp Lịch Vạn Niên phổ thông. Code ghi chú rằng 6tail và Hiệp Kỷ ghi đảo lại |
| Q2 | Danh sách 15 Thần Sát | Giữ 8 cát, 7 hung ở mục 3.4 cho Xem Ngày |
| Q3 | Thần Sát của lá số không có trong nguồn | Dùng bảng do chủ dự án cung cấp ở mục 2.5 |
| Q4 | 15 ô ngoại lệ Thần Sát | Giữ nguyên để khớp 100% nguồn |
| Q5 | Dương Công Kỵ Nhật với tháng nhuận | Áp dụng cả tháng nhuận |
| Q6 | Hoang Ốc khi tuổi mụ dưới 10 | Trả về `None`, nhất quán với điều kiện `lunar_age >= 10` |
| Q7 | Năm dùng cho Tam Tai và tuổi mụ | Dùng năm âm lịch và Chi của năm âm lịch, nhất quán với định nghĩa tuổi mụ |
| Q8 | Chữ con vật của sao Vị, sao Bích | Dùng tên quen dùng: Vị Thổ Trĩ, Bích Thủy Du |
| Q9 | Nhãn Tàng Can | Giữ thứ tự mảng của nguồn; có thêm `main_hidden_stem` để lấy riêng Bản khí |
| Q10 | Lục Phá không có trong nguồn | Chủ dự án xác nhận 6 cặp ở mục 1.3 |
| | 28 Sao | Dùng công thức `(JDN + 11) mod 28` |
| | Tên `enum` | Dùng `Stem` và `Branch` |
| | Giờ Tý Sớm 23:00–23:59 | Mặc định `SplitAtMidnight`: giữ Can Chi ngày hôm đó, khởi Trụ Giờ từ Can ngày hôm sau |

### 6.2. Còn mở

| Mã | Vấn đề | Cách xử lý tạm |
| :-- | :-- | :-- |
| Q11 | **Can Chi tháng nhuận.** Chế độ `LunarNewYear` lấy Can Chi theo tháng mà nó lặp lại, như `amlich.js` của Hồ Ngọc Đức; chưa đối chiếu sách | Giữ như hiện tại |
| Q12 | **Vector kiểm thử mục 5.** Trực, Sao, Thần chưa đối chiếu với lịch in | Test bám theo đặc tả |

Ngoài phạm vi tài liệu này: `SHEN_SHA` còn 136 thần khác, trong đó 4 mã không có tên trong `I18n.ts` (`sn.sanSang`, `sn.guiKu`, `sn.daTui`, `sn.siLi`), và bảng Nghi Kỵ `DAY_YI_JI`, `TIME_YI_JI`.
