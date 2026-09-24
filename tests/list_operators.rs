#![allow(missing_docs)]

use restqs::{
    FieldCatalog, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsQuery, RqsResult,
    RqsValue, parse,
};

fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("age", "users.age")?
        .allow_text("status", "users.status")
}

fn parse_with_limits(input: &str, limits: ParserLimits) -> RqsResult<RqsQuery> {
    let catalog = catalog()?;
    Parser::with_config(&catalog, ParserConfig::with_limits(limits)).parse(input)
}

#[test]
fn in_list_rejects_greater_than() -> RqsResult<()> {
    let result = parse("age>in(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn in_list_rejects_greater_than_or_equal() -> RqsResult<()> {
    let result = parse("age>=in(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn in_list_rejects_less_than() -> RqsResult<()> {
    let result = parse("age<in(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn in_list_rejects_less_than_or_equal() -> RqsResult<()> {
    let result = parse("age<=in(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn list_alias_rejects_greater_than() -> RqsResult<()> {
    let result = parse("age>list(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn list_alias_rejects_greater_than_or_equal() -> RqsResult<()> {
    let result = parse("age>=list(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn list_alias_rejects_less_than() -> RqsResult<()> {
    let result = parse("age<list(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn list_alias_rejects_less_than_or_equal() -> RqsResult<()> {
    let result = parse("age<=list(1,2)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn empty_list_rejects_ordered_comparison() -> RqsResult<()> {
    let result = parse("age>in()", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn encoded_list_rejects_ordered_comparison() -> RqsResult<()> {
    let result = parse("age%3E%3Din%281%2C2%29", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn text_list_rejects_ordered_comparison() -> RqsResult<()> {
    let result =
        parse("status<in(active,pending)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn list_alias_equality_becomes_in() -> RqsResult<()> {
    let query = parse("age=list(1,2)", &catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::In));
    Ok(())
}

#[test]
fn list_alias_inequality_becomes_not_in() -> RqsResult<()> {
    let query = parse("age!=list(1,2)", &catalog()?)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::NotIn)
    );
    Ok(())
}

#[test]
fn list_alias_preserves_typed_items() -> RqsResult<()> {
    let query = parse("age=list(1,2)", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::List(vec![
            RqsValue::Integer(1),
            RqsValue::Integer(2)
        ]))
    );
    Ok(())
}

#[test]
fn explicit_text_containing_list_syntax_remains_a_scalar() -> RqsResult<()> {
    let query = parse("status>str(in(active,pending))", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("in(active,pending)".to_owned()))
    );
    Ok(())
}

#[test]
fn value_size_limit_precedes_list_operator_validation() {
    let result = parse_with_limits(
        "age>in(1,2)",
        ParserLimits {
            max_value_bytes: 6,
            ..ParserLimits::default()
        },
    )
    .map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn item_count_limit_precedes_list_operator_validation() {
    let result = parse_with_limits(
        "age>in(1,2)",
        ParserLimits {
            max_list_items: 1,
            ..ParserLimits::default()
        },
    )
    .map_err(|error| error.error_code());

    assert_eq!(result, Err("too_many_list_items"));
}

#[test]
fn item_type_validation_precedes_list_operator_validation() -> RqsResult<()> {
    let result = parse("age>in(1,invalid)", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
    Ok(())
}
