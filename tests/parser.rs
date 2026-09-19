#![allow(missing_docs)]

use restqs::{
    Field, FieldCatalog, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsValue,
    SortDirection, parse,
};

fn catalog() -> restqs::RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("age", "users.age")?
        .allow_text("status", "users.status")?
        .allow_boolean("active", "users.active")?
        .allow_datetime("created_at", "users.created_at")?
        .allow_uuid("id", "users.id")
}

fn regex_catalog() -> restqs::RqsResult<FieldCatalog> {
    let field = Field::new("email", "users.email", restqs::ValueKind::Text)?.allow_regex();
    FieldCatalog::new().allow(field)
}

#[test]
fn parser_returns_comparison_filter() -> restqs::RqsResult<()> {
    let query = parse("age>=18", &catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Gte));
    Ok(())
}

#[test]
fn parser_casts_integer_value() -> restqs::RqsResult<()> {
    let query = parse("age=18", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(18))
    );
    Ok(())
}

#[test]
fn parser_casts_boolean_value() -> restqs::RqsResult<()> {
    let query = parse("active=true", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Boolean(true))
    );
    Ok(())
}

#[test]
fn parser_maps_in_list_to_in_operator() -> restqs::RqsResult<()> {
    let query = parse("status=in(active,pending)", &catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::In));
    Ok(())
}

#[test]
fn parser_maps_not_equal_list_to_not_in_operator() -> restqs::RqsResult<()> {
    let query = parse("status!=in(active,pending)", &catalog()?)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::NotIn)
    );
    Ok(())
}

#[test]
fn parser_reads_exists_filter() -> restqs::RqsResult<()> {
    let query = parse("status", &catalog()?)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::Exists)
    );
    Ok(())
}

#[test]
fn parser_reads_not_exists_filter() -> restqs::RqsResult<()> {
    let query = parse("!status", &catalog()?)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::NotExists)
    );
    Ok(())
}

#[test]
fn parser_rejects_unknown_field() -> restqs::RqsResult<()> {
    let error = parse("secret=value", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("unknown_field"));
    Ok(())
}

#[test]
fn parser_rejects_regex_without_field_permission() -> restqs::RqsResult<()> {
    let error = parse("status=/active/i", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("regex_disabled"));
    Ok(())
}

#[test]
fn parser_accepts_regex_with_field_permission() -> restqs::RqsResult<()> {
    let query = parse("email=/@example.com$/i", &regex_catalog()?)?;

    assert_eq!(
        query.filters().first().map(Filter::op),
        Some(FilterOp::Regex)
    );
    Ok(())
}

