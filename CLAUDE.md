# Viet Astro Engine - Project Instructions & Guidelines

## 1. Project Overview & Tech Stack

- **Core Engine (`astro-core`)**: Pure Rust (Edition 2021+), `no_std` compatible or minimal `std`, 100% mathematical astronomy.
- **iOS Bridge (`astro-ffi`)**: Mozilla UniFFI for Swift / SwiftUI integration.
- **Web Bridge (`astro-wasm`)**: `wasm-bindgen` and `serde-wasm-bindgen` for WebAssembly.
- **Timezone Standard**: Default timezone is **GMT+7 (Vietnam, Standard Meridian 105.0° East)**. All astronomical calculations must accept configurable timezone offset.

## 2. Core Domain Rules (Strictly Enforced)

1. **Lunar Calendar Accuracy**: Algorithms must follow Jean Meeus / Hồ Ngọc Đức standards, strictly handling New Moon (Sóc) points in GMT+7 and leap months.
2. **Solar Terms (24 Tiết Khí)**: Exact calculation based on Solar Longitude (e.g., Lập Xuân at 315.0°) solved via numerical convergence.
3. **True Solar Time & Bazi**: Support equation of time and geographic longitude adjustments for true local solar time (crucial for Early/Late Rat Hour - Giờ Tý sớm/muộn).
4. **Zero Hallucination on Astrology Rules**: When implementing Bazi (Tứ Trụ) or Zi Wei Dou Shu (Tử Vi), rely _strictly_ on explicit data matrices/lookup tables provided via NotebookLM transcripts or approved source texts (e.g., _Tử Vi Đẩu Số Tân Biên_). Do not invent rules.

## 3. Tool Usage & External Knowledge (Mandatory)

- **Context7 MCP**: Whenever writing or debugging code involving external Rust crates (like `uniffi`, `wasm-bindgen`, `chrono`, etc.) or modern **SwiftUI / WidgetKit / Combine** APIs, you MUST use the Context7 MCP tool to fetch the latest official documentation and correct syntax before generating code. Do not rely on outdated training memory for third-party libraries.
- **XcodeBuildMCP**: Use XcodeBuildMCP to build, test, and inspect iOS targets and simulator states when working on Phase 5 (UI).

## 4. Git Commit & PR Hygiene (Strict)

- **No Co-Authored-By**: NEVER include `Co-Authored-By: Claude` or any attribution to Anthropic/Claude in git commit messages or pull requests. Keep commit history clean and professional.
- **Commit Style**: Use Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `chore:`).

## 5. Engineering Standards

- **Strong Typing**: Leverage Rust's `enum` and `struct` exhaustively (e.g., `enum CelestialStem`, `enum EarthlyBranch`, `enum SolarTerm`).
- **Test-Driven Development**: Every new module or function in `astro-core` must include corresponding `#[cfg(test)]` Unit Tests covering edge cases (leap years, timezone boundaries).
- **No Placeholders**: Never use `TODO`, `// implement later`, or stub functions. Write complete, production-ready code.
