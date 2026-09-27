# Changelog

User-facing changes are tracked in this file.

The project uses the [documented compatibility policy](docs/compatibility.md): compatible patches within each `0.x`
line and migration notes for new incompatible lines, with a narrow documented security-fix exception.

## Unreleased

- Update SQLite execution to SQLx 0.9.0, including its dynamic-SQL marker and argument types, and refresh its locked
  transitive dependencies. Both database fixtures now require Rust 1.94 and run on stable CI; the library keeps
  Rust 1.85 and no runtime dependencies.
- Update the assertion-policy tool to Syn 3.0.6, cargo-audit to 0.22.2, the coverage toolchain to Rust 1.98.1, and
  fuzzing to nightly-2026-09-27. Preserve the coverage and single-assertion gates.
- Refresh digest-pinned test services to PostgreSQL 18.6 and the latest available official MySQL image, 26.7.0.
  Document current dependency versions and unavoidable upstream/MSRV constraints.

## 0.2.0

This release introduces breaking API and validation changes from 0.1.x. Follow
[the migration guide](docs/migration-0.2.md). The library keeps Rust 1.85 as its minimum supported version,
empty default features, and zero dependencies, including when `sqlx` is enabled.

### Added

- `SqlxColumnMap` explicitly maps authorized logical fields to trusted physical columns. Missing or duplicate mappings
  fail with `missing_column_mapping` or `duplicate_column_mapping`; request names never become SQL identifiers.
- `SqlxAdapter::build_with_bind_start` composes PostgreSQL fragments after caller-owned binds. Zero and overflowing
  positions fail explicitly. The tested tenant example preserves its mandatory authorization predicate.
- Repository examples apply a 25-row default, 100-row maximum, and 10,000-row offset cap. Custom application budgets
  and an explicit trusted internal unbounded policy are available, with checked signed database integer conversion.
- SQLite, PostgreSQL 17.6, and MySQL 8.4.6 execution fixtures check filtering, projection, ordering, pagination, binds,
  and supported regex behavior. Driver dependencies remain outside the published crate; PostgreSQL/MySQL fixtures
  use SQLx 0.9.0 on Rust 1.94+, while SQLite uses SQLx 0.8.6 and retains Rust 1.85 support.
- Generated properties and bounded sanitizer-backed fuzz targets exercise decoding, typed plans, limit boundaries,
  bind correspondence, and SQL/value separation with reproducible regression inputs.
- Required CI and release gates enforce one assertion per Rust/Python test and 100% library source-line coverage.
  Release jobs validate and publish the same immutable tagged commit after environment approval.
- Public and internal Rust documentation covers validation and adapter boundaries. README and guide examples run as
  doctests; missing private implementation docs fail Clippy, and docs.rs includes the optional adapter. Integration docs link
  compiled driver sources instead of duplicating them; unsupported adapter designs and stale support links are removed.
- Compatibility policy covers Rust API shape, query behavior, error codes, MSRV, and security fixes. Private security
  reports use GitHub as the sole disclosure channel, with documented maintainer verification.

### Changed

- `Field::new` and catalog builders no longer accept physical columns; `column_name()` accessors are removed.
  `SqlxAdapter::new` now requires an owned `SqlxColumnMap`, and the adapter is no longer `Copy`.
  Core plans contain only logical field identity, value kinds, and capabilities.
- Classification, control budgets, duplicate identity, sort-prefix interpretation, and placeholder formatting are
  separate internal computations coordinated by parser/adapter functions.
- Repeated `sort`, `fields`, `limit`, or `skip` controls fail with `duplicate_control`, including empty values and encoded
  equivalent names. Value-size validation precedes duplicate detection and interpretation of the repeated value.
- Duplicate catalog names fail with `duplicate_field`. The exact control names `sort`, `fields`, `limit`, and `skip`
  fail registration with `reserved_field_name`; use public aliases for affected fields.
