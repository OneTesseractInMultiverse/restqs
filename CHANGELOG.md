# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## 0.1.0 - Unreleased

### Added

Initial RestQS crate with REST Query Syntax parsing, explicit field allowlists, typed query plans, safe parser limits,
SQLx-oriented fragments behind the `sqlx` feature, documentation-only SQLx examples, tests, docs, and release files.

### Fixed

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
