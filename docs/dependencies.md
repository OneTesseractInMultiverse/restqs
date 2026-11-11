# Dependency Maintenance

The published library has **no runtime or development dependencies**, including with its optional `sqlx` feature.
External drivers and test tools live in separate unpublished crates. Updating them does not raise the library's
Rust 1.85 minimum or change a previously published crate.

This inventory describes the 0.2.2 release checkout. The published
[`v0.2.0` snapshot](https://github.com/OneTesseractInMultiverse/restqs/tree/v0.2.0) predates the developer-dependency updates
in [the changelog](../CHANGELOG.md); its release notes retain the versions tested at that time.

## Current Direct Dependencies

Cargo manifests, lockfiles, and GitHub Action releases were reviewed on 2026-11-10. There were no open Dependabot
alerts or pull requests, but a fresh RustSec audit found RUSTSEC-2026-0459 in the SQLite fixture. The update replaces
its vulnerable bundled SQLite, advances Syn and cargo-audit, and refreshes compatible transitive dependencies.
The tables record versions used by this checkout; tool and image pins not changed below retain their earlier
verification dates. Committed lockfiles record transitive resolution.

| Dependency | Version | Scope |
| --- | --- | --- |
| [SQLx](https://crates.io/crates/sqlx) | 0.9.0 release for PostgreSQL/MySQL; upstream revision `10462fa4` for SQLite | Execution fixtures; Rust 1.94+ |
| [Tokio](https://crates.io/crates/tokio) | 1.54.1 | Async fixture runtimes |
| [Proptest](https://crates.io/crates/proptest) | 1.11.0 | Generated properties; Rust 1.85+ |
| [Syn](https://crates.io/crates/syn) | 3.0.10 | Syntax-aware assertion-policy checker |
| [proc-macro2](https://crates.io/crates/proc-macro2) | 1.0.107 | Source spans and tokens for the checker |
| [libfuzzer-sys](https://crates.io/crates/libfuzzer-sys) | 0.4.13 | Isolated fuzz entry points |

SQLite uses `libsqlite3-sys` 0.39.1 and bundled SQLite 3.54.1. Released SQLx 0.9.0 only accepts versions below 0.38,
so the SQLite fixture temporarily pins SQLx to upstream commit
[`10462fa4414c6c7f1ed1d39bb31678e3b726878c`](https://github.com/transact-rs/sqlx/commit/10462fa4414c6c7f1ed1d39bb31678e3b726878c).
This revision permits patched 0.39.x and includes upstream changes since the 0.9.0 release; the complete SQLite
execution suite validates that snapshot. PostgreSQL/MySQL keep the published 0.9.0 dependency. Both driver fixtures
require Rust 1.94 and run on stable CI; core, property, and policy-checker tests retain Rust 1.85 checks.

[RUSTSEC-2026-0459](https://github.com/RustSec/advisory-db/blob/main/crates/libsqlite3-sys/RUSTSEC-2026-0459.md)
concerns memory disclosure when opening a crafted database/WAL file pair. This repository uses fresh in-memory
fixtures, and the published RestQS library does not include SQLite. Applications own their database dependencies
and should audit their own lockfiles; upgrading RestQS alone does not upgrade their SQLite engine.

Remove the Git pin once a published SQLx release accepts patched `libsqlite3-sys` (at least 0.39.0; this checkout uses
0.39.1). Update the SQLite fixture and documented installation snippets together, retain a patched locked version,
and rerun the RustSec audit, driver MSRV check, and complete execution suite. Never suppress the advisory to return
to the older registry release. Fixture examples use SQLx 0.9's `AssertSqlSafe` only for repository-built statements;
request values stay separate binds.

## Tools and CI

| Tool or CI dependency | Pinned release |
| --- | --- |
| Coverage Rust toolchain | 1.98.1 |
| cargo-llvm-cov | 0.9.1 |
| cargo-audit | 0.23.0 |
| cargo-fuzz | 0.13.2 |
| cargo-semver-checks | 0.50.0 |
| Fuzz Rust toolchain | nightly-2026-09-27 |
| actions/checkout | v7.0.1, full commit pinned |
| actions/upload-artifact | v7.0.2, full commit pinned |
| rust-lang/crates-io-auth-action | v1.0.6, full commit pinned |
| PostgreSQL service image | 18.6, manifest digest pinned |
| MySQL service image | 26.7.0, manifest digest pinned |

GitHub Action pins resolve to the reviewed upstream release tags. The database versions identify the official Docker
images used for conformance. At the 2026-09-26 image review, MySQL's `26.7.1` image was unavailable even though its release
notes existed. Production database upgrades are application-owned; these services are disposable test databases.

The service fixture uses ctutils 0.4.8, rustls 0.23.46, rustls-webpki 0.103.16, and webpki-roots 1.0.12 to resolve
RUSTSEC-2026-0385, RUSTSEC-2026-0346 through RUSTSEC-2026-0349, RUSTSEC-2026-0382, and RUSTSEC-2026-0381.
All six manifest scopes must pass the release audit.

## Transitive Constraints

Update to the latest versions accepted by the upstream dependency graph and supported toolchains; do not force a new
major version into a dependency that requires an older incompatible API. Current exceptions are:

- Proptest remains at 1.11.0 for the property fixture's Rust 1.85 coverage. Version 1.12.0 requires Rust 1.88.
- Some driver/property transitive dependencies still require Syn 2. The direct checker uses Syn 3; replacing a nested
  `^2` requirement with 3 is an upstream migration, not a lockfile refresh.
- The released PostgreSQL/MySQL driver's SHA-2 stack uses crypto-common 0.1.7, which requires **exactly**
  generic-array 0.14.7. Updating generic-array to 0.14.9 is rejected by Cargo. Prefer the newer compatible crypto-common and preserve its exact constraint.
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
