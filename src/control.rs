//! Shared query-control names and public-field namespace policy.

use crate::{RqsError, RqsResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Canonical query-control namespace shared by catalog validation and parameter classification.
pub(crate) enum Control {
    /// The `sort` control for ordered logical fields.
    Sort,
    /// The `fields` control for requested logical output fields.
    Projection,
    /// The `limit` control for an explicit unsigned row cap.
    Limit,
    /// The `skip` control for an explicit unsigned offset.
    Offset,
    /// The recognized but unsupported `$text` control.
    TextSearch,
}

impl Control {
    /// Classify exact, case-sensitive control names, including the unsupported `$text` control.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "sort" => Some(Self::Sort),
            "fields" => Some(Self::Projection),
            "limit" => Some(Self::Limit),
            "skip" => Some(Self::Offset),
            "$text" => Some(Self::TextSearch),
            _ => None,
        }
    }
}

/// Reject names owned by query controls so field lookup cannot shadow parser syntax.
pub(crate) fn validate_unreserved_name(name: &str) -> RqsResult<()> {
    if Control::from_name(name).is_some() {
        Err(RqsError::ReservedFieldName {
            field: name.to_owned(),
        })
    } else {
        Ok(())
    }
}
