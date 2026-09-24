#![allow(missing_docs)]

use restqs::{
    Field, FieldCatalog, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsError, RqsQuery,
    RqsResult, RqsValue, ValueKind, parse,
};

fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow(Field::new("name", ValueKind::Text)?.allow_regex())?
        .allow_text("profile._name2")?
        .allow_integer("age")
}

fn parse_query(input: &str) -> RqsResult<RqsQuery> {
    parse(input, &catalog()?)
}

#[test]
fn plain_text_preserves_encoded_greater_than_or_equal() -> RqsResult<()> {
    let query = parse_query("name=a%3E%3Db")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a>=b".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_encoded_greater_than_or_equal() -> RqsResult<()> {
    let query = parse_query("name=str(a%3E%3Db)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a>=b".to_owned()))
    );
    Ok(())
}

#[test]
fn plain_text_preserves_encoded_less_than_or_equal() -> RqsResult<()> {
    let query = parse_query("name=a%3C%3Db")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a<=b".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_encoded_less_than_or_equal() -> RqsResult<()> {
    let query = parse_query("name=str(a%3C%3Db)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a<=b".to_owned()))
    );
    Ok(())
}

#[test]
fn plain_text_preserves_encoded_not_equal() -> RqsResult<()> {
    let query = parse_query("name=a%21%3Db")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a!=b".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_encoded_not_equal() -> RqsResult<()> {
    let query = parse_query("name=str(a%21%3Db)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a!=b".to_owned()))
    );
    Ok(())
}

#[test]
fn plain_text_preserves_encoded_greater_than() -> RqsResult<()> {
    let query = parse_query("name=a%3Eb")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a>b".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_encoded_greater_than() -> RqsResult<()> {
    let query = parse_query("name=str(a%3Eb)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a>b".to_owned()))
    );
    Ok(())
}

#[test]
fn plain_text_preserves_encoded_less_than() -> RqsResult<()> {
    let query = parse_query("name=a%3Cb")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a<b".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_encoded_less_than() -> RqsResult<()> {
    let query = parse_query("name=str(a%3Cb)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a<b".to_owned()))
    );
    Ok(())
}

#[test]
fn plain_text_preserves_encoded_equal() -> RqsResult<()> {
    let query = parse_query("name=a%3Db")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a=b".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_encoded_equal() -> RqsResult<()> {
    let query = parse_query("name=str(a%3Db)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a=b".to_owned()))
    );
    Ok(())
}

#[test]
fn plain_text_preserves_raw_comparison_tokens() -> RqsResult<()> {
    let query = parse_query("name=a>=b<=c!=d>e<f=g")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a>=b<=c!=d>e<f=g".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_text_preserves_raw_comparison_tokens() -> RqsResult<()> {
    let query = parse_query("name=str(a>=b<=c!=d>e<f=g)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("a>=b<=c!=d>e<f=g".to_owned()))
    );
    Ok(())
}

#[test]
fn encoded_greater_than_or_equal_is_recognized_at_field_boundary() -> RqsResult<()> {
    let query = parse_query("name%3E%3Da%3E%3Db%3C%3Dc%21%3Dd%3Ee%3Cf%3Dg")?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Gte));
    Ok(())
}

#[test]
fn encoded_less_than_or_equal_is_recognized_at_field_boundary() -> RqsResult<()> {
    let query = parse_query("name%3C%3Da%3E%3Db%3C%3Dc%21%3Dd%3Ee%3Cf%3Dg")?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Lte));
    Ok(())
}

#[test]
fn encoded_not_equal_is_recognized_at_field_boundary() -> RqsResult<()> {
    let query = parse_query("name%21%3Da%3E%3Db%3C%3Dc%21%3Dd%3Ee%3Cf%3Dg")?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Ne));
    Ok(())
}

#[test]
fn encoded_greater_than_is_recognized_at_field_boundary() -> RqsResult<()> {
    let query = parse_query("name%3Ea%3E%3Db%3C%3Dc%21%3Dd%3Ee%3Cf%3Dg")?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Gt));
    Ok(())
}

#[test]
fn encoded_less_than_is_recognized_at_field_boundary() -> RqsResult<()> {
    let query = parse_query("name%3Ca%3E%3Db%3C%3Dc%21%3Dd%3Ee%3Cf%3Dg")?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Lt));
    Ok(())
}

#[test]
fn encoded_equal_is_recognized_at_field_boundary() -> RqsResult<()> {
    let query = parse_query("name%3Da%3E%3Db%3C%3Dc%21%3Dd%3Ee%3Cf%3Dg")?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Eq));
    Ok(())
}

#[test]
fn text_value_can_begin_with_equal() -> RqsResult<()> {
    let query = parse_query("name==value")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("=value".to_owned()))
    );
    Ok(())
}

#[test]
fn longest_boundary_operator_leaves_following_token_in_value() -> RqsResult<()> {
    let query = parse_query("name>=<=value")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("<=value".to_owned()))
    );
    Ok(())
}

#[test]
fn regex_pattern_preserves_comparison_tokens() -> RqsResult<()> {
    let query = parse_query("name=/a!=b<=c>/")?;

    assert_eq!(
        query
            .filters()
            .first()
            .and_then(Filter::regex_literal)
            .map(|regex| regex.pattern()),
        Some("a!=b<=c>")
    );
    Ok(())
}

#[test]
fn list_text_preserves_comparison_tokens() -> RqsResult<()> {
    let query = parse_query("name=in(a>b,c<=d)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::List(vec![
            RqsValue::Text("a>b".to_owned()),
            RqsValue::Text("c<=d".to_owned())
        ]))
    );
    Ok(())
}

#[test]
fn multibyte_text_preserves_comparison_token() -> RqsResult<()> {
    let query = parse_query("name=é%3E水")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("é>水".to_owned()))
    );
    Ok(())
}

#[test]
fn dotted_field_keeps_value_tokens_out_of_catalog_lookup() -> RqsResult<()> {
    let query = parse_query("profile._name2=a>b")?;

    assert_eq!(
        query
            .filters()
            .first()
            .map(|filter| filter.field().public_name()),
        Some("profile._name2")
    );
    Ok(())
}

#[test]
fn lone_bang_at_field_boundary_is_invalid() {
    let result = parse_query("name!value").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
}

#[test]
fn invalid_boundary_operator_does_not_skip_to_later_equal() {
    let result = parse_query("name!value=tail").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
}

#[test]
fn unknown_field_metadata_excludes_value_tokens() {
    let result = parse_query("missing=confidential%3Etext");

    assert_eq!(
        result,
        Err(RqsError::UnknownField {
            field: "missing".to_owned()
        })
    );
}

#[test]
fn integer_value_with_comparison_token_is_invalid_value() {
    let result = parse_query("age=18%3E20").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn operator_characters_in_value_still_count_toward_byte_limit() -> RqsResult<()> {
    let catalog = catalog()?;
    let config = ParserConfig::with_limits(ParserLimits {
        max_value_bytes: 3,
        ..ParserLimits::default()
    });
    let result = Parser::with_config(&catalog, config)
        .parse("name=a%3Ebc")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
    Ok(())
}

#[test]
fn empty_field_before_long_operator_is_rejected() {
    let result = parse_query(">=value").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
}
