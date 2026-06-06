//! Defensive parser limits.

/// Defensive parser limits used by the default parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserLimits {
    /// Maximum raw query byte length.
    pub max_query_bytes: usize,
    /// Maximum parameter count.
    pub max_parameters: usize,
    /// Maximum raw value byte length.
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
