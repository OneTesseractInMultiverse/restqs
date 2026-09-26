#![allow(missing_docs)]

use restqs::{FieldCatalog, RqsError, parse};

#[test]
fn duplicate_field_message_redacts_malformed_identifiers() {
    let error = RqsError::DuplicateField {
        field: "status\nsecret".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field [redacted] is already registered in the catalog"
    );
}

#[test]
fn duplicate_field_message_redacts_long_registered_names() -> restqs::RqsResult<()> {
    let name = "a".repeat(129);
    let result = FieldCatalog::new()
        .allow_text(&name)?
        .allow_text(&name)
        .map_err(|error| error.to_string());

    assert_eq!(
        result,
        Err("field [redacted] is already registered in the catalog".to_owned())
    );
    Ok(())
}

#[test]
fn reserved_field_message_redacts_malformed_identifiers() {
    let error = RqsError::ReservedFieldName {
        field: "limit\nsecret".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field [redacted] is reserved for query controls"
    );
}

#[test]
fn filter_error_redacts_line_feed() {
    let result =
        parse("bad%0AINJECTED=value", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn filter_error_redacts_carriage_return() {
    let result =
        parse("bad%0DINJECTED=value", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn filter_error_redacts_terminal_escape() {
    let result =
        parse("bad%1B%5B31m=value", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn filter_error_redacts_unicode_direction_override() {
    let result =
        parse("bad%E2%80%AE=value", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn malformed_filter_error_does_not_disclose_value_text() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("name")?;
    let result = parse("name%0A=secret%3Eother", &catalog).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
    Ok(())
}

#[test]
fn sort_error_redacts_invalid_field() {
    let result =
        parse("sort=bad%0AINJECTED", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn projection_error_redacts_invalid_field() {
    let result =
        parse("fields=bad%1B%5B31m", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn existence_error_redacts_invalid_field() {
    let result = parse("!bad%0AINJECTED", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(result, Err("field [redacted] is invalid".to_owned()));
}

#[test]
fn malformed_field_uses_invalid_field_name_code() {
    let result =
        parse("bad%0AINJECTED=value", &FieldCatalog::new()).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_field_name"));
}

#[test]
fn unknown_field_error_preserves_valid_dotted_identifier() {
    let result =
        parse("profile._name2=value", &FieldCatalog::new()).map_err(|error| error.to_string());

    assert_eq!(
        result,
        Err("field profile._name2 is not allowed".to_owned())
    );
}

#[test]
fn invalid_column_error_redacts_sql_text() {
    let message = RqsError::InvalidColumnName {
        column: "users.name; SELECT secret".to_owned(),
    }
    .to_string();

    assert_eq!(message, "column [redacted] is invalid");
}

#[test]
fn missing_mapping_error_redacts_invalid_field() {
    let error = RqsError::MissingColumnMapping {
        field: "name\nsecret".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field [redacted] has no SQL column mapping"
    );
}

#[test]
fn duplicate_mapping_error_redacts_invalid_field() {
    let error = RqsError::DuplicateColumnMapping {
        field: "name=secret".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field [redacted] has more than one SQL column mapping"
    );
}

#[test]
fn manually_constructed_unknown_field_error_redacts_malformed_identifier() {
    let error = RqsError::UnknownField {
        field: "name=secret".to_owned(),
    };

    assert_eq!(error.to_string(), "field [redacted] is not allowed");
}

#[test]
fn unknown_field_error_redacts_line_feed() {
    let error = RqsError::UnknownField {
        field: "name\nINJECTED".to_owned(),
    };

    assert_eq!(error.to_string(), "field [redacted] is not allowed");
}

#[test]
fn unknown_field_error_redacts_terminal_escape() {
    let error = RqsError::UnknownField {
        field: "name\u{1b}[31m".to_owned(),
    };

    assert_eq!(error.to_string(), "field [redacted] is not allowed");
}

#[test]
fn invalid_column_error_redacts_carriage_return() {
    let error = RqsError::InvalidColumnName {
        column: "users.name\rINJECTED".to_owned(),
    };

    assert_eq!(error.to_string(), "column [redacted] is invalid");
}

#[test]
fn invalid_field_error_redacts_nul() {
    let error = RqsError::InvalidFieldName {
        field: "name\0secret".to_owned(),
    };

    assert_eq!(error.to_string(), "field [redacted] is invalid");
}

#[test]
fn invalid_value_error_omits_decoded_value_text() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let result = parse("age=private%0Avalue%1B%5B31m", &catalog).map_err(|error| error.to_string());

    assert_eq!(result, Err("field age needs integer".to_owned()));
    Ok(())
}

#[test]
fn duplicate_filter_error_omits_both_values() -> restqs::RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("status")?;
    let result = parse("status=first_secret&status=second%0Asecret", &catalog)
        .map_err(|error| error.to_string());

    assert_eq!(result, Err("field status repeats operator =".to_owned()));
    Ok(())
}

#[test]
fn missing_value_error_redacts_invalid_field() {
    let error = RqsError::MissingValue {
        field: "bad\nINJECTED".to_owned(),
    };

    assert_eq!(error.to_string(), "field [redacted] needs a value");
}

#[test]
fn invalid_value_error_redacts_invalid_field() {
    let error = RqsError::InvalidValue {
        field: "bad\nINJECTED".to_owned(),
        expected: "integer",
    };

    assert_eq!(error.to_string(), "field [redacted] needs integer");
}

#[test]
fn value_too_large_error_redacts_invalid_field() {
    let error = RqsError::ValueTooLarge {
        field: "bad\nINJECTED".to_owned(),
        max_bytes: 3,
    };

    assert_eq!(error.to_string(), "field [redacted] exceeds 3 bytes");
}

#[test]
fn too_many_list_items_error_redacts_invalid_field() {
    let error = RqsError::TooManyListItems {
        field: "bad\nINJECTED".to_owned(),
        max_items: 2,
    };

    assert_eq!(error.to_string(), "field [redacted] exceeds 2 list items");
}

#[test]
fn regex_disabled_error_redacts_invalid_field() {
    let error = RqsError::RegexDisabled {
        field: "bad\nINJECTED".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field [redacted] does not allow regex filters"
    );
}

#[test]
fn duplicate_filter_error_redacts_invalid_field() {
    let error = RqsError::DuplicateFilter {
        field: "bad\nINJECTED".to_owned(),
        operator: ">",
    };

    assert_eq!(error.to_string(), "field [redacted] repeats operator >");
}

#[test]
fn error_preserves_identifier_at_display_byte_limit() {
    let field = "a".repeat(128);
    let error = RqsError::UnknownField {
        field: field.clone(),
    };

    assert_eq!(error.to_string(), format!("field {field} is not allowed"));
}

#[test]
fn error_redacts_identifier_above_display_byte_limit() {
    let error = RqsError::UnknownField {
        field: "a".repeat(129),
    };

    assert_eq!(error.to_string(), "field [redacted] is not allowed");
}
