Mỗi khi có thay đổi ở các public struct/function trong `astro-core`:

1. Cập nhật các wrapper tương ứng trong `crates/astro-ffi` (UniFFI) và `crates/astro-wasm` (wasm-bindgen).
2. Chạy script `./build-ios.sh` để sinh lại file Swift bindings và `.xcframework`.
3. Chạy `wasm-pack build crates/astro-wasm --target web` để kiểm tra xem WebAssembly có biên dịch thành công không.
4. Báo cáo ngắn gọn các API mới vừa được expose.
