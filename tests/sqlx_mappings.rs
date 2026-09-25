#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

#[path = "../examples/support/in_memory.rs"]
mod in_memory;

use restqs::{
    Field, FieldCatalog, RqsError, RqsQuery, RqsResult, RqsValue, ValueKind,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
    parse,
};

fn plan(raw: &str) -> RqsResult<RqsQuery> {
    let catalog = FieldCatalog::new()
        .allow_integer("age")?
        .allow_text("status")?
        .allow(Field::new("email", ValueKind::Text)?.allow_regex())?;
    parse(raw, &catalog)
}

fn unmapped(raw: &str, dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
    SqlxAdapter::new(dialect, SqlxColumnMap::new())
        .allow_regex()
        .build(&plan(raw)?)
}

#[test]
fn postgres_resolves_a_logical_filter_to_a_physical_column() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&plan("age>=18")?)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some("\"people\".\"years\" >= $1")
    );
    Ok(())
}

#[test]
fn mysql_quotes_a_separate_physical_mapping() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let parts = SqlxAdapter::new(SqlDialect::MySql, columns).build(&plan("age>=18")?)?;

    assert_eq!(parts.where_clause.as_deref(), Some("`people`.`years` >= ?"));
    Ok(())
}

#[test]
fn sqlite_quotes_a_separate_physical_mapping() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let parts = SqlxAdapter::new(SqlDialect::Sqlite, columns).build(&plan("age>=18")?)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some("\"people\".\"years\" >= ?")
    );
    Ok(())
}

#[test]
fn sql_mapping_preserves_typed_bind_values() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("status", "people.state")?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, columns)
        .build(&plan("status=str(active' OR 1=1)")?)?;

    assert_eq!(
        parts.binds,
        vec![RqsValue::Text("active' OR 1=1".to_owned())]
    );
    Ok(())
}

#[test]
fn sort_resolves_the_physical_mapping() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&plan("sort=-age")?)?;

    assert_eq!(parts.order_by.as_deref(), Some("\"people\".\"years\" DESC"));
    Ok(())
}

#[test]
fn projection_resolves_the_physical_mapping() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("status", "people.state")?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&plan("fields=status")?)?;

    assert_eq!(parts.projection, vec!["\"people\".\"state\""]);
    Ok(())
}

#[test]
fn one_plan_can_be_translated_for_another_sql_schema() -> RqsResult<()> {
    let query = plan("age>=18")?;
    let first = SqlxColumnMap::new().map("age", "users.age")?;
    let second = SqlxColumnMap::new().map("age", "people.years")?;
    let _ = SqlxAdapter::new(SqlDialect::Postgres, first).build(&query)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, second).build(&query)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some("\"people\".\"years\" >= $1")
    );
    Ok(())
}

#[test]
fn sql_translation_leaves_the_logical_plan_unchanged() -> RqsResult<()> {
    let query = plan("age>=18&sort=-age&fields=age")?;
    let original = query.clone();
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let _ = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&query)?;

    assert_eq!(query, original);
    Ok(())
}

#[test]
fn the_same_plan_can_be_consumed_by_sql_and_in_memory() -> RqsResult<()> {
    let query = plan("age>=18")?;
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let _ = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&query)?;
    let people = [
        in_memory::Person {
            name: "Alex",
            age: 17,
        },
        in_memory::Person {
            name: "Sam",
            age: 21,
        },
    ];

    assert_eq!(in_memory::eligible_names(&query, &people)?, vec!["Sam"]);
    Ok(())
}

#[test]
fn missing_filter_mapping_reports_the_logical_field() {
    assert_eq!(
        unmapped("age>=18", SqlDialect::Postgres),
        Err(RqsError::MissingColumnMapping {
            field: "age".to_owned()
        })
    );
}

#[test]
fn missing_mysql_mapping_is_rejected() {
    assert_eq!(
        unmapped("age>=18", SqlDialect::MySql).map_err(|error| error.error_code()),
        Err("missing_column_mapping")
    );
}

#[test]
fn missing_sqlite_mapping_is_rejected() {
    assert_eq!(
        unmapped("age>=18", SqlDialect::Sqlite).map_err(|error| error.error_code()),
        Err("missing_column_mapping")
    );
}

#[test]
fn missing_sort_mapping_is_rejected() {
    assert_eq!(
        unmapped("sort=age", SqlDialect::Postgres).map_err(|error| error.error_code()),
        Err("missing_column_mapping")
    );
}

