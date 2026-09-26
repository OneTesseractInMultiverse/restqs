# PostgreSQL and MySQL execution tests

This unpublished fixture tests SQLx repositories against PostgreSQL 17.6 and MySQL 8.4.6. CI pins their Docker image
digests. The PostgreSQL normal-query path uses the same `examples/support/users.rs` builder documented in the integration
guide. Separate catalog configuration explicitly authorizes regex; the public user example still disables it.

Each test creates a single-connection pool and a temporary `users` table, runs one behavioral assertion, then closes the
pool. Closure also runs after setup or query errors, and temporary tables disappear when the connection closes. No shared
persistent tables or databases are created. The suite is opt-in; it never treats missing connection URLs as success.

Use a disposable test database with permission to create temporary tables. For example:

```sh
docker run -d --name restqs-pg-tests -p 127.0.0.1:55432:5432 \
  -e POSTGRES_USER=restqs -e POSTGRES_PASSWORD=restqs-test -e POSTGRES_DB=restqs postgres:17.6
docker run -d --name restqs-mysql-tests -p 127.0.0.1:53306:3306 \
  -e MYSQL_ROOT_PASSWORD=restqs-root-test -e MYSQL_USER=restqs \
  -e MYSQL_PASSWORD=restqs-test -e MYSQL_DATABASE=restqs mysql:8.4.6
# Wait for both databases to be ready before running the suite.
export RESTQS_POSTGRES_URL='postgres://restqs:restqs-test@127.0.0.1:55432/restqs'
export RESTQS_MYSQL_URL='mysql://restqs:restqs-test@127.0.0.1:53306/restqs?ssl-mode=required'
make verify-services
# Remove only the disposable containers created above, including their anonymous volumes.
docker rm -fv restqs-pg-tests restqs-mysql-tests
```

`make test-services` runs only execution checks. A plain `cargo test --manifest-path integration-tests/services/Cargo.toml`
compiles the tests but leaves service tests ignored. The Make targets and stable CI pass `--ignored` explicitly.

The tests cover scalar comparisons, null equality/inequality, list expansion/exclusion, boolean and hostile text binds,
fixed projection and rejection of partial projection, ordering, limits, offsets, and regex. PostgreSQL checks native
case-sensitive and case-insensitive operators; MySQL tests no-flags regex and rejects unsupported flags. Results use
explicit values and ordering rather than assuming insertion order. SQLite conformance lives in the separate SQLite
fixture and additionally checks actual default result caps.

SQLx 0.9.0 (MIT OR Apache-2.0) provides PostgreSQL/MySQL drivers; Tokio (MIT) runs async execution. Their versions and all
transitives are committed in `Cargo.lock` and audited in CI. These maintained upstream dependencies add isolated compile
time, lockfile/update review, and disposable service startup costs. Dependabot checks the manifest weekly. Neither adds
dependencies to the published library. This fixture requires Rust 1.94 or newer; the core and SQLite 0.8.6 fixture retain
Rust 1.85 coverage. SQLx 0.9's optional RSA authentication feature is not enabled; MySQL uses TLS in local and CI examples.

SQLx 0.9 requires `AssertSqlSafe` for dynamically assembled SQL. The private executors accept only statements built from
fixed repository syntax, authorized mapped identifiers, and placeholders. Values are bound separately. This marker
performs no sanitization and must never be applied to raw request SQL. The hostile-value execution test guards this
boundary. SQLite's documented 0.8.6 integration does not require this marker.
