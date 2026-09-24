# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

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
