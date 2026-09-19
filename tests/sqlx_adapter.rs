#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

use restqs::{
    Field, FieldCatalog, RqsValue, ValueKind,
    adapters::sqlx::{SqlDialect, SqlxAdapter},
    parse,
};

fn catalog() -> restqs::RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("age", "users.age")?
        .allow_text("status", "users.status")?
        .allow_datetime("created_at", "users.created_at")
}

fn regex_catalog() -> restqs::RqsResult<FieldCatalog> {
    let field = Field::new("email", "users.email", ValueKind::Text)?.allow_regex();
    FieldCatalog::new().allow(field)
}

#[test]
fn sqlx_adapter_builds_postgres_where_clause() -> restqs::RqsResult<()> {
    let query = parse("age>=18", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"age\" >= $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_preserves_bind_order() -> restqs::RqsResult<()> {
    let query = parse("age>=18&status=active", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(parts.binds.len(), 2);
    Ok(())
}

#[test]
fn sqlx_adapter_builds_mysql_identifier_quotes() -> restqs::RqsResult<()> {
    let query = parse("age=18", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::MySql).build(&query)?;

    assert_eq!(parts.where_clause, Some("`users`.`age` = ?".to_owned()));
    Ok(())
}

#[test]
fn sqlx_adapter_builds_in_clause() -> restqs::RqsResult<()> {
    let query = parse("status=in(active,pending)", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"status\" IN ($1, $2)".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_order_by_clause() -> restqs::RqsResult<()> {
    let query = parse("sort=-created_at", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.order_by,
        Some("\"users\".\"created_at\" DESC".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_ascending_order_by_clause() -> restqs::RqsResult<()> {
    let query = parse("sort=created_at", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.order_by,
        Some("\"users\".\"created_at\" ASC".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_projection_columns() -> restqs::RqsResult<()> {
    let query = parse("fields=status,age", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(parts.projection.len(), 2);
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_regex_not_equal() -> restqs::RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres).allow_regex();
    let error = parse("email!=/admin/", &regex_catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_regex_greater_than() -> restqs::RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres).allow_regex();
    let error = parse("email>/admin/", &regex_catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_regex_greater_than_or_equal() -> restqs::RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres).allow_regex();
    let error = parse("email>=/admin/", &regex_catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_regex_less_than() -> restqs::RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres).allow_regex();
    let error = parse("email</admin/", &regex_catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_pipeline_rejects_regex_less_than_or_equal() -> restqs::RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::Postgres).allow_regex();
    let error = parse("email<=/admin/", &regex_catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn mysql_pipeline_rejects_regex_not_equal() -> restqs::RqsResult<()> {
    let adapter = SqlxAdapter::new(SqlDialect::MySql).allow_regex();
    let error = parse("email!=/admin/", &regex_catalog()?)
        .and_then(|query| adapter.build(&query))
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn postgres_regex_pattern_stays_in_bind_value() -> restqs::RqsResult<()> {
    let query = parse("email=/admin/", &regex_catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres)
        .allow_regex()
        .build(&query)?;

    assert_eq!(parts.binds, vec![RqsValue::Text("admin".to_owned())]);
    Ok(())
}

#[test]
fn mysql_regex_pattern_stays_in_bind_value() -> restqs::RqsResult<()> {
    let query = parse("email=/admin/", &regex_catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::MySql)
        .allow_regex()
        .build(&query)?;

    assert_eq!(parts.binds, vec![RqsValue::Text("admin".to_owned())]);
    Ok(())
}

#[test]
fn sqlx_adapter_rejects_regex_by_default() -> restqs::RqsResult<()> {
    let query = parse("email=/@example.com$/i", &regex_catalog()?)?;
    let error = SqlxAdapter::new(SqlDialect::Postgres)
        .build(&query)
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("adapter_unsupported"));
    Ok(())
}

#[test]
fn sqlx_adapter_builds_exists_clause() -> restqs::RqsResult<()> {
    let query = parse("status", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"status\" IS NOT NULL".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_not_exists_clause() -> restqs::RqsResult<()> {
    let query = parse("!status", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"status\" IS NULL".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_not_in_clause() -> restqs::RqsResult<()> {
    let query = parse("status!=in(active,pending)", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"status\" NOT IN ($1, $2)".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_not_equal_clause() -> restqs::RqsResult<()> {
    let query = parse("age!=18", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"age\" <> $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_less_than_clause() -> restqs::RqsResult<()> {
    let query = parse("age<18", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"age\" < $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_greater_than_clause() -> restqs::RqsResult<()> {
    let query = parse("age>18", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"age\" > $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_less_than_or_equal_clause() -> restqs::RqsResult<()> {
    let query = parse("age<=18", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"age\" <= $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_rejects_empty_list_clause() -> restqs::RqsResult<()> {
    let query = parse("status=in()", &catalog()?)?;
    let error = SqlxAdapter::new(SqlDialect::Postgres)
        .build(&query)
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("adapter_unsupported"));
    Ok(())
}

#[test]
fn sqlx_adapter_builds_mysql_regex_when_enabled() -> restqs::RqsResult<()> {
    let query = parse("email=/@example.com$/", &regex_catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::MySql)
        .allow_regex()
        .build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("`users`.`email` REGEXP ?".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_builds_postgres_case_sensitive_regex() -> restqs::RqsResult<()> {
    let query = parse("email=/@example.com$/", &regex_catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres)
        .allow_regex()
        .build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"email\" ~ $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_keeps_limit_value() -> restqs::RqsResult<()> {
    let query = parse("limit=25", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(parts.limit, Some(25));
    Ok(())
}

#[test]
fn sqlx_adapter_keeps_offset_value() -> restqs::RqsResult<()> {
    let query = parse("skip=50", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(parts.offset, Some(50));
    Ok(())
}

#[test]
fn sqlx_adapter_returns_none_for_empty_where_clause() -> restqs::RqsResult<()> {
    let query = parse("sort=age", &catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres).build(&query)?;

    assert_eq!(parts.where_clause, None);
    Ok(())
}

#[test]
fn sqlx_adapter_builds_postgres_regex_when_enabled() -> restqs::RqsResult<()> {
    let query = parse("email=/@example.com$/i", &regex_catalog()?)?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres)
        .allow_regex()
        .build(&query)?;

    assert_eq!(
        parts.where_clause,
        Some("\"users\".\"email\" ~* $1".to_owned())
    );
    Ok(())
}

#[test]
fn sqlx_adapter_rejects_sqlite_regex() -> restqs::RqsResult<()> {
    let query = parse("email=/@example.com$/", &regex_catalog()?)?;
    let error = SqlxAdapter::new(SqlDialect::Sqlite)
        .allow_regex()
        .build(&query)
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("adapter_unsupported"));
    Ok(())
}
