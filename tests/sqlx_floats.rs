#![allow(missing_docs)]
#![cfg(feature = "sqlx")]

use restqs::{
    FieldCatalog, RqsResult, RqsValue,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
    parse,
};

fn build_float_query(raw: &str, dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
    let catalog = FieldCatalog::new().allow_float("score")?;
    let query = parse(raw, &catalog)?;
    let columns = SqlxColumnMap::new().map("score", "scores.value")?;
    SqlxAdapter::new(dialect, columns).build(&query)
}

#[test]
fn postgres_pipeline_rejects_non_finite_floats() {
    let result =
        build_float_query("score=NaN", SqlDialect::Postgres).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn postgres_preserves_finite_float_bind_types_and_boundaries() -> RqsResult<()> {
    let parts = build_float_query(
        "score>=-1.7976931348623157e308&score=in(5e-324,1.7976931348623157e308)",
        SqlDialect::Postgres,
    )?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Float(f64::MIN),
            RqsValue::Float(f64::from_bits(1)),
            RqsValue::Float(f64::MAX),
        ]
    );
    Ok(())
}

#[test]
fn mysql_pipeline_rejects_non_finite_floats() {
    let result = build_float_query("score=float(inf)", SqlDialect::MySql)
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn mysql_preserves_finite_float_bind_types_and_boundaries() -> RqsResult<()> {
    let parts = build_float_query(
        "score>=-1.7976931348623157e308&score=in(5e-324,1.7976931348623157e308)",
        SqlDialect::MySql,
    )?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Float(f64::MIN),
            RqsValue::Float(f64::from_bits(1)),
            RqsValue::Float(f64::MAX),
        ]
    );
    Ok(())
}

#[test]
fn sqlite_pipeline_rejects_non_finite_floats() {
    let result = build_float_query("score=in(1,NaN)", SqlDialect::Sqlite)
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn sqlite_preserves_finite_float_bind_types_and_boundaries() -> RqsResult<()> {
    let parts = build_float_query(
        "score>=-1.7976931348623157e308&score=in(5e-324,1.7976931348623157e308)",
        SqlDialect::Sqlite,
    )?;

    assert_eq!(
        parts.binds,
        vec![
            RqsValue::Float(f64::MIN),
            RqsValue::Float(f64::from_bits(1)),
            RqsValue::Float(f64::MAX),
        ]
    );
    Ok(())
}
