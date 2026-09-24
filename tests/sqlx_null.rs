#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

use restqs::{
    FieldCatalog, RqsError, RqsResult, RqsValue,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxQueryParts},
    parse,
};

fn columns() -> RqsResult<restqs::adapters::sqlx::SqlxColumnMap> {
    restqs::adapters::sqlx::SqlxColumnMap::new()
        .map("name", "users.name")?
        .map("status", "users.status")?
        .map("age", "users.age")?
        .map("active", "users.active")
}

fn build(query: &str, dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
    let catalog = FieldCatalog::new()
        .allow_text("name")?
        .allow_text("status")?
        .allow_integer("age")?
        .allow_boolean("active")?;
    let query = parse(query, &catalog)?;
    SqlxAdapter::new(dialect, columns()?).build(&query)
}

#[test]
fn postgres_null_equality_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name=null", SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" IS NULL".to_owned())
    );
    Ok(())
}

#[test]
fn postgres_null_equality_consumes_no_binds() -> RqsResult<()> {
    let parts = build("name=null", SqlDialect::Postgres)?;

    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn postgres_null_inequality_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name!=null", SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" IS NOT NULL".to_owned())
    );
    Ok(())
}

#[test]
fn postgres_null_inequality_consumes_no_binds() -> RqsResult<()> {
    let parts = build("name!=null", SqlDialect::Postgres)?;

    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn postgres_mixed_null_predicates_preserve_placeholders() -> RqsResult<()> {
    let parts = build(
        "age>=18&name=null&status!=null&age<65&active=null",
        SqlDialect::Postgres,
    )?;

    assert_eq!(
        parts.where_clause,
        Some(
            concat!(
                "\"users\".\"age\" >= $1",
                " AND \"users\".\"name\" IS NULL",
                " AND \"users\".\"status\" IS NOT NULL",
                " AND \"users\".\"age\" < $2",
                " AND \"users\".\"active\" IS NULL",
            )
            .to_owned()
        )
    );
    Ok(())
}

#[test]
fn postgres_mixed_null_predicates_preserve_bind_order() -> RqsResult<()> {
    let parts = build(
        "age>=18&name=null&status!=null&age<65&active=null",
        SqlDialect::Postgres,
    )?;

    assert_eq!(
        parts.binds,
        vec![RqsValue::Integer(18), RqsValue::Integer(65)]
    );
    Ok(())
}

#[test]
fn postgres_rejects_null_greater_than() {
    let result = build("name>null", SqlDialect::Postgres);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn postgres_rejects_null_greater_than_or_equal() {
    let result = build("name>=null", SqlDialect::Postgres);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn postgres_rejects_null_less_than() {
    let result = build("name<null", SqlDialect::Postgres);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn postgres_rejects_null_less_than_or_equal() {
    let result = build("name<=null", SqlDialect::Postgres);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn mysql_null_equality_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name=null", SqlDialect::MySql)?;

    assert_eq!(
        parts.where_clause,
        Some("`users`.`name` IS NULL".to_owned())
    );
    Ok(())
}

#[test]
fn mysql_null_equality_consumes_no_binds() -> RqsResult<()> {
    let parts = build("name=null", SqlDialect::MySql)?;

    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn mysql_null_inequality_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name!=null", SqlDialect::MySql)?;

    assert_eq!(
        parts.where_clause,
        Some("`users`.`name` IS NOT NULL".to_owned())
    );
    Ok(())
}

#[test]
fn mysql_null_inequality_consumes_no_binds() -> RqsResult<()> {
    let parts = build("name!=null", SqlDialect::MySql)?;

    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn mysql_mixed_null_predicates_preserve_placeholders() -> RqsResult<()> {
    let parts = build(
        "age>=18&name=null&status!=null&age<65&active=null",
        SqlDialect::MySql,
    )?;

    assert_eq!(
        parts.where_clause,
        Some(
            concat!(
                "`users`.`age` >= ?",
                " AND `users`.`name` IS NULL",
                " AND `users`.`status` IS NOT NULL",
                " AND `users`.`age` < ?",
                " AND `users`.`active` IS NULL",
            )
            .to_owned()
        )
    );
    Ok(())
}

#[test]
fn mysql_mixed_null_predicates_preserve_bind_order() -> RqsResult<()> {
    let parts = build(
        "age>=18&name=null&status!=null&age<65&active=null",
        SqlDialect::MySql,
    )?;

    assert_eq!(
        parts.binds,
        vec![RqsValue::Integer(18), RqsValue::Integer(65)]
    );
    Ok(())
}

#[test]
fn mysql_rejects_null_greater_than() {
    let result = build("name>null", SqlDialect::MySql);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn mysql_rejects_null_greater_than_or_equal() {
    let result = build("name>=null", SqlDialect::MySql);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn mysql_rejects_null_less_than() {
    let result = build("name<null", SqlDialect::MySql);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn mysql_rejects_null_less_than_or_equal() {
    let result = build("name<=null", SqlDialect::MySql);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn sqlite_null_equality_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name=null", SqlDialect::Sqlite)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" IS NULL".to_owned())
    );
    Ok(())
}

#[test]
fn sqlite_null_equality_consumes_no_binds() -> RqsResult<()> {
    let parts = build("name=null", SqlDialect::Sqlite)?;

    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn sqlite_null_inequality_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name!=null", SqlDialect::Sqlite)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" IS NOT NULL".to_owned())
    );
    Ok(())
}

#[test]
fn sqlite_null_inequality_consumes_no_binds() -> RqsResult<()> {
    let parts = build("name!=null", SqlDialect::Sqlite)?;

    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn sqlite_mixed_null_predicates_preserve_placeholders() -> RqsResult<()> {
    let parts = build(
        "age>=18&name=null&status!=null&age<65&active=null",
        SqlDialect::Sqlite,
    )?;

    assert_eq!(
        parts.where_clause,
        Some(
            concat!(
                "\"users\".\"age\" >= ?",
                " AND \"users\".\"name\" IS NULL",
                " AND \"users\".\"status\" IS NOT NULL",
                " AND \"users\".\"age\" < ?",
                " AND \"users\".\"active\" IS NULL",
            )
            .to_owned()
        )
    );
    Ok(())
}

#[test]
fn sqlite_mixed_null_predicates_preserve_bind_order() -> RqsResult<()> {
    let parts = build(
        "age>=18&name=null&status!=null&age<65&active=null",
        SqlDialect::Sqlite,
    )?;

    assert_eq!(
        parts.binds,
        vec![RqsValue::Integer(18), RqsValue::Integer(65)]
    );
    Ok(())
}

#[test]
fn sqlite_rejects_null_greater_than() {
    let result = build("name>null", SqlDialect::Sqlite);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn sqlite_rejects_null_greater_than_or_equal() {
    let result = build("name>=null", SqlDialect::Sqlite);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn sqlite_rejects_null_less_than() {
    let result = build("name<null", SqlDialect::Sqlite);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn sqlite_rejects_null_less_than_or_equal() {
    let result = build("name<=null", SqlDialect::Sqlite);

    assert_eq!(
        result,
        Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison"
        })
    );
}

#[test]
fn postgres_first_bind_after_null_predicate_starts_at_one() -> RqsResult<()> {
    let parts = build("name=null&age=18", SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" IS NULL AND \"users\".\"age\" = $1".to_owned())
    );
    Ok(())
}

#[test]
fn postgres_first_bind_after_null_predicate_contains_scalar() -> RqsResult<()> {
    let parts = build("name=null&age=18", SqlDialect::Postgres)?;

    assert_eq!(parts.binds, vec![RqsValue::Integer(18)]);
    Ok(())
}

#[test]
fn explicit_null_text_uses_bound_equality() -> RqsResult<()> {
    let parts = build("name=str(null)", SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" = $1".to_owned())
    );
    Ok(())
}

#[test]
fn explicit_null_text_stays_in_bind_value() -> RqsResult<()> {
    let parts = build("name=str(null)", SqlDialect::Postgres)?;

    assert_eq!(parts.binds, vec![RqsValue::Text("null".to_owned())]);
    Ok(())
}

#[test]
fn encoded_mixed_case_null_uses_null_predicate() -> RqsResult<()> {
    let parts = build("name=%4EuLl", SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"name\" IS NULL".to_owned())
    );
    Ok(())
}

#[test]
fn integer_null_uses_null_predicate() -> RqsResult<()> {
    let parts = build("age=null", SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"age\" IS NULL".to_owned())
    );
    Ok(())
}

#[test]
fn ordered_null_comparison_has_stable_error_code() {
    let result = build("age>null", SqlDialect::Postgres).map_err(|error| error.error_code());

    assert_eq!(result, Err("adapter_unsupported"));
}
