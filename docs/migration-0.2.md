# Migrating to 0.2

Version 0.2 is an unreleased breaking API change. Published 0.1.x applications keep their existing API until they
upgrade. This migration separates the endpoint's logical field allowlist from each SQL repository's physical schema.

## Logical Catalog

Previously, the catalog required a database column even for consumers that never generated SQL:

```rust,ignore
let catalog = FieldCatalog::new()
    .allow_integer("age", "users.age")?
    .allow_text("status", "users.status")?;
let email = Field::new("email", "users.email", ValueKind::Text)?.allow_regex();
```

The new constructors accept logical names and value kinds only:

```rust
use restqs::{Field, FieldCatalog, ValueKind};

let catalog = FieldCatalog::new()
    .allow_integer("age")?
    .allow_text("status")?;
let email = Field::new("email", ValueKind::Text)?.allow_regex();
# Ok::<(), restqs::RqsError>(())
```

Every `allow_*` convenience method drops its column argument. `Field::column_name()` and `FieldRef::column_name()`
are removed. Use `public_name()` for logical identity. Value kinds, endpoint allowlisting,
and per-field regex permission keep their existing meanings. Filters, sort terms, and projections contain logical
field references with no SQL metadata.

## Duplicate Catalog Registrations

`FieldCatalog::allow` and all `allow_*` builders now reject a repeated public name with `RqsError::DuplicateField`
(`duplicate_field`), including an identical definition. Previously the last registration silently replaced the field's
type and regex permission. Matching is exact and case-sensitive, so `status` and `Status` remain distinct names.

Remove overlapping registrations from catalog composition and choose each field's final definition before insertion.
For example, configure regex permission on the field rather than registering the same name again to enable it:

```rust
use restqs::{Field, FieldCatalog, ValueKind};

let email = Field::new("email", ValueKind::Text)?.allow_regex();
let catalog = FieldCatalog::new().allow(email)?;

assert_eq!(catalog.get("email").map(Field::regex_allowed), Some(true));
# Ok::<(), restqs::RqsError>(())
```

There is no replacement API. Build the catalog for each endpoint from its intended definitions, and treat duplicate
errors as configuration failures. Distinct public aliases remain supported: a catalog can register both `age` and
`years`, and a SQL repository can map both to `users.age`. Physical column changes belong in `SqlxColumnMap`; duplicate
logical mapping keys already return `duplicate_column_mapping` independently of this catalog policy.

## Reserved Query-Control Names

The exact lowercase public names `sort`, `fields`, `limit`, and `skip` now return `RqsError::ReservedFieldName`
(`reserved_field_name`) from `Field::new`, every catalog builder, and logical keys in `SqlxColumnMap`. Previously,
registration succeeded but equality syntax such as `limit=5` silently selected a query control instead of a filter.
Rename these public fields and update clients to use an unambiguous alias:

```rust
use restqs::{FieldCatalog, RqsValue, parse};

let catalog = FieldCatalog::new().allow_integer("row_limit")?;
let query = parse("row_limit=5&limit=10", &catalog)?;

assert_eq!(query.filters()[0].value(), Some(&RqsValue::Integer(5)));
# Ok::<(), restqs::RqsError>(())
```

Here `row_limit=5` filters the field, while `limit=10` caps results. SQL repositories can map the alias with
`SqlxColumnMap::new().map("row_limit", "users.limit")?`; physical column names need no change.

The policy is exact and case-sensitive. Names such as `Limit`, `profile.limit`, and `limit_value` remain valid.
Control syntax, including percent-encoded names, keeps its existing behavior. Reserved names used as field references
in sorting, projection, non-equality comparisons, or existence filters now return `reserved_field_name` before catalog
lookup. Malformed names still return `invalid_field_name`, including `$text`; the unsupported `$text=` control still
returns `text_search_unsupported`. Update exhaustive matches on `RqsError` for the new variant.

## SQL Repository Configuration

SQL applications configure a `SqlxColumnMap` from trusted application code, then pass it to `SqlxAdapter::new`:

```rust
use restqs::{FieldCatalog, parse, adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap}};

let catalog = FieldCatalog::new().allow_integer("age")?.allow_text("status")?;
let query = parse("age>=18&status=active&sort=-age&fields=status", &catalog)?;
let columns = SqlxColumnMap::new()
    .map("age", "users.age")?
    .map("status", "users.status")?;
let parts = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&query)?;

assert_eq!(parts.projection, vec!["\"users\".\"status\"".to_owned()]);
# Ok::<(), restqs::RqsError>(())
```

`SqlxAdapter` owns its mapping and is `Clone`, rather than `Copy`. Clone the configuration if several adapters need
the same mapping. The adapter validates physical identifiers when a mapping is registered and resolves every filter,
sort, and projection field through that mapping. Only referenced fields need mappings. There is no fallback that
treats a logical name as a SQL identifier, even if it looks like `users.status`.

Missing mappings return `missing_column_mapping`. Duplicate mapping keys return `duplicate_column_mapping` instead
of replacing a column. Invalid physical identifiers still return `invalid_column_name`; malformed logical names still
return `invalid_field_name`, while reserved control names return `reserved_field_name`. Mapping a field does not authorize
it: the parser continues to reject fields absent from the endpoint catalog. Keep both regex permission gates enabled
where needed.

Different repositories can translate the same parsed plan using different column maps. Non-SQL consumers can use
its logical names and typed values directly; see [the in-memory example](../examples/in_memory.rs). SQL pagination
assembly and bind-value handling are unchanged by this migration.

All query builders and custom adapters that previously read `column_name()` must move that lookup into their own
trusted storage configuration. Do not build a map from request parameters or assume public names are column names.
