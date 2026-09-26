//! Typed values used in RQS plans.

use crate::{
    ParserLimits, RqsError, RqsResult, ValueKind,
    temporal::{is_valid_date, is_valid_datetime},
};

/// Value owned by an RQS plan.
///
/// Parser-created values obey the catalog type and input budgets. This public enum
/// can also be constructed directly; doing so does not perform validation. Adapters
/// and repositories define storage conversion and unsupported-value behavior.
#[derive(Debug, Clone, PartialEq)]
pub enum RqsValue {
    /// Null value.
    Null,
    /// Boolean value.
    Boolean(bool),
    /// Signed 64-bit integer value.
    Integer(i64),
    /// 64-bit floating point value.
    ///
    /// Parsing accepts only finite results, including subnormal values and signed
    /// zero. NaN, infinities, and overflow return [`RqsError::InvalidValue`].
    Float(f64),
    /// UTF-8 text.
    Text(String),
    /// Date string in `YYYY-MM-DD` form.
    ///
    /// Parsed dates follow Gregorian calendar rules with years `0000..=9999`.
    Date(String),
    /// Date-time string in `YYYY-MM-DD[Tt]HH:MM:SS[.digits](Z|z|+HH:MM|-HH:MM)` form.
    ///
    /// Parsed timestamps require a valid Gregorian date and an explicit offset.
    /// Clock and offset hours are `00..=23`; minutes and seconds are `00..=59`.
    /// Leap seconds are unsupported. Fractional seconds require at least one
    /// ASCII digit. Precision, letter case, and offsets are preserved without
    /// normalization, including `-00:00`.
    DateTime(String),
    /// Lowercase UUID text in its original compact or hyphenated layout.
    ///
    /// The parser checks hexadecimal shape, not UUID version or variant semantics.
    Uuid(String),
    /// Homogeneous scalar membership operands in input order.
    ///
    /// Parser-created lists are not nested. An empty list is a valid core value
    /// but the SQL adapter rejects it because portable empty IN syntax is absent.
    List(Vec<RqsValue>),
}

/// Recognize a top-level case-insensitive null, then dispatch a known wrapper or catalog-typed
/// scalar. Wrappers cannot override the catalog kind.
pub(crate) fn parse_value(
    field: &str,
    raw: &str,
    kind: ValueKind,
    limits: ParserLimits,
) -> RqsResult<RqsValue> {
    if raw.eq_ignore_ascii_case("null") {
        return Ok(RqsValue::Null);
    }

    match parse_cast_wrapper(raw) {
        Some(("in" | "list", inner)) => parse_list(field, inner, kind, limits),
        Some((cast, inner)) => parse_casted_scalar(field, cast, inner, kind),
        None => parse_scalar(field, raw, kind),
    }
}

/// Split comma-delimited items, enforce the item count, and parse trimmed scalars of one catalog
/// kind. Empty lists remain valid core values; items are not recursively parsed as wrappers or
/// nulls.
fn parse_list(
    field: &str,
    raw: &str,
    kind: ValueKind,
    limits: ParserLimits,
) -> RqsResult<RqsValue> {
    if raw.trim().is_empty() {
        return Ok(RqsValue::List(Vec::new()));
    }

    let items = raw.split(',').collect::<Vec<_>>();
    if items.len() > limits.max_list_items {
        return Err(RqsError::TooManyListItems {
            field: field.to_owned(),
            max_items: limits.max_list_items,
        });
    }

    let parsed = items
        .into_iter()
        .map(|item| parse_scalar(field, item.trim(), kind))
        .collect::<RqsResult<Vec<_>>>()?;
    Ok(RqsValue::List(parsed))
}

/// Require the wrapper to match the catalog kind before converting its inner scalar.
fn parse_casted_scalar(field: &str, cast: &str, raw: &str, kind: ValueKind) -> RqsResult<RqsValue> {
    let expected = cast_for_kind(kind);
    if cast != expected {
        return Err(RqsError::InvalidValue {
            field: field.to_owned(),
            expected: kind.expected_name(),
        });
    }

    parse_scalar(field, raw, kind)
}

