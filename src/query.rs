//! Database-neutral RQS query plan.

use crate::{Filter, Pagination, Projection, SortTerm};

/// Parsed RQS query plan.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RqsQuery {
    filters: Vec<Filter>,
    sort: Vec<SortTerm>,
    projection: Projection,
    pagination: Pagination,
}

impl RqsQuery {
    /// Create an empty query plan.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return all filters.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Return all sort terms.
    #[must_use]
    pub fn sort(&self) -> &[SortTerm] {
        &self.sort
    }

    /// Return the projection.
    #[must_use]
    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    /// Return pagination.
    #[must_use]
    pub fn pagination(&self) -> Pagination {
        self.pagination
    }

    pub(crate) fn push_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    pub(crate) fn set_sort(&mut self, sort: Vec<SortTerm>) {
        self.sort = sort;
    }

    pub(crate) fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }

    pub(crate) fn pagination_mut(&mut self) -> &mut Pagination {
        &mut self.pagination
    }
}
