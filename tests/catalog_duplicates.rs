#![allow(missing_docs)]

use restqs::{Field, FieldCatalog, RqsError, RqsResult, ValueKind};

#[test]
fn catalog_rejects_an_identical_registration() -> RqsResult<()> {
    let result = FieldCatalog::new()
        .allow_text("status")?
        .allow_text("status")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_field"));
    Ok(())
}

#[test]
fn catalog_rejects_a_duplicate_with_a_different_type() -> RqsResult<()> {
    let result = FieldCatalog::new()
        .allow_text("status")?
        .allow_integer("status")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_field"));
    Ok(())
}

#[test]
fn catalog_rejects_a_duplicate_that_enables_regex() -> RqsResult<()> {
    let replacement = Field::new("email", ValueKind::Text)?.allow_regex();
    let result = FieldCatalog::new()
        .allow_text("email")?
        .allow(replacement)
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_field"));
    Ok(())
}

#[test]
fn catalog_rejects_a_duplicate_that_disables_regex() -> RqsResult<()> {
    let original = Field::new("email", ValueKind::Text)?.allow_regex();
    let result = FieldCatalog::new()
        .allow(original)?
        .allow_text("email")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("duplicate_field"));
    Ok(())
}

#[test]
fn direct_field_registration_reports_the_duplicate_name() -> RqsResult<()> {
    let field = Field::new("profile.status", ValueKind::Text)?;
    let result = FieldCatalog::new().allow(field.clone())?.allow(field);

    assert_eq!(
        result,
        Err(RqsError::DuplicateField {
            field: "profile.status".to_owned()
        })
    );
    Ok(())
}

#[test]
fn catalog_treats_case_variants_as_distinct_names() -> RqsResult<()> {
    let catalog = FieldCatalog::new()
        .allow_text("status")?
        .allow_integer("Status")?;

    assert_eq!(
        catalog.get("Status").map(Field::value_kind),
        Some(ValueKind::Integer)
    );
    Ok(())
}

#[test]
fn catalog_treats_dotted_names_as_distinct_from_their_prefix() -> RqsResult<()> {
    let catalog = FieldCatalog::new()
        .allow_text("profile")?
        .allow_integer("profile.age")?;

    assert_eq!(
        catalog.get("profile.age").map(Field::value_kind),
        Some(ValueKind::Integer)
    );
    Ok(())
}

#[test]
fn field_capabilities_can_be_configured_before_registration() -> RqsResult<()> {
    let field = Field::new("email", ValueKind::Text)?.allow_regex();
    let catalog = FieldCatalog::new().allow(field)?;

    assert_eq!(catalog.get("email").map(Field::regex_allowed), Some(true));
    Ok(())
}
