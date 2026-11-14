# Đặc tả Cơ sở dữ liệu Luận giải Offline (INTERPRETATION_DB_SPEC)

Thiết kế kho văn bản luận giải của Giai đoạn 5A cho ứng dụng iOS và Web, chạy offline 100%, khóa khớp với các `enum` của `astro-core`. Tài liệu này chốt cấu trúc và kế hoạch nạp dữ liệu.

**Trạng thái:** D1–D13 chốt ngày 2026-11-07. Bảng độ sáng phụ tinh, module `content` và V13 đã xong; các batch đang được nạp, xem `docs/PROGRESS.md`.

## 0. Tóm tắt

| Vấn đề | Đề xuất | Mục |
| :-- | :-- | :-- |
| Định dạng lưu trữ | Phương án A: JSON trong git là nguồn duy nhất. Crate build-time `astro-data` biên dịch ra SQLite cho iOS, JSON cho Web | 2 |
| Phần lấy từ Phương án B | Rust chỉ giữ khóa và logic chọn mục. Không có văn bản luận giải trong Rust | 2.1, 9 |
| Khớp `enum` | `astro-data` đọc khóa bằng chính `enum` của `astro-core`. Thêm hoặc đổi biến thể mà thiếu dữ liệu thì build lỗi | 3.1, 10 |
| Luận giải Tử Vi theo cung | Không dùng khối 14 sao × 12 cung × độ sáng. Dùng đoạn luận có điều kiện, bám đúng cách sách viết | 1.2, 7 |
| Chống bịa | Mục lấy từ sách có từ khóa gốc được máy kiểm. Ô sách không viết ghi là `gap`, không điền. Mục tổng hợp đánh dấu `canonical_summary` | 3.3, 3.5 |
| Nguồn | Tử Vi nạp từ PDF *Tân Biên*. Trạch Nhật kết hợp `lunar-ts` và tổng hợp. Bát Tự, Phong Thủy tổng hợp theo sách chuẩn | 12, D4 |

Quy mô dự kiến:

| Phân hệ | Mục theo khóa | Đoạn luận có điều kiện |
| :-- | --: | --: |
| `almanac` | 92 | |
| `bazi` | 134 | |
| `fengshui` | 57 | |
| `tuvi` | 207 | Ước 1.600–1.900 |

Văn bản Tử Vi cần nạp đo được khoảng 340.000 ký tự. Theo số đo thử nghiệm, phần Tử Vi khoảng 1,1 MB SQLite hoặc 860 KB JSON Web. Quy mô này quyết định nhiều lựa chọn ở mục 2.

## 1. Khảo sát đầu vào

### 1.1. Đề bài so với `enum` thực tế

Đọc từ `crates/astro-core/src`. Những dòng in đậm là chỗ đề bài và code khác nhau; schema theo code.

| Đề bài | Kiểu trong code | Số lượng | Ghi chú |
| :-- | :-- | --: | :-- |
| 12 Trực | `Truc` | 12 | |
| 28 Sao | `Mansion` | 28 | |
| 12 Thần Hoàng/Hắc Đạo | `PathGod` | 12 | |
| 15 Thần Sát Trạch Nhật | `AlmanacGod` | 15 | |
| **6 tên ngày kỵ** | `VietnamTaboo` | **9** | Có thêm `NguyetTan`, `ThienDiaChuyenSat`; Sát Chủ tách `SatChuDuong`, `SatChuAm` |
| Tam Tai | `TamTai` | 3 | `Nhap`, `Cu`, `Xuat` |
| Kim Lâu | `KimLau` | 4 | |
| Hoang Ốc | `HoangOc` | 6 | |
| **60 Hoa Giáp Nạp Âm** | `NaYin`, `CanChi` | **30**, 60 | `NaYin` có 30 biến thể, mỗi cái ứng với 2 Can Chi. Tách hai tầng, mục 5 |
| 10 Thiên Can Nhật Chủ | `Stem` | 10 | |
| 10 Thập Thần | `TenGod` | 10 | |
| 12 Vòng Trường Sinh | `LifeStage` | 12 | |
| **12 Thần Sát Mệnh Lý** | `BaziStar` | **10** | Thêm Không Vong và Khôi Canh: chúng là trường của `BaziAnalysis`, không phải biến thể |
| 8 Quái Mệnh | `Trigram` | 8 | |
| 8 Sao Bát Du Niên | `EightMansionsStar` | 8 | |
| **9 Sao, đương vận/thất vận** | `NineStar` | 9 | `luck()` cố định, không theo Vận. Core chưa có trạng thái đương vận, thất vận: D6 |
| 14 Chính Tinh, Phụ Tinh | `TuViStar` | 14 + 91 | Vòng Trường Sinh là 12 biến thể của `TuViStar` |
| **Miếu/Vượng/Đắc/Hãm** | `StarBrightness` | **5** | Có thêm `Binh`. Bảng thực tế mỗi Chính Tinh chỉ có 4 mức, tổng 56 ô; `Binh` chỉ có ở Tử Vi, Thiên Phủ |
| 12 cung chức năng | `TuViPalaceRole` | 12 | |
| Tứ Hóa | `FourTransformations` | 4 | Gắn vào sao gốc, không phải `TuViStar` |

Ba hệ quả kỹ thuật:

- **Thiếu `ALL`.** `TenGod`, `BaziStar`, `VietnamTaboo`, `TamTai`, `KimLau`, `HoangOc`, `StarBrightness` chưa có hằng `ALL`, nên chưa kiểm được độ phủ. Cần thêm trước, mục 9.
- **Trùng tên.** 34 tên biến thể xuất hiện ở từ 2 `enum` trở lên trong phạm vi này, ví dụ `ThienDuc` thuộc `PathGod`, `AlmanacGod`, `TuViStar`; `SinhKhi` thuộc `AlmanacGod`, `EightMansionsStar`; `Tu` thuộc `KimLau`, `LifeStage`, `TuViStar`. Khóa bắt buộc có tên collection.
- **Swift không có chuỗi tên biến thể.** WASM đã trả đúng tên biến thể qua serde, còn Swift nhận `enum` của UniFFI. Lấy tên bằng `String(describing:)` là cách dễ vỡ, nên khóa phải do Rust dựng, mục 9.

### 1.2. Cách *Tân Biên* thực sự viết

Đây là phát hiện chi phối thiết kế Tử Vi. Đo trên lớp văn bản PDF (mục 11.2) của toàn bộ Phần 2, mỗi "đoạn" là một gạch đầu dòng đã gộp dòng xuống, không tính 23 khối Phú giải của cung Mệnh. Đếm bằng biểu thức chính quy nên là số chỉ báo; "vượng" còn trúng cả sao Đế Vượng, nên cột độ sáng là cận trên.

| Chương | Trang | Ký tự | Đoạn | Nêu Miếu, Vượng, Đắc, Hãm, Bình | "sáng sủa", "mờ ám" | "tại `<Chi>`" | "đồng cung" | "đơn thủ" |
| :-- | :-- | --: | --: | --: | --: | --: | --: | --: |
| Đặc tính các sao | 33–51 | 28.122 | 376 | 101 (27%) | 22 | 9 | 35 | 0 |
| Cung Mệnh, Thân | 52–123 | 153.448 | 397 | 280 (71%) | 161 | 38 | 59 | 5 |
| 11 cung còn lại | 124–224 | 158.330 | 1.241 | **11 (0,9%)** | 160 (13%) | 410 (33%) | 519 (42%) | 290 (23%) |

Kết luận:

1. **Độ sáng là trục của cung Mệnh, không phải trục của 11 cung còn lại.** Ở đó sách viết theo Chi và theo sao đồng cung, ví dụ tr. 158: "Sát đồng cung: có uy quyền, nên chuyên về quân sự". Khối 14 × 12 × 4 có 672 ô, nhưng 11 cung còn lại gần như không có lời cho trục độ sáng. Ép đủ ô nghĩa là phải bịa, trái Quy tắc 4 của `CLAUDE.md`.
2. **Sách có đủ đặc tính theo độ sáng ở cấp từng sao.** Mục 3 ghi "Miếu địa: Tỵ, Ngọ, Dần, Thân: thông minh, trung hậu…", nên trục sao × độ sáng là có thật, 56 ô.
3. **Đơn vị tự nhiên là tổ hợp sao, không phải một sao.** Trên 144 trạng thái (12 vị trí Tử Vi × 12 cung) chỉ có 39 bộ Chính Tinh: 14 sao đơn, 24 cặp, 1 bộ rỗng. 24 trong 144 ô là cung vô Chính diệu.
4. **Không đủ 168 ô sao × cung.** Chương Nô Bộc không chia theo Chính Tinh: tr. 168–173 dùng ma trận Mệnh × Nô Bộc rồi phụ tinh, 0/14. Tật Ách thiếu tiêu đề Thiên Phủ, Tử Tức thiếu Thiên Tướng, Huynh Đệ thiếu Thiên Cơ, Thái Âm; bốn sao này chỉ có trong thân bài, cần soát khi nạp. Có tiêu đề riêng: 150 ô.
5. **Sách định nghĩa các từ điều kiện**, nên phần lớn máy xét được, mục 7.5. Riêng chữ "nhiều" không có ngưỡng: "nhiều sao" xuất hiện ở 145 đoạn cung Mệnh và 73 đoạn 11 cung còn lại.
6. **Phụ tinh cũng có điều kiện đắc, hãm.** 68 trên 126 đoạn của mục 4.2.15–4.2.24 nhắc đến chúng: D8.

Mức phủ tổng quan sao là đầy đủ: 85 mục ở tr. 33–52, trong đó 23 mục ghép từ 2 sao trở lên, phủ 105 `TuViStar` và 4 Hóa. Mục 3.73 in "Quan Phù" là Quan Phủ của vòng Bác Sỹ, cùng lỗi đã ghi ở mục 0.4 của `TUVI_SPEC.md`.

### 1.3. Nguồn hiện có

