//! SQLx-oriented SQL fragment adapter.
//!
//! This adapter returns SQL text fragments and typed bind values. It does not
//! run a query. Callers pass the fragments into their own SQLx code.
//!
//! Equality and inequality with [`RqsValue::Null`] produce `IS NULL` and
//! `IS NOT NULL` without bind values. Ordered comparisons with null return
//! [`RqsError::AdapterUnsupported`] with feature `ordered null comparison`.
//!
//! Regex is opt-in. PostgreSQL supports no suffix flags or `i`; MySQL supports
//! no suffix flags. Other recognized flags return [`RqsError::AdapterUnsupported`].
//! SQLite rejects all regex. Patterns retain the database's native regex syntax
//! and are always passed as bind values.

use crate::{
    FieldRef, Filter, FilterOp, RqsError, RqsQuery, RqsResult, RqsValue, SortDirection, SortTerm,
};

pub use super::sqlx_columns::SqlxColumnMap;

/// SQL dialect used for placeholders and identifier quotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlDialect {
    /// PostgreSQL.
    Postgres,
    /// MySQL.
    MySql,
    /// SQLite.
    Sqlite,
}

/// SQLx adapter options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlxAdapter {
    /// Dialect governing identifier quoting, placeholder syntax, and supported operators.
    dialect: SqlDialect,
    /// Trusted logical-to-physical identifier mappings; never populated from request data.
    columns: SqlxColumnMap,
    /// Adapter-level opt-in, independent of each catalog field's regex permission.
    regex_enabled: bool,
}

impl SqlxAdapter {
    /// Create an adapter for a dialect with explicit trusted column mappings.
    #[must_use]
    pub fn new(dialect: SqlDialect, columns: SqlxColumnMap) -> Self {
        Self {
            dialect,
            columns,
            regex_enabled: false,
        }
    }

    /// Enable regex SQL generation for dialects that support it.
    ///
    /// PostgreSQL accepts no suffix flags or `i`. MySQL accepts no suffix flags,
    /// with matching behavior determined by its collation and regex engine.
    /// Unsupported flags and SQLite regex return [`RqsError::AdapterUnsupported`].
    #[must_use]
    pub fn allow_regex(mut self) -> Self {
        self.regex_enabled = true;
        self
    }

    /// Convert an RQS query plan into SQL fragments.
    ///
    /// Every filter, sort, and projection field must have a configured mapping.
    /// Missing mappings return [`RqsError::MissingColumnMapping`].
    /// PostgreSQL placeholders start at `$1`. Filters become AND predicates;
    /// list items are flattened into consecutive binds and null/existence predicates
    /// consume no binds. Pagination is copied as metadata and is not added to SQL.
    ///
    /// # Errors
    ///
    /// Returns mapping errors, bind-position overflow, or [`RqsError::AdapterUnsupported`]
    /// for unsupported dialect/operand combinations (including empty lists, ordered
    /// null comparisons, disabled regex, and unsupported regex flags). No partial
    /// fragments are returned on failure.
    pub fn build(&self, query: &RqsQuery) -> RqsResult<SqlxQueryParts> {
        self.build_with_bind_start(query, 1)
    }

    /// Build fragments starting at an explicit one-based bind position.
    ///
    /// Use `2` when the caller's PostgreSQL statement already binds one value.
    /// Returned binds contain only generated filter values; bind the caller's
    /// values first. MySQL and SQLite retain anonymous `?` placeholders.
    ///
    /// Zero returns [`RqsError::InvalidBindPosition`], even for an empty plan.
    /// Position arithmetic exceeding `usize` returns [`RqsError::BindPositionOverflow`].
    /// Database and driver parameter limits remain the caller's responsibility.
    pub fn build_with_bind_start(
        &self,
        query: &RqsQuery,
        first_bind_position: usize,
    ) -> RqsResult<SqlxQueryParts> {
        validate_bind_start(first_bind_position)?;
        let mut builder = FragmentBuilder::new(
            self.dialect,
            self.regex_enabled,
            &self.columns,
            first_bind_position,
        );
        builder.add_filters(query.filters())?;
        builder.add_sort(query.sort())?;
        builder.add_projection(query)?;
        builder.add_pagination(query);
        Ok(builder.finish())
    }
}

