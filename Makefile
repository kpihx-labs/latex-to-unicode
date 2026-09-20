.PHONY: all build test check fmt lint clean publish

all: check build

build:
	cargo build --release
	wasm-pack build crates/wasm --target web --out-dir pkg
	wasm-pack build crates/wasm --target nodejs --out-dir pkg-node

test:
	cargo test

check: test
	cargo check --all-targets
	cargo fmt --check

fmt:
	cargo fmt

lint:
	cargo clippy --all-targets

clean:
	cargo clean
	rm -rf pkg pkg-node

publish: check
	cargo publish -p latex-to-unicode
	cargo publish -p latex-to-unicode-cli
