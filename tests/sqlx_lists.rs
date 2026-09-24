#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

use restqs::{
    FieldCatalog, RqsResult, RqsValue,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxQueryParts},
    parse,
};

fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("age", "users.age")?
        .allow_text("status", "users.status")
}

fn build_mixed_comparisons(dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
    let query = parse(
        "age>=18&status=in(active,pending)&age!=list(21,65)&status!=archived",
        &catalog()?,
    )?;
    SqlxAdapter::new(dialect).build(&query)
}

#[test]
fn postgres_lists_expand_placeholders_between_scalar_comparisons() -> RqsResult<()> {
    let parts = build_mixed_comparisons(SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause,
        Some(
            concat!(
                "\"users\".\"age\" >= $1 AND \"users\".\"status\" IN ($2, $3)",
                " AND \"users\".\"age\" NOT IN ($4, $5) AND \"users\".\"status\" <> $6"
            )
            .to_owned()
        )
    );
    Ok(())
}

#[test]
fn postgres_lists_expand_to_scalar_binds_in_order() -> RqsResult<()> {
    let parts = build_mixed_comparisons(SqlDialect::Postgres)?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Integer(18),
            RqsValue::Text("active".to_owned()),
            RqsValue::Text("pending".to_owned()),
            RqsValue::Integer(21),
            RqsValue::Integer(65),
            RqsValue::Text("archived".to_owned()),
        ]
    );
    Ok(())
}

#[test]
fn mysql_lists_expand_placeholders_between_scalar_comparisons() -> RqsResult<()> {
    let parts = build_mixed_comparisons(SqlDialect::MySql)?;

    assert_eq!(
        parts.where_clause,
        Some(
            concat!(
                "`users`.`age` >= ? AND `users`.`status` IN (?, ?)",
                " AND `users`.`age` NOT IN (?, ?) AND `users`.`status` <> ?"
            )
            .to_owned()
        )
    );
    Ok(())
}

#[test]
fn mysql_lists_expand_to_scalar_binds_in_order() -> RqsResult<()> {
    let parts = build_mixed_comparisons(SqlDialect::MySql)?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Integer(18),
            RqsValue::Text("active".to_owned()),
            RqsValue::Text("pending".to_owned()),
            RqsValue::Integer(21),
            RqsValue::Integer(65),
            RqsValue::Text("archived".to_owned()),
        ]
    );
    Ok(())
}

#[test]
fn sqlite_lists_expand_placeholders_between_scalar_comparisons() -> RqsResult<()> {
    let parts = build_mixed_comparisons(SqlDialect::Sqlite)?;

    assert_eq!(
        parts.where_clause,
        Some(
            concat!(
                "\"users\".\"age\" >= ? AND \"users\".\"status\" IN (?, ?)",
                " AND \"users\".\"age\" NOT IN (?, ?) AND \"users\".\"status\" <> ?"
            )
            .to_owned()
        )
    );
    Ok(())
}

#[test]
fn sqlite_lists_expand_to_scalar_binds_in_order() -> RqsResult<()> {
    let parts = build_mixed_comparisons(SqlDialect::Sqlite)?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Integer(18),
            RqsValue::Text("active".to_owned()),
            RqsValue::Text("pending".to_owned()),
            RqsValue::Integer(21),
            RqsValue::Integer(65),
            RqsValue::Text("archived".to_owned()),
        ]
    );
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_greater_than_list() -> RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres);
    let result = parse("age>in(1,2)", &catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_greater_than_or_equal_list() -> RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres);
    let result = parse("age>=in(1,2)", &catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_less_than_list() -> RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres);
    let result = parse("age<in(1,2)", &catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_less_than_or_equal_list() -> RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres);
    let result = parse("age<=in(1,2)", &catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}
