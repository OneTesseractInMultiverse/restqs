//! Error types for RQS parsing and adapter translation.

use std::fmt::{self, Display, Formatter};

use crate::identifier::is_dotted_identifier;

const MAX_DIAGNOSTIC_IDENTIFIER_BYTES: usize = 128;

/// Result type used by RestQS.
pub type RqsResult<T> = Result<T, RqsError>;

/// RQS parser and adapter errors.
///
/// Display text replaces malformed field and column identifiers, and identifiers
/// longer than 128 bytes, with `[redacted]`. Error fields and `Debug` output retain
/// the original data and should not be used for untrusted-input logging.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RqsError {
    /// The raw query exceeded the configured byte limit.
    QueryTooLarge {
        /// Maximum accepted byte length.
        max_bytes: usize,
    },
    /// The query contained more parameters than allowed.
    TooManyParameters {
        /// Maximum accepted parameter count.
        max_parameters: usize,
    },
    /// A percent-encoded parameter used invalid syntax.
    InvalidEncoding,
    /// A public field name is not valid.
    InvalidFieldName {
        /// Invalid field name.
        field: String,
    },
    /// A database column name is not valid.
    InvalidColumnName {
        /// Invalid column name.
        column: String,
    },
    /// A query referenced a field that is not in the catalog.
    UnknownField {
        /// Unknown public field name.
        field: String,
    },
    /// A filter used invalid operator syntax or an unsupported operator/value combination.
    ///
    /// Regex literals support equality only; other comparison operators are rejected.
    InvalidOperator,
    /// A filter value was missing.
    MissingValue {
        /// Field that needed a value.
        field: String,
    },
    /// A value failed catalog type casting.
    InvalidValue {
        /// Field that failed parsing.
        field: String,
        /// Expected value type.
        expected: &'static str,
    },
    /// A decoded filter or query-control value exceeded the configured byte limit.
    ValueTooLarge {
        /// Public field or query-control name that received the value.
        field: String,
        /// Maximum accepted byte length.
        max_bytes: usize,
    },
    /// A list contained too many items.
    TooManyListItems {
        /// Field that received the list.
        field: String,
        /// Maximum accepted item count.
        max_items: usize,
    },
    /// Pagination syntax was invalid.
    InvalidPagination {
        /// Parameter name.
        parameter: &'static str,
    },
    /// Pagination value cannot be negative.
    NegativePagination {
        /// Parameter name.
        parameter: &'static str,
    },
    /// The requested limit exceeded the configured maximum.
    LimitTooLarge {
        /// Maximum accepted limit.
        max_limit: u64,
    },
    /// Regex filtering was requested for a field without regex permission.
    RegexDisabled {
        /// Field that received a regex value.
        field: String,
    },
    /// Text search is not part of the current RQS contract.
    TextSearchUnsupported,
    /// A filter repeated the same field and operator.
    DuplicateFilter {
        /// Field used more than once.
        field: String,
        /// Repeated operator.
        operator: &'static str,
    },
    /// SQL translation cannot represent a filter with this adapter.
    AdapterUnsupported {
        /// Unsupported feature name.
        feature: &'static str,
    },
}

impl RqsError {
    /// Return a stable machine-readable error code.
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::QueryTooLarge { .. } => "query_too_large",
            Self::TooManyParameters { .. } => "too_many_parameters",
            Self::InvalidEncoding => "invalid_encoding",
            Self::InvalidFieldName { .. } => "invalid_field_name",
            Self::InvalidColumnName { .. } => "invalid_column_name",
            Self::UnknownField { .. } => "unknown_field",
            Self::InvalidOperator => "invalid_operator",
            Self::MissingValue { .. } => "missing_value",
            Self::InvalidValue { .. } => "invalid_value",
            Self::ValueTooLarge { .. } => "value_too_large",
            Self::TooManyListItems { .. } => "too_many_list_items",
            Self::InvalidPagination { .. } => "invalid_pagination",
            Self::NegativePagination { .. } => "negative_pagination",
            Self::LimitTooLarge { .. } => "limit_too_large",
            Self::RegexDisabled { .. } => "regex_disabled",
            Self::TextSearchUnsupported => "text_search_unsupported",
            Self::DuplicateFilter { .. } => "duplicate_filter",
            Self::AdapterUnsupported { .. } => "adapter_unsupported",
        }
    }
}

impl Display for RqsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueryTooLarge { max_bytes } => {
                write!(formatter, "query exceeds {max_bytes} bytes")
            }
            Self::TooManyParameters { max_parameters } => {
                write!(formatter, "query exceeds {max_parameters} parameters")
            }
            Self::InvalidEncoding => write!(formatter, "query uses invalid percent encoding"),
            Self::InvalidFieldName { field } => {
                write!(
                    formatter,
                    "field {} is invalid",
                    diagnostic_identifier(field)
                )
            }
            Self::InvalidColumnName { column } => write!(
                formatter,
                "column {} is invalid",
                diagnostic_identifier(column)
            ),
            Self::UnknownField { field } => write!(
                formatter,
                "field {} is not allowed",
                diagnostic_identifier(field)
            ),
            Self::InvalidOperator => write!(formatter, "filter operator is invalid"),
            Self::MissingValue { field } => write!(
                formatter,
                "field {} needs a value",
                diagnostic_identifier(field)
            ),
            Self::InvalidValue { field, expected } => {
                write!(
                    formatter,
                    "field {} needs {expected}",
                    diagnostic_identifier(field)
                )
            }
            Self::ValueTooLarge { field, max_bytes } => {
                write!(
                    formatter,
                    "field {} exceeds {max_bytes} bytes",
                    diagnostic_identifier(field)
                )
            }
            Self::TooManyListItems { field, max_items } => {
                write!(
                    formatter,
                    "field {} exceeds {max_items} list items",
                    diagnostic_identifier(field)
                )
            }
            Self::InvalidPagination { parameter } => {
                write!(formatter, "{parameter} pagination value is invalid")
            }
            Self::NegativePagination { parameter } => {
                write!(formatter, "{parameter} pagination value cannot be negative")
            }
            Self::LimitTooLarge { max_limit } => {
                write!(formatter, "limit exceeds {max_limit}")
            }
            Self::RegexDisabled { field } => {
                write!(
                    formatter,
                    "field {} does not allow regex filters",
                    diagnostic_identifier(field)
                )
            }
            Self::TextSearchUnsupported => write!(formatter, "text search is not supported"),
            Self::DuplicateFilter { field, operator } => {
                write!(
                    formatter,
                    "field {} repeats operator {operator}",
                    diagnostic_identifier(field)
                )
            }
            Self::AdapterUnsupported { feature } => {
                write!(formatter, "adapter does not support {feature}")
            }
        }
    }
}

impl std::error::Error for RqsError {}

fn diagnostic_identifier(value: &str) -> &str {
    if value.len() <= MAX_DIAGNOSTIC_IDENTIFIER_BYTES && is_dotted_identifier(value) {
        value
    } else {
        "[redacted]"
    }
}
