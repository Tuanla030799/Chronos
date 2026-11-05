# Đặc Tả Quy Tắc Lịch Dân Gian & Hạn Tuổi Việt Nam (vietnam_custom_rules.md)

Tài liệu này là nguồn chuẩn (Single Source of Truth) cho phần quy tắc Trạch Nhật và xem hạn tuổi theo tập tục dân gian Việt Nam (dựa theo Ngọc Hạp Thông Thư và Lịch Vạn Niên Việt Nam). Khi cài đặt code Rust trong astro-core, bắt buộc tuân thủ 100% các hằng số và công thức dưới đây.

---

## 1. Ngày Kỵ Cố Định Theo Ngày Âm Lịch Hàng Tháng

Các ngày dưới đây xét trực tiếp theo số ngày Âm lịch (lunar_day từ 1 đến 30), áp dụng giống nhau cho mọi tháng trong năm (kể cả tháng nhuận):

- **Ngày Tam Nương Sát** (Kỵ khai trương, xuất hành, cưới hỏi, động thổ, sửa chữa):
  - Danh sách ngày: [3, 7, 13, 18, 22, 27]
- **Ngày Nguyệt Kỵ** ("Mùng năm, mười bốn, hai ba - Đi chơi cũng lỗ huống là đi buôn"):
  - Danh sách ngày: [5, 14, 23]
- **Ngày Nguyệt Tận**:
  - Ngày cuối cùng của mỗi tháng Âm lịch (ngày 29 đối với tháng thiếu, hoặc ngày 30 đối với tháng đủ).

---

## 2. Ngày Dương Công Kỵ Nhật (13 Ngày Đại Kỵ Trong Năm)

Xét theo cặp (lunar_month, lunar_day) Âm lịch:

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

---

## 3. Bảng Ngày Kỵ Theo Địa Chi Của Ngày (Tra Theo Tháng Âm Lịch 1 đến 12)

Xét dựa trên lunar_month (Tháng Âm lịch từ 1 đến 12; tháng nhuận tính như tháng chính) đối chiếu với Địa Chi của ngày (day_branch):

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

---

## 4. Công Thức Tính Hạn Tuổi Làm Nhà & Cưới Hỏi (Theo Tuổi Mụ)

### 4.1. Định nghĩa Tuổi Mụ (lunar_age)

- Công thức: `lunar_age = target_lunar_year - birth_lunar_year + 1`
- Ví dụ: Sinh năm 1990 (Canh Ngọ), xem năm 2026 (Bính Ngọ) -> `lunar_age = 2026 - 1990 + 1 = 37 tuổi mụ`.

### 4.2. Hạn Tam Tai (Xét theo Địa Chi năm sinh birth_branch và Địa Chi năm xem target_branch)

Tam Tai kéo dài 3 năm liên tiếp theo nhóm Tam Hợp cục của Địa Chi năm sinh:

| Nhóm Tam Hợp Tuổi (birth_branch) | 3 Năm Phạm Tam Tai (target_branch) | Chi Tiết Năm Nhập / Cư / Xuất                                                 |
| :------------------------------- | :--------------------------------- | :---------------------------------------------------------------------------- |
| Thân, Tý, Thìn                   | Dần, Mão, Thìn                     | Dần (Năm đầu - Nhập), Mão (Năm giữa - Cư, nặng nhất), Thìn (Năm cuối - Xuất)  |
| Dần, Ngọ, Tuất                   | Thân, Dậu, Tuất                    | Thân (Năm đầu - Nhập), Dậu (Năm giữa - Cư, nặng nhất), Tuất (Năm cuối - Xuất) |
| Tỵ, Dậu, Sửu                     | Hợi, Tý, Sửu                       | Hợi (Năm đầu - Nhập), Tý (Năm giữa - Cư, nặng nhất), Sửu (Năm cuối - Xuất)    |
| Hợi, Mão, Mùi                    | Tỵ, Ngọ, Mùi                       | Tỵ (Năm đầu - Nhập), Ngọ (Năm giữa - Cư, nặng nhất), Mùi (Năm cuối - Xuất)    |

### 4.3. Hạn Kim Lâu (Công thức Cửu Cung chia 9)

- Công thức: Lấy tuổi mụ chia cho 9 lấy phần dư: `rem = lunar_age % 9`
- Bảng đối chiếu kết quả:
  - `rem == 1`: Phạm **Kim Lâu Thân** (Kỵ bản thân người đứng tuổi làm nhà / cưới hỏi).
  - `rem == 3`: Phạm **Kim Lâu Thê** (Gây họa cho người phối ngẫu / vợ chồng).
  - `rem == 6`: Phạm **Kim Lâu Tử** (Gây họa cho con cái trong nhà).
  - `rem == 8`: Phạm **Kim Lâu Súc** (Hại vật nuôi, ảnh hưởng kinh tế gia đình).
  - `rem` thuộc `[0, 2, 4, 5, 7]`: **Không phạm Kim Lâu** (An toàn).

### 4.4. Hạn Hoang Ốc (Khởi tính 6 cung theo bàn tay)

Áp dụng cho `lunar_age >= 10`:

- Gọi `tens = lunar_age / 10` (chữ số hàng chục, chia lấy phần nguyên) và `units = lunar_age % 10` (chữ số hàng đơn vị).
- Công thức tính chỉ số cung Hoang Ốc (`hoang_oc_index` nhận giá trị từ 1 đến 6):
  - `hoang_oc_index = ((tens + units - 1) % 6) + 1`
- Ví dụ kiểm tra công thức:
  - Tuổi 10: `tens = 1`, `units = 0` -> `((1 + 0 - 1) % 6) + 1 = 1` (Nhất Cát).
  - Tuổi 20: `tens = 2`, `units = 0` -> `((2 + 0 - 1) % 6) + 1 = 2` (Nhì Nghi).
  - Tuổi 37: `tens = 3`, `units = 7` -> `((3 + 7 - 1) % 6) + 1 = (9 % 6) + 1 = 4` (Tứ Tấn Tài).
- Bảng đối chiếu 6 cung Hoang Ốc:
  - `1`: **Nhất Cát** (Tốt - Làm nhà an cư lập nghiệp, mọi việc hanh thông).
  - `2`: **Nhì Nghi** (Tốt - Làm nhà hưng vượng, giàu có).
  - `3`: **Tam Địa Sát** (**Xấu** - Phạm hạn Hoang Ốc, gia chủ dễ mắc bệnh tật).
  - `4`: **Tứ Tấn Tài** (Tốt - Làm nhà phúc lộc vẹn toàn).
  - `5`: **Ngũ Thọ Tử** (**Xấu** - Phạm hạn Hoang Ốc nặng, gia đạo chia rẽ, sinh ly tử biệt).
  - `6`: **Lục Hoang Ốc** (**Xấu** - Phạm hạn Hoang Ốc, làm ăn khó thành, nghèo túng).
