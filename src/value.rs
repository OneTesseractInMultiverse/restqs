//! Typed values used in RQS plans.

use crate::{
    ParserLimits, RqsError, RqsResult, ValueKind,
    temporal::{is_valid_date, is_valid_datetime},
};

/// Value owned by an RQS plan.
#[derive(Debug, Clone, PartialEq)]
pub enum RqsValue {
    /// Null value.
    Null,
    /// Boolean value.
    Boolean(bool),
    /// Signed 64-bit integer value.
    Integer(i64),
    /// 64-bit floating point value.
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
    /// UUID string.
    Uuid(String),
    /// List of values.
    List(Vec<RqsValue>),
}

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

fn parse_scalar(field: &str, raw: &str, kind: ValueKind) -> RqsResult<RqsValue> {
    match kind {
        ValueKind::Text => Ok(RqsValue::Text(raw.to_owned())),
        ValueKind::Integer => raw
            .parse::<i64>()
            .map(RqsValue::Integer)
            .map_err(|_| invalid_value(field, kind)),
        ValueKind::Float => raw
            .parse::<f64>()
            .map(RqsValue::Float)
            .map_err(|_| invalid_value(field, kind)),
        ValueKind::Boolean => parse_boolean(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::Date => parse_date(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::DateTime => parse_datetime(raw).ok_or_else(|| invalid_value(field, kind)),
        ValueKind::Uuid => parse_uuid(raw).ok_or_else(|| invalid_value(field, kind)),
    }
}

fn parse_boolean(raw: &str) -> Option<RqsValue> {
    match raw.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(RqsValue::Boolean(true)),
        "0" | "false" | "no" | "off" => Some(RqsValue::Boolean(false)),
        _ => None,
    }
}

fn parse_date(raw: &str) -> Option<RqsValue> {
    is_valid_date(raw).then(|| RqsValue::Date(raw.to_owned()))
}

fn parse_datetime(raw: &str) -> Option<RqsValue> {
    is_valid_datetime(raw).then(|| RqsValue::DateTime(raw.to_owned()))
}

fn parse_uuid(raw: &str) -> Option<RqsValue> {
    if is_hyphenated_uuid(raw) || is_compact_uuid(raw) {
        Some(RqsValue::Uuid(raw.to_ascii_lowercase()))
    } else {
        None
    }
}

fn is_hyphenated_uuid(raw: &str) -> bool {
    raw.len() == 36
        && [8, 13, 18, 23]
            .into_iter()
            .all(|index| raw.as_bytes().get(index) == Some(&b'-'))
        && raw.chars().enumerate().all(|(index, character)| {
            [8, 13, 18, 23].contains(&index) || character.is_ascii_hexdigit()
        })
}

fn is_compact_uuid(raw: &str) -> bool {
    raw.len() == 32 && raw.chars().all(|character| character.is_ascii_hexdigit())
}

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

fn invalid_value(field: &str, kind: ValueKind) -> RqsError {
    RqsError::InvalidValue {
        field: field.to_owned(),
        expected: kind.expected_name(),
    }
}
