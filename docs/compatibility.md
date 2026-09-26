# Compatibility Policy

RestQS follows Cargo's compatible release lines and adds explicit guarantees for query behavior, error codes, and the
minimum supported Rust version (MSRV). Being pre-1.0 does not make patch releases unrestricted. This policy applies to
releases from 0.2 onward; it does not change previously published artifacts. The migration baseline for the 0.2.0
release is the published 0.1.1 artifact recorded below.

## Release Lines

| Version change | RestQS policy |
| --- | --- |
| `0.1.1` → `0.1.2` | Compatible fixes and compatible additions; preserve existing Rust APIs, supported query behavior, error-code meanings, and MSRV, subject to the security exception below. |
| `0.1.x` → `0.2.0` | New incompatible line. API or behavior changes require a migration guide. Cargo requirements such as `restqs = "0.1"` do not select it automatically. |
| `0.2.0` → `0.2.1` | Same compatibility promise within the 0.2 line. A further intentional break requires `0.3.0`. |
| `1.2.3` → `1.2.4` | Compatible fixes and documentation updates. |
| `1.2.3` → `1.3.0` | Compatible new APIs/features; existing supported uses continue working. |
| `1.x` → `2.0.0` | Incompatible API, behavior, or MSRV changes, with migration notes. |

Cargo treats a change in the leftmost nonzero version component as incompatible; it does not inspect behavior.
`0.0.x` versions have no shared compatible line and are not used for supported releases here. Prerelease versions are
explicit opt-ins whose APIs may change before the final release. See the
[Cargo compatibility reference](https://doc.rust-lang.org/cargo/reference/semver.html).

The project supports vulnerability fixes on the latest published minor line, as stated in [SECURITY.md](../SECURITY.md).
A compatibility promise within a line does not promise indefinite maintenance of older lines. Unreleased `main` is not
a supported release; applications using a Git/path dependency must review its migration notes before updating.

## Rust API Shape

A change is incompatible when existing supported downstream source stops compiling. Review public paths, signatures,
trait implementations and bounds, feature availability, and construction/pattern-matching syntax, including these cases:

| Change | Classification |
| --- | --- |
| Remove/rename an item, change an argument/type, tighten a bound, remove `Copy` | Incompatible. The 0.2 field constructors and SQL adapter changes are examples. |
| Add a field to a struct whose fields are all public, such as `ParserLimits` | Incompatible: downstream struct literals can break, even with `Default` or an optional field type. Adding a private field also breaks those literals. |
| Add a variant to an exhaustive enum such as `RqsValue`, `ValueKind`, `FilterOp`, or `SqlDialect` | Incompatible: downstream exhaustive matches can break. Changing variant fields can also be incompatible. |
| Add `#[non_exhaustive]` to an already published exhaustive type | Incompatible; the attribute is not a retroactive compatibility escape hatch. |
| Add a variant to the already non-exhaustive `RqsError` | Usually source-compatible; still review its effect on accepted input and error-code behavior separately. Existing code must keep a fallback match arm. |
| Add an independent method, type, or opt-in feature without changing existing signatures or behavior | Compatible after checking trait resolution, feature interactions, and downstream usage. |

Private implementation refactors do not require an incompatible line when observable behavior stays the same. SQL
mapping and application/database execution remain adapter/repository responsibilities; the core plan remains independent
of storage. Renaming fields in the public plan or changing its interpretation is a contract change even if parsing still
succeeds.

## Query and Error Behavior

Compatibility covers supported query syntax, value interpretation, documented validation precedence, parser limits,
plan contents, and adapter semantics, including placeholder/bind correspondence. A previously supported query must keep
its meaning within a compatible line. Removing syntax, tightening a default limit, or rejecting previously supported
values normally requires an incompatible release. Adding syntax must not reinterpret existing valid queries.

Existing `RqsError::error_code()` strings and meanings stay stable within a compatible line. Removing/renaming/reusing a
code, or intentionally assigning a different code to an already specified failure, is incompatible. A code for a new
opt-in feature can be additive. New stricter validation needs a behavior review even when its Rust enum addition is
source-compatible. Human-readable `Display` wording and `Debug` formatting are not stable machine interfaces; use
`error_code()` for classification. Redaction guarantees must still hold when messages change.

Examples are integration guidance rather than additional public Rust APIs, but changes affecting authorization,
projection, pagination, or returned rows still need tests and release/migration notes. Execution timeouts, result budgets,
and database behavior remain application-owned.

## Stricter Validation and Security Fixes

The default is to put intentional accepted-input or validation-precedence changes in an incompatible line. The 0.2
migration therefore records duplicate controls/catalog names, reserved names, finite floats, list/operator rejection,
and regex-flag enforcement alongside the source API changes. Calling a change a bug fix does not by itself exempt it.

A compatible patch may reject input that was already explicitly documented as invalid, or close a demonstrated security
boundary bypass, when preserving the old behavior would leave users exposed. This narrow exception is for validation
and safe failure behavior; it does not permit unrelated source API breaks or an MSRV increase. Prefer preserving existing
error codes for existing failure classes. If a security correction must change acceptance or error classification, explain
the affected requests, security rationale, resulting code, and client migration in the changelog and migration notes;
coordinate disclosure privately when necessary. Do not silently present it as behavior-preserving.

The published 0.1.1 release already tightened calendar validation, operator boundaries, regex/operator combinations, and
value-size enforcement, and corrected SQL null semantics and error redaction. Its changelog remains the historical
record. Version 0.2.0 deliberately groups further validation changes with its incompatible API migration.

## Rust and Tooling Versions

The library MSRV is the `rust-version` in `Cargo.toml`, currently **Rust 1.85**, tested with and without `sqlx`. Keep that
MSRV for all patches in a `0.x` line. Raising the library MSRV requires the next `0.x` line before 1.0, and a major version
after 1.0 under this project's stricter guarantee. Announce the old/new MSRV and reason in release notes and update CI.

Developer tooling and unpublished fixtures may require newer Rust when their requirements are explicit. The SQLx 0.9
service fixture requires Rust 1.94; the coverage job uses pinned Rust 1.97.1; fuzzing uses pinned nightly. These do not
raise the library MSRV or add dependencies to its published manifest. Application dependencies may impose their own
MSRV. Keep installation examples clear about that distinction.

## Public API Comparison Baseline

Use the published **0.1.1** release as the baseline for preparing 0.2.0:

- Tag: [`v0.1.1`](https://github.com/OneTesseractInMultiverse/restqs/releases/tag/v0.1.1).
- Peeled commit: `973f2167facc75727e67a19f25bcc819511798d7` (the annotated tag object has a different ID).
- Manifest: version `0.1.1`, edition `2024`, Rust `1.85`, empty default features, optional `sqlx`.
- Released API: [restqs 0.1.1 rustdoc](https://docs.rs/restqs/0.1.1/restqs/).

From a full Git checkout, install the pinned comparison tool and run both feature surfaces:

```sh
cargo +1.97.1 install cargo-semver-checks --locked --version 0.50.0
git fetch origin tag v0.1.1
git rev-parse 'v0.1.1^{commit}'
cargo +1.97.1 semver-checks --baseline-rev 973f2167facc75727e67a19f25bcc819511798d7 --only-explicit-features --release-type patch
cargo +1.97.1 semver-checks --baseline-rev 973f2167facc75727e67a19f25bcc819511798d7 --all-features --release-type patch
```

Check that the peeled tag matches the recorded commit before comparison. `--release-type patch` deliberately asks for
compatibility with 0.1.1 even while the manifest says 0.2.0, so expected breaking changes are reported instead of hidden
by the version bump. For this transition a nonzero result is expected: removed column accessors/constructor arguments,
`SqlxAdapter`'s new mapping argument, and removal of its `Copy` implementation need migration. The tool also reports
changed `RqsError` discriminant positions after variant insertions. RestQS exposes error-code strings for machine
classification, not numeric enum ordinals; do not serialize memory layout or `Debug` output. This observation is recorded
as part of the deliberately incompatible 0.2 transition, not suppressed in the checker. Review every diagnostic
against [the 0.2 migration guide](migration-0.2.md); an unexplained new break must be resolved before release. Do not use
`|| true` to turn tool/build failures into successful verification.

The initial comparison for this 0.2 preparation reported three failing check categories for core and four with `sqlx`:
changed error discriminants, removed column accessors, changed constructor arity, and (with `sqlx`) removed `Copy`.
These are reviewed migration findings for this new incompatible line; there were no tool/build errors. Keep this record
with the migration guide and investigate additional findings on later comparisons.

On a compatible patch release both comparisons must pass against the latest published version in that line. After a new
release, record its tag and full commit as the next baseline; never move the old tag. For the first release of a new line,
retain the preceding published line as the migration comparison baseline until the new version is published. Before a
project's first-ever publication, review and archive its complete exported API and immutable release commit instead of
claiming a comparison to a nonexistent published version.

The comparison tool is a maintainer-only check, not a parser dependency or an automatic merge gate. Its pinned version
requires a newer toolchain than the library; use the command above. Automated API analysis cannot prove query behavior,
error-code meaning, MSRV, or database semantics. Review the source diff, changelog, focused tests, and migration guide as
part of the [release checklist](release-checklist.md). Record the baseline, command results, accepted changes, and migration
links in the release-preparation PR.