| Collection | Có trong tay | Phủ được | Còn thiếu |
| :-- | :-- | :-- | :-- |
| `tuvi.*` | *Tân Biên* tr. 17–20, 32–224 | 105 sao, Tứ Hóa, 12 cung | Nô Bộc theo Chính Tinh |
| `almanac.taboo`, `.tam_tai`, `.kim_lau`, `.hoang_oc` | `docs/vietnam_custom_rules.md` | Một dòng ý nghĩa cho phần lớn mục | Lời cho Nguyệt Tận, Dương Công, Thiên Địa Chuyển Sát; cách hóa giải |
| `almanac.mansion` | `LunarUtil.XIU_SONG`, giấy phép MIT | 28 bài thơ chữ Hán, nêu rõ việc nên, việc kỵ | Bản tiếng Việt, phải dịch hoặc có nguồn Việt |
| `almanac.truc`, `.path_god`, `.god` | Tên và Cát, Hung | | Toàn bộ lời luận |
| `bazi.nayin` | *Tân Biên* tr. 241–243 | 30 nguyên thể, như "Vàng dưới biển" | Tính cách, vận mệnh chung |
| `bazi.can_chi`, `.day_master`, `.ten_god`, `.life_stage`, `.shen_sha` | Bảng tra trong đặc tả | | Toàn bộ lời luận |
| `fengshui.*` | Ma trận 8×8, tên, ngũ hành, màu | | Toàn bộ lời luận |

Không có nguồn nào cho cách hóa giải hạn: D5. `vietnam_custom_rules.md` không nói đến. *Tân Biên* không nhắc Tam Tai, Kim Lâu, Hoang Ốc; chữ "giải trừ" trong sách luôn nói về tác dụng của sao trên lá số, và lời nói đầu tr. 4 còn chê việc cúng sao cầu giải nạn.

## 2. Định dạng lưu trữ và chia sẻ đa nền tảng

### 2.1. So sánh hai phương án

| Tiêu chí | A: JSON nguồn, biên dịch ra SQLite và JSON | B: bảng ngắn trong Rust, văn bản dài ngoài |
| :-- | :-- | :-- |
| Sửa một câu | Sửa JSON, build lại dữ liệu | Bảng ngắn phải build lại Rust, XCFramework, WASM |
| Dung lượng WASM | Không đổi, 279.538 byte | Tăng theo văn bản, ước thêm 50–100% với khoảng 300 mục ngắn |
| Tính chất `astro-core` | Giữ nguyên: không phụ thuộc, không I/O | Thêm văn bản luận giải vào lõi tính toán |
| Quy trình duyệt | Một đường, một định dạng | Hai đường, ranh giới "ngắn" và "dài" phải tự quy định |
| Khớp `enum` | Kiểm lúc build dữ liệu | Trình biên dịch kiểm: đây là ưu điểm thật của B |
| Nạp từ NotebookLM | Trực tiếp ra JSON | Phải sinh mã Rust cho phần bảng ngắn |
| Review trong git | Diff JSON theo đoạn | Diff lẫn giữa mã và nội dung |
| Tốc độ đọc | Một lần tra khóa | Nhanh hơn, nhưng không đo được khác biệt ở quy mô này |

**Đề xuất: Phương án A, kèm một phần của B.** Ưu điểm kiểm `enum` của B lấy lại được bằng cách cho `astro-data` đọc khóa qua chính `enum` của `astro-core`. Điều đáng giữ trong Rust là logic chọn khóa, vì nó phải giống nhau trên iOS và Web. Nó không cần chứa văn bản, mục 9.

### 2.2. Kiến trúc

```text
Sách PDF, NotebookLM, bảng của chủ dự án
        │  gói nguồn trong .ref/sources/ (không commit)
        ▼
data/interpretations/**/*.json          nguồn chuẩn duy nhất, có trong git
        │  cargo run -p astro-data -- validate
        │  đọc khóa bằng enum của astro-core, kiểm độ phủ, nguồn, điều kiện
        ▼
build/content/                          sinh ra, không commit
  ├── chronos_content.sqlite            → bundle iOS, Widget
  ├── web/<module>.<hash>.json          → Web, Service Worker
  └── manifest.json

Khi chạy:  astro-core trả khóa và facts → app đọc văn bản theo khóa
```

### 2.3. Cây thư mục

```text
data/interpretations/
  sources.json                     danh mục nguồn, mục 3.3
  vocab/activities.json            từ vựng việc nên làm, việc kỵ
  vocab/star_groups.json           nhóm sao, mục 7.6
  almanac/  bazi/  fengshui/  tuvi/
    <batch>.<collection>.json      ví dụ tuvi/tv-15.passage.json
  fixtures/                        lá số mẫu và đoạn mong đợi, mục 10
crates/astro-data/                 công cụ build-time
.ref/sources/<nguồn>/pages.txt     văn bản nguồn theo trang, ngoài git
```

Mỗi file thuộc đúng một batch và một collection để diff và review theo batch:

```json
{ "collection": "tuvi.passage", "batch": "TV-15", "entries": [], "gaps": [] }
```

### 2.4. Crate `astro-data`

- Là crate thứ tư của workspace, **chỉ chạy lúc build**, không phải dependency của `astro-ffi`, `astro-wasm`. App không link nó.
- Phụ thuộc: `astro-core` với feature `serde`, `serde`, `serde_json`, `sha2`, `rusqlite` với feature `bundled`.
- Lệnh: `validate [--complete]`, `build [--release]`, `stats`, `scan`, `review --batch <mã>`.
- `build-ios.sh` và `wasm-pack` build theo package nên không bị ảnh hưởng. `cargo test --workspace` sẽ biên dịch SQLite C một lần.

### 2.5. iOS

- `chronos_content.sqlite` nằm trong bundle của app. Widget Extension có bundle riêng nên file phải thuộc cả hai target, hoặc dùng App Group; chốt ở Phase 5.
- File build với `journal_mode = DELETE`, đúng khuyến nghị của SQLite cho phương tiện chỉ đọc. Không để ở WAL.
- Mở bằng URI `file:<đường dẫn>?immutable=1`: SQLite mở chỉ đọc, bỏ khóa và dò thay đổi. Điều kiện là file không bao giờ bị sửa, đúng với bundle iOS.
- Chỉ cần `SELECT` theo khóa nên dùng SQLite hệ thống là đủ. Phase 5 tra Context7 trước khi viết phần Swift.

Dùng JSON cho cả hai nền tảng cũng chạy được ở quy mô này. SQLite được chọn vì đọc theo neo không phải giải mã cả file, hợp với giới hạn bộ nhớ của Widget.

### 2.6. Web

- Mỗi phân hệ một file `web/<module>.<hash>.json`, tên theo mã băm nội dung. Ước tính `tuvi` khoảng 1 MB, các phân hệ khác dưới 150 KB.
- Service Worker precache vào Cache Storage, gọi `navigator.storage.persist()`. Mở phân hệ nào thì tải file đó vào một `Map`.
- **Không dùng IndexedDB ở bản đầu.** Dữ liệu chỉ đọc, tra theo khóa, mỗi file nhỏ, nên IndexedDB chỉ thêm bước chép dữ liệu và đường nâng cấp. Xem lại nếu sau này có tìm kiếm toàn văn hoặc dữ liệu vượt khoảng 10 MB.

### 2.7. Phiên bản

`manifest.json`, chép vào bảng `meta` của SQLite:

```json
{
  "schema_version": 2,
  "engine_version": "0.1.0",
  "content_hash": "sha256:…",
  "modules": { "almanac": { "file": "almanac.3f9a1c.json", "entries": 92 } }
}
```

- `content_hash` tính trên các hàng đã biên dịch, không tính trên file SQLite, vì byte của file phụ thuộc phiên bản SQLite. Nó cũng là định danh phiên bản nội dung.
- SQLite đặt `PRAGMA user_version = schema_version` và `PRAGMA application_id = 1128813134` (chữ `CHRN`).
- Engine và dữ liệu luôn đi cùng nhau: iOS trong cùng bản app, Web trong cùng precache. Vì vậy không có đường cập nhật dữ liệu riêng ở bản đầu.

## 3. Quy ước chung của schema

### 3.1. Khóa

Dạng `<collection>/<Phần>[/<Phần>]`. Mỗi phần là **tên biến thể Rust nguyên văn**, trùng tên serde và tên trong khai báo TypeScript của `astro-wasm`.

```text
almanac.truc/Kien
almanac.god/ThienDuc                 khác almanac.path_god/ThienDuc
bazi.can_chi/Giap/Ty
fengshui.eight_star_use/SinhKhi/Bep
tuvi.star_brightness/TuVi/Mieu
```

Không dùng số thứ tự, không đổi chữ hoa thường, không bỏ tên collection. Trong JSON, `key` khử tuần tự thẳng vào kiểu Rust: `"Kien"` vào `Truc`, `["TuVi", "Mieu"]` vào `(TuViStar, StarBrightness)`.

### 3.2. Trường chung

| Trường | Kiểu | Bắt buộc | Ý nghĩa |
| :-- | :-- | :-- | :-- |
| `key` hoặc `keys` | khóa | Có | `keys` khi một lời dùng cho nhiều khóa, như mục ghép Kình Dương, Đà La |
| `summary` | chuỗi, tối đa 160 ký tự | Có | Một câu cho thẻ hoặc ô lịch |
| `body` | mảng chuỗi | Không | Mỗi phần tử một đoạn. Dùng mảng để diff git theo đoạn |
| `aliases` | mảng chuỗi | Không | Tên khác: Bảo Quang, Lục Bại, Lò Trung Hỏa |
| `source_ref` | `SourceRef` | Có | Mục 3.3 |
| `sensitivity` | mảng thẻ | Không | Mục 3.6 |
| `status` | `draft`, `approved` | Có | Mục 3.4 |
| `correction` | chuỗi | Không | Ghi chỗ dữ liệu khác bản in vì sửa lỗi in, nêu căn cứ. Của đoạn Tử Vi (`tuvi.passage`) thì vào file build từ schema 2 (D20) để app hiện "Đính chính so với bản in"; của mục (`entry`) vẫn chỉ dùng để duyệt |

**Không lặp dữ liệu của core.** JSON không chứa tên chính, Cát Hung, ngũ hành hay bảng Miếu Hãm. `astro-data` lấy `name_vi()`, `luck()`, `is_hoang_dao()`, `nature()` từ core rồi ghi vào file build, nên không thể lệch với engine.

Văn bản là chuỗi thuần, không Markdown, không biến thay thế. App không ghép câu mới.

### 3.3. Nguồn

`sources.json` khai báo mỗi nguồn một lần:

```json
{
  "id": "tan_bien",
  "title": "Tử Vi Đẩu Số Tân Biên",
  "author": "Vân Đằng Thái Thứ Lang",
  "edition": "Internet Archive TuViDdauSoTanBien-VDThaiThuLang-DV, bản đánh máy lại 2014",
  "rights": "unverified",
  "packet": ".ref/sources/tan_bien/pages.txt"
}
```

