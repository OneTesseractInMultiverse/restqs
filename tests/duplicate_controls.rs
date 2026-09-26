//! Duplicate controls checks using explicit, single-assertion cases.

use restqs::{
    FieldCatalog, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsError, RqsQuery,
    RqsResult, RqsValue, parse,
};

/// Parse controls with the suite catalog and supplied decoded-value byte budget.
fn parse_controls(raw: &str) -> RqsResult<RqsQuery> {
    let catalog = FieldCatalog::new()
        .allow_integer("age")?
        .allow_text("status")?;
    parse(raw, &catalog)
}

#[test]
fn duplicate_limit_is_rejected() {
    let result = parse_controls("limit=1&limit=100").map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_control"));
}

#[test]
fn duplicate_skip_is_rejected() {
    let result = parse_controls("skip=1&skip=100").map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_control"));
}

#[test]
fn duplicate_sort_is_rejected() {
    let result = parse_controls("sort=age&sort=-age").map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_control"));
}

#[test]
fn duplicate_projection_is_rejected() {
    let result = parse_controls("fields=age&fields=status").map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_control"));
}

#[test]
fn duplicate_limit_rejects_reversed_values() {
    let result = parse_controls("limit=100&limit=1");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_limit_rejects_empty_first_value() {
    let result = parse_controls("limit=&limit=1");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_limit_rejects_two_empty_values() {
    let result = parse_controls("limit=&limit=");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_limit_rejects_encoded_first_name() {
    let result = parse_controls("%6Cimit=1&limit=100");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_limit_rejects_encoded_second_name() {
    let result = parse_controls("limit=1&%6Cimit=100");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_limit_rejects_identical_values() {
    let result = parse_controls("limit=1&limit=1");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_skip_rejects_reversed_values() {
    let result = parse_controls("skip=100&skip=1");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "skip" })
    );
}

#[test]
fn duplicate_skip_rejects_empty_first_value() {
    let result = parse_controls("skip=&skip=1");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "skip" })
    );
}

#[test]
fn duplicate_skip_rejects_two_empty_values() {
    let result = parse_controls("skip=&skip=");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "skip" })
    );
}

#[test]
fn duplicate_skip_rejects_encoded_first_name() {
    let result = parse_controls("%73kip=1&skip=100");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "skip" })
    );
}

#[test]
fn duplicate_skip_rejects_encoded_second_name() {
    let result = parse_controls("skip=1&%73kip=100");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "skip" })
    );
}

#[test]
fn duplicate_skip_rejects_identical_values() {
    let result = parse_controls("skip=1&skip=1");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "skip" })
    );
}

#[test]
fn duplicate_sort_rejects_reversed_values() {
    let result = parse_controls("sort=-age&sort=age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn duplicate_sort_rejects_empty_first_value() {
    let result = parse_controls("sort=&sort=age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn duplicate_sort_rejects_two_empty_values() {
    let result = parse_controls("sort=&sort=");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn duplicate_sort_rejects_encoded_first_name() {
    let result = parse_controls("%73ort=age&sort=-age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn duplicate_sort_rejects_encoded_second_name() {
    let result = parse_controls("sort=age&%73ort=-age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn duplicate_sort_rejects_identical_values() {
    let result = parse_controls("sort=age&sort=age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn duplicate_projection_rejects_reversed_values() {
    let result = parse_controls("fields=status&fields=age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn duplicate_projection_rejects_empty_first_value() {
    let result = parse_controls("fields=&fields=age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn duplicate_projection_rejects_two_empty_values() {
    let result = parse_controls("fields=&fields=");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn duplicate_projection_rejects_encoded_first_name() {
    let result = parse_controls("%66ields=age&fields=status");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn duplicate_projection_rejects_encoded_second_name() {
    let result = parse_controls("fields=age&%66ields=status");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn duplicate_projection_rejects_identical_values() {
    let result = parse_controls("fields=age&fields=age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn duplicate_control_is_detected_after_filters_and_other_controls() {
    let result = parse_controls("sort=age&age>18&fields=status&sort=-age");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "sort" })
    );
}

#[test]
fn all_distinct_controls_preserve_range_filter_operators() -> RqsResult<()> {
    let query = parse_controls("limit=5&skip=2&sort=-age&fields=status&age>=18&age<65")?;
    let operators = query.filters().iter().map(Filter::op).collect::<Vec<_>>();

    assert_eq!(operators, vec![FilterOp::Gte, FilterOp::Lt]);
    Ok(())
}

#[test]
fn control_duplicate_state_is_fresh_for_each_parse() -> RqsResult<()> {
    let catalog = FieldCatalog::new();
    let parser = Parser::new(&catalog);
    let _ = parser.parse("limit=5")?;
    let query = parser.parse("limit=10")?;

    assert_eq!(query.pagination().limit(), Some(10));
    Ok(())
}

#[test]
fn case_variant_field_does_not_collide_with_a_control() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("Limit")?;
    let query = parse("Limit=2&limit=5", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(2))
    );
    Ok(())
}

#[test]
fn duplicate_control_precedes_repeated_value_syntax_validation() {
    let result = parse_controls("limit=5&limit=bad");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl { parameter: "limit" })
    );
}

#[test]
fn duplicate_control_precedes_repeated_field_resolution() {
    let result = parse_controls("fields=age&fields=unknown");

    assert_eq!(
        result,
        Err(RqsError::DuplicateControl {
            parameter: "fields"
        })
    );
}

#[test]
fn value_size_limit_precedes_duplicate_control_validation() {
    let catalog = FieldCatalog::new();
    let parser = Parser::with_config(
        &catalog,
        ParserConfig::with_limits(ParserLimits {
            max_value_bytes: 2,
            ..ParserLimits::default()
        }),
    );
    let result = parser.parse("limit=5&limit=100");

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "limit".to_owned(),
            max_bytes: 2
        })
    );
}
