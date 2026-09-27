# Dependency Maintenance

The published library has **no runtime or development dependencies**, including with its optional `sqlx` feature.
External drivers and test tools live in separate unpublished crates. Updating them does not raise the library's
Rust 1.85 minimum or change a previously published crate.

## Current Direct Dependencies

Checked against crates.io and upstream release metadata on 2026-09-26. Versions below are the latest stable releases
available at that check; prereleases are not selected. Committed lockfiles record transitive resolution.

| Dependency | Version | Scope |
| --- | --- | --- |
| [SQLx](https://crates.io/crates/sqlx) | 0.9.0 | SQLite and PostgreSQL/MySQL execution fixtures; Rust 1.94+ |
| [Tokio](https://crates.io/crates/tokio) | 1.53.1 | Async fixture runtimes |
| [Proptest](https://crates.io/crates/proptest) | 1.11.0 | Generated properties; Rust 1.85+ |
| [Syn](https://crates.io/crates/syn) | 3.0.6 | Syntax-aware assertion-policy checker |
| [proc-macro2](https://crates.io/crates/proc-macro2) | 1.0.107 | Source spans and tokens for the checker |
| [libfuzzer-sys](https://crates.io/crates/libfuzzer-sys) | 0.4.13 | Isolated fuzz entry points |

SQLite uses `libsqlite3-sys` 0.37.0 and bundled SQLite 3.51.3 through SQLx. Both driver fixtures use the same SQLx release
and run on stable CI. Core, property, and policy-checker tests retain Rust 1.85 checks. Fixture examples use SQLx 0.9's
`AssertSqlSafe` only for repository-built statements; request values stay separate binds.

## Tools and CI

| Tool or CI dependency | Pinned release |
| --- | --- |
| Coverage Rust toolchain | 1.98.1 |
| cargo-llvm-cov | 0.9.1 |
| cargo-audit | 0.22.2 |
| cargo-fuzz | 0.13.2 |
| cargo-semver-checks | 0.50.0 |
| Fuzz Rust toolchain | nightly-2026-09-27 (latest published nightly manifest at the check) |
| actions/checkout | v7.0.1, full commit pinned |
| actions/upload-artifact | v7.0.1, full commit pinned |
| rust-lang/crates-io-auth-action | v1.0.5, full commit pinned |
| PostgreSQL service image | 18.6, manifest digest pinned |
| MySQL service image | 26.7.0, manifest digest pinned |

GitHub Action pins already matched their latest upstream releases. The database versions identify the latest available
stable official Docker images used for conformance; MySQL's `26.7.1` image was unavailable at the check even though its
release notes exist. Production database upgrades are application-owned; these services are disposable test databases.

## Transitive Constraints

Update to the latest versions accepted by the upstream dependency graph and supported toolchains; do not force a new
major version into a dependency that requires an older incompatible API. Current exceptions are:

- Some driver/property transitive dependencies still require Syn 2. The direct checker uses Syn 3; replacing a nested
  `^2` requirement with 3 is an upstream migration, not a lockfile refresh.
- SQLx's SHA-2 stack uses crypto-common 0.1.7, which requires **exactly** generic-array 0.14.7. Updating generic-array to
  0.14.9 is rejected by Cargo. Prefer the newer compatible crypto-common and preserve its exact constraint.
- The property fixture keeps wasip2 1.0.1+wasi-0.2.4 under its Rust 1.85 resolution policy. The newer
  1.0.4+wasi-0.2.12 requires Rust 1.87 and is not compatible with that fixture's declared minimum.

Other older major versions can coexist when required by upstream ranges. `cargo update --verbose` reports available
alternatives; inspect reverse dependency trees before changing constraints. A passing audit is distinct from having
no newer releases, and current versions still need behavior tests.

## Update Procedure

Dependabot checks all six Cargo manifest scopes and GitHub Actions weekly. Review image and tool pins against upstream
release metadata as part of dependency maintenance; `latest` tags are discovery aids, never committed deployment pins.

1. Check stable releases on crates.io, GitHub release/tag references, official Docker Hub tags/digests, and Rust channel
   manifests. Confirm the minimum Rust version and migration requirements before changing a manifest.
2. Update exact direct versions, then run `cargo update --manifest-path <path>/Cargo.toml --verbose` for each isolated
   crate. Review lockfile additions/removals, licenses, RustSec status, and any constrained older versions.
3. Preserve full action commit hashes and image manifest digests. Update Makefile/CI tool pins together and keep
   installation and integration docs consistent with executable examples.
4. Run `make verify`, `make verify-sqlite`, `make verify-properties`, `make verify-services` with disposable databases,
   `make fuzz-smoke`, `make coverage`, and `make audit`. Keep MSRV tests and the package-content check.
5. Record changed versions, upstream constraints, and validation in the PR. A dependency-only developer-tool update
   does not require republishing an unchanged, dependency-free library.
