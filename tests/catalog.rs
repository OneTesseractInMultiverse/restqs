#![allow(missing_docs)]

use restqs::{Field, FieldCatalog, RqsError, ValueKind};

#[test]
fn catalog_returns_allowlisted_field() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("status", "users.status")?;

    assert_eq!(
        catalog.get("status").map(Field::column_name),
        Some("users.status")
    );
    Ok(())
}

#[test]
fn catalog_rejects_invalid_public_field_name() {
    let error = Field::new("status;drop", "users.status", ValueKind::Text)
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_field_name"));
}

#[test]
fn catalog_rejects_invalid_column_name() {
    let error = Field::new("status", "users.status;drop", ValueKind::Text)
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_column_name"));
}

#[test]
fn catalog_rejects_empty_column_segment() {
    let error =
        Field::new("status", "users..status", ValueKind::Text).map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_column_name"));
}

#[test]
fn field_can_allow_regex() -> restqs::RqsResult<()> {
    let field = Field::new("email", "users.email", ValueKind::Text)?.allow_regex();

    assert!(field.regex_allowed());
    Ok(())
}

#[test]
fn empty_catalog_has_zero_fields() {
    assert_eq!(FieldCatalog::new().len(), 0);
}

#[test]
fn empty_catalog_reports_empty() {
    assert!(FieldCatalog::new().is_empty());
}

#[test]
fn value_kind_expected_names_are_stable() {
    let names = [
        ValueKind::Text.expected_name(),
        ValueKind::Integer.expected_name(),
        ValueKind::Float.expected_name(),
        ValueKind::Boolean.expected_name(),
        ValueKind::Date.expected_name(),
        ValueKind::DateTime.expected_name(),
        ValueKind::Uuid.expected_name(),
    ];

    assert_eq!(
        names,
        [
            "text", "integer", "float", "boolean", "date", "datetime", "uuid"
        ]
    );
}

#[test]
fn field_returns_value_kind() -> restqs::RqsResult<()> {
    let field = Field::new("score", "users.score", ValueKind::Float)?;

    assert_eq!(field.value_kind(), ValueKind::Float);
    Ok(())
}

#[test]
fn catalog_allows_float_field() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_float("score", "users.score")?;

    assert_eq!(
        catalog.get("score").map(Field::column_name),
        Some("users.score")
    );
    Ok(())
}

#[test]
fn catalog_allow_builder_rejects_invalid_field_name() {
    let error = FieldCatalog::new()
        .allow_text("status/drop", "users.status")
        .map_err(|error| error.error_code());

    assert_eq!(error, Err("invalid_field_name"));
}

#[test]
fn unknown_field_error_uses_stable_code() {
    let error = RqsError::UnknownField {
        field: "secret".to_owned(),
    };

    assert_eq!(error.error_code(), "unknown_field");
}
