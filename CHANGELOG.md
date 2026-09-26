# Changelog

User-facing changes are tracked in this file.

The project uses semantic versioning.

## Unreleased - 0.2.0

### Added

`SqlxAdapter::build_with_bind_start` accepts an explicit one-based first bind position for composing PostgreSQL
fragments after caller-owned parameters. Standalone `build` still starts at `$1`; MySQL and SQLite keep anonymous
placeholders. Zero positions and arithmetic overflow return `invalid_bind_position` and `bind_position_overflow`.
A tested tenant repository example preserves the authorization predicate and binds tenant, filters, then pagination.

### Changed

The core catalog and plan now contain logical field identity, value kind, and query capabilities without SQL column
metadata. `Field::new` and all `FieldCatalog::allow_*` builders drop the physical column argument; `column_name()`
accessors are removed. `SqlxAdapter::new` requires an owned `SqlxColumnMap` and is no longer `Copy`. Every SQL filter,
sort term, and projection resolves through that trusted configuration. Missing and duplicate entries return the new
`missing_column_mapping` and `duplicate_column_mapping` error codes. Physical identifier validation moves into the
adapter. This breaking API change targets 0.2.0; see [Migrating to 0.2](docs/migration-0.2.md).

Parameter classification, control-size policy, and duplicate-filter identity and rejection are now pure internal
computations. The parser coordinates them before updating query state. This refactor preserves syntax, stable error
codes, and filter validation precedence without changing the public API.

Sort-prefix interpretation is now a pure computation, separate from authorized field resolution and sort-term
construction. Bare, descending, and percent-encoded ascending sort terms retain their syntax and validation errors.

SQL placeholder formatting is now a pure computation over the dialect and explicit bind position, separate from bind
insertion. PostgreSQL numbering, MySQL and SQLite anonymous placeholders, SQL fragments, and bind order are unchanged.

Bind-order tests and the integration example now compare complete typed value sequences. Focused mixed scalar/list
tests cover input order and repeated list values instead of relying on bind counts.

Error-display tests now check rendered messages for each error variant and hostile-input redaction instead of counting
messages. Stable error-code checks remain separate from display formatting and safety checks.

### Fixed

The PostgreSQL and SQLite user repository examples now validate projection before execution for their fixed `(id, name)`
response. Omitted or empty fields and exactly `id,name` in either order are accepted. Partial or additional selections
return an explicit adapter error rather than failing during row decoding or being silently ignored. Both examples use
shared, compiled repository helpers with catalog-authorized fields and trusted column mappings.


Repeated `sort`, `fields`, `limit`, and `skip` controls now return `duplicate_control` instead of silently using the
last value. Identical and empty values and percent-encoded equivalent names follow the same rule. Control value-size
checks precede duplicate detection, which precedes interpreting the repeated value. Single empty controls and distinct
range-filter operators retain their behavior. See [the migration guide](docs/migration-0.2.md#repeated-query-controls).

Ordered comparisons (`>`, `>=`, `<`, `<=`) with `in(...)` or `list(...)` values now return `invalid_operator` during
parsing, preventing list values from reaching SQLx as scalar binds. Equality and inequality still produce `In` and
`NotIn` filters with one bind per item. Value-size, item-count, and item-type validation retain their precedence.

Catalog builders now reject duplicate public names with `duplicate_field`, including identical definitions, instead
of silently replacing the field's type or regex permission. Validation runs before insertion. Names remain exact and
case-sensitive, and distinct aliases may still share a physical SQL column. Configure each field before registering
it; see [catalog migration](docs/migration-0.2.md#duplicate-catalog-registrations).

Logical field names `sort`, `fields`, `limit`, and `skip` now fail registration with `reserved_field_name` instead of
silently becoming query controls in equality expressions. The catalog and parser share one control-name policy, and
logical SQL mapping keys follow it too. Reserved names in field references also return this error. Matching is exact
and case-sensitive; existing control syntax and physical SQL identifiers are unchanged. Use a public alias for affected
fields; see [the migration guide](docs/migration-0.2.md#reserved-query-control-names).

The SQLx repository examples now apply parsed limits and offsets before execution and bind pagination after filter
values. A shared, tested example module handles PostgreSQL placeholder numbering, SQLite's `LIMIT -1 OFFSET ?` form
for offset-only requests, and checked signed-integer conversion. Values above `i64::MAX` fail before SQLx query
creation. The guide explicitly documents that an omitted limit leaves results uncapped and requires application policy.

Regex suffix flags are no longer silently discarded or ignored. The parser accepts unique lowercase `i`, `m`, `s`,
and `x` flags, preserving their order, and returns the new `invalid_regex_flags` error for unknown or repeated flags.
The SQLx adapter accepts no flags or `i` for PostgreSQL, and no flags for MySQL; other recognized flags return
`adapter_unsupported`. SQLite still rejects all regex. Previously accepted requests with invalid or unsupported flags
now fail explicitly. Both permission gates and bound patterns are preserved.

Float parsing now rejects NaN, positive and negative infinity, and overflow to infinity with `invalid_value` across
raw scalars, `float(...)` wrappers, and list items. Finite extremes, subnormal values, signed zero, and underflow rounded
to zero remain accepted. SQL adapters preserve accepted float bind values. See
[the float migration policy](docs/migration-0.2.md#finite-float-values) for previously accepted special values.

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
