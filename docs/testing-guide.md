# Testing Guide

RestQS tests mirror the library boundary. Parser tests check RQS input and returned plans. Catalog tests check
identifier validation. Value tests check type parsing. Adapter tests check generated fragments and bind order.

The ordinary unit suite runs in memory without database libraries. It needs no web servers, credentials, environment
variables, or special setup. A contributor can clone the repository, install local tools with `make setup`, and run `make test`.

```mermaid
flowchart LR
  catalog["Catalog tests"] --> gate["make verify"]
  parser["Parser tests"] --> gate
  value["Value tests"] --> gate
  adapter["Adapter tests"] --> gate
  docs["Doc tests"] --> gate
```

## One Assertion Rule

Each test function uses one assertion. A test can prepare data, parse input, and call helper functions. The final
verification checks one fact.

```rust
use restqs::{FieldCatalog, FilterOp, parse};

let catalog = FieldCatalog::new().allow_integer("age")?;
let query = parse("age>=18", &catalog)?;

assert_eq!(query.filters()[0].op(), FilterOp::Gte);
# Ok::<(), restqs::RqsError>(())
```

Run `make test-policy` to check every tracked or unignored Rust and Python source file, including the isolated fixtures
and the checker itself. Rust syntax and token trees identify `#[test]`, namespaced `#[tokio::test]`-style attributes,
`cfg_attr` tests, ignored tests, and disabled feature/platform branches. Comments and string contents do not count.
Each explicit test must contain exactly one `assert!`, `assert_eq!`, `assert_ne!`, debug-assert, or proptest assertion;
closures and nested macro arguments count toward that test. Nested functions are checked separately. Fuzz targets also
require one assertion. Python uses its AST to count `assert` and unittest/mock assertion calls, including exception
context managers. Files that cannot be parsed fail the check.

Helpers return values or errors for the test to assert; assertions in non-test Rust/Python functions are rejected.
Opaque item macros, custom assertion macros, and generated/parameterized test attributes are rejected until the checker
has explicit support with regression tests. Use ordinary named tests and property runners that return a result, as in
the robustness fixture. The allowlist covers standard value/format macros and `prop_oneof!`; it does not expand arbitrary
macros or execute code. Rustdoc/Markdown snippets and externally generated code are reviewed separately, not counted as
source test functions. Python tests use the `test_` naming convention and ordinary unittest functions, without generated
tests or assertion aliases. This structural gate cannot prove that an assertion checks one meaningful fact; review must.

This rule follows the library design. A function either coordinates work or computes a value. A test either proves one
behavior or one failure mode.

## Correctness Tests

Coverage alone does not prove behavior. Tests must prove correctness. Each operator needs a positive case. Each failure
mode needs a negative case. Each adapter path needs expected fragment text and bind order.

Compare the complete ordered `RqsValue` vector in a bind test. A length-only assertion cannot detect reordered or
replaced values. Cover list expansion between scalar filters and before a scalar filter, including unsorted and
repeated list values. Keep each expected sequence in its test and setup helpers free of assertions.

Use concrete input. Prefer `age>=18` over synthetic placeholders. Prefer a real catalog entry such as `users.age` over a
vague `table.column` example.

## Failure Tests

Failure tests are part of the public contract. They prove safe defaults and error boundaries.

Important failure cases include:

- Unknown public fields.
- Duplicate catalog names, including identical definitions and changes to type or regex permission.
- Invalid database column identifiers.
- Missing or duplicate adapter column mappings, including sort-only and projection-only plans.
- Wrong value type for a field.
- Duplicate filters with the same field and operator.
- Regex on a field that does not allow regex.
- Unknown or duplicate regex suffix flags, and recognized flags unsupported by each SQL dialect.
- Text search through `$text=`.
- Pagination above the configured limit.
- Too many parameters or list items.

Error-code tests track compatibility-sensitive `RqsError::error_code()` values. Keep display tests separate: compare
rendered messages against concrete expected text for each relevant error variant. Display wording can change for
clarity when the corresponding tests are updated deliberately.

Safety tests use hostile field and column names to check newline, carriage-return, NUL, and terminal-escape redaction.
Also verify that malformed input and invalid or duplicated filter values do not appear in rendered messages. Preserve
the distinction between valid identifiers, which remain visible within the display byte limit, and malformed or long
identifiers, which become `[redacted]`. Counting messages does not test any of these properties.

## Repository Example Checks

The logical plan and column-mapping tests verify that SQL configuration cannot authorize fields, and that one plan
can be consumed with different SQL schemas or by the [in-memory example](../examples/in_memory.rs). Run the latter
without SQL support using `cargo run --example in_memory --no-default-features`.

The [repository pagination tests](../tests/repository_pagination.rs) compile the same helper module used by the SQLx
integration examples. They verify SQL and complete bind sequences for PostgreSQL and SQLite without database services
or a SQLx dependency. Run them with `cargo test --all-features --test repository_pagination`; `make test` includes them.

The [user repository tests](../tests/repository_users.rs) compile the documented fixed-response builders. They check
that accepted projections select both decoder columns and that partial, extra, and unauthorized selections fail before
execution. Run `cargo test --all-features --test repository_users` or `make test`.

## SQLite Execution Checks

Run `make verify-sqlite` for the isolated SQLx fixture, or `make test-sqlite` to run only its tests. A separate manifest
and committed lockfile keep these dependencies outside the published library and `make verify`. The fixture uses a
fresh in-memory SQLite database per test and closes its pool even when the query returns an error. It checks actual
returned ordering against explicit values, including tie-breaking and sorting before pagination.