/// SQL fragments and bind values ready for caller-owned repository code.
///
/// These public fields are data, not an execution or sanitization API. Preserve
/// the generated SQL/bind correspondence when composing a statement. Add trusted
/// authorization predicates, result budgets, pagination binds, and schema-aware
/// decoding in the repository before passing SQL to a driver.
#[derive(Debug, Clone, PartialEq)]
pub struct SqlxQueryParts {
    /// Optional `WHERE` clause without the `WHERE` keyword.
    pub where_clause: Option<String>,
    /// Projection columns, already quoted for the dialect.
    pub projection: Vec<String>,
    /// Optional `ORDER BY` clause without the `ORDER BY` keyword.
    pub order_by: Option<String>,
    /// Limit value.
    pub limit: Option<u64>,
    /// Offset value.
    pub offset: Option<u64>,
    /// Generated filter bind values in placeholder order, excluding caller-owned binds.
    pub binds: Vec<RqsValue>,
}

/// Mutable translation state for one plan; discarded if any field or operand cannot be
/// translated.
struct FragmentBuilder<'a> {
    /// Dialect governing identifier quoting, placeholder syntax, and supported operators.
    dialect: SqlDialect,
    /// One-based offset reserved for the first generated filter bind.
    first_bind_position: usize,
    /// Trusted logical-to-physical identifier mappings; never populated from request data.
    columns: &'a SqlxColumnMap,
    /// Adapter-level opt-in, independent of each catalog field's regex permission.
    regex_enabled: bool,
    /// Translated filter predicates accumulated in request order, without WHERE.
    clauses: Vec<String>,
    /// Requested selection; empty means the repository chooses its default response.
    projection: Vec<String>,
    /// Quoted ordering clause without ORDER BY, or None for no requested ordering.
    order_by: Option<String>,
    /// Explicit unsigned row cap, or None when the request omitted a limit.
    limit: Option<u64>,
    /// Explicit unsigned skip count, or None when the request omitted an offset.
    offset: Option<u64>,
    /// Generated filter values in placeholder order, excluding caller-owned prefix values.
    binds: Vec<RqsValue>,
}

impl<'a> FragmentBuilder<'a> {
    /// Initialize empty fragment state with trusted mappings and an already validated one-based
    /// bind start.
    fn new(
        dialect: SqlDialect,
        regex_enabled: bool,
        columns: &'a SqlxColumnMap,
        first_bind_position: usize,
    ) -> Self {
        Self {
            dialect,
            first_bind_position,
            columns,
            regex_enabled,
            clauses: Vec::new(),
            projection: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
            binds: Vec::new(),
        }
    }

    /// Translate predicates in request order and accumulate clauses and their corresponding bind
    /// values.
    fn add_filters(&mut self, filters: &[Filter]) -> RqsResult<()> {
        for filter in filters {
            let clause = self.filter_clause(filter)?;
            self.clauses.push(clause);
        }
        Ok(())
    }

    /// Resolve every sort field and store its dialect-quoted ordering clause.
    fn add_sort(&mut self, sort: &[SortTerm]) -> RqsResult<()> {
        self.order_by = sort_clause(self.dialect, self.columns, sort)?;
        Ok(())
    }

    /// Resolve every selected field and store quoted columns in requested order.
    fn add_projection(&mut self, query: &RqsQuery) -> RqsResult<()> {
        self.projection =
            projection_columns(self.dialect, self.columns, query.projection().fields())?;
        Ok(())
    }

    /// Copy unsigned pagination metadata; the repository owns limits, conversion, and SQL
    /// pagination syntax.
    fn add_pagination(&mut self, query: &RqsQuery) {
        self.limit = query.pagination().limit();
        self.offset = query.pagination().offset();
    }

    /// Consume builder state, joining predicates with AND and preserving bind and projection
    /// order.
    fn finish(self) -> SqlxQueryParts {
        SqlxQueryParts {
            where_clause: if self.clauses.is_empty() {
                None
            } else {
                Some(self.clauses.join(" AND "))
            },
            projection: self.projection,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            binds: self.binds,
        }
    }

