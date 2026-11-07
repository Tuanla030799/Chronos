Nạp một batch dữ liệu luận giải theo `docs/INTERPRETATION_DB_SPEC.md`. Tham số: mã batch, ví dụ `TV-04`. Danh sách batch, trang và nguồn ở mục 11.3 của đặc tả.

## Nguyên tắc cứng

1. **Chỉ dùng văn bản nguồn.** Với *Tân Biên*, nguồn là `.ref/sources/tan_bien/pages.txt` (khối `=== tr. N ===`). Không dùng kiến thức ngoài, không bổ sung ý sách không viết. `canonical_summary` chỉ dùng cho batch mà mục 11.3 cho phép, không bao giờ cho Tử Vi.
2. **Diễn đạt lại, không chép (D3).** Viết ngắn gọn, rõ nghĩa, tiếng Việt hiện đại. Không chép từ 12 từ liên tiếp trở lên của trang (V12). Nguồn *Tân Biên* có `rights = paraphrased`: không dùng `quote`.
3. **Không làm nhẹ, không làm nặng.** Giữ "tất", "ắt", "không thể", "chuyên", "khủng khiếp", "chắc chắn". Không đổi thành "dễ", "khó", "hay", "rất nặng". Không thêm "có thể" khi sách khẳng định.
4. **Giữ sắc thái xấu.** "mánh lới, thủ đoạn" là mánh khóe, thủ đoạn, không phải mưu mẹo; "thị phi hình sự" giữ chữ hình sự.
5. **Mỗi "ngoại trừ", "nhưng nếu", "rất cần"** phải có điều kiện loại trừ ở đoạn chung, bằng `not_with`, `tuan_triet: Neither` hoặc `condition_text`, để hai đoạn trái nhau không cùng hiện.
6. **Mọi chỗ khác bản in**, kể cả sửa một chữ, ghi `correction` kèm căn cứ: trang khác, bảng khác, vị trí sao không thể có.
7. **Không chắc thì không đoán.** Tên tắt mơ hồ (Tướng: Thiên Tướng hay Tướng Quân; Hình, Kỵ trong câu phú là Kình, Đà ở tr. 56), điều kiện "nhiều sao" không có ngưỡng: để vào `condition_text`. Ô sách không viết: ghi `gap`. "Lộc" không ghi rõ Lộc Tồn hay Hóa Lộc: dùng fact `any_loc` (D15 của đặc tả), không phải `condition_text`. "Sinh ban ngày/đêm", "ngày âm 1–15": dùng `birth_time_of_day`, `lunar_day_range`.
8. **Gắn `sensitivity`**: `Death`, `Illness` (cả thương tích, tàn tật), `Legal`, `Sexuality`, `FamilyLoss`. Giữ đúng nghĩa, không tô nhẹ.

## Cách mã hóa điều kiện (mục 7 của đặc tả)