`rights` nhận `open`, `licensed`, `paraphrased` hoặc `unverified`. `paraphrased` là quyết định của chủ dự án chỉ phát hành bản diễn đạt lại của nguồn đó. `packet` là văn bản nguồn chia khối theo trang, không có nếu nguồn không kiểm được theo trang. Mỗi mục dẫn nguồn bằng `source_ref`:

```json
{
  "source": "tan_bien",
  "source_type": "paraphrase",
  "pages": [158],
  "keywords": ["Sát đồng cung: có uy quyền"]
}
```

| Trường | Ý nghĩa |
| :-- | :-- |
| `source_type` | Mục dưới |
| `pages` | Các trang chứa lời gốc |
| `locator` | Thay cho `pages` với nguồn không theo trang, ví dụ `LunarUtil.XIU_SONG`, `mục 1` |
| `keywords` | Vài cụm từ gốc, mỗi cụm tối đa 15 từ, giữ nguyên cả lỗi đánh máy. `validate` kiểm từng cụm có trong `packet` ở đúng trang. Không đưa vào file build |

| `source_type` | Ý nghĩa | `keywords` |
| :-- | :-- | :-- |
| `paraphrase` | Mặc định theo D3: diễn đạt lại ngắn gọn bằng ngôn ngữ hiện đại | Bắt buộc |
| `quote` | Nguyên văn, chỉ cho câu phú ngắn | Bắt buộc |
| `translation` | Dịch từ Hán văn | Bắt buộc |
| `canonical_summary` | Tổng hợp theo tinh thần một sách chuẩn, không trích từ trang cụ thể: D4 | Không có |

Từ khóa gốc là phép kiểm máy chính chống bịa: mục không tìm được từ khóa trong nguồn thì không qua `validate`.

`canonical_summary` không kiểm được bằng máy, nên chỉ dùng cho batch được D4 cho phép. `citation` của mục này ghi "Tổng hợp theo truyền thống *<tên sách>*", không ghi số trang và không viết như lời trích từ sách.

### 3.4. Trạng thái

- `draft`: vừa sinh, chưa duyệt. Chỉ vào bản build dev, có đánh dấu.
- `approved`: chủ dự án đã duyệt. Chỉ trạng thái này vào `build --release`.

Khóa chưa có mục `approved` thì app chỉ hiện tên và Cát Hung từ core.

### 3.5. Khoảng trống

Ô sách không viết được ghi rõ trong mảng `gaps` của file, không điền:

```json
{ "key": ["NoBoc", "TuVi"], "gap": "Chương Nô Bộc không chia theo Chính Tinh", "checked": "tan_bien tr. 168–173" }
```

`gap` là kết luận đã kiểm chứng, không phải việc còn nợ, và không vào file build. `validate --complete` yêu cầu mỗi khóa có mục hoặc `gap`; `validate` thường và `stats` chỉ báo khóa còn thiếu, vì các batch nạp dần.

### 3.6. Từ vựng

- `activities.json`: danh sách có kiểm soát cho `good_for`, `avoid`, ví dụ `CuoiHoi`, `KhaiTruong`, `XuatHanh`, `DongTho`, `SuaChua`, `MaiTang`, `CauTai`, `KyKet`. Bắt đầu từ từ ngữ trong `vietnam_custom_rules.md`, bổ sung từ `LunarUtil.YI_JI` (142 mục) khi cần. Dùng thẻ để sau này lọc được ngày theo việc mà không phải gắn lại dữ liệu.
- `sensitivity`: `Death`, `Illness`, `Legal`, `Sexuality`, `FamilyLoss`, dùng cho D9.

## 4. Phân hệ 1: `almanac`

| Collection | Khóa | Số mục | Trường riêng |
| :-- | :-- | --: | :-- |
| `almanac.truc` | `Truc` | 12 | `good_for`, `avoid` |
| `almanac.mansion` | `Mansion` | 28 | `good_for`, `avoid`, `verse` |
| `almanac.path_god` | `PathGod` | 12 | `good_for`, `avoid` |
| `almanac.god` | `AlmanacGod` | 15 | `good_for`, `avoid` |
| `almanac.taboo` | `VietnamTaboo` | 9 | `avoid`, `verse` |
| `almanac.restriction` | `AgeTopic`: `TamTai`, `KimLau`, `HoangOc` | 3 | `advice`: tổng quan từng loại hạn |
| `almanac.tam_tai` | `TamTai` | 3 | `advice` |
| `almanac.kim_lau` | `KimLau` | 4 | `advice` |
| `almanac.hoang_oc` | `HoangOc` | 6 | `advice` |

`good_for`, `avoid` là mảng `{ "activity": "CuoiHoi", "note": "…" }`, `note` không bắt buộc. Tên, Cát Hung và điều kiện phạm hạn đã có trong core nên không lưu lại.

`advice` là chuỗi tùy chọn, chỉ ghi lưu ý thực tế trong dân gian, như thận trọng khi khởi sự lớn hoặc tục mượn tuổi khi động thổ. Không ghi nghi thức cúng sao, giải hạn: D5. Không có thì bỏ trường, app không hiện mục này.

Ví dụ, lời lấy từ mục 1 của `vietnam_custom_rules.md`:

```json
{
  "key": "TamNuongSat",
  "summary": "Kỵ khai trương, xuất hành, cưới hỏi, động thổ, sửa chữa.",
  "avoid": [
    { "activity": "KhaiTruong" }, { "activity": "XuatHanh" }, { "activity": "CuoiHoi" },
    { "activity": "DongTho" }, { "activity": "SuaChua" }
  ],
  "source_ref": {
    "source": "vn_custom_rules", "source_type": "quote", "locator": "mục 1",
    "keywords": ["Kỵ khai trương, xuất hành, cưới hỏi, động thổ, sửa chữa"]
  },
  "status": "draft"
}
```

## 5. Phân hệ 2: `bazi`

| Collection | Khóa | Số mục | Trường riêng |
| :-- | :-- | --: | :-- |
| `bazi.nayin` | `NaYin` | 30 | `image`, `traits`, `fortune` |
| `bazi.can_chi` | `(Stem, Branch)` | 60 | `traits`, `fortune` |
| `bazi.day_master` | `Stem` | 10 | `traits` |
| `bazi.ten_god` | `TenGod` | 10 | |
| `bazi.life_stage` | `LifeStage` | 12 | |
| `bazi.shen_sha` | `BaziStar`, thêm `KhongVong`, `KhoiCanh` | 12 | |

- **Hai tầng của 60 Hoa Giáp.** `bazi.nayin` là tầng bắt buộc: 30 Nạp Âm, trong đó `image` là nguyên thể lấy từ *Tân Biên*. `bazi.can_chi` chỉ có mục khi nguồn thật sự viết riêng từng tuổi, còn lại ghi `gap`. App hiện cả hai tầng nếu có. `validate` chỉ nhận 60 cặp hợp lệ, sinh bằng `CanChi::from_cycle_index`.
- `KhongVong`, `KhoiCanh` không có trong `BaziStar`: `bazi_keys` sinh chúng từ `khong_vong_day`, `khong_vong_year`, `is_khoi_canh`.
- `bazi.life_stage` chỉ dùng cho Bát Tự. Vòng Trường Sinh của Tử Vi có lời riêng ở `tuvi.star`.

## 6. Phân hệ 3: `fengshui`

| Collection | Khóa | Số mục | Trường riêng |
| :-- | :-- | --: | :-- |
| `fengshui.kua` | `Trigram` | 8 | |
| `fengshui.eight_star` | `EightMansionsStar` | 8 | |
| `fengshui.eight_star_use` | `(EightMansionsStar, EightUse)` | 32 | `verdict` |
| `fengshui.nine_star` | `NineStar` | 9 | `auspicious_meaning`, `inauspicious_meaning` |

`EightUse` là `HuongNha`, `Bep`, `BanLamViec`, `KetHon`, ứng với các đường tính đã có:

| `EightUse` | Lấy sao từ |
| :-- | :-- |
| `HuongNha`, `Bep`, `BanLamViec` | `Trigram::star_at(direction)`, `direction` do người dùng chọn |
| `KetHon` | `kua_compatibility(first, second).star` |

`verdict` là `Luck` tùy chọn cho riêng ngữ cảnh. Nếu nguồn cho biết cát hung ở một ngữ cảnh khác `EightMansionsStar::luck()`, ví dụ khi đặt bếp, app dùng `verdict` và không tô màu theo `luck()`.

Trạng thái của sao trong Vận theo D6, nguồn *Thẩm Thị Huyền Không Học*, `star` và `period` từ 1 đến 9:

| Trạng thái | Điều kiện | Văn bản dùng |
| :-- | :-- | :-- |
| Đương Vận, Vượng Khí | `star == period`. Vận 5: Ngũ Hoàng vẫn là Chính thần nhưng bản chất sát khí, cần tĩnh | `auspicious_meaning` |
| Sinh Khí | `star == (period % 9) + 1`, đúng 1 vận kế tiếp | `auspicious_meaning` |
| Thoái Khí | `star == ((period + 7) % 9) + 1`, ví dụ Vận 9 là Bát Bạch | `auspicious_meaning`, kèm nhãn Thoái khí |
| Thất Vận: Tử Khí, Sát Khí | Các sao còn lại. Nhị Hắc, Ngũ Hoàng khi thất vận là đại hung sát | `inauspicious_meaning` |

Ví dụ Vận 9: Cửu Tử là Đương Vận, Nhất Bạch là Sinh Khí, Bát Bạch là Thoái Khí, sáu sao còn lại Thất Vận, kể cả Nhị Hắc.

## 7. Phân hệ 4: `tuvi`

### 7.1. Các lớp

| Lớp | Collection | Khóa | Số lượng | Nguồn |
| :-- | :-- | :-- | --: | :-- |
| Tổng quan sao | `tuvi.star` | `TuViStar` | 105 | tr. 33–52 |
| Sao theo độ sáng | `tuvi.star_brightness` | `(TuViStar, StarBrightness)` | 86 | tr. 33–43 |
| Tứ Hóa | `tuvi.transformation` | `FourTransformations` | 4 | Mục 3.23–3.26 |
| Giới thiệu cung | `tuvi.palace` | `TuViPalaceRole` | 12 | Đoạn mở đầu mỗi chương |
| Theo cung | `tuvi.passage` | Neo `(cung, sao)` | Ước 1.600–1.900 đoạn | tr. 52–224 |