    /// Resolve the physical column before dispatching the normalized operator to its translation
    /// path.
    fn filter_clause(&mut self, filter: &Filter) -> RqsResult<String> {
        let column = quoted_field(self.dialect, self.columns, filter.field())?;
        match filter.op() {
            FilterOp::Exists => Ok(format!("{column} IS NOT NULL")),
            FilterOp::NotExists => Ok(format!("{column} IS NULL")),
            FilterOp::Regex => self.regex_clause(filter, &column),
            FilterOp::In | FilterOp::NotIn => self.list_clause(filter, &column),
            FilterOp::Eq => self.comparison_clause(filter, &column, "="),
            FilterOp::Ne => self.comparison_clause(filter, &column, "<>"),
            FilterOp::Gt => self.comparison_clause(filter, &column, ">"),
            FilterOp::Gte => self.comparison_clause(filter, &column, ">="),
            FilterOp::Lt => self.comparison_clause(filter, &column, "<"),
            FilterOp::Lte => self.comparison_clause(filter, &column, "<="),
        }
    }

    /// Produce a null predicate without binding, or bind a scalar and format its comparison;
    /// reject missing operands.
    fn comparison_clause(
        &mut self,
        filter: &Filter,
        column: &str,
        operator: &str,
    ) -> RqsResult<String> {
        let Some(value) = filter.value() else {
            return Err(RqsError::AdapterUnsupported {
                feature: "missing value",
            });
        };
        if let Some(clause) = null_comparison_clause(column, operator, value)? {
            return Ok(clause);
        }
        let placeholder = self.push_bind(value.clone())?;
        Ok(format_comparison(column, operator, &placeholder))
    }

    /// Expand nonempty membership operands into individual placeholders; reject missing lists and
    /// incompatible operators.
    fn list_clause(&mut self, filter: &Filter, column: &str) -> RqsResult<String> {
        let Some(RqsValue::List(values)) = filter.value() else {
            return Err(RqsError::AdapterUnsupported {
                feature: "list value",
            });
        };
        if values.is_empty() {
            return Err(RqsError::AdapterUnsupported {
                feature: "empty list",
            });
        }
        let placeholders = values
            .iter()
            .map(|value| self.push_bind(value.clone()))
            .collect::<RqsResult<Vec<_>>>()?
            .join(", ");
        let operator = match filter.op() {
            FilterOp::In => "IN",
            FilterOp::NotIn => "NOT IN",
            _ => {
                return Err(RqsError::AdapterUnsupported {
                    feature: "list operator",
                });
            }
        };
        Ok(format!("{column} {operator} ({placeholders})"))
    }

    /// Check adapter opt-in, literal presence, and dialect flag support before binding the raw
    /// pattern.
    fn regex_clause(&mut self, filter: &Filter, column: &str) -> RqsResult<String> {
        if !self.regex_enabled {
            return Err(RqsError::AdapterUnsupported { feature: "regex" });
        }
        let Some(regex) = filter.regex_literal() else {
            return Err(RqsError::AdapterUnsupported {
                feature: "regex literal",
            });
        };
        let operator = regex_operator(self.dialect, regex.flags())?;
        let placeholder = self.push_bind(RqsValue::Text(regex.pattern().to_owned()))?;
        Ok(format_comparison(column, operator, &placeholder))
    }

    /// Check position arithmetic and format the placeholder before appending the value, leaving
    /// bind state unchanged on overflow.
    fn push_bind(&mut self, value: RqsValue) -> RqsResult<String> {
        let position = bind_position(self.first_bind_position, self.binds.len())?;
        let placeholder = format_placeholder(self.dialect, position);
        self.binds.push(value);
        Ok(placeholder)
    }
}

/// Reject zero because every numbered placeholder position is one-based.
fn validate_bind_start(position: usize) -> RqsResult<()> {
    if position == 0 {
        Err(RqsError::InvalidBindPosition)
    } else {
        Ok(())
    }
}

