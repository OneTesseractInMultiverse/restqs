#![allow(missing_docs)]

use restqs::RqsError;

#[test]
fn reserved_field_message_identifies_the_control_collision() {
    let error = RqsError::ReservedFieldName {
        field: "limit".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field limit is reserved for query controls"
    );
}

#[test]
fn query_size_message_reports_the_byte_limit() {
    let error = RqsError::QueryTooLarge { max_bytes: 8192 };

    assert_eq!(error.to_string(), "query exceeds 8192 bytes");
}

#[test]
fn parameter_count_message_reports_the_limit() {
    let error = RqsError::TooManyParameters {
        max_parameters: 128,
    };

    assert_eq!(error.to_string(), "query exceeds 128 parameters");
}

#[test]
fn invalid_encoding_message_describes_percent_encoding() {
    assert_eq!(
        RqsError::InvalidEncoding.to_string(),
        "query uses invalid percent encoding"
    );
}

#[test]
fn invalid_field_message_preserves_a_safe_identifier() {
    let error = RqsError::InvalidFieldName {
        field: "profile._name2".to_owned(),
    };

    assert_eq!(error.to_string(), "field profile._name2 is invalid");
}

#[test]
fn invalid_column_message_preserves_a_safe_identifier() {
    let error = RqsError::InvalidColumnName {
        column: "users.status".to_owned(),
    };

    assert_eq!(error.to_string(), "column users.status is invalid");
}

#[test]
fn missing_mapping_message_names_the_logical_field() {
    let error = RqsError::MissingColumnMapping {
        field: "status".to_owned(),
    };

    assert_eq!(error.to_string(), "field status has no SQL column mapping");
}

#[test]
fn duplicate_mapping_message_names_the_logical_field() {
    let error = RqsError::DuplicateColumnMapping {
        field: "status".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field status has more than one SQL column mapping"
    );
}

#[test]
fn unknown_field_message_preserves_a_safe_identifier() {
    let error = RqsError::UnknownField {
        field: "profile._name2".to_owned(),
    };

    assert_eq!(error.to_string(), "field profile._name2 is not allowed");
}

#[test]
fn invalid_operator_message_describes_the_syntax_error() {
    assert_eq!(
        RqsError::InvalidOperator.to_string(),
        "filter operator is invalid"
    );
}

#[test]
fn missing_value_message_names_the_field() {
    let error = RqsError::MissingValue {
        field: "age".to_owned(),
    };

    assert_eq!(error.to_string(), "field age needs a value");
}

#[test]
fn invalid_value_message_reports_the_expected_type() {
    let error = RqsError::InvalidValue {
        field: "age".to_owned(),
        expected: "integer",
    };

    assert_eq!(error.to_string(), "field age needs integer");
}

#[test]
fn oversized_value_message_reports_the_field_and_byte_limit() {
    let error = RqsError::ValueTooLarge {
        field: "name".to_owned(),
        max_bytes: 2048,
    };

    assert_eq!(error.to_string(), "field name exceeds 2048 bytes");
}

#[test]
fn list_size_message_reports_the_field_and_item_limit() {
    let error = RqsError::TooManyListItems {
        field: "status".to_owned(),
        max_items: 100,
    };

    assert_eq!(error.to_string(), "field status exceeds 100 list items");
}

#[test]
fn invalid_pagination_message_names_the_parameter() {
    let error = RqsError::InvalidPagination { parameter: "limit" };

    assert_eq!(error.to_string(), "limit pagination value is invalid");
}

#[test]
fn negative_pagination_message_names_the_parameter() {
    let error = RqsError::NegativePagination { parameter: "skip" };

    assert_eq!(
        error.to_string(),
        "skip pagination value cannot be negative"
    );
}

#[test]
fn excessive_limit_message_reports_the_maximum() {
    let error = RqsError::LimitTooLarge { max_limit: 100 };

    assert_eq!(error.to_string(), "limit exceeds 100");
}

#[test]
fn regex_permission_message_names_the_field() {
    let error = RqsError::RegexDisabled {
        field: "email".to_owned(),
    };

    assert_eq!(
        error.to_string(),
        "field email does not allow regex filters"
    );
}

#[test]
fn invalid_regex_flags_message_describes_the_allowed_flags() {
    assert_eq!(
        RqsError::InvalidRegexFlags.to_string(),
        "regex flags must be unique letters from i, m, s, x"
    );
}

#[test]
fn text_search_message_describes_the_unsupported_feature() {
    assert_eq!(
        RqsError::TextSearchUnsupported.to_string(),
        "text search is not supported"
    );
}

#[test]
fn duplicate_filter_message_reports_the_field_and_operator() {
    let error = RqsError::DuplicateFilter {
        field: "age".to_owned(),
        operator: ">=",
    };

    assert_eq!(error.to_string(), "field age repeats operator >=");
}

#[test]
fn unsupported_adapter_message_names_the_feature() {
    let error = RqsError::AdapterUnsupported {
        feature: "sqlite regex",
    };

    assert_eq!(error.to_string(), "adapter does not support sqlite regex");
}