Ba tầng đề bài yêu cầu cho 14 Chính Tinh đều có: tổng quan ở `tuvi.star`, ảnh hưởng Miếu Hãm ở `tuvi.star_brightness`, ý nghĩa tại 12 cung ở `tuvi.passage`. Phụ tinh và Tứ Hóa dùng cùng cấu trúc, phần hội hợp nằm ở `when`.

`tuvi.star` có thêm `profile`, ghi đúng tiêu đề sách:

```json
{ "dau": "NamBacDau", "element": "Dương Thổ", "labels": ["Đế tinh"] }
```

`tuvi.star_brightness` chỉ có các khóa mà `TuViStar::brightness_at` thực sự trả về: 56 của Chính Tinh, 30 của 17 phụ tinh. Ví dụ tr. 33, khóa `["TuVi", "Mieu"]`: "thông minh, trung hậu, nghiêm cẩn, uy nghi".

### 7.2. Đoạn luận có điều kiện

Một đoạn là một ý của sách, gồm **neo** để tìm nhanh và **điều kiện** để lọc:

```json
{
  "id": "quan-loc/tu-vi/01",
  "anchor": { "role": "QuanLoc", "stars": ["TuVi", "ThienPhu"] },
  "when": [
    { "solo": true, "branches": ["Ngo"] },
    { "with_all": ["TuVi", "ThienPhu"] }
  ],
  "body": ["Sự nghiệp vẻ vang, vừa có địa vị vừa giàu có."],
  "source_ref": {
    "source": "tan_bien", "source_type": "paraphrase", "pages": [158, 160],
    "keywords": ["Đơn thủ tại Ngọ, Phủ đồng cung", "Tử đồng cung: công danh hiến hách"]
  },
  "status": "draft"
}
```

Câu gốc là "Đơn thủ tại Ngọ, Phủ đồng cung: công danh hiển hách…". Đây là **hai trường hợp** chứ không phải một: Tử Vi ở Ngọ thì Thiên Phủ ở Tuất, còn Tử Vi và Thiên Phủ chỉ đồng cung tại Dần, Thân. Vì vậy `when` là mảng các phương án. Đọc nhầm thành một điều kiện gộp sẽ bị `validate` chặn, mục 10.

| Trường của đoạn | Ý nghĩa |
| :-- | :-- |
| `id` | Ổn định, dạng `<cung>/<sao>/<số>`, số theo thứ tự trong sách |
| `anchor.role` | `TuViPalaceRole` |
| `anchor.stars` | Đoạn áp dụng khi một trong các sao này tọa thủ. Nhận cả tên nhóm `@LucSat` |
| `anchor.transformations` | Thay cho `stars`, neo theo `FourTransformations` |
| `anchor.no_main_star` | Thay cho `stars`, cung vô Chính diệu |
| Không có cả ba | Neo toàn cung: dùng cho bộ sao trải trên nhiều cung, không sao nào bắt buộc tọa thủ |
| `section` | `DaiCuong`, `NuMenh`, `NamMenh`, `PhuGiai`, theo tiêu đề nhỏ của sách; để nhóm khi hiển thị |
| `verse` | Câu phú của khối Phú giải, nguyên văn cả lỗi in, không lược. Đây là câu phú truyền thống, không phải lời tác giả, nên V12 không áp dụng; V5 kiểm nó có trên trang |
| `when` | Mảng phương án, thỏa một phương án là đạt. Bỏ trống nghĩa là luôn áp dụng |
| `condition_text` | Phần điều kiện máy không xét được, mục 7.5 |

### 7.3. Trường của `when`

Trong một phương án, các trường là quan hệ VÀ. Trong một danh sách là HOẶC, trừ khi tên có `_all`.

| Trường | Kiểu | Đúng khi | Chữ trong sách |
| :-- | :-- | :-- | :-- |
| `branches` | `Branch[]` | Cung đóng tại một trong các Chi | "tại Tỵ, Hợi" |
| `brightness` | `StarBrightness[]` | Sao neo hoặc Hóa neo có một trong các độ sáng: Chính Tinh, 17 phụ tinh, Hóa Kỵ | "miếu, vượng hay đắc địa" |
| `solo` | bool | Không có Chính Tinh khác trong cung | "đơn thủ" |
| `with_all` | sao | Mọi sao đều ở cung này, được liệt kê cả sao neo | "Sát đồng cung" |
| `not_with` | sao | Không sao nào ở cung này | Loại trừ ở đoạn chung khi sách viết "nhưng nếu" |
| `meets_any`, `meets_all` | sao | Một, hoặc mọi sao hội hợp | "gặp Kình, Đà" |
| `not_meets` | sao | Không sao nào hội hợp | "vô Hình Kỵ" |
| `paired_with` | sao | Sao ở cung nhị hợp | "gặp Nguyệt nhị hợp" |
| `flanked_by` | sao | Sao ở hai cung kề bên | "giáp" |
| `transformations`, `transformations_all` | `FourTransformations[]` | Một, hoặc mọi Hóa hội hợp | "gặp Khoa, Quyền" |
| `with_transformations` | `FourTransformations[]` | Có Hóa này trong cung | "gặp Quyền đồng cung" |
| `tuan_triet` | `Tuan`, `Triet`, `Either`, `Neither` | Cung có, hoặc không có, Tuần, Triệt | "Tuần, Triệt án ngữ" |
| `than_here` | bool | Thân cư cung này | "Thân cư Phúc Đức" |
| `is_menh` | bool | Cung đang xét chính là cung Mệnh. Neo Mệnh còn được áp cho cung Thân, nên đoạn nói riêng về cung Mệnh cần `true` | "Mệnh vô Chính diệu", "Mệnh tại Tý" |
| `polarity` | `TuViPolarity[]` | Âm Dương Nam Nữ | "Dương Nam, Âm Nữ" |
| `harmony` | `YinYangHarmony` | Thuận lý, nghịch lý giữa năm sinh và cung Mệnh | "thuận lý" |
| `ban_menh` | `FiveElements[]` | Ngũ hành của Bản Mệnh | "Kim Mệnh, cung Mệnh an tại Tỵ" |
| `gender` | `Gender` | Giới tính | "Nữ mệnh" |
| `year_stems`, `year_branches` | `Stem[]`, `Branch[]` | Can, Chi năm sinh | "Tuổi Giáp, Đinh, Kỷ" |
| `menh_any`, `menh_all` | sao | Một, hoặc mọi sao ở cung Mệnh | Ma trận Mệnh × Nô Bộc |
| `birth_time_of_day` | `BirthTimeOfDay`: `Day`, `Night` | Giờ sinh. `Day`: Mão, Thìn, Tỵ, Ngọ, Mùi, Thân. `Night`: Dậu, Tuất, Hợi, Tý, Sửu, Dần | "sinh ban ngày", "sinh ban đêm" |
| `lunar_day_range` | `{ from, to }`, 1–30 | Ngày sinh âm lịch nằm trong khoảng, hai đầu đều tính | "từ mồng Một đến Rằm", ngày cụ thể |
| `opposite_only` | sao | Một trong các sao này đóng tại đúng cung xung chiếu, không phải tam hợp hay đồng cung | "chỉ xung chiếu", để phân biệt với tam hợp chiếu |
| `in_than` | sao | Một trong các sao này đóng tại cung an Thân, bất kể đang xét cung nào | "sao X thủ cung Thân" khi đang luận cung Mệnh |
| `any_loc` | bool | Lộc Tồn hoặc Hóa Lộc có mặt, tọa thủ hoặc hội hợp tùy ngữ cảnh | "gặp Lộc" khi sách không nói rõ Lộc Tồn hay Hóa Lộc |
| `element_relation`, `element_relation_all` | `{ pair, relation }[]` | Một, hoặc mọi quan hệ sinh khắc giữa hai Ngũ hành của `pair` đúng với `relation`. `pair`: `BanMenhCung` (Bản Mệnh, Ngũ hành Cung), `BanMenhChinhTinh` (Bản Mệnh, Ngũ hành Chính Tinh), `CungChinhTinh` (Ngũ hành Cung, Ngũ hành Chính Tinh). `relation`: `Sinh`, `DuocSinh`, `Khac`, `BiKhac`, `BinhHoa`, đọc từ tên thứ nhất của `pair` sang tên thứ hai | "Mệnh Thủy gặp sao Kim sinh phù", "cung Mệnh sinh Chính diệu, Chính diệu lại sinh Bản Mệnh" |

Ngũ hành Chính Tinh chỉ có ở 14 Chính Tinh (`TuViStar::element`), lấy từ tiêu đề mỗi sao ở `tuvi.star` (tr. 33–38), không suy đoán cho phụ tinh.

### 7.4. Neo, độ phủ và quy tắc đọc

- 12 × 14 = 168 neo Chính Tinh, mỗi neo phải có đoạn hoặc `gap`. Dự kiến 14 `gap` của Nô Bộc.
- **Cặp sao chỉ viết một lần.** Sách lặp cùng một lời ở hai mục, như "Tướng đồng cung" ở 8.1 và "Tử đồng cung" ở 8.11. Gộp thành một đoạn hai neo, `with_all` gồm cả hai sao, `keywords` giữ cả hai nơi, nếu không app hiện một câu hai lần. Một đoạn được nhiều neo trả về chỉ hiện một lần.
- **Bộ sao.** Tiêu đề một cặp cùng loại như "Xương, Khúc" hiểu là một trong hai, dùng `anchor.stars`. Tiêu đề từ 3 tên trở lên hiểu là phải hội đủ, dùng `meets_all` với neo toàn cung, trừ khi V7 cho thấy không thể xảy ra.
- **"Cũng như trên"** viết đầy đủ để đoạn đứng độc lập.
- **Cung vô Chính diệu** dùng `anchor.no_main_star`. Sách có lời cho Mệnh vô Chính diệu ở tr. 53–54.
- **Mượn sao xung chiếu.** Sách dặn "Coi Chính diệu xung chiếu như Chính diệu tọa thủ" ở đúng 8 cung: Phụ Mẫu, Phúc Đức, Quan Lộc, Nô Bộc, Thiên Di, Tài Bạch, Tử Tức, Huynh Đệ (tr. 131, 142, 166, 171, 180, 197, 207, 224). Mệnh và Điền Trạch có lời riêng, Tật Ách và Thê Thiếp không có mục này, nên không mượn. Ở 8 cung đó, `tuvi_palace_query` trả thêm một lookup: neo là hai Chính Tinh đối cung, `branch`, `with`, `bright` theo cung đối, phần còn lại theo cung trống. Cung trống luôn đối diện đúng 2 Chính Tinh, nên lookup này không bao giờ "đơn thủ".
- Mục 4.2 của sách đặt tên "tọa thủ tại cung Mệnh - Thân", nên `tuvi_palace_query` trả cả neo `Menh` cho cung có `is_than`.
- Đoạn chung của cung như 15.1 "Tiên quyết" neo vào nhóm sao, ví dụ `@NamDau`.

