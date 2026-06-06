# Open Source Practices

RestQS is meant to be easy to inspect, test, and change. Project policy files live near the code, so contributors can
find rules without external systems.

`LICENSE` contains the license text. `CONTRIBUTING.md` explains the contribution process. `SECURITY.md` gives the
private report path for security issues. `CODE_OF_CONDUCT.md` sets conduct rules. `CHANGELOG.md` records user-facing
releases. `SUPPORT.md` routes usage questions, public bugs, feature requests, and private security reports.

## Maintenance Goals

The crate keeps its public contract narrow. The core parser has no runtime dependencies. Database adapters live behind
Cargo features. Public error codes stay stable inside a compatible release line.

This structure gives contributors clear ownership boundaries. Parser changes modify parsing tests. Value changes modify
value tests. Adapter changes modify adapter tests and integration docs. Public API changes update README, `/docs`, and
the changelog.

## Review Standards

Pull requests stay small enough for careful review. A good contribution states the behavior change, why the change
belongs in this crate, the tests that prove the behavior, and any public API or error contract change.

Review focuses on correctness, clear boundaries, tests, and documentation. A change that mixes parsing with SQL
generation needs revision. A change that uses a raw query field as a SQL identifier needs revision.

## Dependency Policy

Runtime dependencies stay narrow. The core parser currently uses the Rust standard library only. Database adapters
belong behind Cargo feature flags.

A dependency needs clear maintenance value. Review the license, release activity, transitive tree, and RustSec status
before adoption. CI runs a dependency advisory audit.

## Documentation Policy

Documentation must explain the safety boundary. Examples must include a field catalog before parsing. SQL examples must
show bind values or explain that the repository owns bind calls.

Diagrams can clarify flow, boundaries, and adapter roles. Tables can summarize operators, value kinds, and error codes.
Long design explanations work best as paragraphs with small examples between them.

## Compatibility Policy

The crate follows semantic versioning. Patch releases fix behavior and docs. Minor releases add compatible APIs or
adapters. Major releases can change public APIs, error codes, or plan shape.

`RqsError::error_code()` values are compatibility-sensitive. Treat removal or meaning changes as breaking changes.
