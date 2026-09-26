CARGO ?= cargo
PYTHON ?= python3
POLICY_MANIFEST := tools/test-policy/Cargo.toml
POLICY_CARGO = CARGO_TARGET_DIR=target/test-policy $(CARGO)
COVERAGE_TOOLCHAIN := 1.97.1
COVERAGE_VERSION := 0.9.1
SQLITE_MANIFEST := integration-tests/sqlite/Cargo.toml
SQLITE_CARGO = CARGO_TARGET_DIR=target/sqlite-integration $(CARGO)
SERVICE_MANIFEST := integration-tests/services/Cargo.toml
SERVICE_CARGO = CARGO_TARGET_DIR=target/service-integration $(CARGO)
PROPERTY_MANIFEST := integration-tests/robustness/Cargo.toml
PROPERTY_CARGO = CARGO_TARGET_DIR=target/robustness $(CARGO)
FUZZ_TOOLCHAIN ?= nightly-2026-09-24
FUZZ_SECONDS ?= 15
FUZZ_RUNS ?= 10000
FUZZ_OPTIONS = -max_total_time=$(FUZZ_SECONDS) -runs=$(FUZZ_RUNS) -max_len=4096 -timeout=5 -rss_limit_mb=1024 -seed=5394771 -dict=fuzz/rqs.dict

.PHONY: all audit build check clippy coverage coverage-setup test-policy verify-test-policy doc doc-internal fmt fmt-check help lint package package-list publish-dry-run setup test test-doc test-sqlite test-services test-properties verify verify-sqlite verify-services verify-properties fuzz-setup fuzz-smoke

all: verify

help:
	@printf '%s\n' \
		'build           Build the crate' \
		'check           Check default and all-feature configurations' \
		'clippy          Run Clippy with warnings denied' \
		'test-policy     Check Rust and Python test assertions' \
		'verify-test-policy Test, lint, and run the policy checkers' \
		'coverage-setup  Install pinned Rust and coverage tools' \
		'coverage        Run cargo llvm-cov with a 100 percent line gate' \
		'doc             Build documentation with and without sqlx' \
		'doc-internal    Build rustdoc including private implementation items' \
		'fmt             Format core and example Rust code' \
		'fmt-check       Check formatting' \
		'lint            Alias for clippy' \
		'package         Verify crate package contents' \
		'package-list    List files included in the crate package' \
		'publish-dry-run Run cargo publish dry-run' \
		'setup           Install local developer tools' \
		'test            Run all tests' \
		'test-doc        Run rustdoc examples' \
		'test-sqlite     Execute the isolated SQLite repository tests' \
		'verify-sqlite   Format-check, lint, and test the SQLite fixture' \
		'test-services   Execute opt-in PostgreSQL and MySQL tests (URLs required)' \
		'verify-services Format-check, lint, and execute the service fixture' \
		'test-properties Run reproducible parser and adapter properties' \
		'verify-properties Format-check, lint, and run the property fixture' \
		'fuzz-setup      Install pinned fuzzing tools' \
		'fuzz-smoke      Run bounded decoding, parsing, and adapter fuzz targets' \
		'verify          Run the local quality gate'

setup:
	$(CARGO) install cargo-audit --locked
	$(MAKE) coverage-setup

build:
	$(CARGO) build --all-features

check:
	$(CARGO) check --all-targets --no-default-features
	$(CARGO) check --all-targets --all-features

clippy:
	$(CARGO) clippy --all-targets --no-default-features -- -D warnings
	$(CARGO) clippy --all-targets --all-features -- -D warnings

lint: clippy

coverage-setup:
	rustup toolchain install $(COVERAGE_TOOLCHAIN) --profile minimal --component llvm-tools-preview
	cargo +$(COVERAGE_TOOLCHAIN) install cargo-llvm-cov --locked --version $(COVERAGE_VERSION)

coverage:
	cargo +$(COVERAGE_TOOLCHAIN) llvm-cov --all-features --all-targets --show-missing-lines --fail-under-lines 100 --fail-uncovered-lines 0

