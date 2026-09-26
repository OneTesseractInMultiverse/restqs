# SQLite repository execution tests

This unpublished fixture executes the repository documented in `docs/integrations.md` through SQLx against a fresh
in-memory database per test. Its SQL builder is the same `examples/support/users.rs` module compiled by the unit suite.
It checks actual row order, multiple sort terms, filtering, fixed projection, pagination, and queries without sorting.
Each test has one assertion. The pool closes after each result, including query errors; no files or services are used.

From the repository root:

```sh
make test-sqlite
make verify-sqlite
```

The core crate and `make verify` remain dependency-free. This separate manifest and lockfile contain the fixture's
SQLx 0.8.6 and Tokio 1.53.1 dependencies, with bundled SQLite 3.46.0 from `libsqlite3-sys` 0.30.1. The fixture has been
checked on Rust 1.85 and stable. Both Rust CI jobs run formatting, Clippy, and these tests; the security job audits its
lockfile. Dependabot checks this manifest weekly. PostgreSQL and MySQL execution coverage remains separate work.

SQLx (MIT OR Apache-2.0) supplies the real database binding and row-decoding path. Tokio (MIT) supplies the async runtime.
Both are maintained upstream; SQLx 0.9 is available, but this fixture intentionally exercises the documented 0.8 API and
keeps Rust 1.85 compatibility. Their maintenance cost is isolated compilation time, a committed dependency lockfile,
and dependency update review. Neither adds parser or adapter runtime behavior to the published library. The lockfile
passed `cargo audit` when introduced. The lockfile keeps `yoke-derive` at 0.8.2 because 0.8.3 uses an API unavailable
on Rust 1.85; CI checks this minimum version so future dependency updates cannot silently break it.

Upstream: [SQLx](https://github.com/launchbadge/sqlx), [Tokio](https://github.com/tokio-rs/tokio).
