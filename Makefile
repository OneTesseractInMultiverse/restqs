CARGO ?= cargo

.PHONY: all audit build check clippy coverage doc fmt fmt-check help lint package package-list publish-dry-run setup test test-doc verify

all: verify

help:
	@printf '%s\n' \
		'build           Build the crate' \
		'check           Run cargo check for all targets and features' \
		'clippy          Run Clippy with warnings denied' \
		'coverage        Run cargo llvm-cov with a 100 percent line gate' \
		'doc             Build docs.rs-style documentation' \
		'fmt             Format all Rust code' \
		'fmt-check       Check formatting' \
		'lint            Alias for clippy' \
		'package         Verify crate package contents' \
		'package-list    List files included in the crate package' \
		'publish-dry-run Run cargo publish dry-run' \
		'setup           Install local developer tools' \
		'test            Run all tests' \
		'test-doc        Run rustdoc examples' \
		'verify          Run the local quality gate'

setup:
	$(CARGO) install cargo-audit --locked
	$(CARGO) install cargo-llvm-cov --locked

build:
	$(CARGO) build --all-features

check:
	$(CARGO) check --all-targets --all-features

clippy:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

lint: clippy

coverage:
	$(CARGO) llvm-cov --all-features --all-targets --show-missing-lines --fail-under-lines 100

doc:
	RUSTDOCFLAGS="--cfg docsrs -D warnings" $(CARGO) doc --no-deps --all-features

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
	$(CARGO) test --all-targets --all-features

test-doc:
	$(CARGO) test --doc --all-features

audit:
	$(CARGO) generate-lockfile
	$(CARGO) audit
	@rm -f Cargo.lock

verify: fmt-check check lint test test-doc doc