See [the fixture README](https://github.com/OneTesseractInMultiverse/restqs/tree/main/integration-tests/sqlite) for tested
versions and dependency rationale. Stable Rust CI verifies this SQLx 0.9 fixture (Rust 1.94 minimum),
and the security job audits its lockfile. No database service setup is required. PostgreSQL and MySQL execution suites are opt-in locally and run in the required stable CI job.

## PostgreSQL and MySQL Execution Checks

See [the service fixture README](https://github.com/OneTesseractInMultiverse/restqs/tree/main/integration-tests/services)
for versions, disposable Docker commands, and connection URLs. `make verify-services` checks formatting, lints, and runs
all service tests explicitly. Each uses a fresh temporary table and single-connection pool with cleanup after errors.
No URL means a failing setup, never a silently skipped success. Plain Cargo tests leave these service tests ignored.

The service fixture uses SQLx 0.9.0 on Rust 1.94+; the core, properties, and policy checker still run on Rust 1.85. Stable CI gates merges
on both database services as well as SQLite. Every fixture has a separate audited lockfile and no core dependencies.

## Generated Properties and Fuzzing

`make verify-properties` runs the isolated proptest fixture: bounded Unicode and structured queries, configurable limit
boundaries, resolved plan metadata, valid operator/value combinations, bind contents, placeholder numbering, and value/SQL
separation. Default runs use 256 cases per property and seed 5394771 on stable and Rust 1.85. Each test has one assertion.
For longer local runs use `PROPTEST_CASES=10000 PROPTEST_RNG_SEED=2026 make test-properties`.

`make fuzz-setup` installs pinned tooling; `make fuzz-smoke` runs decoding, parsing, and adapter targets with committed
regression seeds and a syntax dictionary. Each smoke target stops after 10,000 executions or 15 seconds, with 4 KiB
inputs, per-input timeout, and memory limits. Stable CI runs the smoke suite in the required Rust job. The library and
ordinary unit suite acquire no property-testing or fuzzing dependencies.

See [the property fixture](https://github.com/OneTesseractInMultiverse/restqs/tree/main/integration-tests/robustness) and
[the fuzz guide](https://github.com/OneTesseractInMultiverse/restqs/tree/main/fuzz) for shrinking, artifact replay, longer
runs, and adding minimized failures to the regression corpus. CI preserves failing inputs for 14 days. These bounded
runs complement example tests and source coverage; they cannot establish exhaustive safety.

## Coverage Command

Run coverage after installing local tools:

```sh
make setup
make coverage
```

Coverage uses pinned Rust 1.98.1 with `llvm-tools-preview` and cargo-llvm-cov 0.9.1 (`make coverage-setup` installs both).
The command checks every measured file under the library's `src/` with all features and all root test targets, requiring
100% total line coverage and zero uncovered lines across the measured files. Example executables, external fixtures, checker tooling, doctests, and
compiler-generated code are outside this source-line metric; their separate suites still run. No changed-line exemption
or reduced threshold is used. A newly uncovered source line fails the gate.

The coverage command fails on any uncovered source line. The numeric region summary can still show missed compiler
subregions from error propagation. The gate uses source-line coverage, which matches the project goal for readable Rust
code.

## Local Gates

Run the main test suite:

```sh
make test
```

Run the full local gate:

```sh
make verify
```

`make verify` checks formatting, type checking, Clippy, tests, doc tests, docs.rs-style documentation, and
`make verify-test-policy` (checker format/lint/tests plus repository assertions). Run `make coverage` separately before
sending a pull request; it requires the pinned coverage tools.

The `Protect main` ruleset requires `Rust stable`, `Rust 1.85.0`, `Security audit`, and `Quality policies`, all from GitHub
Actions. `Quality policies` runs `make verify-test-policy` and `make coverage` on every PR and main push, without path
filters. The reusable release CI uses the same job at the resolved release commit. Workflow YAML alone does not enforce
merge protection: inspect `gh api repos/OneTesseractInMultiverse/restqs/rulesets/17356771` when changing job names.

## Test Review

Review tests with the same care as source code. A good test has a clear name, one assertion, no external service, and a
direct link to required behavior. A test that only increases coverage without proving behavior needs revision.

## Documentation Checks

`make test-doc` compiles and runs Rust examples from crate rustdoc, the README, and the API guide with both feature
selections; adapter, security, and migration-guide examples run with `sqlx`. The historical 0.1 constructor snippet is
intentionally marked `ignore`; its 0.2 replacements are compiled. Driver execution code lives in the isolated conformance fixtures
linked by the integration guide. Change those sources and run the appropriate fixture instead of maintaining a
second driver implementation in Markdown.

`make doc` denies rustdoc warnings and broken intra-doc links. `make doc-internal` includes private implementation
items for maintainer review. Clippy denies missing private-item docs across the library, fixtures, and Rust policy tool; public API docs already fail compilation
when missing. Document contracts, failure conditions, ordering, and trust boundaries rather than paraphrasing syntax.
Test function names describe their one expected fact; module docs explain suite scope and helper docs explain setup.

Before a release, inspect `cargo package --list` for obsolete or unintended files and run Clippy across the core and
isolated crates. Delete unused implementation only when compiler/reference checks establish it has no supported role;
preserve migration notes, historical changelog entries, and deliberately defensive adapter checks.
