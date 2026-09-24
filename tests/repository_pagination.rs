#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

#[path = "../examples/support/pagination.rs"]
mod pagination;

use pagination::{SqlStatement, append_postgres_pagination, append_sqlite_pagination};
use restqs::{
    FieldCatalog, RqsResult, RqsValue,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxQueryParts},
    parse,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn query_parts(raw: &str, dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
    let catalog = FieldCatalog::new()
        .allow_integer("id", "users.id")?
        .allow_text("status", "users.status")?;
    let query = parse(raw, &catalog)?;
    SqlxAdapter::new(dialect).build(&query)
}

fn base_sql(parts: &SqlxQueryParts) -> String {
    let mut sql = "SELECT * FROM users".to_owned();
    if let Some(clause) = &parts.where_clause {
        sql.push_str(" WHERE ");
        sql.push_str(clause);
    }
    if let Some(order_by) = &parts.order_by {
        sql.push_str(" ORDER BY ");
        sql.push_str(order_by);
    }
    sql
}

fn postgres(raw: &str) -> TestResult<SqlStatement> {
    let parts = query_parts(raw, SqlDialect::Postgres)?;
    Ok(append_postgres_pagination(&base_sql(&parts), &parts)?)
}

fn sqlite(raw: &str) -> TestResult<SqlStatement> {
    let parts = query_parts(raw, SqlDialect::Sqlite)?;
    Ok(append_sqlite_pagination(&base_sql(&parts), &parts)?)
}

fn numeric_parts(limit: Option<u64>, offset: Option<u64>) -> SqlxQueryParts {
    SqlxQueryParts {
        where_clause: None,
        projection: Vec::new(),
        order_by: None,
        limit,
        offset,
        binds: Vec::new(),
    }
}

#[test]
fn postgres_appends_limit_after_filter_and_sort() -> TestResult {
    let statement = postgres("status=active&sort=id&limit=25")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" = $1 ORDER BY \"users\".\"id\" ASC LIMIT $2"
    );
    Ok(())
}

#[test]
fn postgres_binds_limit_after_filter() -> TestResult {
    let statement = postgres("status=active&limit=25")?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Text("active".to_owned()), RqsValue::Integer(25)]
    );
    Ok(())
}

#[test]
fn postgres_numbers_pagination_after_list_bind_values() -> TestResult {
    let statement = postgres("status=null&id=in(10,20,30)&limit=1&skip=2")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" IS NULL AND \"users\".\"id\" IN ($1, $2, $3) LIMIT $4 OFFSET $5"
    );
    Ok(())
}

#[test]
fn postgres_binds_limit_then_offset_after_list_values() -> TestResult {
    let statement = postgres("status=null&id=in(10,20,30)&skip=2&limit=1")?;

    assert_eq!(
        statement.binds,
        vec![
            RqsValue::Integer(10),
            RqsValue::Integer(20),
            RqsValue::Integer(30),
            RqsValue::Integer(1),
            RqsValue::Integer(2)
        ]
    );
    Ok(())
}

#[test]
fn postgres_supports_offset_without_limit() -> TestResult {
    let statement = postgres("status=active&skip=2")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" = $1 OFFSET $2"
    );
    Ok(())
}

#[test]
fn postgres_binds_only_offset_when_limit_is_absent() -> TestResult {
    let statement = postgres("status=active&skip=2")?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Text("active".to_owned()), RqsValue::Integer(2)]
    );
    Ok(())
}

#[test]
fn postgres_starts_pagination_at_first_placeholder_without_filter_binds() -> TestResult {
    let statement = postgres("status=null&limit=1&skip=1")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" IS NULL LIMIT $1 OFFSET $2"
    );
    Ok(())
}

#[test]
fn postgres_preserves_zero_limit_and_offset_binds() -> TestResult {
    let statement = postgres("limit=0&skip=0")?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Integer(0), RqsValue::Integer(0)]
    );
    Ok(())
}

#[test]
fn postgres_keeps_zero_limit_in_sql() -> TestResult {
    let statement = postgres("limit=0")?;

    assert_eq!(statement.sql, "SELECT * FROM users LIMIT $1");
    Ok(())
}

#[test]
fn postgres_omitted_pagination_leaves_sql_uncapped() -> TestResult {
    let statement = postgres("status=active")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" = $1"
    );
    Ok(())
}

#[test]
fn postgres_omitted_pagination_preserves_filter_binds() -> TestResult {
    let statement = postgres("status=active")?;

    assert_eq!(statement.binds, vec![RqsValue::Text("active".to_owned())]);
    Ok(())
}

#[test]
fn sqlite_appends_limit_after_filter_and_sort() -> TestResult {
    let statement = sqlite("status=active&sort=id&limit=25")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" = ? ORDER BY \"users\".\"id\" ASC LIMIT ?"
    );
    Ok(())
}

#[test]
fn sqlite_binds_limit_after_filter() -> TestResult {
    let statement = sqlite("status=active&limit=25")?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Text("active".to_owned()), RqsValue::Integer(25)]
    );
    Ok(())
}