### 7.5. Điều kiện máy xét được

Lấy từ Phần 1 mục 9 và Phần 2 mục 2 của sách:

| Từ | Định nghĩa của sách | Cách xét |
| :-- | :-- | :-- |
| Đơn thủ | tr. 33: an tại một cung, không gặp Chính diệu khác cùng cung | `main_stars` có đúng 1 sao |
| Đồng cung | tr. 33 | Cùng `TuViPalace` |
| Tam chiếu | tr. 18: "Ba cung chiếu lẫn nhau" | `Branch::trine()` |
| Xung chiếu | tr. 19: "Hai cung đối nhau, chiếu lẫn nhau" | `Branch::clash()` |
| Nhị hợp | tr. 20 | `Branch::six_combination()`, fact `pair` |
| Hội hợp | tr. 33: "gặp nhau tại cùng một cung, hay chiếu lẫn nhau" | Cùng cung, tam chiếu, xung chiếu: D7 |
| Sáng sủa tốt đẹp | tr. 33: sao đặc tính tốt, hoặc an tại miếu, vượng, đắc địa | `nature()` và `brightness`, chưa dùng ở bản đầu |
| Mờ ám xấu xa | tr. 33: sao đặc tính xấu, hoặc an tại hãm địa | Như trên |

**Điều kiện không xét được thì giữ nguyên lời, không tự đặt ngưỡng.** Sách không cho biết "nhiều" là bao nhiêu sao, nên phần đó vào `condition_text`, các trường khác vẫn lọc bình thường:

```json
{
  "id": "menh/tu-vi/05",
  "anchor": { "role": "Menh", "stars": ["TuVi"] },
  "section": "NuMenh",
  "when": [{ "gender": "Female" }],
  "condition_text": "gặp nhiều sao sáng sủa tốt đẹp hội hợp",
  "body": ["…"]
}
```

App hiện dạng "Nếu gặp nhiều sao sáng sủa tốt đẹp hội hợp: …", không khẳng định điều kiện đó đúng với lá số. Tên viết tắt chưa chắc nghĩa cũng vào `condition_text`.

### 7.6. Nhóm sao

`star_groups.json` khai báo nhóm, mỗi nhóm có nguồn, `validate` thay `@Tên` thành danh sách `TuViStar`:

```json
{ "id": "LucSat", "stars": ["KinhDuong", "DaLa", "HoaTinh", "LinhTinh", "DiaKhong", "DiaKiep"], "source": "tan_bien tr. 32" }
```

"Sát tinh" đồng nghĩa với Lục Sát: mục 1.11 liệt kê sáu sao, các mục 3.14, 3.16, 3.21 ghi đúng sáu sao đó là Sát tinh.

Nhóm hay gặp, đếm trong sách: Tả Hữu, Xương Khúc, Hỏa Linh, Không Kiếp, Khôi Việt, Nhật Nguyệt, Kình Đà, Song Hao, Đào Hồng, Long Phượng, Sát Phá Liêm Tham, Cơ Nguyệt Đồng Lương, Khoa Quyền Lộc, Lục Bại, Nam Đẩu, Bắc Đẩu. Thành phần từng nhóm đọc từ sách lúc nạp, không suy đoán.

### 7.7. Bẫy khi đọc sách

| Chữ trong sách | Có thể là | Cách xử lý |
| :-- | :-- | :-- |
| Hình, Kỵ trong câu phú | Kình Dương, Đà La (tr. 56 chú rõ), không phải Thiên Hình, Hóa Kỵ | Đọc phần giải của câu phú |
| Lộc | Lộc Tồn hoặc Hóa Lộc | Xác định theo ngữ cảnh, không rõ thì để `condition_text` |
| Tướng | Thiên Tướng hoặc Tướng Quân, như "Tướng, Binh, Thai" tr. 198–207 | Như trên |
| Sát | Thất Sát; "Sát tinh" là cả nhóm | Không nhầm hai dạng |
| Tử | Tử Vi, hoặc sao Tử của vòng Trường Sinh | Theo ngữ cảnh |
| Đế, Nhật, Nguyệt | Tử Vi, Thái Dương, Thái Âm | Bảng tên tắt trong skill nạp |
| Quan Phù, Quan Phủ | `QuanPhuThaiTue`, `QuanPhuBacSy` | Theo vòng sao |
| Phúc Đức | Sao `TuViStar::PhucDuc` hoặc cung `TuViPalaceRole::PhucDuc` | Theo ngữ cảnh |
| Thê Thiếp (Phu Quân) | `TuViPalaceRole::PhuThe` | Chương viết theo góc nhìn người chồng, D9 |

## 8. File build

### 8.1. SQLite

```sql
PRAGMA journal_mode = DELETE;
PRAGMA application_id = 1128813134;
PRAGMA user_version = 2;

CREATE TABLE meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) WITHOUT ROWID;

-- Mọi collection tra theo một khóa.
CREATE TABLE entry (
  key         TEXT PRIMARY KEY,           -- 'almanac.truc/Kien'
  collection  TEXT NOT NULL,
  title       TEXT NOT NULL,              -- từ name_vi() của core
  luck        TEXT,                       -- từ core: Cat, Trung, Hung
  summary     TEXT NOT NULL,
  body        TEXT NOT NULL,              -- mảng JSON các đoạn
  extra       TEXT,                       -- JSON: good_for, avoid, verse, advice, profile
  citation    TEXT NOT NULL,              -- 'Tử Vi Đẩu Số Tân Biên, tr. 33'
  sensitivity INTEGER NOT NULL DEFAULT 0, -- mặt nạ bit
  draft       INTEGER NOT NULL DEFAULT 0  -- 1 ở bản dev với mục chưa duyệt
) WITHOUT ROWID;
CREATE INDEX entry_collection ON entry (collection);

CREATE TABLE passage (
  id             INTEGER PRIMARY KEY,     -- sinh lúc build, theo thứ tự sách
  stable_id      TEXT NOT NULL UNIQUE,    -- 'quan-loc/tu-vi/01'
  section        TEXT,
  verse          TEXT,
  gates          TEXT NOT NULL,           -- JSON, mục 8.3
  condition_text TEXT,
  correction     TEXT,                    -- chỗ khác bản in và căn cứ (D20); NULL nếu không có
  body           TEXT NOT NULL,
  citation       TEXT NOT NULL,
  sensitivity    INTEGER NOT NULL DEFAULT 0,
  draft          INTEGER NOT NULL DEFAULT 0,
  tier           TEXT NOT NULL            -- Primary, hoặc More khi có condition_text
);

-- Một đoạn có thể gắn nhiều neo: 'QuanLoc/TuVi', 'Menh/HoaLoc', 'Menh/~NoMain', 'QuanLoc/~Any'.
CREATE TABLE passage_anchor (
  anchor     TEXT NOT NULL,
  passage_id INTEGER NOT NULL REFERENCES passage (id),
  PRIMARY KEY (anchor, passage_id)
) WITHOUT ROWID;
```

### 8.2. JSON Web

Cùng nội dung, mỗi phân hệ một file. Parity giữa hai đầu ra được kiểm ở mục 10.

```json
{
  "entries": { "tuvi.star/TuVi": { "title": "Tử Vi", "summary": "…", "body": ["…"], "citation": "…" } },
  "passages": [{ "id": 412, "gates": [], "body": ["…"], "citation": "…" }],
  "anchors": { "QuanLoc/TuVi": [412, 413] }
}
```

### 8.3. Cổng lọc

`astro-data` biên dịch mỗi phương án của `when` thành một cổng gồm các fact dạng chuỗi. Một đoạn đạt khi có cổng đạt, hoặc khi không có cổng nào:

```json
[
  { "all": ["solo"], "any": [["branch:Ngo"]], "none": [] },
  { "all": ["with:ThienPhu"], "any": [], "none": [] }
]
```

Một cổng đạt khi có mọi fact của `all`, mỗi nhóm của `any` có ít nhất một fact, và không có fact nào của `none`. `brightness` biên dịch theo sao neo, thành `bright:TuVi:Mieu`.

## 9. Module `content` trong `astro-core`

`crates/astro-core/src/content.rs`: không văn bản, không I/O, không dependency. Swift và TypeScript không tự ghép chuỗi khóa và không lặp logic lá số. `astro-data` dùng chính module này cho V10, V13.

| Hàm Rust | Swift, JS | Trả về |
| :-- | :-- | :-- |
| `almanac_day_keys(&AlmanacDay)` | `almanacDayKeys` | Khóa Trực, Sao, Thần, Thần Sát, ngày kỵ |
| `age_restriction_keys(&AgeRestrictions)` | `ageRestrictionKeys` | Khóa hạn tuổi, tổng quan trước trường hợp |
| `bazi_keys(&BaziAnalysis)` | `baziKeys` | Khóa Nhật Chủ, từng trụ, Thần Sát |
| `kua_key`, `eight_star_key`, `eight_star_use_key`, `kua_compatibility_keys` | Cùng tên camelCase | Khóa Bát Trạch |
| `nine_star_key(star)` | `nineStarKey` | Khóa Cửu Tinh; `NineStar::phase_in(period)` cho biết dùng văn bản nào |
| `tuvi_palace_query(&TuViChart, Branch)` | `tuviPalaceQuery` | `PalaceQuery { keys, lookups }` |
| `gate_matches(&[Gate], &[String])` | `gateMatches` | Một đoạn có đạt không. Swift truyền chuỗi JSON của cột `gates`, JS truyền mảng `Gate` |

Luồng đọc một cung Tử Vi:

1. `tuvi_chart` lập lá số.
2. `tuvi_palace_query` trả `keys` và các `lookups`, mỗi lookup gồm `anchors` và `facts`.
3. Với từng lookup, app lấy đoạn theo neo: SQL `passage_anchor`, hoặc `anchors` trong `Map`.
4. `gate_matches` lọc đoạn bằng `facts` của chính lookup đó. Một đoạn chỉ lấy một lần.
5. App hiện `tier = Primary` trước, gom `More` vào "Xem thêm", nhóm theo `section`.

