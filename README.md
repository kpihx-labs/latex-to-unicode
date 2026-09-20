# latex-to-unicode

High-performance, zero-dependency LaTeX-to-Unicode mathematical renderer written in Rust.

Converts LaTeX math expressions and full Markdown documents with math blocks (`$...$` and `$$...$$`) into beautiful, high-fidelity Unicode text suitable for terminal user interfaces (TUI), CLI tools, editors, and web apps.

## Architecture

- `crates/core`: Zero-dependency Rust parser and Unicode mathematical renderer.
- `crates/cli`: Blazing-fast CLI utility (`latex-to-unicode`).
- `crates/wasm`: WebAssembly bindings compiled with `wasm-pack` for Node.js, Bun, and browsers.
- `crates/c-api`: C-ABI shared library (`liblatex_to_unicode_c_api.so`) for ultra-low latency LuaJIT FFI in Neovim.

## Features

- Over 200 mathematical symbols (Greek letters, logic, sets, calculus, operators, arrows).
- Multi-line 2D block rendering for matrices, cases, and systems (`pmatrix`, `bmatrix`, `cases`).
- Smart superscript and subscript conversions with multi-character handling.
- Fractions, binomial coefficients, roots (`\sqrt[n]{x}`), and box enclosures (`\boxed{...}`).
- Combining accents (`\vec{v}`, `\hat{x}`, `\bar{z}`, `\dot{u}`).
- Full Markdown document streaming and inline/block math transformation.

## License

MIT License. Copyright (c) 2026 Ivann H. KAMDEM POUOKAM (KpihX).
