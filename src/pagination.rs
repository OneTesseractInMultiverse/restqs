//! Pagination data in an RQS plan.

/// Pagination selected by `limit=` and `skip=`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pagination {
    /// Explicit unsigned row cap, or None when the request omitted a limit.
    limit: Option<u64>,
    /// Explicit unsigned skip count, or None when the request omitted an offset.
    offset: Option<u64>,
}

impl Pagination {
    /// Create pagination metadata without applying parser or repository limits.
    ///
    /// None preserves omission; Some(0) is an explicit zero. Consumers converting
    /// to signed database integers must check range before binding. Constructing
    /// this value does not change an existing query plan.
    #[must_use]
    pub fn new(limit: Option<u64>, offset: Option<u64>) -> Self {
        Self { limit, offset }
    }

    /// Return the limit.
    #[must_use]
    pub fn limit(&self) -> Option<u64> {
        self.limit
    }

    /// Return the offset.
    #[must_use]
    pub fn offset(&self) -> Option<u64> {
        self.offset
    }

    /// Record an explicit limit after the parser has validated its syntax and configured maximum.
    pub(crate) fn set_limit(&mut self, value: u64) {
        self.limit = Some(value);
    }

    /// Record an explicit offset after unsigned parsing; repository execution caps apply
    /// separately.
    pub(crate) fn set_offset(&mut self, value: u64) {
        self.offset = Some(value);
    }
}