Neo `Menh/<sao>` được thêm cho cung có `is_than`; mục 7.4 nêu các quy tắc còn lại.

| Fact | Nghĩa |
| :-- | :-- |
| `branch:<Branch>` | Chi của cung |
| `solo`, `no_main`, `than`, `tuan`, `triet`, `is_menh` | Trạng thái cung |
| `with:<TuViStar>` | Sao ở cùng cung |
| `meet:<TuViStar>` | Sao hội hợp: cùng cung, tam chiếu, xung chiếu |
| `pair:<TuViStar>` | Sao ở cung nhị hợp |
| `flank:<TuViStar>` | Sao ở cung kề bên |
| `hoa:<FourTransformations>` | Hóa hội hợp |
| `hoa_with:<FourTransformations>` | Hóa trong cung |
| `bright:<TuViStar hoặc Hóa>:<StarBrightness>` | Độ sáng của sao, Hóa trong cung |
| `gender:<Gender>`, `polarity:<TuViPolarity>`, `harmony:<YinYangHarmony>`, `ban_menh:<FiveElements>`, `year_stem:<Stem>`, `year_branch:<Branch>` | Người xem |
| `menh:<TuViStar>` | Sao tại cung Mệnh |
| `oppose:<TuViStar>` | Sao đóng tại đúng cung xung chiếu, tách riêng khỏi `meet` để xét "chỉ xung chiếu" |
| `than_sit:<TuViStar>` | Sao đóng tại cung an Thân, luôn có mặt dù đang xét cung nào |
| `birth_time:<BirthTimeOfDay>` | Giờ sinh ban ngày hay ban đêm |
| `lunar_day:<1..30>` | Ngày sinh âm lịch |
| `any_loc` | Lộc Tồn hoặc Hóa Lộc có mặt, tọa thủ hoặc hội hợp |
| `element:<ElementPair>:<ElementRelation>` | Quan hệ sinh khắc giữa hai Ngũ hành của `ElementPair`, mục 7.3 |

## 10. Kiểm tra tự động

`cargo run -p astro-data -- validate` chạy các phép kiểm sau. Lỗi nào cũng dừng build.

| Mã | Kiểm tra |
| :-- | :-- |
| V1 | JSON đúng schema, `deny_unknown_fields`. Trường riêng chỉ dùng đúng collection |
| V2 | Mọi khóa khử tuần tự được vào `enum` thật của `astro-core` |
| V3 | Không khóa nào trùng, kể cả giữa các file. Với `--complete`: mọi giá trị trong `ALL` có mục hoặc `gap` |
| V4 | Mọi mục có `source_ref`, `source` có trong `sources.json`, `source_type` đúng quy tắc mục 3.3 |
| V5 | `keywords` có trong `packet` ở đúng trang. Thiếu `packet` trên máy thì báo bỏ qua, không im lặng |
| V6 | Thẻ `activity`, `sensitivity`, nhóm `@Tên` tồn tại |
| V7 | `when` thỏa được trên 144 trạng thái Chính Tinh, bảng độ sáng và bảng Tứ Hóa: bắt `with_all` sai vị trí, cặp sao không bao giờ đồng cung, độ sáng không tồn tại như Tử Vi hãm địa, Hóa không thể đồng cung. Báo lỗi ngay, không cần quét lá số |
| V8 | `summary` không quá 160 ký tự, văn bản không rỗng |
| V9 | SQLite và JSON Web cùng số mục, cùng nội dung, cùng `content_hash` |
| V10 | `fixtures/tuvi_palace.json`: 5 vector của `TUVI_SPEC.md` mục 6.2 cho đúng danh sách đoạn của Quan Lộc, chạy ở Rust. Hai cầu nối có test riêng cho cùng API |
| V11 | `build --release` bỏ mục chưa `approved`; từ chối nếu mục `approved` dựa trên nguồn `rights = unverified`, hoặc dùng `quote`, `translation` với nguồn `paraphrased` |
| V12 | Mục `paraphrase` không chép nguyên một đoạn từ 12 từ liên tiếp trở lên của trang nguồn: D3. `verse` không thuộc phép này nhưng phải nguyên văn, không lược, tối đa 40 từ |
| V13 | Quét lá số thật của một chu kỳ 60 năm bằng `content`: mọi đoạn phải xuất hiện ít nhất một lần. Dừng ngay khi đã thấy đủ. Chạy trong `validate`, `build`, `cargo test`; lệnh `scan` in thêm tải từng cung |

`stats` in độ phủ từng collection: `approved`, `draft`, `gap`, thiếu. Test `every_key_of_the_content_module_exists_in_the_registry` đảm bảo khóa do `content` sinh luôn có trong registry.

## 11. Kế hoạch nạp dữ liệu

### 11.1. Quy trình một batch

1. **Chuẩn bị nguồn** trong `.ref/sources/<nguồn>/pages.txt`, chia khối theo trang: `=== tr. 158 ===`.
2. **Sinh JSON**, mỗi phiên một batch, bằng skill mới `/ingest-content` theo khuôn của `/ingest-rules`. Quy tắc cứng:
   - Chỉ dùng văn bản nguồn, không dùng kiến thức ngoài. Không chắc thì để `condition_text` hoặc `gap`.
   - Không làm nhẹ lời sách: giữ "tất", "không thể", "chuyên", "khủng khiếp"; không đổi thành "dễ", "khó", "hay", "rất nặng".
   - Mỗi "ngoại trừ", "nhưng nếu" phải có điều kiện loại trừ ở đoạn chung, bằng `not_with`, `tuan_triet` hoặc `condition_text`, để hai đoạn trái nhau không cùng hiện.
   - Mỗi chỗ khác bản in, kể cả sửa một chữ, ghi `correction` kèm căn cứ.
3. **Kiểm máy:** `validate` phải sạch.
4. **Duyệt:** `review --batch` in từng cặp nguyên văn và JSON cạnh nhau, ở `build/content/review/`. Đọc hết các đoạn có `correction`, `sensitivity`, và mọi đoạn về phụ tinh, bộ sao, vô Chính diệu: mục 11.5 cho thấy lỗi tập trung ở đó. Đoạn về Chính Tinh đọc mẫu 20%. Mẫu có lỗi thì đọc cả batch.
5. **Chốt:** đổi `status` thành `approved`, commit riêng từng batch, cập nhật `docs/PROGRESS.md`.

### 11.2. Chuẩn bị nguồn

**Tân Biên: dùng lớp văn bản của PDF, không dùng `_djvu.txt`.** PDF của Internet Archive có lớp văn bản số, dấu tiếng Việt đúng. `crates/astro-data/tools/extract_pdf_pages.swift` đọc bằng PDFKit và ghi ra `pages.txt`. File `_djvu.txt` là OCR, sai nhiều: "đổng cung", "Cù Môn", "Khôi , Viêt". Sáu phần PDF khớp số trang in: dv01 từ tr. 1, dv02 tr. 31, dv03 tr. 81, dv04 tr. 132, dv05 tr. 181, dv06 tr. 225.

Lớp văn bản vẫn cần soát:

- Còn lỗi đánh máy, ví dụ tr. 158 "hiến hách" là "hiển hách". Sửa ở `body`, giữ nguyên ở `keywords`.
- Có chữ Cyrillic trông giống chữ Latin, như "Тhìn", "đượс". `astro-data` đổi 18 ký tự này trước khi so từ khóa.
- Vài dòng đầu trang có thể bị đảo thứ tự, như tiêu đề 8.1 ở tr. 158.
- Bảng và sơ đồ mất cấu trúc, như tr. 18–20: đọc từ ảnh trang.

**NotebookLM:** yêu cầu trích nguyên văn, không tóm tắt, mỗi đoạn kèm tên sách và số trang, xuất bảng `Khóa | Trang | Nguyên văn`. Câu trả lời tóm tắt của NotebookLM không dùng làm nguồn vì không có neo để kiểm.

### 11.3. Các batch

Giới hạn mỗi batch: **tối đa khoảng 25.000 ký tự nguồn**, để một batch nằm trong sức review. Đầu ra JSON lớn hơn dự kiến, xem mục 11.5, nên nên viết bằng dạng gọn rồi mới bung thành JSON.

**Tử Vi, nguồn đã sẵn:**

| Batch | Nội dung | Trang | Ký tự |
| :-- | :-- | :-- | --: |
| TV-01 | 14 Chính Tinh: `star`, `star_brightness` | 33–39 | 7.328 |
| TV-02 | 91 phụ tinh, Tứ Hóa: `star`, `transformation`; nhóm sao | 32, 39–52 | 20.794 |
| TV-03 | Mệnh: nhận định chung, vô Chính diệu | 52–55 | 7.546 |
| TV-04 | Mệnh: Tử Vi, Liêm Trinh, Thiên Đồng | 55–65 | 19.891 |
| TV-05 | Mệnh: Vũ Khúc, Thái Dương | 65–74 | 20.575 |
| TV-06 | Mệnh: Thiên Cơ, Thiên Phủ, Thái Âm, Thiên Tướng | 74–81, 90–92 | 18.668 |
| TV-07 | Mệnh: Tham Lang, Cự Môn | 81–90 | 21.849 |
| TV-08 | Mệnh: Thiên Lương, Thất Sát, Phá Quân | 92–101 | 19.524 |
| TV-09 | Mệnh: Xương Khúc, Khôi Việt, Lộc Tồn, Tả Hữu | 101–110 | 18.041 |
| TV-10 | Mệnh: Kình Dương, Đà La, Hỏa Linh, Không Kiếp | 110–117 | 13.797 |
| TV-11 | Mệnh: Tứ Hóa, Lục Bại | 117–124 | 13.267 |
| TV-12 | Phụ Mẫu, Nô Bộc | 124–131, 168–173 | 19.757 |
| TV-13 | Phúc Đức, mục 6.1 | 132–142 | 23.151 |
| TV-14 | Phúc Đức mục 6.2, Điền Trạch | 143–157 | 23.000 |
| TV-15 | Quan Lộc | 158–167 | 17.831 |
| TV-16 | Thiên Di, Tật Ách | 174–189 | 21.660 |
| TV-17 | Tài Bạch, Huynh Đệ | 190–197, 217–224 | 23.258 |
| TV-18 | Tử Tức | 198–207 | 13.618 |
| TV-19 | Thê Thiếp | 208–216 | 15.833 |

