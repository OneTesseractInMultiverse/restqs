#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

use restqs::{
    Field, FieldCatalog, RqsResult, RqsValue, ValueKind,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
    parse,
};

const WITH_REGEX: &str = concat!(
    "deleted=null&age>=18&status=in(active,pending)",
    "&email=/@example.com$/&status!=in(archived,blocked)&age<65&!deleted",
);

const WITHOUT_REGEX: &str = concat!(
    "deleted=null&age>=18&status=in(active,pending)",
    "&status!=in(archived,blocked)&age<65&!deleted",
);

fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("age")?
        .allow_text("status")?
        .allow_text("deleted")?
        .allow(Field::new("email", ValueKind::Text)?.allow_regex())
}

fn adapter(dialect: SqlDialect) -> RqsResult<SqlxAdapter> {
    let columns = SqlxColumnMap::new()
        .map("age", "users.age")?
        .map("status", "users.status")?
        .map("deleted", "users.deleted")?
        .map("email", "users.email")?;
    Ok(SqlxAdapter::new(dialect, columns).allow_regex())
}

fn build(raw: &str, dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
    let query = parse(raw, &catalog()?)?;
    adapter(dialect)?.build(&query)
}

#[test]
fn postgres_numbers_comparisons_lists_and_regex_in_one_sequence() -> RqsResult<()> {
    let parts = build(WITH_REGEX, SqlDialect::Postgres)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some(concat!(
            "\"users\".\"deleted\" IS NULL",
            " AND \"users\".\"age\" >= $1",
            " AND \"users\".\"status\" IN ($2, $3)",
            " AND \"users\".\"email\" ~ $4",
            " AND \"users\".\"status\" NOT IN ($5, $6)",
            " AND \"users\".\"age\" < $7",
            " AND \"users\".\"deleted\" IS NULL",
        ))
    );
    Ok(())
}

#[test]
fn postgres_mixed_filters_preserve_complete_bind_contents_and_order() -> RqsResult<()> {
    let parts = build(WITH_REGEX, SqlDialect::Postgres)?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Integer(18),
            RqsValue::Text("active".to_owned()),
            RqsValue::Text("pending".to_owned()),
            RqsValue::Text("@example.com$".to_owned()),
            RqsValue::Text("archived".to_owned()),
            RqsValue::Text("blocked".to_owned()),
            RqsValue::Integer(65),
        ]
    );
    Ok(())
}

#[test]
fn mysql_keeps_anonymous_placeholders_across_comparisons_lists_and_regex() -> RqsResult<()> {
    let parts = build(WITH_REGEX, SqlDialect::MySql)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some(concat!(
            "`users`.`deleted` IS NULL",
            " AND `users`.`age` >= ?",
            " AND `users`.`status` IN (?, ?)",
            " AND `users`.`email` REGEXP ?",
            " AND `users`.`status` NOT IN (?, ?)",
            " AND `users`.`age` < ?",
            " AND `users`.`deleted` IS NULL",
        ))
    );
    Ok(())
}

#[test]
fn mysql_mixed_filters_preserve_complete_bind_contents_and_order() -> RqsResult<()> {
    let parts = build(WITH_REGEX, SqlDialect::MySql)?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Integer(18),
            RqsValue::Text("active".to_owned()),
            RqsValue::Text("pending".to_owned()),
            RqsValue::Text("@example.com$".to_owned()),
            RqsValue::Text("archived".to_owned()),
            RqsValue::Text("blocked".to_owned()),
            RqsValue::Integer(65),
        ]
    );
    Ok(())
}

#[test]
fn sqlite_keeps_anonymous_placeholders_across_comparisons_and_lists() -> RqsResult<()> {
    let parts = build(WITHOUT_REGEX, SqlDialect::Sqlite)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some(concat!(
            "\"users\".\"deleted\" IS NULL",
            " AND \"users\".\"age\" >= ?",
            " AND \"users\".\"status\" IN (?, ?)",
            " AND \"users\".\"status\" NOT IN (?, ?)",
            " AND \"users\".\"age\" < ?",
            " AND \"users\".\"deleted\" IS NULL",
        ))
    );
    Ok(())
}

#[test]
fn sqlite_mixed_filters_preserve_complete_bind_contents_and_order() -> RqsResult<()> {
    let parts = build(WITHOUT_REGEX, SqlDialect::Sqlite)?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Integer(18),
            RqsValue::Text("active".to_owned()),
            RqsValue::Text("pending".to_owned()),
            RqsValue::Text("archived".to_owned()),
            RqsValue::Text("blocked".to_owned()),
            RqsValue::Integer(65),
        ]
    );
    Ok(())
}

#[test]
fn postgres_numbering_continues_past_a_list_and_two_digit_positions() -> RqsResult<()> {
    let parts = build(
        "age=in(1,2,3,4,5,6,7,8,9,10)&status=active",
        SqlDialect::Postgres,
    )?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some(concat!(
            "\"users\".\"age\" IN ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            " AND \"users\".\"status\" = $11",
        ))
    );
    Ok(())
}

#[test]
fn reusing_an_adapter_starts_a_fresh_placeholder_sequence() -> RqsResult<()> {
    let catalog = catalog()?;
    let adapter = adapter(SqlDialect::Postgres)?;
    let _ = adapter.build(&parse(WITH_REGEX, &catalog)?)?;
    let parts = adapter.build(&parse("age=21", &catalog)?)?;

    assert_eq!(
        parts.where_clause.as_deref(),
        Some("\"users\".\"age\" = $1")
    );
    Ok(())
}
