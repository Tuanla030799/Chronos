//! UniFFI's binding generator, built from the same `uniffi` as the astro-ffi library so the
//! generated Swift always matches the scaffolding compiled into it.
//!
//! `cargo run -p astro-ffi --features bindgen --bin uniffi-bindgen -- generate --library <lib> --language swift --out-dir <dir>`

fn main() {
    uniffi::uniffi_bindgen_main()
}
