.PHONY: build test fmt lint clean

# Build the contract to WASM via Stellar CLI
build:
	stellar contract build

# Run tests against the native host.
# .cargo/config.toml defaults to wasm32-unknown-unknown for `stellar contract build`,
# so we override the target here to run tests natively.
test:
	cargo test --target $$(rustc -vV | sed -n 's/^host: //p')

# Format all Rust code
fmt:
	cargo fmt --all

# Lint with Clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Remove build artefacts
clean:
	cargo clean