- Ordered list comparisons fail with `invalid_operator`; equality and inequality retain `In` and `NotIn` semantics.
- Regex suffixes must be unique lowercase `i`, `m`, `s`, or `x`. Unknown/repeated flags fail with `invalid_regex_flags`.
  PostgreSQL translates no flags or `i`, MySQL translates no flags, and unsupported dialect/flag combinations fail
  with `adapter_unsupported`. SQLite continues to reject regex.
- Float conversion rejects NaN, infinities, and overflow with `invalid_value`, including casts and list items.
  Finite extremes, subnormals, signed zero, and underflow rounded to zero remain accepted.

### Fixed

- Repository examples apply authorized ordering before pagination and bind pagination after all filter values.
  PostgreSQL numbering and SQLite offset-only syntax are covered by shared executable helpers.
- Fixed `(id, name)` responses reject incompatible projections before execution. Omitted/empty projection or exactly
  both fields in either order is supported; dynamic projections require an application-owned decoder.
- Focused tests verify complete typed bind order and error-display redaction, separately from stable error codes.

## 0.1.1 - 2026-09-24

### Fixed

Release validation and publication now use the same immutable tagged commit. Publication requires stable and minimum
supported Rust checks, both feature configurations, the dependency audit, source coverage, package verification, and
approval through the configured `crates-io` environment. Manual publication accepts an existing release tag from `main`.

Default-feature Clippy and documentation builds now pass without suppressing warnings. CI and local verification check
the parser both with and without the `sqlx` feature.

Date and date-time parsing now validates Gregorian month lengths and leap years, clock components, and numeric offsets.
Timestamps accept `Z`/`z`, positive and negative offsets, and fractional seconds while preserving the decoded string.
The documented format requires four-digit years, `T`/`t`, seconds `00..59`, and an explicit offset; leap seconds are
unsupported. Scalars, typed wrappers, and list items share validation and return `invalid_value` for invalid input.

Filter operators are recognized at the field boundary instead of being selected from anywhere in the decoded parameter.
Plain text, cast wrappers, lists, and regex patterns can contain comparison tokens without having value text mistaken
for a field name. Percent-encoded comparison syntax remains supported. The longest supported operator at the boundary
is consumed, and the remaining value text is preserved for value parsing. A lone `!` after a field now returns
`invalid_operator` instead of `invalid_field_name`; leading `!` and existence filters retain their existing behavior.

The SQLx adapter translates null equality and inequality to `IS NULL` and `IS NOT NULL` in PostgreSQL, MySQL, and SQLite.
These predicates no longer consume binds or placeholders, so subsequent scalar binds retain their correct positions.
Ordered comparisons with null now return `adapter_unsupported` with feature metadata `ordered null comparison`.
The core plan remains unchanged, and explicit text values such as `str(null)` remain ordinary bound comparisons.

Regex literals used with `!=`, `>`, `>=`, `<`, or `<=` now return `invalid_operator` instead of silently becoming positive
matches. Regex matching supports equality only; negation is unsupported. Value-size validation precedes operator
validation, which precedes field permission. Equality regex matching still requires both field and adapter permission.

Error Display messages redact malformed field and column identifiers and identifiers longer than 128 bytes. This
prevents control-character injection and disclosure of value text misidentified as a field. Error fields and Debug
output retain the original input.

`max_value_bytes` now rejects oversized regex values and `sort`, `fields`, `limit`, and `skip` controls, closing paths
that bypassed the value-size check. The limit counts decoded UTF-8 bytes, including wrappers, regex delimiters and flags,
and complete lists. Existence filters have no value to limit. Oversized regex values return `value_too_large` before
regex permission is checked; values within the limit still require permission.

### Changed

The parser validates field syntax before catalog lookup. Malformed nonempty field names now return
`invalid_field_name` instead of `unknown_field`; valid names absent from the catalog still return `unknown_field`.

## 0.1.0 - 2026-06-06

### Added

Initial RestQS crate with REST Query Syntax parsing, explicit field allowlists, typed query plans, safe parser limits,
SQLx-oriented fragments behind the `sqlx` feature, documentation-only SQLx examples, tests, docs, and release files.