#[test]
fn sqlite_appends_limit_and_offset_after_list_placeholders() -> TestResult {
    let statement = sqlite("status=null&id=in(10,20,30)&limit=1&skip=2")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" IS NULL AND \"users\".\"id\" IN (?, ?, ?) LIMIT ? OFFSET ?"
    );
    Ok(())
}

#[test]
fn sqlite_binds_limit_then_offset_after_list_values() -> TestResult {
    let statement = sqlite("status=null&id=in(10,20,30)&skip=2&limit=1")?;

    assert_eq!(
        statement.binds,
        vec![
            RqsValue::Integer(10),
            RqsValue::Integer(20),
            RqsValue::Integer(30),
            RqsValue::Integer(1),
            RqsValue::Integer(2)
        ]
    );
    Ok(())
}

#[test]
fn sqlite_offset_only_uses_fixed_unlimited_sentinel() -> TestResult {
    let statement = sqlite("status=active&skip=2")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" = ? LIMIT -1 OFFSET ?"
    );
    Ok(())
}

#[test]
fn sqlite_offset_only_does_not_bind_the_sentinel() -> TestResult {
    let statement = sqlite("status=active&skip=2")?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Text("active".to_owned()), RqsValue::Integer(2)]
    );
    Ok(())
}

#[test]
fn sqlite_preserves_zero_limit_and_offset_binds() -> TestResult {
    let statement = sqlite("limit=0&skip=0")?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Integer(0), RqsValue::Integer(0)]
    );
    Ok(())
}

#[test]
fn sqlite_keeps_zero_limit_in_sql() -> TestResult {
    let statement = sqlite("limit=0")?;

    assert_eq!(statement.sql, "SELECT * FROM users LIMIT ?");
    Ok(())
}

#[test]
fn sqlite_omitted_pagination_leaves_sql_uncapped() -> TestResult {
    let statement = sqlite("status=active")?;

    assert_eq!(
        statement.sql,
        "SELECT * FROM users WHERE \"users\".\"status\" = ?"
    );
    Ok(())
}

#[test]
fn sqlite_omitted_pagination_preserves_filter_binds() -> TestResult {
    let statement = sqlite("status=active")?;

    assert_eq!(statement.binds, vec![RqsValue::Text("active".to_owned())]);
    Ok(())
}

#[test]
fn postgres_accepts_maximum_signed_pagination_values() -> TestResult {
    let parts = numeric_parts(
        Some(9_223_372_036_854_775_807),
        Some(9_223_372_036_854_775_807),
    );
    let statement = append_postgres_pagination("SELECT * FROM users", &parts)?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Integer(i64::MAX), RqsValue::Integer(i64::MAX)]
    );
    Ok(())
}

#[test]
fn sqlite_accepts_maximum_signed_pagination_values() -> TestResult {
    let parts = numeric_parts(
        Some(9_223_372_036_854_775_807),
        Some(9_223_372_036_854_775_807),
    );
    let statement = append_sqlite_pagination("SELECT * FROM users", &parts)?;

    assert_eq!(
        statement.binds,
        vec![RqsValue::Integer(i64::MAX), RqsValue::Integer(i64::MAX)]
    );
    Ok(())
}

#[test]
fn postgres_rejects_limit_one_above_signed_range() {
    let parts = numeric_parts(Some(9_223_372_036_854_775_808), None);

    assert!(append_postgres_pagination("SELECT * FROM users", &parts).is_err());
}

#[test]
fn postgres_rejects_offset_one_above_signed_range() {
    let parts = numeric_parts(None, Some(9_223_372_036_854_775_808));

    assert!(append_postgres_pagination("SELECT * FROM users", &parts).is_err());
}

#[test]
fn sqlite_rejects_limit_one_above_signed_range() {
    let parts = numeric_parts(Some(9_223_372_036_854_775_808), None);

    assert!(append_sqlite_pagination("SELECT * FROM users", &parts).is_err());
}

#[test]
fn sqlite_rejects_offset_one_above_signed_range() {
    let parts = numeric_parts(None, Some(9_223_372_036_854_775_808));

    assert!(append_sqlite_pagination("SELECT * FROM users", &parts).is_err());
}

#[test]
fn postgres_rejects_maximum_unsigned_limit() {
    let parts = numeric_parts(Some(u64::MAX), None);

    assert!(append_postgres_pagination("SELECT * FROM users", &parts).is_err());
}

#[test]
fn sqlite_rejects_maximum_unsigned_limit() {
    let parts = numeric_parts(Some(u64::MAX), None);

    assert!(append_sqlite_pagination("SELECT * FROM users", &parts).is_err());
}

#[test]
fn postgres_rejects_oversized_offset_from_parser() -> RqsResult<()> {
    let parts = query_parts("skip=18446744073709551615", SqlDialect::Postgres)?;

    assert!(append_postgres_pagination("SELECT * FROM users", &parts).is_err());
    Ok(())
}

#[test]
fn sqlite_rejects_oversized_offset_from_parser() -> RqsResult<()> {
    let parts = query_parts("skip=18446744073709551615", SqlDialect::Sqlite)?;

    assert!(append_sqlite_pagination("SELECT * FROM users", &parts).is_err());
    Ok(())
}
