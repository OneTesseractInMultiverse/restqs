//! Trusted physical column configuration for the SQLx adapter.

use std::collections::BTreeMap;

use crate::{
    FieldRef, RqsError, RqsResult, catalog::validate_public_name, identifier::is_dotted_identifier,
};

/// Explicit mappings from logical query fields to trusted SQL column identifiers.
///
/// Construct this configuration from application code, never request input.
/// A mapping does not authorize a field; authorization belongs to the core catalog.
/// Missing entries never fall back to using the public field name as SQL.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SqlxColumnMap {
    columns: BTreeMap<String, String>,
}

impl SqlxColumnMap {
    /// Create an empty mapping.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one logical field's physical column and return the updated map.
    ///
    /// Logical names follow the public query grammar; reserved query-control names
    /// return [`RqsError::ReservedFieldName`]. Columns must be dotted ASCII
    /// identifiers; expressions, quotes, and empty segments are rejected.
    /// Registering a logical name twice returns [`RqsError::DuplicateColumnMapping`].
    pub fn map(
        mut self,
        public_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> RqsResult<Self> {
        let public_name = public_name.into();
        let column_name = column_name.into();
        validate_public_name(&public_name)?;
        validate_column_name(&column_name)?;
        validate_new_mapping(&self.columns, &public_name)?;
        self.columns.insert(public_name, column_name);
        Ok(self)
    }

    /// Resolve an authorized logical field to its configured physical column.
    ///
    /// Returns [`RqsError::MissingColumnMapping`] when the field is unmapped.
    pub fn resolve(&self, field: &FieldRef) -> RqsResult<&str> {
        self.columns
            .get(field.public_name())
            .map(String::as_str)
            .ok_or_else(|| RqsError::MissingColumnMapping {
                field: field.public_name().to_owned(),
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

fn validate_new_mapping(columns: &BTreeMap<String, String>, name: &str) -> RqsResult<()> {
    if columns.contains_key(name) {
        Err(RqsError::DuplicateColumnMapping {
            field: name.to_owned(),
        })
    } else {
        Ok(())
    }
}
