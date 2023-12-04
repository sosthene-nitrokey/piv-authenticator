.NOTPARALLEL:

export RUST_LOG ?= info,cargo_tarpaulin=off
TEST_FEATURES ?=vpicc,pivy-tests,opensc-tests,rsa

.PHONY: build-cortex-m4
build-cortex-m4:
	cargo build --target thumbv7em-none-eabi

.PHONY: test
test:
	cargo test --features $(TEST_FEATURES)

.PHONY: check
check:
	RUSTLFAGS='-Dwarnings' cargo check --all-features --all-targets

.PHONY: lint
lint:
	cargo fmt --check
	cargo check --all-features --all-targets
	cargo clippy --all-targets --all-features -- -Dwarnings
	RUSTDOCFLAGS='-Dwarnings' cargo doc --all-features
	
.PHONY: tarpaulin
tarpaulin:
	cargo tarpaulin --features $(TEST_FEATURES) -o Html -o Xml

.PHONY: vpicc-example
vpicc-example:
	cargo run --example vpicc --features vpicc,rsa
	
.PHONY: ci
ci: lint tarpaulin
	