test-policy:
	$(POLICY_CARGO) run --manifest-path $(POLICY_MANIFEST) --locked
	$(PYTHON) .github/scripts/test_policy.py

verify-test-policy:
	$(CARGO) fmt --manifest-path $(POLICY_MANIFEST) --all -- --check
	$(POLICY_CARGO) clippy --manifest-path $(POLICY_MANIFEST) --all-targets --locked -- -D warnings
	$(POLICY_CARGO) test --manifest-path $(POLICY_MANIFEST) --locked
	$(PYTHON) -m unittest discover -s .github/scripts -p 'test_test_policy.py'
	$(MAKE) test-policy

doc:
	RUSTDOCFLAGS="--cfg docsrs -D warnings" $(CARGO) doc --no-deps --no-default-features
	RUSTDOCFLAGS="--cfg docsrs -D warnings" $(CARGO) doc --no-deps --all-features

doc-internal:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features --document-private-items

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

package:
	$(CARGO) package

package-list:
	$(CARGO) package --list

publish-dry-run: verify
	$(CARGO) publish --dry-run

test:
	$(CARGO) test --all-targets --no-default-features
	$(CARGO) test --all-targets --all-features

test-doc:
	$(CARGO) test --doc --no-default-features
	$(CARGO) test --doc --all-features

test-sqlite:
	$(SQLITE_CARGO) test --manifest-path $(SQLITE_MANIFEST) --locked

verify-sqlite:
	$(CARGO) fmt --manifest-path $(SQLITE_MANIFEST) --all -- --check
	$(SQLITE_CARGO) clippy --manifest-path $(SQLITE_MANIFEST) --all-targets --locked -- -D warnings
	$(MAKE) test-sqlite

test-services:
	$(SERVICE_CARGO) test --manifest-path $(SERVICE_MANIFEST) --locked -- --ignored

verify-services:
	$(CARGO) fmt --manifest-path $(SERVICE_MANIFEST) --all -- --check
	$(SERVICE_CARGO) clippy --manifest-path $(SERVICE_MANIFEST) --all-targets --locked -- -D warnings
	$(MAKE) test-services

test-properties:
	$(PROPERTY_CARGO) test --manifest-path $(PROPERTY_MANIFEST) --locked

verify-properties:
	$(CARGO) fmt --manifest-path $(PROPERTY_MANIFEST) --all -- --check
	$(PROPERTY_CARGO) clippy --manifest-path $(PROPERTY_MANIFEST) --all-targets --locked -- -D warnings
	$(MAKE) test-properties

fuzz-setup:
	rustup toolchain install $(FUZZ_TOOLCHAIN) --profile minimal --component rust-src
	$(CARGO) install cargo-fuzz --locked --version 0.13.2

fuzz-smoke:
	$(CARGO) fmt --manifest-path fuzz/Cargo.toml --all -- --check
	$(CARGO) metadata --manifest-path fuzz/Cargo.toml --locked --no-deps --format-version 1 > /dev/null
	mkdir -p fuzz/corpus/decoding fuzz/corpus/parsing fuzz/corpus/adapters
	cargo +$(FUZZ_TOOLCHAIN) fuzz run decoding fuzz/corpus/decoding fuzz/seeds/decoding -- $(FUZZ_OPTIONS)
	cargo +$(FUZZ_TOOLCHAIN) fuzz run parsing fuzz/corpus/parsing fuzz/seeds/parsing -- $(FUZZ_OPTIONS)
	cargo +$(FUZZ_TOOLCHAIN) fuzz run adapters fuzz/corpus/adapters fuzz/seeds/adapters -- $(FUZZ_OPTIONS)

audit:
	$(CARGO) generate-lockfile
	$(CARGO) audit
	$(CARGO) audit --file integration-tests/sqlite/Cargo.lock
	$(CARGO) audit --file integration-tests/services/Cargo.lock
	$(CARGO) audit --file integration-tests/robustness/Cargo.lock
	$(CARGO) audit --file fuzz/Cargo.lock
	$(CARGO) audit --file tools/test-policy/Cargo.lock
	@rm -f Cargo.lock

verify: fmt-check check lint test test-doc doc verify-test-policy