- `anchor.stars`: một trong các sao này tọa thủ. Tiêu đề một cặp cùng loại ("Xương, Khúc") là một trong hai.
- Tiêu đề bộ từ 3 tên trở lên là phải hội đủ: neo toàn cung (bỏ `stars`) và `meets_all`.
- Cặp sao sách viết hai lần (ở mục mỗi sao) chỉ ghi **một** đoạn: `anchor.stars` gồm cả hai, `with_all` gồm cả hai, `p` và `kw` lấy cả hai nơi.
- "Đơn thủ tại X, Y đồng cung" là **hai phương án** trong mảng `when`, không gộp.
- "Cũng như trên": viết lại đầy đủ để đoạn đứng độc lập.
- "Sáng sủa tốt đẹp" của một sao có bảng độ sáng: `brightness: [Mieu, Vuong, Dac]` (Chính Tinh) hoặc `[Dac]` (phụ tinh); "mờ ám xấu xa": `[Ham]`. Độ sáng phụ tinh ở `docs/TUVI_SPEC.md` mục 6.1.
- Fact có sẵn: `branches`, `brightness`, `solo`, `with_all`, `not_with`, `meets_any`, `meets_all`, `not_meets`, `paired_with`, `flanked_by`, `transformations`, `transformations_all`, `with_transformations`, `tuan_triet` (`Tuan`, `Triet`, `Either`, `Neither`), `than_here`, `is_menh`, `gender`, `polarity` (`DuongNam`, `AmNam`, `DuongNu`, `AmNu`), `harmony` (`ThuanLy`, `NghichLy`), `ban_menh` (ngũ hành Bản Mệnh: `Kim`, `Moc`, `Thuy`, `Hoa`, `Tho`), `year_stems`, `year_branches`, `menh_any`, `menh_all`.
- Fact mới (Task 5A.5, mục 7.3 của đặc tả): `birth_time_of_day` (`Day` giờ Mão–Thân, `Night` giờ Dậu–Dần), `lunar_day_range` (`{ "from": 1, "to": 15 }`, 1–30), `opposite_only` (sao chỉ đóng tại đúng cung xung chiếu, không phải tam hợp), `in_than` (sao đóng tại cung an Thân, dù đang xét cung nào), `any_loc` (bool, có Lộc Tồn hoặc Hóa Lộc), `element_relation`, `element_relation_all` (một, hoặc mọi quan hệ phải đúng: `[{ "pair": "BanMenhCung" | "BanMenhChinhTinh" | "CungChinhTinh", "relation": "Sinh" | "DuocSinh" | "Khac" | "BiKhac" | "BinhHoa" }]`, đọc từ tên thứ nhất của `pair` sang tên thứ hai; "và" giữa hai quan hệ dùng `element_relation_all`, "hoặc" dùng `element_relation`).
- Tên `Branch`: `Ty` là Tý, `Ti` là Tỵ, `Than` là Thân, `Thin` là Thìn. Nhóm sao viết `@Ten` theo `data/interpretations/vocab/star_groups.json`.
- `section`: `DaiCuong`, `NuMenh`, `NamMenh`, `PhuGiai`. Khối Phú giải: câu phú vào `v` **nguyên văn, không lược, không chuẩn hóa chính tả** (V5 kiểm nó có trên trang); lời giải diễn đạt lại vào `b`.
- Đoạn nói về chính cung Mệnh (Chi của Mệnh, Mệnh vô Chính diệu, thuận lý…) cần `is_menh: true`: neo Mệnh còn được áp cho cung Thân. Đoạn "cung Mệnh - Thân" của mục 4.2 thì không cần.

## Các bước

1. **Đọc nguồn** theo trang của batch, ví dụ:
   `awk '/^=== tr\. 55 ===/{f=1} /^=== tr\. 66 ===/{f=0} f' .ref/sources/tan_bien/pages.txt`
2. **Viết dạng gọn** vào `build/ingest/<batch>.jsonl`, mỗi dòng một object (đặc tả của dạng gọn ở đầu `crates/astro-data/src/compact.rs`):
   ```
   {"collection":"tuvi.passage","batch":"TV-04","source":"tan_bien"}
   {"id":"menh/tu-vi/01","a":{"role":"Menh","stars":["TuVi"]},"sec":"DaiCuong","w":[{"brightness":["Mieu","Vuong","Dac"]}],"b":["…"],"p":[55],"kw":["Tử Vi miếu, vượng hay đắc địa tọa thủ"]}
   {"gap":["Menh","TuVi"],"why":"…","checked":"tan_bien tr. 55–60"}
   ```
   `id` dạng `<cung>/<sao hoặc nhóm>/<số hai chữ số>`, theo thứ tự sách. `kw` là cụm nguyên văn tối đa 15 từ, giữ cả lỗi đánh máy, nằm trọn trong một trang của `p`.
3. **Bung ra JSON:** `cargo run -q -p astro-data -- expand build/ingest/<batch>.jsonl`. File `data/interpretations/<module>/<batch>.<collection>.json` bị ghi đè.
4. **Kiểm:** `cargo run -q --release -p astro-data -- validate`. Gồm V1–V12 và V13 (quét 510.432 lá số, mọi đoạn phải có lá số hiện ra). Sửa ở file dạng gọn rồi bung lại cho đến khi sạch. V7, V13 báo "không bao giờ xảy ra" thường là đọc sai điều kiện hoặc lỗi in của sách: đối chiếu chương khác trước khi ghi `correction`.
5. **Xem tải:** `cargo run -q --release -p astro-data -- scan`, và `stats` cho số mục, ký tự, `correction`, `condition_text`.
6. **Bảng duyệt:** `cargo run -q -p astro-data -- review --batch <batch>`, ra `build/content/review/<batch>.md`.
7. **Không commit.** Báo cáo: số mục, đoạn, gap; các `correction` và căn cứ; các chỗ phân vân đã để `condition_text`; kết quả validate, scan.

Mục mới luôn ở `draft`. Chỉ chủ dự án đổi sang `approved`.
