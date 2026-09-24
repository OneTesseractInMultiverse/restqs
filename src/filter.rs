//! Filter expression types and parsing helpers.

use crate::{
    FieldRef, ParserLimits, RqsError, RqsResult, RqsValue, limits::validate_value_size,
    value::parse_value,
};

/// Supported filter operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FilterOp {
    /// Equals.
    Eq,
    /// Not equals.
    Ne,
    /// Greater than.
    Gt,
    /// Greater than or equal.
    Gte,
    /// Less than.
    Lt,
    /// Less than or equal.
    Lte,
    /// Field is not null.
    Exists,
    /// Field is null.
    NotExists,
    /// Field is in list.
    In,
    /// Field is not in list.
    NotIn,
    /// Regex match, parsed only from equality with a regex literal.
    Regex,
}

impl FilterOp {
    /// Return the RQS operator token.
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Exists => "exists",
            Self::NotExists => "not_exists",
            Self::In => "in",
            Self::NotIn => "not_in",
            Self::Regex => "regex",
        }
    }
}

/// Regex literal parsed from slash form.
///
/// Suffix flags are unique lowercase letters from `i`, `m`, `s`, and `x`.
/// Adapters must translate each requested flag or explicitly reject it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexLiteral {
    pattern: String,
    flags: String,
}

impl RegexLiteral {
    /// Return the raw pattern without slash delimiters.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Return validated regex flags in input order, without duplicates.
    #[must_use]
    pub fn flags(&self) -> &str {
        &self.flags
    }

    #[cfg(all(test, feature = "sqlx"))]
    pub(crate) fn new_for_test(pattern: &str, flags: &str) -> Self {
        Self {
            pattern: pattern.to_owned(),
            flags: flags.to_owned(),
        }
    }
}

/// One filter in an RQS plan.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    field: FieldRef,
    op: FilterOp,
    value: Option<RqsValue>,
    regex: Option<RegexLiteral>,
}

impl Filter {
    /// Create a filter.
    #[must_use]
    pub(crate) fn new(field: FieldRef, op: FilterOp, value: Option<RqsValue>) -> Self {
        Self {
            field,
            op,
            value,
            regex: None,
        }
    }

    /// Create a regex filter.
    #[must_use]
    pub(crate) fn regex(field: FieldRef, regex: RegexLiteral) -> Self {
        Self {
            field,
            op: FilterOp::Regex,
            value: None,
            regex: Some(regex),
        }
    }

    /// Return the field.
    #[must_use]
    pub fn field(&self) -> &FieldRef {
        &self.field
    }

    /// Return the operator.
    #[must_use]
    pub fn op(&self) -> FilterOp {
        self.op
    }

    /// Return the value.
    #[must_use]
    pub fn value(&self) -> Option<&RqsValue> {
        self.value.as_ref()
    }

    /// Return the regex literal.
    #[must_use]
    pub fn regex_literal(&self) -> Option<&RegexLiteral> {
        self.regex.as_ref()
    }
}

/// Parse an RQS value into a filter.
pub(crate) fn build_value_filter(
    field: FieldRef,
    op: FilterOp,
    raw_value: &str,
    limits: ParserLimits,
) -> RqsResult<Filter> {
    validate_value_size(field.public_name(), raw_value, limits.max_value_bytes)?;
    if raw_value.is_empty() {
        return Err(RqsError::MissingValue {
            field: field.public_name().to_owned(),
        });
    }

    if let Some(regex) = parse_regex_literal(raw_value) {
        validate_regex_operator(op)?;
        if !field.regex_allowed() {
            return Err(RqsError::RegexDisabled {
                field: field.public_name().to_owned(),
            });
        }
        validate_regex_flags(regex.flags())?;
        return Ok(Filter::regex(field, regex));
    }

    let value = parse_value(field.public_name(), raw_value, field.value_kind(), limits)?;
    let op = list_operator(op, &value);
    Ok(Filter::new(field, op, Some(value)))
}

fn validate_regex_operator(op: FilterOp) -> RqsResult<()> {
    if op == FilterOp::Eq {
        Ok(())
    } else {
        Err(RqsError::InvalidOperator)
    }
}

fn validate_regex_flags(flags: &str) -> RqsResult<()> {
    let mut seen = 0_u8;
    for flag in flags.bytes() {
        let bit = match flag {
            b'i' => 1,
            b'm' => 2,
            b's' => 4,
            b'x' => 8,
            _ => return Err(RqsError::InvalidRegexFlags),
        };
        if seen & bit != 0 {
            return Err(RqsError::InvalidRegexFlags);
        }
        seen |= bit;
    }
    Ok(())
}

fn list_operator(op: FilterOp, value: &RqsValue) -> FilterOp {
    match (op, value) {
        (FilterOp::Eq, RqsValue::List(_)) => FilterOp::In,
        (FilterOp::Ne, RqsValue::List(_)) => FilterOp::NotIn,
        _ => op,
    }
}

fn parse_regex_literal(raw: &str) -> Option<RegexLiteral> {
    if !raw.starts_with('/') {
        return None;
    }

    let offset = raw[1..].rfind('/')?;
    let end = offset + 1;

    let pattern = raw[1..end].to_owned();
    let flags = raw[end + 1..].to_owned();
    Some(RegexLiteral { pattern, flags })
}
