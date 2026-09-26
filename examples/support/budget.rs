//! Application-owned result budgets; parser limits only constrain input shape.

use std::fmt::{self, Display, Formatter};

use restqs::adapters::sqlx::SqlxQueryParts;

/// Explicit repository execution policy, independent of request parameters.
#[derive(Debug, Clone, Copy)]
pub struct QueryBudget {
    /// None opts trusted internal work out of result caps.
    bounds: Option<Bounds>,
}

#[derive(Debug, Clone, Copy)]
/// Validated application-owned result and offset caps.
struct Bounds {
    /// Positive row cap used when the request omits a limit.
    default_limit: u64,
    /// Inclusive explicit row cap within signed database range.
    max_limit: u64,
    /// Inclusive skip cap within signed database range.
    max_offset: u64,
}

/// A repository policy rejected configuration or request pagination.
#[derive(Debug, PartialEq, Eq)]
pub enum BudgetError {
    /// Defaults must be positive and within signed database bounds and the maximum.
    InvalidConfiguration,
    /// An explicit request limit exceeds the repository budget.
    LimitExceeded,
    /// A request offset exceeds the repository budget.
    OffsetExceeded,
}

impl Display for BudgetError {
    /// Format a stable policy explanation without embedding request values.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidConfiguration => "invalid repository result budget",
            Self::LimitExceeded => "request exceeds repository result limit",
            Self::OffsetExceeded => "request exceeds repository offset limit",
        })
    }
}

impl std::error::Error for BudgetError {}

impl Default for QueryBudget {
    /// Use a 25-row default, 100-row maximum, and 10,000-row offset cap for public endpoint
    /// examples.
    fn default() -> Self {
        Self {
            bounds: Some(Bounds {
                default_limit: 25,
                max_limit: 100,
                max_offset: 10_000,
            }),
        }
    }
}

impl QueryBudget {
    /// Configure positive default and maximum row caps and an inclusive offset cap.
    pub fn bounded(
        default_limit: u64,
        max_limit: u64,
        max_offset: u64,
    ) -> Result<Self, BudgetError> {
        if default_limit == 0
            || default_limit > max_limit
            || i64::try_from(max_limit).is_err()
            || i64::try_from(max_offset).is_err()
        {
            return Err(BudgetError::InvalidConfiguration);
        }
        Ok(Self {
            bounds: Some(Bounds {
                default_limit,
                max_limit,
                max_offset,
            }),
        })
    }

    /// Opt trusted internal work into uncapped results and offsets.
    ///
    /// This must be chosen by application code, never by a query-string control.
    /// Parser limits and checked database integer conversion still apply.
    pub fn unbounded_internal() -> Self {
        Self { bounds: None }
    }

    /// Compute effective pagination without mutating the supplied fragments.
    pub fn apply(self, mut parts: SqlxQueryParts) -> Result<SqlxQueryParts, BudgetError> {
        if let Some(bounds) = self.bounds {
            let limit = parts.limit.unwrap_or(bounds.default_limit);
            if limit > bounds.max_limit {
                return Err(BudgetError::LimitExceeded);
            }
            if parts
                .offset
                .is_some_and(|offset| offset > bounds.max_offset)
            {
                return Err(BudgetError::OffsetExceeded);
            }
            parts.limit = Some(limit);
        }
        Ok(parts)
    }
}
