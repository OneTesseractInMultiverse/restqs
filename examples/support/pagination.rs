//! Repository-owned pagination assembly shared by the integration examples.

use std::num::TryFromIntError;

use restqs::{RqsValue, adapters::sqlx::SqlxQueryParts};

/// A complete statement and its values in placeholder order.
#[derive(Debug, PartialEq)]
pub struct SqlStatement {
    /// Repository SQL with pagination appended.
    pub sql: String,
    /// Filter values followed by supplied limit and offset values.
    pub binds: Vec<RqsValue>,
}

#[derive(Clone, Copy)]
/// Pagination converted to the signed integer representation used by database binds.
struct CheckedPagination {
    /// Validated signed row cap, preserving explicit zero.
    limit: Option<i64>,
    /// Validated signed skip count.
    offset: Option<i64>,
}

/// Append PostgreSQL pagination after the filter placeholders.
///
/// `base_sql` must already contain the SELECT, filters, and any ordering, with
/// exactly the placeholders described by `parts.binds`. It must not contain a
/// trailing semicolon or pagination. An omitted limit leaves the result uncapped.
/// Values outside the signed 64-bit range return an error before binding.
pub fn append_postgres_pagination(
    base_sql: &str,
    parts: &SqlxQueryParts,
) -> Result<SqlStatement, TryFromIntError> {
    let pagination = checked_pagination(parts)?;
    let clause = postgres_clause(pagination, parts.binds.len());
    Ok(assemble_statement(base_sql, parts, &clause, pagination))
}

/// Append SQLite pagination in positional bind order.
///
/// `base_sql` has the same requirements as [`append_postgres_pagination`], but
/// uses `?` placeholders. Offset-only requests use the fixed `LIMIT -1` sentinel;
/// it does not consume a bind. An omitted limit leaves the result uncapped.
/// Values outside the signed 64-bit range return an error before binding.
pub fn append_sqlite_pagination(
    base_sql: &str,
    parts: &SqlxQueryParts,
) -> Result<SqlStatement, TryFromIntError> {
    let pagination = checked_pagination(parts)?;
    let clause = sqlite_clause(pagination);
    Ok(assemble_statement(base_sql, parts, clause, pagination))
}

/// Convert supplied pagination to signed database integers before statement construction; reject
/// overflow.
fn checked_pagination(parts: &SqlxQueryParts) -> Result<CheckedPagination, TryFromIntError> {
    Ok(CheckedPagination {
        limit: parts.limit.map(i64::try_from).transpose()?,
        offset: parts.offset.map(i64::try_from).transpose()?,
    })
}

/// Place limit and offset after the existing bind vector using numbered PostgreSQL placeholders.
fn postgres_clause(pagination: CheckedPagination, filter_binds: usize) -> String {
    match (pagination.limit, pagination.offset) {
        (Some(_), Some(_)) => {
            format!(" LIMIT ${} OFFSET ${}", filter_binds + 1, filter_binds + 2)
        }
        (Some(_), None) => format!(" LIMIT ${}", filter_binds + 1),
        (None, Some(_)) => format!(" OFFSET ${}", filter_binds + 1),
        (None, None) => String::new(),
    }
}

/// Select fixed SQLite pagination syntax, including the no-bind `LIMIT -1` sentinel for
/// offset-only requests.
fn sqlite_clause(pagination: CheckedPagination) -> &'static str {
    match (pagination.limit, pagination.offset) {
        (Some(_), Some(_)) => " LIMIT ? OFFSET ?",
        (Some(_), None) => " LIMIT ?",
        (None, Some(_)) => " LIMIT -1 OFFSET ?",
        (None, None) => "",
    }
}

/// Append trusted pagination syntax and extend filter binds with limit then offset, preserving
/// positional order.
fn assemble_statement(
    base_sql: &str,
    parts: &SqlxQueryParts,
    clause: &str,
    pagination: CheckedPagination,
) -> SqlStatement {
    let mut binds = parts.binds.clone();
    binds.extend(
        pagination
            .limit
            .into_iter()
            .chain(pagination.offset)
            .map(RqsValue::Integer),
    );
    SqlStatement {
        sql: format!("{base_sql}{clause}"),
        binds,
    }
}
