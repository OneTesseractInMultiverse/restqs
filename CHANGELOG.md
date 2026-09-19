# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## 0.1.0 - Unreleased

### Added

Initial RestQS crate with REST Query Syntax parsing, explicit field allowlists, typed query plans, safe parser limits,
SQLx-oriented fragments behind the `sqlx` feature, documentation-only SQLx examples, tests, docs, and release files.

### Fixed

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
