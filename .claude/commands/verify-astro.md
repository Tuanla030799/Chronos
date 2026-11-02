Hãy chạy `cargo test --package astro-core` và kiểm tra kết quả đầu ra đối chiếu với các quy tắc bất biến sau:

1. Múi giờ mặc định phải là GMT+7 (`tz_offset = 7.0`).
2. Kiểm tra các điểm biên: Ngày Sóc rơi vào khoảng 23:00 - 23:59 giờ Việt Nam, tháng nhuận Âm lịch, và thời khắc giao tiết Lập Xuân (Kinh độ Mặt Trời = 315.0°).
3. Nếu có bất kỳ sai lệch nào dù chỉ 1 phút ở Tiết khí hoặc lệch 1 ngày ở Âm lịch, hãy in ra giá trị biến trung gian (`JDN`, `SunLongitude`, `NewMoonDay`) để debug ngay.
