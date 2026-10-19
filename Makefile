.PHONY: build test check fmt clean

build:
	cargo build --target wasm32v1-none --release --locked
	mkdir -p target/wasm32v1-none/optimized
	stellar contract optimize \
		--wasm target/wasm32v1-none/release/blend_v2_1_receiver.wasm \
		--wasm-out target/wasm32v1-none/optimized/blend_v2_1_receiver.wasm

test:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features --locked -- -D warnings
	cargo test --all-features --locked

check:
	cargo check --all-targets --all-features --locked

fmt:
	cargo fmt --all

clean:
	cargo clean