#[test]
fn parser_rejects_regex_not_equal() -> restqs::RqsResult<()> {
    let error = parse("email!=/admin/", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_rejects_regex_greater_than() -> restqs::RqsResult<()> {
    let error = parse("email>/admin/", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_rejects_regex_greater_than_or_equal() -> restqs::RqsResult<()> {
    let error = parse("email>=/admin/", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_rejects_regex_less_than() -> restqs::RqsResult<()> {
    let error = parse("email</admin/", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_rejects_regex_less_than_or_equal() -> restqs::RqsResult<()> {
    let error = parse("email<=/admin/", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_rejects_encoded_regex_not_equal() -> restqs::RqsResult<()> {
    let error =
        parse("email%21%3D%2Fadmin%2F", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_checks_regex_operator_before_permission() -> restqs::RqsResult<()> {
    let error = parse("status!=/active/", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_checks_regex_size_before_operator() -> restqs::RqsResult<()> {
    let catalog = regex_catalog()?;
    let config = ParserConfig::with_limits(ParserLimits {
        max_value_bytes: 3,
        ..ParserLimits::default()
    });
    let error = Parser::with_config(&catalog, config)
        .parse("email!=/admin/")
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("value_too_large"));
    Ok(())
}

#[test]
fn parser_preserves_not_equal_for_scalar_text() -> restqs::RqsResult<()> {
    let query = parse("email!=admin", &regex_catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Ne));
    Ok(())
}

#[test]
fn parser_preserves_slashes_in_explicit_text_cast() -> restqs::RqsResult<()> {
    let query = parse("email!=str(/admin/)", &regex_catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("/admin/".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_preserves_not_equal_for_unclosed_regex_text() -> restqs::RqsResult<()> {
    let query = parse("email!=/admin", &regex_catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Ne));
    Ok(())
}

#[test]
fn parser_treats_single_slash_as_text() -> restqs::RqsResult<()> {
    let query = parse("email=/", &regex_catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Eq));
    Ok(())
}

#[test]
fn parser_treats_unclosed_regex_as_text() -> restqs::RqsResult<()> {
    let query = parse("email=/active", &regex_catalog()?)?;

    assert_eq!(query.filters().first().map(Filter::op), Some(FilterOp::Eq));
    Ok(())
}

#[test]
fn parser_rejects_text_search() -> restqs::RqsResult<()> {
    let error = parse("$text=hello", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("text_search_unsupported"));
    Ok(())
}

#[test]
fn parser_reads_descending_sort() -> restqs::RqsResult<()> {
    let query = parse("sort=-created_at", &catalog()?)?;

    assert_eq!(
        query.sort().first().map(restqs::SortTerm::direction),
        Some(SortDirection::Desc)
    );
    Ok(())
}

#[test]
fn parser_reads_projection_fields() -> restqs::RqsResult<()> {
    let query = parse("fields=status,age", &catalog()?)?;

    assert_eq!(query.projection().fields().len(), 2);
    Ok(())
}

#[test]
fn parser_rejects_empty_projection_field() -> restqs::RqsResult<()> {
    let error = parse("fields=status,", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_field_name"));
    Ok(())
}

#[test]
fn parser_reads_empty_projection() -> restqs::RqsResult<()> {
    let query = parse("fields=", &catalog()?)?;

    assert!(query.projection().is_empty());
    Ok(())
}

#[test]
fn parser_reads_ascending_sort_prefix() -> restqs::RqsResult<()> {
    let query = parse("sort=%2Bcreated_at", &catalog()?)?;

    assert_eq!(
        query.sort().first().map(restqs::SortTerm::direction),
        Some(SortDirection::Asc)
    );
    Ok(())
}

#[test]
fn parser_reads_empty_sort() -> restqs::RqsResult<()> {
    let query = parse("sort=", &catalog()?)?;

    assert_eq!(query.sort().len(), 0);
    Ok(())
}

#[test]
fn parser_rejects_empty_sort_field() -> restqs::RqsResult<()> {
    let error = parse("sort=created_at,", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_field_name"));
    Ok(())
}

#[test]
fn parser_reads_limit() -> restqs::RqsResult<()> {
    let query = parse("limit=20", &catalog()?)?;

    assert_eq!(query.pagination().limit(), Some(20));
    Ok(())
}

#[test]
fn parser_reads_empty_limit_as_zero() -> restqs::RqsResult<()> {
    let query = parse("limit=", &catalog()?)?;

    assert_eq!(query.pagination().limit(), Some(0));
    Ok(())
}

#[test]
fn parser_reads_skip() -> restqs::RqsResult<()> {
    let query = parse("skip=20", &catalog()?)?;

    assert_eq!(query.pagination().offset(), Some(20));
    Ok(())
}

#[test]
fn parser_rejects_invalid_limit() -> restqs::RqsResult<()> {
    let error = parse("limit=abc", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_pagination"));
    Ok(())
}

#[test]
fn parser_rejects_limit_above_default_maximum() -> restqs::RqsResult<()> {
    let error = parse("limit=101", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("limit_too_large"));
    Ok(())
}

#[test]
fn parser_rejects_negative_skip() -> restqs::RqsResult<()> {
    let error = parse("skip=-1", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("negative_pagination"));
    Ok(())
}

#[test]
fn parser_decodes_percent_encoded_values() -> restqs::RqsResult<()> {
    let query = parse("status=active%20user", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("active user".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_rejects_invalid_percent_encoding() -> restqs::RqsResult<()> {
    let error = parse("status=%ZZ", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_encoding"));
    Ok(())
}

#[test]
fn parser_rejects_percent_encoding_without_high_digit() -> restqs::RqsResult<()> {
    let error = parse("status=%", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_encoding"));
    Ok(())
}

#[test]
fn parser_rejects_percent_encoding_without_low_digit() -> restqs::RqsResult<()> {
    let error = parse("status=%A", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_encoding"));
    Ok(())
}

#[test]
fn parser_rejects_percent_encoding_with_invalid_low_digit() -> restqs::RqsResult<()> {
    let error = parse("status=%AZ", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_encoding"));
    Ok(())
}

#[test]
fn parser_decodes_plus_as_space() -> restqs::RqsResult<()> {
    let query = parse("status=active+user", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("active user".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_decodes_uppercase_hex() -> restqs::RqsResult<()> {
    let query = parse("status=active%2Fpending", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("active/pending".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_decodes_lowercase_hex() -> restqs::RqsResult<()> {
    let query = parse("status=active%2fpending", &catalog()?)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("active/pending".to_owned()))
    );
    Ok(())
}

#[test]
fn parser_rejects_too_many_parameters() -> restqs::RqsResult<()> {
    let limits = ParserLimits {
        max_parameters: 1,
        ..ParserLimits::default()
    };
    let catalog = catalog()?;
    let parser = Parser::with_config(&catalog, ParserConfig::with_limits(limits));
    let error = parser
        .parse("age=18&status=active")
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("too_many_parameters"));
    Ok(())
}

#[test]
fn parser_rejects_empty_filter_field() -> restqs::RqsResult<()> {
    let error = parse("=active", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_rejects_empty_not_exists_field() -> restqs::RqsResult<()> {
    let error = parse("!", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_field_name"));
    Ok(())
}

#[test]
fn parser_rejects_missing_filter_value() -> restqs::RqsResult<()> {
    let error = parse("age=", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("missing_value"));
    Ok(())
}

#[test]
fn parser_rejects_duplicate_filter_operator() -> restqs::RqsResult<()> {
    let error = parse("age>1&age>2", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("duplicate_filter"));
    Ok(())
}

#[test]
fn parser_allows_distinct_range_filters() -> restqs::RqsResult<()> {
    let query = parse("age>1&age<10", &catalog()?)?;

    assert_eq!(query.filters().len(), 2);
    Ok(())
}

#[test]
fn parser_rejects_query_above_byte_limit() -> restqs::RqsResult<()> {
    let limits = ParserLimits {
        max_query_bytes: 3,
        ..ParserLimits::default()
    };
    let catalog = catalog()?;
    let parser = Parser::with_config(&catalog, ParserConfig::with_limits(limits));
    let error = parser.parse("age=18").map_err(|error| error.error_code());

    assert_eq!(error, Err("query_too_large"));
    Ok(())
}

#[test]
fn parser_rejects_too_many_list_items() -> restqs::RqsResult<()> {
    let limits = ParserLimits {
        max_list_items: 1,
        ..ParserLimits::default()
    };
    let catalog = catalog()?;
    let parser = Parser::with_config(&catalog, ParserConfig::with_limits(limits));
    let error = parser
        .parse("status=in(active,pending)")
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("too_many_list_items"));
    Ok(())
}

#[test]
fn parser_rejects_wrong_typed_value() -> restqs::RqsResult<()> {
    let error = parse("age=abc", &catalog()?).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_value"));
    Ok(())
}
