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
are removed. Use `public_name()` for logical identity. Value kinds, endpoint allowlisting, public query-name grammar,
and per-field regex permission keep their existing meanings. Filters, sort terms, and projections contain logical
field references with no SQL metadata.

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
of replacing a column. Invalid physical identifiers still return `invalid_column_name`; invalid logical names still
return `invalid_field_name`. Mapping a field does not authorize it: the parser continues to reject fields absent from
the endpoint catalog. Keep both regex permission gates enabled where needed.

Different repositories can translate the same parsed plan using different column maps. Non-SQL consumers can use
its logical names and typed values directly; see [the in-memory example](../examples/in_memory.rs). SQL pagination
assembly and bind-value handling are unchanged by this migration.

All query builders and custom adapters that previously read `column_name()` must move that lookup into their own
trusted storage configuration. Do not build a map from request parameters or assume public names are column names.
