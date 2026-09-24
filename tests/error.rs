#![allow(missing_docs)]

use restqs::RqsError;

#[test]
fn all_error_codes_are_stable() {
    let errors = [
        RqsError::QueryTooLarge { max_bytes: 1 },
        RqsError::TooManyParameters { max_parameters: 1 },
        RqsError::InvalidEncoding,
        RqsError::InvalidFieldName {
            field: "bad".to_owned(),
        },
        RqsError::InvalidColumnName {
            column: "bad".to_owned(),
        },
        RqsError::MissingColumnMapping {
            field: "status".to_owned(),
        },
        RqsError::DuplicateColumnMapping {
            field: "status".to_owned(),
        },
        RqsError::UnknownField {
            field: "bad".to_owned(),
        },
        RqsError::InvalidOperator,
        RqsError::MissingValue {
            field: "age".to_owned(),
        },
        RqsError::InvalidValue {
            field: "age".to_owned(),
            expected: "integer",
        },
        RqsError::ValueTooLarge {
            field: "name".to_owned(),
            max_bytes: 1,
        },
        RqsError::TooManyListItems {
            field: "status".to_owned(),
            max_items: 1,
        },
        RqsError::InvalidPagination { parameter: "limit" },
        RqsError::NegativePagination { parameter: "skip" },
        RqsError::LimitTooLarge { max_limit: 1 },
        RqsError::RegexDisabled {
            field: "email".to_owned(),
        },
        RqsError::InvalidRegexFlags,
        RqsError::TextSearchUnsupported,
        RqsError::DuplicateFilter {
            field: "age".to_owned(),
            operator: ">",
        },
        RqsError::AdapterUnsupported { feature: "regex" },
    ];

    assert_eq!(
        errors.map(|error| error.error_code()),
        [
            "query_too_large",
            "too_many_parameters",
            "invalid_encoding",
            "invalid_field_name",
            "invalid_column_name",
            "missing_column_mapping",
            "duplicate_column_mapping",
            "unknown_field",
            "invalid_operator",
            "missing_value",
            "invalid_value",
            "value_too_large",
            "too_many_list_items",
            "invalid_pagination",
            "negative_pagination",
            "limit_too_large",
            "regex_disabled",
            "invalid_regex_flags",
            "text_search_unsupported",
            "duplicate_filter",
            "adapter_unsupported"
        ]
    );
}

#[test]
fn all_error_display_messages_are_safe() {
    let messages = [
        RqsError::QueryTooLarge { max_bytes: 1 }.to_string(),
        RqsError::TooManyParameters { max_parameters: 1 }.to_string(),
        RqsError::InvalidEncoding.to_string(),
        RqsError::InvalidFieldName {
            field: "bad".to_owned(),
        }
        .to_string(),
        RqsError::InvalidColumnName {
            column: "bad".to_owned(),
        }
        .to_string(),
        RqsError::MissingColumnMapping {
            field: "status".to_owned(),
        }
        .to_string(),
        RqsError::DuplicateColumnMapping {
            field: "status".to_owned(),
        }
        .to_string(),
        RqsError::UnknownField {
            field: "bad".to_owned(),
        }
        .to_string(),
        RqsError::InvalidOperator.to_string(),
        RqsError::MissingValue {
            field: "age".to_owned(),
        }
        .to_string(),
        RqsError::InvalidValue {
            field: "age".to_owned(),
            expected: "integer",
        }
        .to_string(),
        RqsError::ValueTooLarge {
            field: "name".to_owned(),
            max_bytes: 1,
        }
        .to_string(),
        RqsError::TooManyListItems {
            field: "status".to_owned(),
            max_items: 1,
        }
        .to_string(),
        RqsError::InvalidPagination { parameter: "limit" }.to_string(),
        RqsError::NegativePagination { parameter: "skip" }.to_string(),
        RqsError::LimitTooLarge { max_limit: 1 }.to_string(),
        RqsError::RegexDisabled {
            field: "email".to_owned(),
        }
        .to_string(),
        RqsError::InvalidRegexFlags.to_string(),
        RqsError::TextSearchUnsupported.to_string(),
        RqsError::DuplicateFilter {
            field: "age".to_owned(),
            operator: ">",
        }
        .to_string(),
        RqsError::AdapterUnsupported { feature: "regex" }.to_string(),
    ];

    assert_eq!(messages.len(), 21);
}
