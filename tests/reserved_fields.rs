#![allow(missing_docs)]

use restqs::{Field, FieldCatalog, Filter, RqsError, RqsResult, RqsValue, ValueKind, parse};

#[test]
fn catalog_rejects_limit_field() {
    let result = FieldCatalog::new()
        .allow_integer("limit")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn catalog_rejects_skip_field() {
    let result = FieldCatalog::new()
        .allow_integer("skip")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn catalog_rejects_sort_field() {
    let result = FieldCatalog::new()
        .allow_text("sort")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn catalog_rejects_fields_field() {
    let result = FieldCatalog::new()
        .allow_text("fields")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn field_constructor_reports_the_reserved_name() {
    let result = Field::new("limit", ValueKind::Integer);

    assert_eq!(
        result,
        Err(RqsError::ReservedFieldName {
            field: "limit".to_owned()
        })
    );
}

#[test]
fn text_search_field_keeps_its_syntax_error() {
    let result = Field::new("$text", ValueKind::Text).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_field_name"));
}

#[test]
fn case_variant_remains_an_equality_filter() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("Limit")?;
    let query = parse("Limit=5", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(5))
    );
    Ok(())
}

#[test]
fn dotted_suffix_remains_an_equality_filter() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("profile.limit")?;
    let query = parse("profile.limit=5", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(5))
    );
    Ok(())
}

#[test]
fn dotted_prefix_remains_an_equality_filter() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("limit.value")?;
    let query = parse("limit.value=5", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(5))
    );
    Ok(())
}

#[test]
fn similar_name_remains_an_equality_filter() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("limit_value")?;
    let query = parse("limit_value=5", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(5))
    );
    Ok(())
}

#[test]
fn alias_can_filter_alongside_a_control() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("row_limit")?;
    let query = parse("row_limit=5&limit=10", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Integer(5))
    );
    Ok(())
}

#[test]
fn control_keeps_its_value_alongside_an_alias() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("row_limit")?;
    let query = parse("row_limit=5&limit=10", &catalog)?;

    assert_eq!(query.pagination().limit(), Some(10));
    Ok(())
}

#[test]
fn encoded_control_name_keeps_its_behavior() -> RqsResult<()> {
    let query = parse("l%69mit=10", &FieldCatalog::new())?;

    assert_eq!(query.pagination().limit(), Some(10));
    Ok(())
}

#[test]
fn sort_rejects_a_reserved_field_reference() {
    let result = parse("sort=-limit", &FieldCatalog::new()).map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn projection_rejects_a_reserved_field_reference() {
    let result = parse("fields=skip", &FieldCatalog::new()).map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn non_equality_filter_rejects_a_reserved_field_reference() {
    let result = parse("fields!=name", &FieldCatalog::new()).map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn existence_filter_rejects_a_reserved_field_reference() {
    let result = parse("sort", &FieldCatalog::new()).map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}

#[test]
fn encoded_field_reference_cannot_bypass_the_reserved_policy() {
    let result = parse("sort=l%69mit", &FieldCatalog::new()).map_err(|error| error.error_code());

    assert_eq!(result, Err("reserved_field_name"));
}
