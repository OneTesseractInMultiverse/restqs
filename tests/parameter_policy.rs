#![allow(missing_docs)]

use restqs::{
    Field, FieldCatalog, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsError, RqsResult,
    RqsValue, SortDirection, ValueKind, parse,
};

fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("age")?
        .allow_text("status")?
        .allow(Field::new("email", ValueKind::Text)?.allow_regex())
}

#[test]
fn repeated_sort_replaces_the_previous_terms() -> RqsResult<()> {
    let query = parse("sort=status&sort=-age", &catalog()?)?;
    let terms = query
        .sort()
        .iter()
        .map(|term| (term.field().public_name(), term.direction()))
        .collect::<Vec<_>>();

    assert_eq!(terms, vec![("age", SortDirection::Desc)]);
    Ok(())
}

#[test]
fn repeated_empty_sort_clears_the_previous_terms() -> RqsResult<()> {
    let query = parse("sort=status&sort=", &catalog()?)?;

    assert!(query.sort().is_empty());
    Ok(())
}

#[test]
fn repeated_projection_replaces_the_previous_fields() -> RqsResult<()> {
    let query = parse("fields=status&fields=age", &catalog()?)?;
    let fields = query
        .projection()
        .fields()
        .iter()
        .map(|field| field.public_name())
        .collect::<Vec<_>>();

    assert_eq!(fields, vec!["age"]);
    Ok(())
}

#[test]
fn repeated_empty_projection_clears_the_previous_fields() -> RqsResult<()> {
    let query = parse("fields=status&fields=", &catalog()?)?;

    assert!(query.projection().is_empty());
    Ok(())
}

#[test]
fn repeated_limit_uses_the_last_value() -> RqsResult<()> {
    let query = parse("limit=20&limit=5", &catalog()?)?;

    assert_eq!(query.pagination().limit(), Some(5));
    Ok(())
}

#[test]
fn repeated_empty_limit_replaces_the_previous_value_with_zero() -> RqsResult<()> {
    let query = parse("limit=20&limit=", &catalog()?)?;

    assert_eq!(query.pagination().limit(), Some(0));
    Ok(())
}

#[test]
fn repeated_skip_uses_the_last_value() -> RqsResult<()> {
    let query = parse("skip=20&skip=5", &catalog()?)?;

    assert_eq!(query.pagination().offset(), Some(5));
    Ok(())
}

#[test]
fn repeated_empty_skip_replaces_the_previous_value_with_zero() -> RqsResult<()> {
    let query = parse("skip=20&skip=", &catalog()?)?;

    assert_eq!(query.pagination().offset(), Some(0));
    Ok(())
}

#[test]
fn an_invalid_earlier_control_is_not_hidden_by_a_later_value() -> RqsResult<()> {
    let result = parse("limit=bad&limit=5", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidPagination { parameter: "limit" })
    );
    Ok(())
}

#[test]
fn duplicate_filter_reports_logical_field_and_normalized_operator() -> RqsResult<()> {
    let result = parse("status=in(active)&status=in(pending)", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::DuplicateFilter {
            field: "status".to_owned(),
            operator: "in",
        })
    );
    Ok(())
}

#[test]
fn duplicate_not_in_filters_share_the_normalized_identity() -> RqsResult<()> {
    let result = parse("status!=in(active)&status!=in(pending)", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::DuplicateFilter {
            field: "status".to_owned(),
            operator: "not_in",
        })
    );
    Ok(())
}

#[test]
fn regex_identity_ignores_pattern_and_flags() -> RqsResult<()> {
    let result = parse("email=/one/&email=/two/i", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::DuplicateFilter {
            field: "email".to_owned(),
            operator: "regex",
        })
    );
    Ok(())
}

#[test]
fn existence_filters_participate_in_duplicate_rejection() -> RqsResult<()> {
    let result = parse("status&status", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::DuplicateFilter {
            field: "status".to_owned(),
            operator: "exists",
        })
    );
    Ok(())
}

#[test]
fn controls_do_not_reset_seen_filter_identities() -> RqsResult<()> {
    let result = parse("age>18&sort=age&age>21", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::DuplicateFilter {
            field: "age".to_owned(),
            operator: ">",
        })
    );
    Ok(())
}

#[test]
fn scalar_and_list_equality_keep_distinct_filter_identities() -> RqsResult<()> {
    let query = parse("status=active&status=in(active,pending)", &catalog()?)?;
    let operators = query.filters().iter().map(Filter::op).collect::<Vec<_>>();

    assert_eq!(operators, vec![FilterOp::Eq, FilterOp::In]);
    Ok(())
}

#[test]
fn distinct_fields_with_the_same_operator_preserve_filter_order() -> RqsResult<()> {
    let query = parse("status=active&age=18", &catalog()?)?;
    let fields = query
        .filters()
        .iter()
        .map(|filter| filter.field().public_name())
        .collect::<Vec<_>>();

    assert_eq!(fields, vec!["status", "age"]);
    Ok(())
}

#[test]
fn invalid_value_is_reported_before_duplicate_filter() -> RqsResult<()> {
    let result = parse("age=18&age=bad", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
    Ok(())
}

#[test]
fn invalid_regex_flags_are_reported_before_duplicate_filter() -> RqsResult<()> {
    let result = parse("email=/one/&email=/two/z", &catalog()?);

    assert_eq!(result, Err(RqsError::InvalidRegexFlags));
    Ok(())
}

#[test]
fn control_value_size_is_checked_before_pagination_syntax() -> RqsResult<()> {
    let catalog = catalog()?;
    let parser = Parser::with_config(
        &catalog,
        ParserConfig::with_limits(ParserLimits {
            max_value_bytes: 2,
            ..ParserLimits::default()
        }),
    );
    let result = parser.parse("limit=bad");

    assert_eq!(
        result,
        Err(RqsError::ValueTooLarge {
            field: "limit".to_owned(),
            max_bytes: 2,
        })
    );
    Ok(())
}

#[test]
fn unsupported_text_search_precedes_value_size_validation() -> RqsResult<()> {
    let catalog = catalog()?;
    let parser = Parser::with_config(
        &catalog,
        ParserConfig::with_limits(ParserLimits {
            max_value_bytes: 0,
            ..ParserLimits::default()
        }),
    );

    assert_eq!(
        parser.parse("$text=hello"),
        Err(RqsError::TextSearchUnsupported)
    );
    Ok(())
}

#[test]
fn percent_decoding_precedes_control_classification() -> RqsResult<()> {
    let query = parse("%6Cimit%3D7", &catalog()?)?;

    assert_eq!(query.pagination().limit(), Some(7));
    Ok(())
}

#[test]
fn control_prefix_inside_a_filter_value_stays_text() -> RqsResult<()> {
    let query = parse("status=sort=-age", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("sort=-age".to_owned()))
    );
    Ok(())
}

#[test]
fn the_same_parser_has_fresh_duplicate_state_for_each_parse() -> RqsResult<()> {
    let catalog = catalog()?;
    let parser = Parser::new(&catalog);
    let _ = parser.parse("age=18")?;
    let query = parser.parse("age=21")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(21))
    );
    Ok(())
}
