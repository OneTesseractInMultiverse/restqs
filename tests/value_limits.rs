#![allow(missing_docs)]

use restqs::{
    Field, FieldCatalog, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsError, RqsQuery,
    RqsResult, RqsValue, SortDirection, SortTerm, ValueKind,
};

fn parse_with_limits(input: &str, limits: ParserLimits) -> RqsResult<RqsQuery> {
    let catalog = FieldCatalog::new()
        .allow_integer("id", "users.id")?
        .allow_text("name", "users.name")?
        .allow(Field::new("email", "users.email", ValueKind::Text)?.allow_regex())?;
    Parser::with_config(&catalog, ParserConfig::with_limits(limits)).parse(input)
}

fn parse_with_value_limit(input: &str, max_value_bytes: usize) -> RqsResult<RqsQuery> {
    parse_with_limits(
        input,
        ParserLimits {
            max_value_bytes,
            ..ParserLimits::default()
        },
    )
}

#[test]
fn regex_above_value_limit_is_rejected() {
    let result = parse_with_value_limit("email=/abcdefghij/", 3);

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "email".to_owned(),
            max_bytes: 3,
        })
    );
}

#[test]
fn regex_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("email=/a/", 3)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::Regex)
    );
    Ok(())
}

#[test]
fn regex_one_byte_above_value_limit_is_rejected() {
    let result = parse_with_value_limit("email=/ab/", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn regex_flags_count_toward_value_limit() {
    let result = parse_with_value_limit("email=/a/i", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn regex_limit_counts_utf8_bytes() {
    let result = parse_with_value_limit("email=/é/", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn multibyte_regex_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("email=/é/", 4)?;

    assert_eq!(
        query
            .filters()
            .first()
            .and_then(Filter::regex_literal)
            .map(|regex| regex.pattern()),
        Some("é")
    );
    Ok(())
}

#[test]
fn encoded_regex_at_decoded_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("email=%2F%C3%A9%2F", 4)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::Regex)
    );
    Ok(())
}

#[test]
fn encoded_regex_above_decoded_value_limit_is_rejected() {
    let result =
        parse_with_value_limit("email=%2F%C3%A9%2F", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn regex_within_value_limit_still_requires_field_permission() {
    let result = parse_with_value_limit("name=/a/", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("regex_disabled"));
}

#[test]
fn regex_size_is_checked_before_permission() {
    let result = parse_with_value_limit("name=/ab/", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn sort_above_value_limit_reports_control_name() {
    let result = parse_with_value_limit("sort=name", 3);

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "sort".to_owned(),
            max_bytes: 3,
        })
    );
}

#[test]
fn sort_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("sort=name", 4)?;

    assert_eq!(
        query.sort().first().map(|term| term.field().public_name()),
        Some("name")
    );
    Ok(())
}

#[test]
fn sort_limit_applies_to_complete_term_list() {
    let result = parse_with_value_limit("sort=id,name", 6).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn encoded_sort_prefix_counts_as_one_decoded_byte() -> RqsResult<()> {
    let query = parse_with_value_limit("sort=%2Bname", 5)?;

    assert_eq!(
        query.sort().first().map(SortTerm::direction),
        Some(SortDirection::Asc)
    );
    Ok(())
}

#[test]
fn projection_above_value_limit_reports_control_name() {
    let result = parse_with_value_limit("fields=name", 3);

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "fields".to_owned(),
            max_bytes: 3,
        })
    );
}

#[test]
fn projection_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("fields=name", 4)?;

    assert_eq!(
        query
            .projection()
            .fields()
            .iter()
            .map(|field| field.public_name())
            .collect::<Vec<_>>(),
        vec!["name"]
    );
    Ok(())
}

#[test]
fn projection_limit_applies_to_complete_field_list() {
    let result = parse_with_value_limit("fields=id,name", 6).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn limit_above_value_limit_reports_control_name() {
    let result = parse_with_value_limit("limit=0025", 3);

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "limit".to_owned(),
            max_bytes: 3,
        })
    );
}

