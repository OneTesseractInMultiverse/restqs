//! Shared query-control names and public-field namespace policy.

use crate::{RqsError, RqsResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Control {
    Sort,
    Projection,
    Limit,
    Offset,
    TextSearch,
}

impl Control {
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

pub(crate) fn validate_unreserved_name(name: &str) -> RqsResult<()> {
    if Control::from_name(name).is_some() {
        Err(RqsError::ReservedFieldName {
            field: name.to_owned(),
        })
    } else {
        Ok(())
    }
}