**Các phân hệ khác**, theo D4:

| Batch | Nội dung | Số mục | Nguồn | `source_type` |
| :-- | :-- | --: | :-- | :-- |
| AL-01 | `taboo`, `tam_tai`, `kim_lau`, `hoang_oc`, `restriction` | 25 | `vietnam_custom_rules.md`; 3 ngày kỵ thiếu lời theo *Ngọc Hạp Thông Thư* | `quote`, `canonical_summary` |
| AL-02 | `mansion` | 28 | `LunarUtil.XIU_SONG` | `translation` |
| AL-03 | `truc`, `path_god` | 24 | *Ngọc Hạp Thông Thư*, *Hiệp Kỷ Biện Phương Thư* | `canonical_summary` |
| AL-04 | `god` | 15 | Như trên, đối chiếu `DAY_YI_JI` | `canonical_summary` |
| BZ-01 | `nayin`: `image` | 30 | *Tân Biên* tr. 241–243 | `paraphrase` |
| BZ-02 | `nayin`: `traits`, `fortune`; `can_chi` | 90 | *Tam Mệnh Thông Hội* | `canonical_summary` |
| BZ-03 | `day_master`, `ten_god` | 20 | *Tử Bình Chân Thuyên* | `canonical_summary` |
| BZ-04 | `life_stage`, `shen_sha` | 24 | *Tam Mệnh Thông Hội* | `canonical_summary` |
| FS-01 | `kua`, `eight_star` | 16 | *Bát Trạch Minh Cảnh* | `canonical_summary` |
| FS-02 | `eight_star_use` | 32 | *Bát Trạch Minh Cảnh* | `canonical_summary` |
| FS-03 | `nine_star` | 9 | *Thẩm Thị Huyền Không Học* | `canonical_summary` |

Mục `canonical_summary` dài 2–4 câu.

### 11.4. Thứ tự

1. **Hạ tầng và thử nghiệm:** `astro-data`, các `ALL`, TV-01 cho khóa dày, TV-15 cho đoạn có điều kiện. Dừng lại đo token và sai sót, sửa schema nếu cần. Sau hai batch này đổi schema sẽ đắt.
2. **Module `content`:** dựng khóa và facts, đưa lên Swift và WASM.
3. **Trạch Nhật:** AL-01 đến AL-04. Đây là phần app dùng hằng ngày.
4. **Tử Vi:** các batch TV còn lại.
5. **Bát Tự, Phong Thủy:** BZ, FS.

### 11.5. Kết quả thử nghiệm TV-01, TV-15

Đo ngày 2026-11-07, trước khi duyệt.

| | TV-01 | TV-15 |
| :-- | --: | --: |
| Trang nguồn | 33–39 | 158–167 |
| Ký tự trên các trang nguồn | 9.994 | 18.649 |
| Mục | 42 và 2 `gap` | 116 đoạn, 1 mục cung |
| Ký tự văn bản sau khi diễn đạt lại | 4.578 | 9.195 |
| Tỷ lệ văn bản so với nguồn | 0,46 | 0,49 |
| JSON nguồn | 27.499 byte | 85.813 byte |
| Mục có `correction` | 2 | 10 |
| Mục có `condition_text` | 0 | 24 |
| Mục có `sensitivity` | 6 | 3 |

Phủ neo: 14/14 Chính Tinh của Quan Lộc. Gộp cặp sao bớt được 29 đoạn lặp, từ 145 xuống 116.

**Dung lượng build:** SQLite 94.208 byte, JSON Web 72.455 byte, nén gzip còn 9.905 byte. Hai batch chiếm 8,4% lượng nguồn của Tử Vi, nên toàn bộ Tử Vi ước khoảng 1,1 MB SQLite, 860 KB JSON Web, 120 KB sau nén, 1,35 MB JSON nguồn.

**Token:** JSON đầu ra của TV-15 khoảng 80.000 ký tự, tức khoảng 32 nghìn token theo giả định 2,5 ký tự mỗi token, gấp đôi ước tính ban đầu. Viết bằng dạng gọn chỉ còn 27.000 ký tự, khoảng 11 nghìn token. Giả định này chưa đo bằng tokenizer.

**Validator:** bắt được 2 lỗi của chính dữ liệu vừa sinh (V12 chép quá sát, V7 "Cơ, Vũ" không thể cùng hội hợp), và V7 bản mở rộng bắt được "gặp Quyền đồng cung". Các lỗi in của sách phát hiện khi đọc đối chiếu; V7 chỉ xác nhận được lỗi về vị trí như "tại Thìn", còn "Tý, Ngọ" ở 8.5 thì không, vì Thái Dương đơn thủ tại Tý vẫn có thật. 71 test, gồm 14 test trên dữ liệu thật có cố tình gài lỗi.

**Kiểm định độc lập** trên 160 mục, đọc đối chiếu từng mục với nguyên văn:

| Kết quả | Số lượng |
| :-- | --: |
| Dòng sách bị bỏ sót | 0 |
| Lỗi đổi nghĩa nặng | 0 |
| Nhầm Chi, nhầm sao | 0 |
| Lỗi nặng về điều kiện | 4 (2,5%): 3 đã sửa, 1 chưa nạp (D10) |
| Lỗi nhẹ, chủ yếu làm nhẹ lời sách | 28 (17,5%), đã sửa, chưa kiểm định lại |
| Đính chính được đồng ý | 8/8 |

Lỗi nặng tập trung: 0/79 ở Chính Tinh và Sát tinh, 4/38 ở phụ tinh, bộ sao, vô Chính diệu. Các batch nhiều phụ tinh như TV-02, TV-09 đến TV-11 có thể có tỷ lệ cao hơn 2,5%.

**Quét 510.432 lá số thật** (60 năm, mọi tháng không nhuận, ngày, giờ, hai giới tính) sau khi sửa: 116/116 đoạn đều xuất hiện. Trước khi sửa có 2 đoạn không bao giờ hiện, V7 bản đầu không thấy. Mỗi cung Quan Lộc hiện trung bình 8,6 đoạn, ít nhất 1, nhiều nhất 25; chỉ 2,3 đoạn không kèm "Nếu". Phân bố có hai đỉnh, ở khoảng 2–3 và 14–15 đoạn: cung có Sát tinh hiện thêm 12 đoạn của ma trận Sát tinh, xem D12.

**Sau khi có D8 và `content`:** 14 đoạn chuyển từ `condition_text` sang `brightness`, còn 22 đoạn có `condition_text`. Mỗi cung Quan Lộc giảm còn trung bình 6,3 đoạn, 2,6 đoạn chắc chắn. Cung có Sát tinh hiện thêm 6 đoạn thay vì 12, cả 12 nếu có cả Sát tinh đắc lẫn hãm. Build: SQLite 98.304 byte, JSON Web 76.127 byte. `astro_wasm_bg.wasm` tăng từ 279.538 lên 342.187 byte do API `content`.

### 11.6. Cụm TV-02 đến TV-06

Nạp ngày 2026-11-08 bằng `/ingest-content`, năm batch chạy song song, mọi mục còn `draft`.

| Batch | Mục | Gap | `correction` | `condition_text` | JSON |
| :-- | --: | --: | --: | --: | --: |
| TV-02 | 97 (91 phụ tinh, 4 Hóa, 14 khóa độ sáng) | 16 | 20 | 0 | 91.839 byte |
| TV-03 | 50 đoạn, 1 mục cung | 0 | 5 | 36 | 49.075 byte |
| TV-04 | 110 đoạn | 0 | 11 | 34 | 107.022 byte |
| TV-05 | 105 đoạn | 0 | 17 | 45 | 100.052 byte |
| TV-06 | 107 đoạn | 0 | 17 | 39 | 94.143 byte |

- Dạng gọn tổng 238.049 byte, bằng 54% JSON bung ra.
- Validate sạch, 488/488 đoạn xuất hiện trên 510.432 lá số. Cung Mệnh hiện trung bình 18,1 đoạn, 5,8 đoạn chắc chắn, ít nhất 9, nhiều nhất 39.
- Tích hợp sau khi nạp phát hiện và sửa:
  - Neo Mệnh áp cho cung Thân làm đoạn về chính cung Mệnh hiện sai ở cung Thân: thêm fact `is_menh`.
  - V12 chặn câu phú, khiến 45 câu bị lược thành "…": nay câu phú phải nguyên văn, đã khôi phục từ trang.
  - Thiếu `NamMenh`.
  - Đọc ngược nghĩa "tuy cô bệnh tật" (tr. 68): câu song song tr. 78 in "vô".
- Không có cặp sao lặp giữa các batch: chương Mệnh viết tự sự theo từng sao, không lặp lời như chương Quan Lộc.
- Điều kiện sách dùng mà schema chưa diễn đạt, đang ở `condition_text`: sao của cung Thân khi đang xét Mệnh, "chỉ xung chiếu", ngũ hành sinh khắc giữa cung, Chính diệu và Bản Mệnh, sinh ban ngày/đêm, ngày âm 1–15, "Lộc" chưa rõ Lộc Tồn hay Hóa Lộc.

## 12. Quyết định