#[test]
fn limit_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("limit=025", 3)?;

    assert_eq!(query.pagination().limit(), Some(25));
    Ok(())
}

#[test]
fn encoded_limit_counts_decoded_bytes() -> RqsResult<()> {
    let query = parse_with_value_limit("limit=%30%32%35", 3)?;

    assert_eq!(query.pagination().limit(), Some(25));
    Ok(())
}

#[test]
fn offset_above_value_limit_reports_control_name() {
    let result = parse_with_value_limit("skip=1000", 3);

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "skip".to_owned(),
            max_bytes: 3,
        })
    );
}

#[test]
fn offset_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("skip=100", 3)?;

    assert_eq!(query.pagination().offset(), Some(100));
    Ok(())
}

#[test]
fn scalar_at_multibyte_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("name=é", 2)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("é".to_owned()))
    );
    Ok(())
}

#[test]
fn scalar_one_byte_above_value_limit_is_rejected() {
    let result = parse_with_value_limit("name=éx", 2).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn encoded_scalar_counts_decoded_bytes() -> RqsResult<()> {
    let query = parse_with_value_limit("name=%C3%A9", 2)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("é".to_owned()))
    );
    Ok(())
}

#[test]
fn cast_wrapper_counts_toward_value_limit() {
    let result = parse_with_value_limit("name=str(a)", 5).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn list_above_value_limit_is_rejected() {
    let result = parse_with_value_limit("name=in(a,b)", 6).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn list_at_value_limit_is_accepted() -> RqsResult<()> {
    let query = parse_with_value_limit("name=in(a,b)", 7)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::List(vec![
            RqsValue::Text("a".to_owned()),
            RqsValue::Text("b".to_owned())
        ]))
    );
    Ok(())
}

#[test]
fn null_literal_counts_toward_value_limit() {
    let result = parse_with_value_limit("name=null", 3).map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
}

#[test]
fn exists_filter_has_no_value_to_limit() -> RqsResult<()> {
    let query = parse_with_value_limit("name", 0)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::Exists)
    );
    Ok(())
}

#[test]
fn not_exists_filter_has_no_value_to_limit() -> RqsResult<()> {
    let query = parse_with_value_limit("!name", 0)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::NotExists)
    );
    Ok(())
}

#[test]
fn empty_filter_value_remains_missing_at_zero_limit() {
    let result = parse_with_value_limit("name=", 0).map_err(|error| error.error_code());

    assert_eq!(result, Err("missing_value"));
}

#[test]
fn empty_sort_is_valid_at_zero_limit() -> RqsResult<()> {
    let query = parse_with_value_limit("sort=", 0)?;

    assert!(query.sort().is_empty());
    Ok(())
}

#[test]
fn empty_projection_is_valid_at_zero_limit() -> RqsResult<()> {
    let query = parse_with_value_limit("fields=", 0)?;

    assert!(query.projection().is_empty());
    Ok(())
}

#[test]
fn empty_limit_remains_zero_at_zero_value_limit() -> RqsResult<()> {
    let query = parse_with_value_limit("limit=", 0)?;

    assert_eq!(query.pagination().limit(), Some(0));
    Ok(())
}

#[test]
fn empty_offset_remains_zero_at_zero_value_limit() -> RqsResult<()> {
    let query = parse_with_value_limit("skip=", 0)?;

    assert_eq!(query.pagination().offset(), Some(0));
    Ok(())
}

#[test]
fn encoded_input_still_obeys_raw_query_byte_limit() {
    let limits = ParserLimits {
        max_query_bytes: 7,
        max_value_bytes: 1,
        ..ParserLimits::default()
    };
    let result = parse_with_limits("name=%61", limits).map_err(|error| error.error_code());

    assert_eq!(result, Err("query_too_large"));
}
