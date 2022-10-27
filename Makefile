SOURCE_FILES := $(shell test -e src/ && find src -type f)

policy.wasm: $(SOURCE_FILES) Cargo.*
	cargo build --target=wasm32-wasi --release
	cp target/wasm32-wasi/release/*.wasm policy.wasm

annotated-policy.wasm: policy.wasm metadata.yml
	kwctl annotate -m metadata.yml -o annotated-policy.wasm policy.wasm

.PHONY: fmt
fmt:
	cargo fmt --all -- --check

.PHONY: lint
lint:
	cargo clippy --workspace -- -D warnings

.PHONY: e2e-tests
e2e-tests: annotated-policy.wasm
	bats e2e.bats

.PHONY: test
test: check-policy-version fmt lint
	cargo test --workspace

.PHONY: expected-policy-version
expected-policy-version:
	cargo run --quiet --manifest-path crates/policy-version-helper/Cargo.toml -- --manifest-path Cargo.toml build

.PHONY: check-policy-version
check-policy-version:
	cargo run --quiet --manifest-path crates/policy-version-helper/Cargo.toml -- --manifest-path Cargo.toml check

.PHONY: clean
clean:
	cargo clean
	cargo clean --manifest-path crates/policy-version-helper/Cargo.toml
	cargo clean --manifest-path crates/versions/Cargo.toml
	rm -f policy.wasm annotated-policy.wasm
