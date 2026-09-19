//! Defensive parser limits.

use crate::{RqsError, RqsResult};

/// Defensive parser limits used by the default parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserLimits {
    /// Maximum raw query byte length.
    pub max_query_bytes: usize,
    /// Maximum parameter count.
    pub max_parameters: usize,
    /// Maximum decoded UTF-8 value byte length for filters and query controls.
    ///
    /// Includes cast wrappers, regex delimiters and flags, and complete sort or
    /// projection lists. Existence filters have no value to limit.
    pub max_value_bytes: usize,
    /// Maximum list item count.
    pub max_list_items: usize,
    /// Maximum accepted limit value.
    pub max_limit: u64,
}

impl Default for ParserLimits {
    fn default() -> Self {
        Self {
            max_query_bytes: 8 * 1024,
            max_parameters: 128,
            max_value_bytes: 2 * 1024,
            max_list_items: 100,
            max_limit: 100,
        }
    }
}

pub(crate) fn validate_value_size(field: &str, value: &str, max_bytes: usize) -> RqsResult<()> {
    if value.len() > max_bytes {
        Err(RqsError::ValueTooLarge {
            field: field.to_owned(),
            max_bytes,
        })
    } else {
        Ok(())
    }
}
