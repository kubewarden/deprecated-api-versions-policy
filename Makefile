SOURCE_FILES := $(shell test -e src/ && find src -type f)
VERSION := $(shell sed --posix -n 's,^version = \"\(.*\)\",\1,p' Cargo.toml)

policy.wasm: $(SOURCE_FILES) Cargo.*
	cargo build --target=wasm32-wasip1 --release
	cp target/wasm32-wasip1/release/*.wasm policy.wasm

artifacthub-pkg.yml: metadata.yml Cargo.toml
	kwctl scaffold artifacthub --metadata-path metadata.yml --version $(VERSION) \
		--questions-path questions-ui.yml --output artifacthub-pkg.yml

annotated-policy.wasm: policy.wasm metadata.yml
	kwctl annotate -m metadata.yml -u README.md -o annotated-policy.wasm policy.wasm

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
test: check-policy-metadata check-policy-version fmt lint
	cargo test --workspace

.PHONY: expected-policy-metadata
expected-policy-metadata:
	cargo run --quiet --manifest-path crates/policy-metadata-helper/Cargo.toml -- --metadata-path metadata.yml build

.PHONY: check-policy-metadata
check-policy-metadata:
	cargo run --quiet --manifest-path crates/policy-metadata-helper/Cargo.toml -- --metadata-path metadata.yml check

.PHONY: clean
clean:
	cargo clean
	cargo clean --manifest-path crates/policy-metadata-helper/Cargo.toml
	cargo clean --manifest-path crates/versions/Cargo.toml
	rm -f policy.wasm annotated-policy.wasm artifacthub-pkg.yml