/// Compute the next position using checked addition, reporting overflow instead of wrapping.
fn bind_position(first: usize, preceding_binds: usize) -> RqsResult<usize> {
    first
        .checked_add(preceding_binds)
        .ok_or(RqsError::BindPositionOverflow)
}

/// Format a placeholder for an explicit one-based bind position.
fn format_placeholder(dialect: SqlDialect, position: usize) -> String {
    match dialect {
        SqlDialect::Postgres => format!("${position}"),
        SqlDialect::MySql | SqlDialect::Sqlite => "?".to_owned(),
    }
}

/// Select only dialect-supported regex semantics; reject unsupported flags instead of silently
/// dropping them.
fn regex_operator(dialect: SqlDialect, flags: &str) -> RqsResult<&'static str> {
    match (dialect, flags) {
        (SqlDialect::Postgres, "") => Ok("~"),
        (SqlDialect::Postgres, "i") => Ok("~*"),
        (SqlDialect::Postgres, _) => Err(RqsError::AdapterUnsupported {
            feature: "postgres regex flags",
        }),
        (SqlDialect::MySql, "") => Ok("REGEXP"),
        (SqlDialect::MySql, _) => Err(RqsError::AdapterUnsupported {
            feature: "mysql regex flags",
        }),
        (SqlDialect::Sqlite, _) => Err(RqsError::AdapterUnsupported {
            feature: "sqlite regex",
        }),
    }
}

/// Translate null equality/inequality without binds, reject ordered null comparisons, and return
/// None for non-null values.
fn null_comparison_clause(
    column: &str,
    operator: &str,
    value: &RqsValue,
) -> RqsResult<Option<String>> {
    match (value, operator) {
        (RqsValue::Null, "=") => Ok(Some(format_comparison(column, "IS", "NULL"))),
        (RqsValue::Null, "<>") => Ok(Some(format_comparison(column, "IS NOT", "NULL"))),
        (RqsValue::Null, _) => Err(RqsError::AdapterUnsupported {
            feature: "ordered null comparison",
        }),
        _ => Ok(None),
    }
}

/// Join a trusted column, operator, and SQL operand with spaces; values must already be
/// placeholders or fixed literals.
fn format_comparison(column: &str, operator: &str, operand: &str) -> String {
    format!("{column} {operator} {operand}")
}

/// Return no clause for empty sorting; otherwise resolve and join all terms in priority order.
fn sort_clause(
    dialect: SqlDialect,
    columns: &SqlxColumnMap,
    sort: &[SortTerm],
) -> RqsResult<Option<String>> {
    if sort.is_empty() {
        return Ok(None);
    }
    let terms = sort
        .iter()
        .map(|term| sort_term(dialect, columns, term))
        .collect::<RqsResult<Vec<_>>>()?;
    Ok(Some(terms.join(", ")))
}

