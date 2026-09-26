//! Database-neutral RQS query plan.

use crate::{Filter, Pagination, Projection, SortTerm};

/// Owned, database-neutral plan produced by the parser.
///
/// Downstream code can inspect or clone a plan but cannot mutate its authorized
/// nodes. Filters preserve input order and represent conjunction in the SQL
/// adapter. Empty sorting/projection and omitted pagination leave defaults to
/// the consumer; an empty plan does not itself cap results or authorize rows.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RqsQuery {
    /// Validated predicates in input order; SQL translation combines them with AND.
    filters: Vec<Filter>,
    /// Requested sort terms in priority order; empty means no requested ordering.
    sort: Vec<SortTerm>,
    /// Requested selection; empty means the repository chooses its default response.
    projection: Projection,
    /// Explicit request pagination; omitted values do not imply an execution cap.
    pagination: Pagination,
}

impl RqsQuery {
    /// Create an empty query plan.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return authorized filters in input order, including existence and regex nodes.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Return requested sort terms in priority order; an empty slice requests no ordering.
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

    /// Append a parser-validated filter, preserving request order for downstream bind generation.
    pub(crate) fn push_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    /// Store the parser-resolved sort sequence in its requested order.
    pub(crate) fn set_sort(&mut self, sort: Vec<SortTerm>) {
        self.sort = sort;
    }

    /// Store the parser-resolved selection; an empty selection delegates defaults to the
    /// consumer.
    pub(crate) fn set_projection(&mut self, projection: Projection) {
        self.projection = projection;
    }

    /// Expose mutable pagination only within the core while the parser builds a plan.
    pub(crate) fn pagination_mut(&mut self) -> &mut Pagination {
        &mut self.pagination
    }
}
