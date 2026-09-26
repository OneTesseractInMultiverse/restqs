#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

use restqs::{
    Field, FieldCatalog, RqsError, RqsResult, RqsValue, ValueKind,
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

fn build_at(raw: &str, dialect: SqlDialect, first: usize) -> RqsResult<SqlxQueryParts> {
    let query = parse(raw, &catalog()?)?;
    adapter(dialect)?.build_with_bind_start(&query, first)
}

#[test]
fn postgres_composes_mixed_binds_after_a_caller_parameter() -> RqsResult<()> {
    let parts = build_at(WITH_REGEX, SqlDialect::Postgres, 2)?;
    assert_eq!(
        parts.where_clause.as_deref(),
        Some(concat!(
            "\"users\".\"deleted\" IS NULL",
            " AND \"users\".\"age\" >= $2",
            " AND \"users\".\"status\" IN ($3, $4)",
            " AND \"users\".\"email\" ~ $5",
            " AND \"users\".\"status\" NOT IN ($6, $7)",
            " AND \"users\".\"age\" < $8",
            " AND \"users\".\"deleted\" IS NULL",
        ))
    );
    Ok(())
}

#[test]
fn starting_position_does_not_add_caller_binds() -> RqsResult<()> {
    let parts = build_at(WITH_REGEX, SqlDialect::Postgres, 2)?;
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
fn explicit_start_does_not_change_subsequent_standalone_builds() -> RqsResult<()> {
    let adapter = adapter(SqlDialect::Postgres)?;
    let query = parse("age=21", &catalog()?)?;
    let _ = adapter.build_with_bind_start(&query, 12)?;
    assert_eq!(
        adapter.build(&query)?.where_clause.as_deref(),
        Some("\"users\".\"age\" = $1")
    );
    Ok(())
}

#[test]
fn numbering_crosses_digit_boundaries_from_an_explicit_start() -> RqsResult<()> {
    let parts = build_at("age=in(18,21)&status=active", SqlDialect::Postgres, 9)?;
    assert_eq!(
        parts.where_clause.as_deref(),
        Some("\"users\".\"age\" IN ($9, $10) AND \"users\".\"status\" = $11")
    );
    Ok(())
}

#[test]
fn final_representable_position_does_not_require_a_successor() -> RqsResult<()> {
    let parts = build_at("age=18&deleted=null", SqlDialect::Postgres, usize::MAX)?;
    assert_eq!(
        parts.where_clause,
        Some(format!(
            "\"users\".\"age\" = ${} AND \"users\".\"deleted\" IS NULL",
            usize::MAX
        ))
    );
    Ok(())
}

#[test]
fn bindless_filters_do_not_advance_the_start() -> RqsResult<()> {
    let parts = build_at("deleted=null&!status", SqlDialect::Postgres, usize::MAX)?;
    assert!(parts.binds.is_empty());
    Ok(())
}

#[test]
fn postgres_rejects_zero_start_for_empty_plan() {
    assert_eq!(
        build_at("", SqlDialect::Postgres, 0),
        Err(RqsError::InvalidBindPosition)
    );
}

#[test]
fn postgres_rejects_zero_start_for_filter() {
    assert_eq!(
        build_at("age=18", SqlDialect::Postgres, 0),
        Err(RqsError::InvalidBindPosition)
    );
}

#[test]
fn postgres_checks_position_overflow() {
    assert_eq!(
        build_at("age=18&status=active", SqlDialect::Postgres, usize::MAX),
        Err(RqsError::BindPositionOverflow)
    );
}

#[test]
fn mysql_rejects_zero_start_for_empty_plan() {
    assert_eq!(
        build_at("", SqlDialect::MySql, 0),
        Err(RqsError::InvalidBindPosition)
    );
}

#[test]
fn mysql_rejects_zero_start_for_filter() {
    assert_eq!(
        build_at("age=18", SqlDialect::MySql, 0),
        Err(RqsError::InvalidBindPosition)
    );
}

#[test]
fn mysql_checks_position_overflow() {
    assert_eq!(
        build_at("age=18&status=active", SqlDialect::MySql, usize::MAX),
        Err(RqsError::BindPositionOverflow)
    );
}

#[test]
fn mysql_keeps_anonymous_placeholders_with_an_explicit_start() -> RqsResult<()> {
    let parts = build_at("age=18", SqlDialect::MySql, 12)?;
    assert_eq!(parts.where_clause.as_deref(), Some(r#"`users`.`age` = ?"#));
    Ok(())
}

#[test]
fn sqlite_rejects_zero_start_for_empty_plan() {
    assert_eq!(
        build_at("", SqlDialect::Sqlite, 0),
        Err(RqsError::InvalidBindPosition)
    );
}

#[test]
fn sqlite_rejects_zero_start_for_filter() {
    assert_eq!(
        build_at("age=18", SqlDialect::Sqlite, 0),
        Err(RqsError::InvalidBindPosition)
    );
}

#[test]
fn sqlite_checks_position_overflow() {
    assert_eq!(
        build_at("age=18&status=active", SqlDialect::Sqlite, usize::MAX),
        Err(RqsError::BindPositionOverflow)
    );
}

#[test]
fn sqlite_keeps_anonymous_placeholders_with_an_explicit_start() -> RqsResult<()> {
    let parts = build_at("age=18", SqlDialect::Sqlite, 12)?;
    assert_eq!(parts.where_clause.as_deref(), Some(r#""users"."age" = ?"#));
    Ok(())
}

#[test]
fn overflow_propagates_from_list_binds() {
    assert_eq!(
        build_at("age=in(1,2)", SqlDialect::Postgres, usize::MAX),
        Err(RqsError::BindPositionOverflow)
    );
}

#[test]
fn overflow_propagates_from_regex_binds() {
    assert_eq!(
        build_at("age=18&email=/example/", SqlDialect::Postgres, usize::MAX),
        Err(RqsError::BindPositionOverflow)
    );
}
