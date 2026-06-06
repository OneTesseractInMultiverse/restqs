//! Sort terms in an RQS plan.

use crate::FieldRef;

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// One sort term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortTerm {
    field: FieldRef,
    direction: SortDirection,
}

impl SortTerm {
    /// Create a sort term.
    #[must_use]
    pub fn new(field: FieldRef, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Return the field.
    #[must_use]
    pub fn field(&self) -> &FieldRef {
        &self.field
    }

    /// Return the direction.
    #[must_use]
    pub fn direction(&self) -> SortDirection {
        self.direction
    }
}
