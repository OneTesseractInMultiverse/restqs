//! Allowlisted fields and database column metadata.

use std::collections::BTreeMap;

use crate::{RqsError, RqsResult};

/// Value type expected by an allowlisted field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    /// UTF-8 text.
    Text,
    /// Signed 64-bit integer.
    Integer,
    /// 64-bit floating point number.
    Float,
    /// Boolean value.
    Boolean,
    /// Calendar date in `YYYY-MM-DD` form.
    Date,
    /// Date-time text in RFC3339-like form.
    DateTime,
    /// UUID text.
    Uuid,
}

impl ValueKind {
    /// Return a human-readable type name.
    #[must_use]
    pub fn expected_name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::DateTime => "datetime",
            Self::Uuid => "uuid",
        }
    }
}

/// One public field mapped to one database column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    public_name: String,
    column_name: String,
    value_kind: ValueKind,
    regex_allowed: bool,
}

impl Field {
    /// Create one allowlisted field mapping.
    pub fn new(
        public_name: impl Into<String>,
        column_name: impl Into<String>,
        value_kind: ValueKind,
    ) -> RqsResult<Self> {
        let public_name = public_name.into();
        let column_name = column_name.into();
        validate_public_name(&public_name)?;
        validate_column_name(&column_name)?;
        Ok(Self {
            public_name,
            column_name,
            value_kind,
            regex_allowed: false,
        })
    }

    /// Permit regex values for this field.
    #[must_use]
    pub fn allow_regex(mut self) -> Self {
        self.regex_allowed = true;
        self
    }

    /// Return the public query field name.
    #[must_use]
    pub fn public_name(&self) -> &str {
        &self.public_name
    }

    /// Return the allowlisted database column.
    #[must_use]
    pub fn column_name(&self) -> &str {
        &self.column_name
    }

    /// Return the expected value type.
    #[must_use]
    pub fn value_kind(&self) -> ValueKind {
        self.value_kind
    }

    /// Return true when regex values are allowed.
    #[must_use]
    pub fn regex_allowed(&self) -> bool {
        self.regex_allowed
    }

    pub(crate) fn to_ref(&self) -> FieldRef {
        FieldRef {
            public_name: self.public_name.clone(),
            column_name: self.column_name.clone(),
            value_kind: self.value_kind,
            regex_allowed: self.regex_allowed,
        }
    }
}

/// Resolved field data stored in an RQS plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldRef {
    public_name: String,
    column_name: String,
    value_kind: ValueKind,
    regex_allowed: bool,
}

impl FieldRef {
    /// Return the public query field name.
    #[must_use]
    pub fn public_name(&self) -> &str {
        &self.public_name
    }

    /// Return the allowlisted database column.
    #[must_use]
    pub fn column_name(&self) -> &str {
        &self.column_name
    }

    /// Return the expected value type.
    #[must_use]
    pub fn value_kind(&self) -> ValueKind {
        self.value_kind
    }

    /// Return true when regex values are allowed.
    #[must_use]
    pub fn regex_allowed(&self) -> bool {
        self.regex_allowed
    }
}

/// Explicit allowlist for fields that can appear in RQS input.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldCatalog {
    fields: BTreeMap<String, Field>,
}

impl FieldCatalog {
    /// Create an empty catalog.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a field and return the updated catalog.
    pub fn allow(mut self, field: Field) -> RqsResult<Self> {
        let name = field.public_name().to_owned();
        self.fields.insert(name, field);
        Ok(self)
    }

    /// Insert a text field.
    pub fn allow_text(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::Text)
    }

    /// Insert an integer field.
    pub fn allow_integer(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::Integer)
    }

    /// Insert a float field.
    pub fn allow_float(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::Float)
    }

    /// Insert a boolean field.
    pub fn allow_boolean(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::Boolean)
    }

    /// Insert a date field.
    pub fn allow_date(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::Date)
    }

    /// Insert a date-time field.
    pub fn allow_datetime(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::DateTime)
    }

    /// Insert a UUID field.
    pub fn allow_uuid(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        self.allow_kind(public_name, column_name, ValueKind::Uuid)
    }

    /// Return a field by public name.
    #[must_use]
    pub fn get(&self, public_name: &str) -> Option<&Field> {
        self.fields.get(public_name)
    }

    /// Return true when no fields are configured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Return the number of configured fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    fn allow_kind(
        self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
        value_kind: ValueKind,
    ) -> RqsResult<Self> {
        let field = Field::new(public_name, column_name, value_kind)?;
        self.allow(field)
    }
}

#[cfg(test)]
impl FieldRef {
    pub(crate) fn new_for_test(
        public_name: &str,
        column_name: &str,
        value_kind: ValueKind,
        regex_allowed: bool,
    ) -> Self {
        Self {
            public_name: public_name.to_owned(),
            column_name: column_name.to_owned(),
            value_kind,
            regex_allowed,
        }
    }
}

fn validate_public_name(name: &str) -> RqsResult<()> {
    if is_dotted_identifier(name) {
        Ok(())
    } else {
        Err(RqsError::InvalidFieldName {
            field: name.to_owned(),
        })
    }
}

fn validate_column_name(name: &str) -> RqsResult<()> {
    if is_dotted_identifier(name) {
        Ok(())
    } else {
        Err(RqsError::InvalidColumnName {
            column: name.to_owned(),
        })
    }
}

fn is_dotted_identifier(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_identifier)
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
