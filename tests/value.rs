#![allow(missing_docs)]

use restqs::{FieldCatalog, RqsValue, parse};

fn scalar_catalog() -> restqs::RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_text("name", "users.name")?
        .allow_integer("age", "users.age")?
        .allow_float("score", "users.score")?
        .allow_boolean("active", "users.active")?
        .allow_date("created_on", "users.created_on")?
        .allow_datetime("created_at", "users.created_at")?
        .allow_uuid("id", "users.id")
}

#[test]
fn parser_casts_uuid_values() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_uuid("id", "users.id")?;
    let query = parse("id=550e8400-e29b-41d4-a716-446655440000", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Uuid(
            "550e8400-e29b-41d4-a716-446655440000".to_owned()
        ))
    );
    Ok(())
}

#[test]
fn parser_casts_date_values() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_date("created_on", "users.created_on")?;
    let query = parse("created_on=2026-06-06", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Date("2026-06-06".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_casts_datetime_values() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_datetime("created_at", "users.created_at")?;
    let query = parse("created_at=2026-06-06T12:30:00Z", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::DateTime("2026-06-06T12:30:00Z".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_casts_datetime_values_with_offset() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_datetime("created_at", "users.created_at")?;
    let query = parse("created_at=2026-06-06T12:30:00%2B00:00", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::DateTime("2026-06-06T12:30:00+00:00".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_rejects_wrong_cast_wrapper() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age", "users.age")?;
    let error = parse("age=str(18)", &catalog).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_unknown_boolean_values() -> restqs::RqsResult<()> {
    let error = parse("active=maybe", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_unclosed_cast_wrapper() -> restqs::RqsResult<()> {
    let error = parse("age=int(18", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_unknown_cast_wrapper() -> restqs::RqsResult<()> {
    let error = parse("age=decimal(18)", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_accepts_typed_text_cast() -> restqs::RqsResult<()> {
    let query = parse("name=str(alice)", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Text("alice".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_casts_null_value() -> restqs::RqsResult<()> {
    let query = parse("name=null", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Null)
    );
    Ok(())
}

#[test]
fn parser_casts_float_values() -> restqs::RqsResult<()> {
    let query = parse("score=1.5", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Float(1.5))
    );
    Ok(())
}

#[test]
fn parser_casts_boolean_false_values() -> restqs::RqsResult<()> {
    let query = parse("active=off", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Boolean(false))
    );
    Ok(())
}

#[test]
fn parser_casts_empty_list_values() -> restqs::RqsResult<()> {
    let query = parse("name=in()", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::List(Vec::new()))
    );
    Ok(())
}

#[test]
fn parser_casts_compact_uuid_values() -> restqs::RqsResult<()> {
    let query = parse("id=550e8400e29b41d4a716446655440000", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Uuid(
            "550e8400e29b41d4a716446655440000".to_owned()
        ))
    );
    Ok(())
}

#[test]
fn parser_rejects_invalid_date_values() -> restqs::RqsResult<()> {
    let error =
        parse("created_on=20260606", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_invalid_datetime_values() -> restqs::RqsResult<()> {
    let error = parse("created_at=2026-06-06 12:30:00", &scalar_catalog()?)
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_invalid_float_values() -> restqs::RqsResult<()> {
    let error = parse("score=abc", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_invalid_integer_list_item() -> restqs::RqsResult<()> {
    let error = parse("age=in(1,nope)", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_invalid_uuid_values() -> restqs::RqsResult<()> {
    let error = parse("id=not-a-uuid", &scalar_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}

#[test]
fn parser_rejects_values_above_byte_limit() -> restqs::RqsResult<()> {
    let limits = restqs::ParserLimits {
        max_value_bytes: 3,
        ..restqs::ParserLimits::default()
    };
    let catalog = scalar_catalog()?;
    let parser = restqs::Parser::with_config(&catalog, restqs::ParserConfig::with_limits(limits));
    let error = parser
        .parse("name=active")
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("value_too_large"));
    Ok(())
}

#[test]
fn parser_accepts_typed_integer_cast() -> restqs::RqsResult<()> {
    let query = parse("age=int(18)", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Integer(18))
    );
    Ok(())
}

#[test]
fn parser_accepts_typed_float_cast() -> restqs::RqsResult<()> {
    let query = parse("score=float(1.5)", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Float(1.5))
    );
    Ok(())
}

#[test]
fn parser_accepts_typed_boolean_cast() -> restqs::RqsResult<()> {
    let query = parse("active=bool(yes)", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Boolean(true))
    );
    Ok(())
}

#[test]
fn parser_accepts_typed_date_cast() -> restqs::RqsResult<()> {
    let query = parse("created_on=date(2026-06-06)", &scalar_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Date("2026-06-06".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_accepts_typed_datetime_cast() -> restqs::RqsResult<()> {
    let query = parse(
        "created_at=datetime(2026-06-06T12:30:00Z)",
        &scalar_catalog()?,
    )?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::DateTime("2026-06-06T12:30:00Z".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_accepts_typed_uuid_cast() -> restqs::RqsResult<()> {
    let query = parse(
        "id=uuid(550e8400-e29b-41d4-a716-446655440000)",
        &scalar_catalog()?,
    )?;

    assert_eq!(
        query.filters().first().and_then(restqs::Filter::value),
        Some(&RqsValue::Uuid(
            "550e8400-e29b-41d4-a716-446655440000".to_owned()
        ))
    );
    Ok(())
}
