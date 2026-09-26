#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

#[path = "../examples/support/pagination.rs"]
mod pagination;
#[path = "../examples/support/users.rs"]
mod users;

use pagination::SqlStatement;
use restqs::{RqsValue, parse};
use users::{postgres_users_statement, sqlite_users_statement, users_catalog};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn postgres(raw: &str) -> TestResult<SqlStatement> {
    let query = parse(raw, &users_catalog()?)?;
    postgres_users_statement(&query)
}

fn sqlite(raw: &str) -> TestResult<SqlStatement> {
    let query = parse(raw, &users_catalog()?)?;
    sqlite_users_statement(&query)
}

#[test]
fn postgres_default_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        postgres("")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users"#
    );
    Ok(())
}

#[test]
fn postgres_empty_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        postgres("fields=")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users"#
    );
    Ok(())
}

#[test]
fn postgres_explicit_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        postgres("fields=id,name")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users"#
    );
    Ok(())
}

#[test]
fn postgres_reversed_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        postgres("fields=name,id")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users"#
    );
    Ok(())
}

#[test]
fn postgres_rejects_status_only_before_execution() {
    assert_eq!(
        postgres("fields=status").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn postgres_rejects_id_only_before_execution() {
    assert_eq!(
        postgres("fields=id").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn postgres_rejects_name_only_before_execution() {
    assert_eq!(
        postgres("fields=name").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn postgres_rejects_extra_field_before_execution() {
    assert_eq!(
        postgres("fields=id,name,status").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn postgres_rejects_unauthorized_projection_fields() {
    assert_eq!(
        postgres("fields=secret").map_err(|error| error.to_string()),
        Err("field secret is not allowed".to_owned())
    );
}

#[test]
fn postgres_projection_preserves_filter_and_pagination_binds() -> TestResult {
    let statement = postgres("fields=name,id&status=active&limit=1&skip=2")?;
    assert_eq!(
        statement.binds,
        vec![
            RqsValue::Text("active".to_owned()),
            RqsValue::Integer(1),
            RqsValue::Integer(2)
        ]
    );
    Ok(())
}

#[test]
fn sqlite_default_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        sqlite("")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users WHERE 1 = 1"#
    );
    Ok(())
}

#[test]
fn sqlite_empty_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        sqlite("fields=")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users WHERE 1 = 1"#
    );
    Ok(())
}

#[test]
fn sqlite_explicit_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        sqlite("fields=id,name")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users WHERE 1 = 1"#
    );
    Ok(())
}

#[test]
fn sqlite_reversed_projection_selects_both_decoder_columns() -> TestResult {
    assert_eq!(
        sqlite("fields=name,id")?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users WHERE 1 = 1"#
    );
    Ok(())
}

#[test]
fn sqlite_rejects_status_only_before_execution() {
    assert_eq!(
        sqlite("fields=status").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn sqlite_rejects_id_only_before_execution() {
    assert_eq!(
        sqlite("fields=id").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn sqlite_rejects_name_only_before_execution() {
    assert_eq!(
        sqlite("fields=name").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn sqlite_rejects_extra_field_before_execution() {
    assert_eq!(
        sqlite("fields=id,name,status").map_err(|error| error.to_string()),
        Err("adapter does not support users projection other than id and name".to_owned())
    );
}

#[test]
fn sqlite_rejects_unauthorized_projection_fields() {
    assert_eq!(
        sqlite("fields=secret").map_err(|error| error.to_string()),
        Err("field secret is not allowed".to_owned())
    );
}

#[test]
fn sqlite_projection_preserves_filter_and_pagination_binds() -> TestResult {
    let statement = sqlite("fields=name,id&status=active&limit=1&skip=2")?;
    assert_eq!(
        statement.binds,
        vec![
            RqsValue::Text("active".to_owned()),
            RqsValue::Integer(1),
            RqsValue::Integer(2)
        ]
    );
    Ok(())
}

#[test]
fn postgres_projection_preserves_filter_order_and_pagination_sql() -> TestResult {
    let statement = postgres("fields=name,id&status=active&sort=-id&limit=1&skip=2")?;
    assert_eq!(
        statement.sql,
        r#"SELECT "users"."id", "users"."name" FROM users WHERE "users"."status" = $1 ORDER BY "users"."id" DESC LIMIT $2 OFFSET $3"#
    );
    Ok(())
}

#[test]
fn sqlite_projection_preserves_filter_and_pagination_sql() -> TestResult {
    let statement = sqlite("fields=name,id&status=active&limit=1&skip=2")?;
    assert_eq!(
        statement.sql,
        r#"SELECT "users"."id", "users"."name" FROM users WHERE "users"."status" = ? LIMIT ? OFFSET ?"#
    );
    Ok(())
}