/// Resolve a field and append its fixed ASC/DESC direction using dialect identifier quoting.
fn sort_term(dialect: SqlDialect, columns: &SqlxColumnMap, term: &SortTerm) -> RqsResult<String> {
    let column = quoted_field(dialect, columns, term.field())?;
    let direction = match term.direction() {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    Ok(format!("{column} {direction}"))
}

/// Resolve and quote every projection field in input order; fail if any explicit mapping is
/// missing.
fn projection_columns(
    dialect: SqlDialect,
    columns: &SqlxColumnMap,
    fields: &[FieldRef],
) -> RqsResult<Vec<String>> {
    fields
        .iter()
        .map(|field| quoted_field(dialect, columns, field))
        .collect()
}

/// Resolve trusted column metadata before quoting; never derive SQL identifiers from request
/// field names.
fn quoted_field(
    dialect: SqlDialect,
    columns: &SqlxColumnMap,
    field: &FieldRef,
) -> RqsResult<String> {
    let column = columns.resolve(field)?;
    Ok(quote_column(dialect, column))
}

/// Quote each segment of a previously validated dotted column identifier independently.
fn quote_column(dialect: SqlDialect, column: &str) -> String {
    column
        .split('.')
        .map(|part| quote_identifier(dialect, part))
        .collect::<Vec<_>>()
        .join(".")
}

/// Quote an already validated ASCII identifier segment for the dialect; this is not an
/// arbitrary-string escaping API.
fn quote_identifier(dialect: SqlDialect, value: &str) -> String {
    match dialect {
        SqlDialect::Postgres | SqlDialect::Sqlite => format!("\"{value}\""),
        SqlDialect::MySql => format!("`{value}`"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FieldRef, Projection, RegexLiteral, RqsQuery, SortDirection, SortTerm, ValueKind};

    #[test]
    fn postgres_placeholder_formats_the_first_position() {
        assert_eq!(format_placeholder(SqlDialect::Postgres, 1), "$1");
    }

    #[test]
    fn postgres_placeholder_formats_an_explicit_later_position() {
        assert_eq!(format_placeholder(SqlDialect::Postgres, 12), "$12");
    }

    #[test]
    fn mysql_placeholder_does_not_include_the_position() {
        assert_eq!(format_placeholder(SqlDialect::MySql, 12), "?");
    }

    #[test]
    fn sqlite_placeholder_does_not_include_the_position() {
        assert_eq!(format_placeholder(SqlDialect::Sqlite, 12), "?");
    }

    /// Build the trusted status, age, timestamp, and email mapping used by adapter invariant
    /// tests.
    fn columns() -> RqsResult<SqlxColumnMap> {
        SqlxColumnMap::new()
            .map("status", "users.status")?
            .map("age", "users.age")?
            .map("created_at", "users.created_at")?
            .map("email", "users.email")
    }

    /// Create an authorized text field for directly constructed adapter test plans.
    fn text_field() -> FieldRef {
        FieldRef::new_for_test("status", ValueKind::Text, false)
    }

    /// Create an authorized integer field for directly constructed adapter test plans.
    fn integer_field() -> FieldRef {
        FieldRef::new_for_test("age", ValueKind::Integer, false)
    }

    /// Create an authorized timestamp field for directly constructed adapter test plans.
    fn datetime_field() -> FieldRef {
        FieldRef::new_for_test("created_at", ValueKind::DateTime, false)
    }

    /// Create an opt-in email field for direct adapter regex policy tests.
    fn regex_field() -> FieldRef {
        FieldRef::new_for_test("email", ValueKind::Text, true)
    }

    /// Wrap a directly constructed filter in a plan so adapter guards can be tested independently
    /// of parsing.
    fn query_with_filter(filter: Filter) -> RqsQuery {
        let mut query = RqsQuery::new();
        query.push_filter(filter);
        query
    }

    #[test]
    fn adapter_build_covers_unit_public_query_path() -> RqsResult<()> {
        let mut query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Gte,
            Some(RqsValue::Integer(18)),
        ));
        query.set_sort(vec![SortTerm::new(datetime_field(), SortDirection::Desc)]);
        query.set_projection(Projection::new(vec![text_field()]));
        query.pagination_mut().set_limit(10);
        query.pagination_mut().set_offset(20);
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.order_by);

        assert_eq!(result, Ok(Some("\"users\".\"created_at\" DESC".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_ascending_sort_path() -> RqsResult<()> {
        let mut query = RqsQuery::new();
        query.set_sort(vec![SortTerm::new(datetime_field(), SortDirection::Asc)]);
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.order_by);

        assert_eq!(result, Ok(Some("\"users\".\"created_at\" ASC".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_empty_query_path() -> RqsResult<()> {
        let query = RqsQuery::new();
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(None));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_build_error_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            text_field(),
            FilterOp::In,
            Some(RqsValue::List(Vec::new())),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map_err(|error| error.error_code());

        assert_eq!(result, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_mysql_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Eq,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::MySql, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("`users`.`age` = ?".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_not_exists_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(text_field(), FilterOp::NotExists, None));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"status\" IS NULL".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_not_equal_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Ne,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" <> $1".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_greater_than_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Gt,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" > $1".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_less_than_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Lt,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" < $1".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_less_than_or_equal_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Lte,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" <= $1".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_exists_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(text_field(), FilterOp::Exists, None));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(
            result,
            Ok(Some("\"users\".\"status\" IS NOT NULL".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_regex_disabled_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", "i"),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map_err(|error| error.error_code());

        assert_eq!(result, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_regex_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", "i"),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .allow_regex()
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"email\" ~* $1".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_case_sensitive_regex_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", ""),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .allow_regex()
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"email\" ~ $1".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_mysql_regex_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", ""),
        ));
        let result = SqlxAdapter::new(SqlDialect::MySql, columns()?)
            .allow_regex()
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("`users`.`email` REGEXP ?".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_sqlite_regex_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", ""),
        ));
        let result = SqlxAdapter::new(SqlDialect::Sqlite, columns()?)
            .allow_regex()
            .build(&query)
            .map_err(|error| error.error_code());

        assert_eq!(result, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_in_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            text_field(),
            FilterOp::In,
            Some(RqsValue::List(vec![RqsValue::Text("active".to_owned())])),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"status\" IN ($1)".to_owned())));
        Ok(())
    }

    #[test]
    fn adapter_build_covers_unit_not_in_path() -> RqsResult<()> {
        let query = query_with_filter(Filter::new(
            text_field(),
            FilterOp::NotIn,
            Some(RqsValue::List(vec![RqsValue::Text("active".to_owned())])),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres, columns()?)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(
            result,
            Ok(Some("\"users\".\"status\" NOT IN ($1)".to_owned()))
        );
        Ok(())
    }

    #[test]
    fn null_comparison_preserves_existing_bind_state() -> RqsResult<()> {
        let filter = Filter::new(text_field(), FilterOp::Eq, Some(RqsValue::Null));
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns, 1);
        builder.binds.push(RqsValue::Integer(18));
        builder.filter_clause(&filter)?;

        assert_eq!(builder.binds, vec![RqsValue::Integer(18)]);
        Ok(())
    }

    #[test]
    fn comparison_filter_without_value_is_rejected() -> RqsResult<()> {
        let filter = Filter::new(text_field(), FilterOp::Eq, None);
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns, 1);
        let error = builder
            .filter_clause(&filter)
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn list_filter_without_list_value_is_rejected() -> RqsResult<()> {
        let filter = Filter::new(
            text_field(),
            FilterOp::In,
            Some(RqsValue::Text("active".to_owned())),
        );
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns, 1);
        let error = builder
            .filter_clause(&filter)
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn list_clause_rejects_non_list_operator() -> RqsResult<()> {
        let filter = Filter::new(
            text_field(),
            FilterOp::Eq,
            Some(RqsValue::List(vec![RqsValue::Text("active".to_owned())])),
        );
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns, 1);
        let error = builder
            .list_clause(&filter, "\"users\".\"status\"")
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn regex_filter_without_literal_is_rejected() -> RqsResult<()> {
        let filter = Filter::new(text_field(), FilterOp::Regex, None);
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, true, &columns, 1);
        let error = builder
            .filter_clause(&filter)
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
        Ok(())
    }

    #[test]
    fn postgres_unsupported_flags_do_not_add_a_bind() -> RqsResult<()> {
        let filter = Filter::regex(regex_field(), RegexLiteral::new_for_test("a.b", "s"));
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, true, &columns, 1);
        let _ = builder.filter_clause(&filter);

        assert!(builder.binds.is_empty());
        Ok(())
    }

    #[test]
    fn mysql_unsupported_flags_do_not_add_a_bind() -> RqsResult<()> {
        let filter = Filter::regex(regex_field(), RegexLiteral::new_for_test("admin", "i"));
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::MySql, true, &columns, 1);
        let _ = builder.filter_clause(&filter);

        assert!(builder.binds.is_empty());
        Ok(())
    }

    #[test]
    fn sqlite_regex_does_not_add_a_bind() -> RqsResult<()> {
        let filter = Filter::regex(regex_field(), RegexLiteral::new_for_test("admin", ""));
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Sqlite, true, &columns, 1);
        let _ = builder.filter_clause(&filter);

        assert!(builder.binds.is_empty());
        Ok(())
    }
}
