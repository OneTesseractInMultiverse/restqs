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
    dialect: SqlDialect,
    columns: SqlxColumnMap,
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
    pub fn build(&self, query: &RqsQuery) -> RqsResult<SqlxQueryParts> {
        let mut builder = FragmentBuilder::new(self.dialect, self.regex_enabled, &self.columns);
        builder.add_filters(query.filters())?;
        builder.add_sort(query.sort())?;
        builder.add_projection(query)?;
        builder.add_pagination(query);
        Ok(builder.finish())
    }
}

/// SQL fragments and bind values ready for caller-owned SQLx code.
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
    /// Bind values in placeholder order.
    pub binds: Vec<RqsValue>,
}

struct FragmentBuilder<'a> {
    dialect: SqlDialect,
    columns: &'a SqlxColumnMap,
    regex_enabled: bool,
    clauses: Vec<String>,
    projection: Vec<String>,
    order_by: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
    binds: Vec<RqsValue>,
}

impl<'a> FragmentBuilder<'a> {
    fn new(dialect: SqlDialect, regex_enabled: bool, columns: &'a SqlxColumnMap) -> Self {
        Self {
            dialect,
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

    fn add_filters(&mut self, filters: &[Filter]) -> RqsResult<()> {
        for filter in filters {
            let clause = self.filter_clause(filter)?;
            self.clauses.push(clause);
        }
        Ok(())
    }

    fn add_sort(&mut self, sort: &[SortTerm]) -> RqsResult<()> {
        self.order_by = sort_clause(self.dialect, self.columns, sort)?;
        Ok(())
    }

    fn add_projection(&mut self, query: &RqsQuery) -> RqsResult<()> {
        self.projection =
            projection_columns(self.dialect, self.columns, query.projection().fields())?;
        Ok(())
    }

    fn add_pagination(&mut self, query: &RqsQuery) {
        self.limit = query.pagination().limit();
        self.offset = query.pagination().offset();
    }

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
        let placeholder = self.push_bind(value.clone());
        Ok(format_comparison(column, operator, &placeholder))
    }

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
            .collect::<Vec<_>>()
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
        let placeholder = self.push_bind(RqsValue::Text(regex.pattern().to_owned()));
        Ok(format_comparison(column, operator, &placeholder))
    }

    fn push_bind(&mut self, value: RqsValue) -> String {
        self.binds.push(value);
        let position = self.binds.len();
        format_placeholder(self.dialect, position)
    }
}

/// Format a placeholder for an explicit one-based bind position.
fn format_placeholder(dialect: SqlDialect, position: usize) -> String {
    match dialect {
        SqlDialect::Postgres => format!("${position}"),
        SqlDialect::MySql | SqlDialect::Sqlite => "?".to_owned(),
    }
}

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

fn format_comparison(column: &str, operator: &str, operand: &str) -> String {
    format!("{column} {operator} {operand}")
}

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

fn sort_term(dialect: SqlDialect, columns: &SqlxColumnMap, term: &SortTerm) -> RqsResult<String> {
    let column = quoted_field(dialect, columns, term.field())?;
    let direction = match term.direction() {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };
    Ok(format!("{column} {direction}"))
}

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

fn quoted_field(
    dialect: SqlDialect,
    columns: &SqlxColumnMap,
    field: &FieldRef,
) -> RqsResult<String> {
    let column = columns.resolve(field)?;
    Ok(quote_column(dialect, column))
}

fn quote_column(dialect: SqlDialect, column: &str) -> String {
    column
        .split('.')
        .map(|part| quote_identifier(dialect, part))
        .collect::<Vec<_>>()
        .join(".")
}

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

    fn columns() -> RqsResult<SqlxColumnMap> {
        SqlxColumnMap::new()
            .map("status", "users.status")?
            .map("age", "users.age")?
            .map("created_at", "users.created_at")?
            .map("email", "users.email")
    }

    fn text_field() -> FieldRef {
        FieldRef::new_for_test("status", ValueKind::Text, false)
    }

    fn integer_field() -> FieldRef {
        FieldRef::new_for_test("age", ValueKind::Integer, false)
    }

    fn datetime_field() -> FieldRef {
        FieldRef::new_for_test("created_at", ValueKind::DateTime, false)
    }

    fn regex_field() -> FieldRef {
        FieldRef::new_for_test("email", ValueKind::Text, true)
    }

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
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns);
        builder.binds.push(RqsValue::Integer(18));
        builder.filter_clause(&filter)?;

        assert_eq!(builder.binds, vec![RqsValue::Integer(18)]);
        Ok(())
    }

    #[test]
    fn comparison_filter_without_value_is_rejected() -> RqsResult<()> {
        let filter = Filter::new(text_field(), FilterOp::Eq, None);
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns);
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
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns);
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
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false, &columns);
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
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, true, &columns);
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
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, true, &columns);
        let _ = builder.filter_clause(&filter);

        assert!(builder.binds.is_empty());
        Ok(())
    }

    #[test]
    fn mysql_unsupported_flags_do_not_add_a_bind() -> RqsResult<()> {
        let filter = Filter::regex(regex_field(), RegexLiteral::new_for_test("admin", "i"));
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::MySql, true, &columns);
        let _ = builder.filter_clause(&filter);

        assert!(builder.binds.is_empty());
        Ok(())
    }

    #[test]
    fn sqlite_regex_does_not_add_a_bind() -> RqsResult<()> {
        let filter = Filter::regex(regex_field(), RegexLiteral::new_for_test("admin", ""));
        let columns = columns()?;
        let mut builder = FragmentBuilder::new(SqlDialect::Sqlite, true, &columns);
        let _ = builder.filter_clause(&filter);

        assert!(builder.binds.is_empty());
        Ok(())
    }
}
