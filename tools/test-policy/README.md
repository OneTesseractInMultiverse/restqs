# Test policy checker

Run `make test-policy` from the repository root to inspect Rust and Python source, or `make verify-test-policy` to also
format-check, lint, and test the checkers. The Rust CLI accepts explicit paths for isolated reproductions; no arguments
uses Git's tracked and unignored source inventory. Diagnostics include path, line, column, and assertion count.

The Rust library is pure syntax analysis. The CLI owns Git/filesystem reads and output. The Python companion uses the
standard-library AST and the same source inventory convention. Neither evaluates test input or expands procedural macros.
See [the testing guide](../../docs/testing-guide.md#one-assertion-rule) for the full policy and documented limitations.

This unpublished tool has an isolated lockfile. `syn` 3.0.6 parses Rust syntax and `proc-macro2` 1.0.107 retains token
structure and source locations. Both are MIT/Apache-2.0, actively maintained Rust ecosystem crates. The checker is tested
on Rust 1.85 in CI; Syn's `full` and `visit` features provide function/attribute traversal. The small
transitive tree (`quote`, `unicode-ident`) is locked, audited by RustSec in CI, and monitored by Dependabot. These
build-time developer dependencies never enter the library package or runtime parser/adapter boundary. Updating their
versions requires rerunning the checker regressions, repository scan, MSRV check, and audit.

Regression tests use in-memory source strings and exactly one assertion each. They cover zero/multiple assertions,
comments and raw strings, source locations, async/conditional/ignored tests, nested helpers/closures/macros, generated
tests, malformed Rust, Python exception checks, and fuzz targets. Helpers prepare values and never assert.
