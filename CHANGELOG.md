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

### Changed

The parser validates field syntax before catalog lookup. Malformed nonempty field names now return
`invalid_field_name` instead of `unknown_field`; valid names absent from the catalog still return `unknown_field`.