/// Convert one scalar by catalog kind and map conversion failures to the field's typed error.
fn parse_scalar(field: &str, raw: &str, kind: ValueKind) -> RqsResult<RqsValue> {
    match kind {
        ValueKind::Text => Ok(RqsValue::Text(raw.to_owned())),
        ValueKind::Integer => raw
            .parse::<i64>()
            .map(RqsValue::Integer)
            .map_err(|_| invalid_value(field, kind)),
        ValueKind::Float => parse_float(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::Boolean => parse_boolean(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::Date => parse_date(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::DateTime => parse_datetime(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::Uuid => parse_uuid(raw).ok_or_else(|| invalid_value(field, kind)),
    }
}

/// Parse a finite f64, preserving signed zero and subnormals; reject NaN, infinity, and overflow.
fn parse_float(raw: &str) -> Option<RqsValue> {
    let value = raw.parse::<f64>().ok()?;
    value.is_finite().then_some(RqsValue::Float(value))
}

/// Accept case-insensitive true/yes/on/1 and false/no/off/0; all other spellings return None.
fn parse_boolean(raw: &str) -> Option<RqsValue> {
    match raw.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(RqsValue::Boolean(true)),
        "0" | "false" | "no" | "off" => Some(RqsValue::Boolean(false)),
        _ => None,
    }
}

/// Validate a Gregorian date and preserve its original text for the repository's storage
/// conversion.
fn parse_date(raw: &str) -> Option<RqsValue> {
    is_valid_date(raw).then(|| RqsValue::Date(raw.to_owned()))
}

/// Validate an offset timestamp and preserve precision, case, and offset rather than normalizing
/// it.
fn parse_datetime(raw: &str) -> Option<RqsValue> {
    is_valid_datetime(raw).then(|| RqsValue::DateTime(raw.to_owned()))
}

/// Validate compact or hyphenated hexadecimal UUID text and lowercase it while preserving its
/// layout.
fn parse_uuid(raw: &str) -> Option<RqsValue> {
    if is_hyphenated_uuid(raw) || is_compact_uuid(raw) {
        Some(RqsValue::Uuid(raw.to_ascii_lowercase()))
    } else {
        None
    }
}

/// Require exactly 32 ASCII hexadecimal digits in the conventional 8-4-4-4-12 layout.
fn is_hyphenated_uuid(raw: &str) -> bool {
    raw.len() == 36
        && [8, 13, 18, 23]
            .into_iter()
            .all(|index| raw.as_bytes().get(index) == Some(&b'-'))
        && raw.chars().enumerate().all(|(index, character)| {
            [8, 13, 18, 23].contains(&index) || character.is_ascii_hexdigit()
        })
}

/// Require exactly 32 ASCII hexadecimal digits without separators.
fn is_compact_uuid(raw: &str) -> bool {
    raw.len() == 32 && raw.chars().all(|character| character.is_ascii_hexdigit())
}

/// Recognize a supported lowercase wrapper with a final closing parenthesis; unknown wrappers
/// remain scalar text.
fn parse_cast_wrapper(raw: &str) -> Option<(&str, &str)> {
    let open = raw.find('(')?;
    if !raw.ends_with(')') {
        return None;
    }
    let cast = &raw[..open];
    let inner = &raw[open + 1..raw.len() - 1];
    match cast {
        "str" | "int" | "float" | "bool" | "date" | "datetime" | "uuid" | "in" | "list" => {
            Some((cast, inner))
        }
        _ => None,
    }
}

/// Map each catalog kind to its sole accepted scalar-wrapper name.
fn cast_for_kind(kind: ValueKind) -> &'static str {
    match kind {
        ValueKind::Text => "str",
        ValueKind::Integer => "int",
        ValueKind::Float => "float",
        ValueKind::Boolean => "bool",
        ValueKind::Date => "date",
        ValueKind::DateTime => "datetime",
        ValueKind::Uuid => "uuid",
    }
}

/// Construct a conversion error containing the logical field and expected kind, without copying
/// the rejected value.
fn invalid_value(field: &str, kind: ValueKind) -> RqsError {
    RqsError::InvalidValue {
        field: field.to_owned(),
        expected: kind.expected_name(),
    }
}
