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

The core crate remains dependency-free. `make verify` also builds the isolated syntax-policy tool, which has its own
dependencies. This separate manifest and lockfile contain the fixture's
SQLx 0.9.0 and Tokio 1.53.1 dependencies, with bundled SQLite 3.51.3 from `libsqlite3-sys` 0.37.0.
The fixture requires Rust 1.94 or newer and runs on stable CI. Core tests and generated properties still run on
Rust 1.85; driver dependency requirements do not change the published library's MSRV. The security job audits this
lockfile, and Dependabot checks the manifest weekly. PostgreSQL/MySQL execution uses the sibling `services` fixture.

SQLx (MIT OR Apache-2.0) supplies real database binding and row decoding; Tokio (MIT) supplies the async runtime.
Their maintenance cost is isolated compilation time, a committed lockfile, and dependency update review. Neither adds
runtime dependencies to the library. SQLx 0.9 requires `AssertSqlSafe` for dynamic SQL and no longer gives
`SqliteArguments` a lifetime parameter. The repository marker accepts only fixed SQL, trusted mapped columns, and
placeholders; request values stay separate binds. It is not a sanitizer for arbitrary SQL.

The former `yoke-derive` 0.8.2 MSRV workaround is no longer needed. See
[dependency maintenance](../../docs/dependencies.md) for current versions and upstream constraints.

Upstream: [SQLx](https://github.com/launchbadge/sqlx), [Tokio](https://github.com/tokio-rs/tokio).
