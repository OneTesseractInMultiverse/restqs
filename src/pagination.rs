//! Pagination data in an RQS plan.

/// Pagination selected by `limit=` and `skip=`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Pagination {
    limit: Option<u64>,
    offset: Option<u64>,
}

impl Pagination {
    /// Create pagination.
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

    pub(crate) fn set_limit(&mut self, value: u64) {
        self.limit = Some(value);
    }

    pub(crate) fn set_offset(&mut self, value: u64) {
        self.offset = Some(value);
    }
}
