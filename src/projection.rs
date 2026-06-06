//! Projection fields in an RQS plan.

use crate::FieldRef;

/// Projection selected by `fields=`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Projection {
    fields: Vec<FieldRef>,
}

impl Projection {
    /// Create a projection from fields.
    #[must_use]
    pub fn new(fields: Vec<FieldRef>) -> Self {
        Self { fields }
    }

    /// Return projection fields.
    #[must_use]
    pub fn fields(&self) -> &[FieldRef] {
        &self.fields
    }

    /// Return true when no projection was requested.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}