#[test]
fn missing_projection_mapping_is_rejected() {
    assert_eq!(
        unmapped("fields=status", SqlDialect::Postgres).map_err(|error| error.error_code()),
        Err("missing_column_mapping")
    );
}

#[test]
fn missing_mapping_in_later_sort_term_is_rejected() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let result = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&plan("sort=age,status")?);

    assert_eq!(
        result,
        Err(RqsError::MissingColumnMapping {
            field: "status".to_owned()
        })
    );
    Ok(())
}

#[test]
fn missing_mapping_in_later_projection_is_rejected() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let result = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&plan("fields=age,status")?);

    assert_eq!(
        result,
        Err(RqsError::MissingColumnMapping {
            field: "status".to_owned()
        })
    );
    Ok(())
}

#[test]
fn sql_shaped_public_name_does_not_become_a_column() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("users.status")?;
    let query = parse("users.status=active", &catalog)?;
    let result = SqlxAdapter::new(SqlDialect::Postgres, SqlxColumnMap::new()).build(&query);

    assert_eq!(
        result,
        Err(RqsError::MissingColumnMapping {
            field: "users.status".to_owned()
        })
    );
    Ok(())
}

#[test]
fn mapping_does_not_authorize_an_unknown_field() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("secret", "users.secret")?;
    let adapter = SqlxAdapter::new(SqlDialect::Postgres, columns);
    let result =
        parse("secret=value", &FieldCatalog::new()).and_then(|query| adapter.build(&query));

    assert_eq!(
        result,
        Err(RqsError::UnknownField {
            field: "secret".to_owned()
        })
    );
    Ok(())
}

#[test]
fn unused_catalog_fields_do_not_require_sql_mappings() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "people.years")?;
    let result = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&plan("age>=18")?);

    assert!(result.is_ok());
    Ok(())
}

#[test]
fn empty_plan_needs_no_sql_mappings() -> RqsResult<()> {
    let result = SqlxAdapter::new(SqlDialect::Postgres, SqlxColumnMap::new()).build(&plan("")?)?;

    assert_eq!(result.where_clause, None);
    Ok(())
}

#[test]
fn configured_mapping_resolves_without_quoting() -> RqsResult<()> {
    let columns = SqlxColumnMap::new().map("age", "app.people.years")?;
    let query = plan("age>=18")?;

    assert_eq!(
        columns.resolve(query.filters()[0].field())?,
        "app.people.years"
    );
    Ok(())
}

#[test]
fn mapping_rejects_invalid_logical_name() {
    let result = SqlxColumnMap::new()
        .map("status;drop", "users.status")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_field_name"));
}

#[test]
fn mapping_rejects_a_reserved_logical_name() {
    let result = SqlxColumnMap::new()
        .map("limit", "users.limit")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn alias_can_map_to_a_physical_column_named_like_a_control() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("row_limit")?;
    let query = parse("row_limit=5&limit=10", &catalog)?;
    let columns = SqlxColumnMap::new().map("row_limit", "users.limit")?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, columns).build(&query)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some("\"users\".\"limit\" = $1")
    );
    Ok(())
}

#[test]
fn mapping_rejects_sql_in_column_name() {
    let result = SqlxColumnMap::new()
        .map("status", "users.status;drop")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_column_name"));
}

#[test]
fn mapping_rejects_empty_column_segment() {
    let result = SqlxColumnMap::new()
        .map("status", "users..status")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_column_name"));
}

#[test]
fn mapping_rejects_empty_column_name() {
    let result = SqlxColumnMap::new()
        .map("status", "")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_column_name"));
}

#[test]
fn mapping_rejects_duplicate_logical_key() -> RqsResult<()> {
    let result = SqlxColumnMap::new()
        .map("status", "users.status")?
        .map("status", "users.secret");

    assert_eq!(
        result,
        Err(RqsError::DuplicateColumnMapping {
            field: "status".to_owned()
        })
    );
    Ok(())
}

#[test]
fn mapping_rejects_duplicate_identical_entry() -> RqsResult<()> {
    let result = SqlxColumnMap::new()
        .map("status", "users.status")?
        .map("status", "users.status")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_column_mapping"));
    Ok(())
}

#[test]
fn invalid_column_display_redacts_sql_text() {
    let result = SqlxColumnMap::new()
        .map("status", "users.status; SELECT secret")
        .map_err(|error| error.to_string());

    assert_eq!(result, Err("column [redacted] is invalid".to_owned()));
}