### 12.1. Đã chốt ngày 2026-11-07

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| D1 | Kiến trúc | Phương án A, lớp khóa trong Rust. Web dùng Cache Storage và `Map`, không dùng IndexedDB ở bản đầu |
| D2 | Mô hình Tử Vi | Đoạn có điều kiện thay khối 14 × 12 × độ sáng |
| D3 | Văn phong và bản quyền | Diễn đạt lại ngắn gọn, rõ nghĩa theo ngôn ngữ hiện đại để hiển thị trên di động, không chép nguyên đoạn dài. `quote` chỉ cho câu phú ngắn. Giữ `source_ref` gồm số trang và từ khóa gốc để máy kiểm. Kiểm tình trạng bản quyền trước khi phát hành, rồi mới đổi `rights` |
| D4 | Nguồn ngoài Tử Vi | Tử Vi: PDF *Tân Biên*. Trạch Nhật: `XIU_SONG`, `DAY_YI_JI` của `LunarUtil.ts` kết hợp *Ngọc Hạp Thông Thư*, *Hiệp Kỷ Biện Phương Thư*. Bát Tự, Phong Thủy: được tổng hợp 2–4 câu mỗi mục theo *Tam Mệnh Thông Hội* (Nạp Âm, Trường Sinh), *Tử Bình Chân Thuyên* (Thập Thần, Nhật Chủ), *Bát Trạch Minh Cảnh* (8 Quái, Bát Du Niên), *Thẩm Thị Huyền Không Học* (Cửu Tinh), đánh dấu `canonical_summary`, miễn kiểm từng câu theo trang |
| D5 | Hóa giải hạn | Không đưa nghi thức cúng sao, giải hạn. `advice` để trống, hoặc chỉ ghi lưu ý thực tế trong dân gian |
| D6 | Đương vận, thất vận | Bốn trạng thái ở mục 6, theo *Thẩm Thị Huyền Không Học*. Mỗi `NineStar` có hai trường văn bản `auspicious_meaning`, `inauspicious_meaning`; bỏ collection `nine_star_phase` |
| D7 | Hội hợp | `meet` gồm cùng cung, tam chiếu, xung chiếu. Nhị hợp là fact riêng `pair`. App vẫn hiện cung nhị hợp là cung cần xem, theo tr. 20 |
| D8 | Độ sáng phụ tinh | Đã làm: `TUVI_SPEC.md` mục 6.1, `TuViStar::brightness_at`, `FourTransformations::brightness_at`. Xem D13 |
| D9 | Nội dung nặng | Gắn thẻ `sensitivity` lúc nạp, giữ đúng nghĩa trong JSON. App ẩn sau một lần chạm các đoạn `Death`, `Sexuality`, kèm ghi chú đây là tham khảo văn hóa, không phải lời khuyên y tế hay pháp lý. Không nạp "Luận đoán chết và đám tang" tr. 269–270, "Luận số tiểu nhi" tr. 238–240 |

Dò bằng biểu thức chính quy, khoảng 200 đoạn có từ ngữ nặng: 76 ở cung Mệnh, 100 ở 11 cung còn lại, 20 ở Đặc tính các sao.

**Về D4.** D4 là ngoại lệ có chủ đích của Quy tắc 4 trong `CLAUDE.md`, chỉ áp dụng cho các batch ghi `canonical_summary` ở mục 11.3. Những sách đó không có trong dự án, nên mục tổng hợp không đối chiếu được với nguyên bản và phải được chủ dự án duyệt như mọi mục khác.

### 12.2. Chốt sau thử nghiệm

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| — | Dữ liệu thử | Duyệt toàn bộ TV-01, TV-15 và các đính chính lỗi in |
| D6a | Sinh Khí trong Cửu Tinh | Chỉ đúng 1 vận kế tiếp, `star == (period % 9) + 1`. Vận 9: Nhị Hắc thuộc nhóm Thất Vận |
| D6b | Văn bản của Thoái Khí | Dùng `auspicious_meaning` kèm nhãn Thoái khí. Chỉ sao Thất Vận mới dùng `inauspicious_meaning` |
| D10 | Mục 8.43 "Đào, Hồng tọa thủ tại Tý: Tuổi Tý, Dậu…" | Không nạp: Đào Hoa ở Tý chỉ có tuổi Hợi, Mão, Mùi, Hồng Loan ở Tý chỉ có tuổi Mão |
| D11 | "Khoa, Quyền, Lộc" ở 8.20, 9.2.4 | Phải hội đủ cả ba Hóa, trong cung hoặc tam hợp, xung chiếu: `transformations_all` |
| D12 | Đoạn "Nếu…" | Build gắn `tier` cho mỗi đoạn. App hiện đoạn `Primary` lên đầu, gom đoạn `More` có `condition_text` vào mục "Xem thêm" |

### 12.3. Chốt sau Task 5A.3

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| D13 | Bảng D8 của Task 5A.3 khác *Tân Biên* tr. 39 ở hai nhóm. Hỏa, Linh: bảng ghi Đắc tại Dần, Ngọ, Tuất, Mão, Tỵ; sách ghi Dần, Mão, Thìn, Tỵ, Ngọ. Xương, Khúc: bảng ghi Tỵ, Dậu, Sửu, Thân, Tý, Thìn; sách ghi Thìn, Tuất, Sửu, Mùi, Tỵ, Hợi | Giữ theo tr. 39 của sách, như đã cài. Chốt cùng 17 phụ tinh và Hóa Kỵ, và quy tắc mượn Chính diệu xung chiếu ở 8 cung |
| — | `rights` của *Tân Biên* | `paraphrased`: dữ liệu chỉ diễn đạt lại theo D3, không chép nguyên văn. V11 chặn mọi mục `quote` của nguồn này |

### 12.4. Chốt Task 5A.5

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| D14 | "Phục" đi cùng Song Hao ở TV-03 | Trong vòng Bác Sỹ, Tiểu Hao (3) và Đại Hao (9) xung nhau, Phục Binh (10) nằm sát Đại Hao nên không bao giờ hội hợp (tam hợp, xung chiếu) cùng Song Hao. Lỗi nhầm vị trí sao của sách, như D10: bỏ điều kiện Phục Binh khỏi tổ hợp Song Hao, ghi `correction` |
| D15 | "Lộc" khi sách không ghi rõ Lộc Tồn hay Hóa Lộc (mục 7.7) | Quy ước chuẩn: khớp Lộc Tồn HOẶC Hóa Lộc, tọa thủ hoặc hội hợp tùy ngữ cảnh. Fact mới `any_loc`, mục 7.3, 9. "Song Lộc" vẫn là có cả hai cùng hội hợp, không dùng `any_loc` |
| D16 | Giảm `condition_text` ở cung Mệnh sau TV-02 đến TV-06 (mục 11.6) | Thêm 6 trường `when` tính được từ `TuViChart`: `birth_time_of_day`, `lunar_day_range`, `opposite_only`, `in_than`, `any_loc`, `element_relation`, cùng fact tương ứng ở mục 9. Ngũ hành Chính Tinh lấy từ `profile.element` đã duyệt của `tuvi.star` (TV-01), không suy đoán thêm |

### 12.5. Chốt Task 5A.8

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| D17 | `rights` của 6 nguồn mới (`vn_custom_rules`, `lunar_ts`, `ngoc_hap_thong_thu`, `hiep_ky_bien_phuong_thu`, `tam_menh_thong_hoi`, `tu_binh_chan_thuyen`, `bat_trach_minh_canh`, `tham_thi_huyen_khong_hoc`) | `vn_custom_rules` (`open`, tài liệu của chính dự án), `lunar_ts` (`licensed`, MIT như đã khai ở `THIRD_PARTY_NOTICES.md`); bốn sách Hán cổ (`ngoc_hap_thong_thu`, `hiep_ky_bien_phuong_thu`, `tam_menh_thong_hoi`, `tu_binh_chan_thuyen`, `bat_trach_minh_canh`, `tham_thi_huyen_khong_hoc`) đặt `open`: đều là cổ thư hàng trăm năm đã thuộc phạm vi công cộng, và mục `canonical_summary` là văn bản tự tổng hợp, không trích nguyên văn một bản dịch hiện đại nào, nên không cần `unverified` chặn `build --release` |
| D18 | `bazi.can_chi` (60 cặp Hoa Giáp) không có nguồn riêng cho từng cặp ngoài Nạp Âm (đã ở `bazi.nayin`) và ngũ hành Can/Chi (đã ở core) | Ghi `gap` cho cả 60 cặp, không tự đặt lời tán riêng để tránh trùng dữ liệu core và tránh suy diễn ngoài nguồn, đúng tinh thần mục 3.5 và mục 5 |
| D19 | `bazi.nayin` cần cả `image` (dự kiến `paraphrase` từ *Tân Biên* tr. 241–243) và `traits`, `fortune` (dự kiến `canonical_summary` theo *Tam Mệnh Thông Hội*), nhưng một mục chỉ có một `source_ref` | Gộp cả ba trường vào một mục `canonical_summary` nguồn `tam_menh_thong_hoi`: bảng nguyên thể 30 Nạp Âm của *Tân Biên* vốn cũng chỉ là chép lại từ *Tam Mệnh Thông Hội*, nên quy về một nguồn là hợp lý hơn tách hai `source_ref` mà schema không cho phép |

### 12.6. Chốt Task 5B.4

| Mã | Vấn đề | Quyết định |
| :-- | :-- | :-- |
| D20 | Ứng dụng muốn cho người đọc thấy chỗ dữ liệu khác bản in (Task 5B.4), trong khi `correction` trước đây chỉ để duyệt | Từ 2026-11-14, `correction` của `tuvi.passage` vào file build: cột `passage.correction` của SQLite và trường `correction` của JSON Web. `schema_version` lên 2; app kiểm `user_version = 2`. `correction` của `entry` không đổi. Văn bản đính chính viết cho người duyệt nhưng đã là tiếng Việt rõ ràng có dẫn trang, nên hiện nguyên văn, thu gọn một dòng cho đến khi chạm |

## 13. Phạm vi

**Ngoài phạm vi Task 5A.1, có thể thêm sau mà không đổi schema:**

| Nội dung | Nguồn | Gợi ý |
| :-- | :-- | :-- |
| Cách cục, khoảng 50 cách như Tử Phủ Triều Viên | *Tân Biên* tr. 229–233, 6.336 ký tự | `tuvi.pattern`, dùng lại `when` |
| Vận hạn: sao nhập Đại Hạn, Tiểu Hạn | tr. 248–268, 41.482 ký tự | `tuvi.han_passage`, 2 batch |
| Sinh địa, Vượng địa, Bại địa, Tuyệt địa của cung Mệnh | tr. 52 | Cần thêm bảng vào core |
| Lời cho `FiveElementsBureau`, `MenhCucRelation`, `YinYangHarmony`, Tuần, Triệt | tr. 31–32 và các chương | Thêm collection nhỏ |
| Nghi Kỵ theo ngày | `DAY_YI_JI`, `TIME_YI_JI` | Đã nêu ngoài phạm vi ở `BAZI_ALMANAC_SPEC.md` mục 6.2 |

Không làm: tìm kiếm toàn văn, đa ngôn ngữ, cập nhật dữ liệu qua mạng, ghép câu tự động trên thiết bị.

**Việc tiếp theo:**

1. ~~Task 5A.4: skill `/ingest-content` với dạng viết gọn, chạy các batch còn lại.~~ Xong ở Task 5A.8: 11/11 batch AL/BZ/FS đã nạp và duyệt.
2. ~~Thêm `XIU_SONG` vào `THIRD_PARTY_NOTICES.md` khi nạp AL-02.~~ Xong ở Task 5A.8.
