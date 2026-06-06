//! SQLx-oriented SQL fragment adapter.
//!
//! This adapter returns SQL text fragments and typed bind values. It does not
//! run a query. Callers pass the fragments into their own SQLx code.

use crate::{Filter, FilterOp, RqsError, RqsQuery, RqsResult, RqsValue, SortDirection};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlxAdapter {
    dialect: SqlDialect,
    regex_enabled: bool,
}

impl SqlxAdapter {
    /// Create an adapter for a dialect.
    #[must_use]
    pub fn new(dialect: SqlDialect) -> Self {
        Self {
            dialect,
            regex_enabled: false,
        }
    }

    /// Enable regex SQL generation for dialects that support it.
    #[must_use]
    pub fn allow_regex(mut self) -> Self {
        self.regex_enabled = true;
        self
    }

    /// Convert an RQS query plan into SQL fragments.
    pub fn build(&self, query: &RqsQuery) -> RqsResult<SqlxQueryParts> {
        let mut builder = FragmentBuilder::new(self.dialect, self.regex_enabled);
        builder.add_filters(query.filters())?;
        builder.add_sort(query.sort());
        builder.add_projection(query);
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

struct FragmentBuilder {
    dialect: SqlDialect,
    regex_enabled: bool,
    clauses: Vec<String>,
    projection: Vec<String>,
    order_by: Option<String>,
    limit: Option<u64>,
    offset: Option<u64>,
    binds: Vec<RqsValue>,
}

impl FragmentBuilder {
    fn new(dialect: SqlDialect, regex_enabled: bool) -> Self {
        Self {
            dialect,
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

    fn add_sort(&mut self, sort: &[crate::SortTerm]) {
        if sort.is_empty() {
            return;
        }

        self.order_by = Some(
            sort.iter()
                .map(|term| {
                    let direction = match term.direction() {
                        SortDirection::Asc => "ASC",
                        SortDirection::Desc => "DESC",
                    };
                    format!(
                        "{} {direction}",
                        quote_column(self.dialect, term.field().column_name())
                    )
                })
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    fn add_projection(&mut self, query: &RqsQuery) {
        self.projection = query
            .projection()
            .fields()
            .iter()
            .map(|field| quote_column(self.dialect, field.column_name()))
            .collect();
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
        let column = quote_column(self.dialect, filter.field().column_name());
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
        let placeholder = self.push_bind(value.clone());
        Ok(format!("{column} {operator} {placeholder}"))
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
        let placeholder = self.push_bind(RqsValue::Text(regex.pattern().to_owned()));
        match self.dialect {
            SqlDialect::Postgres if regex.flags().contains('i') => {
                Ok(format!("{column} ~* {placeholder}"))
            }
            SqlDialect::Postgres => Ok(format!("{column} ~ {placeholder}")),
            SqlDialect::MySql => Ok(format!("{column} REGEXP {placeholder}")),
            SqlDialect::Sqlite => Err(RqsError::AdapterUnsupported {
                feature: "sqlite regex",
            }),
        }
    }

    fn push_bind(&mut self, value: RqsValue) -> String {
        self.binds.push(value);
        match self.dialect {
            SqlDialect::Postgres => format!("${}", self.binds.len()),
            SqlDialect::MySql | SqlDialect::Sqlite => "?".to_owned(),
        }
    }
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

    fn text_field() -> FieldRef {
        FieldRef::new_for_test("status", "users.status", ValueKind::Text, false)
    }

    fn integer_field() -> FieldRef {
        FieldRef::new_for_test("age", "users.age", ValueKind::Integer, false)
    }

    fn datetime_field() -> FieldRef {
        FieldRef::new_for_test("created_at", "users.created_at", ValueKind::DateTime, false)
    }

    fn regex_field() -> FieldRef {
        FieldRef::new_for_test("email", "users.email", ValueKind::Text, true)
    }

    fn query_with_filter(filter: Filter) -> RqsQuery {
        let mut query = RqsQuery::new();
        query.push_filter(filter);
        query
    }

    #[test]
    fn adapter_build_covers_unit_public_query_path() {
        let mut query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Gte,
            Some(RqsValue::Integer(18)),
        ));
        query.set_sort(vec![SortTerm::new(datetime_field(), SortDirection::Desc)]);
        query.set_projection(Projection::new(vec![text_field()]));
        query.pagination_mut().set_limit(10);
        query.pagination_mut().set_offset(20);
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.order_by);

        assert_eq!(result, Ok(Some("\"users\".\"created_at\" DESC".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_ascending_sort_path() {
        let mut query = RqsQuery::new();
        query.set_sort(vec![SortTerm::new(datetime_field(), SortDirection::Asc)]);
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.order_by);

        assert_eq!(result, Ok(Some("\"users\".\"created_at\" ASC".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_empty_query_path() {
        let query = RqsQuery::new();
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(None));
    }

    #[test]
    fn adapter_build_covers_unit_build_error_path() {
        let query = query_with_filter(Filter::new(
            text_field(),
            FilterOp::In,
            Some(RqsValue::List(Vec::new())),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map_err(|error| error.error_code());

        assert_eq!(result, Err("adapter_unsupported"));
    }

    #[test]
    fn adapter_build_covers_unit_mysql_path() {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Eq,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::MySql)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("`users`.`age` = ?".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_not_exists_path() {
        let query = query_with_filter(Filter::new(text_field(), FilterOp::NotExists, None));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"status\" IS NULL".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_not_equal_path() {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Ne,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" <> $1".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_greater_than_path() {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Gt,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" > $1".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_less_than_path() {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Lt,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" < $1".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_less_than_or_equal_path() {
        let query = query_with_filter(Filter::new(
            integer_field(),
            FilterOp::Lte,
            Some(RqsValue::Integer(18)),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"age\" <= $1".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_exists_path() {
        let query = query_with_filter(Filter::new(text_field(), FilterOp::Exists, None));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(
            result,
            Ok(Some("\"users\".\"status\" IS NOT NULL".to_owned()))
        );
    }

    #[test]
    fn adapter_build_covers_unit_regex_disabled_path() {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", "i"),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map_err(|error| error.error_code());

        assert_eq!(result, Err("adapter_unsupported"));
    }

    #[test]
    fn adapter_build_covers_unit_regex_path() {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", "i"),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .allow_regex()
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"email\" ~* $1".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_case_sensitive_regex_path() {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", ""),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .allow_regex()
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"email\" ~ $1".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_mysql_regex_path() {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", ""),
        ));
        let result = SqlxAdapter::new(SqlDialect::MySql)
            .allow_regex()
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("`users`.`email` REGEXP ?".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_sqlite_regex_path() {
        let query = query_with_filter(Filter::regex(
            regex_field(),
            RegexLiteral::new_for_test("@example.com$", ""),
        ));
        let result = SqlxAdapter::new(SqlDialect::Sqlite)
            .allow_regex()
            .build(&query)
            .map_err(|error| error.error_code());

        assert_eq!(result, Err("adapter_unsupported"));
    }

    #[test]
    fn adapter_build_covers_unit_in_path() {
        let query = query_with_filter(Filter::new(
            text_field(),
            FilterOp::In,
            Some(RqsValue::List(vec![RqsValue::Text("active".to_owned())])),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(result, Ok(Some("\"users\".\"status\" IN ($1)".to_owned())));
    }

    #[test]
    fn adapter_build_covers_unit_not_in_path() {
        let query = query_with_filter(Filter::new(
            text_field(),
            FilterOp::NotIn,
            Some(RqsValue::List(vec![RqsValue::Text("active".to_owned())])),
        ));
        let result = SqlxAdapter::new(SqlDialect::Postgres)
            .build(&query)
            .map(|parts| parts.where_clause);

        assert_eq!(
            result,
            Ok(Some("\"users\".\"status\" NOT IN ($1)".to_owned()))
        );
    }

    #[test]
    fn comparison_filter_without_value_is_rejected() {
        let filter = Filter::new(text_field(), FilterOp::Eq, None);
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false);
        let error = builder
            .filter_clause(&filter)
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
    }

    #[test]
    fn list_filter_without_list_value_is_rejected() {
        let filter = Filter::new(
            text_field(),
            FilterOp::In,
            Some(RqsValue::Text("active".to_owned())),
        );
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false);
        let error = builder
            .filter_clause(&filter)
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
    }

    #[test]
    fn list_clause_rejects_non_list_operator() {
        let filter = Filter::new(
            text_field(),
            FilterOp::Eq,
            Some(RqsValue::List(vec![RqsValue::Text("active".to_owned())])),
        );
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, false);
        let error = builder
            .list_clause(&filter, "\"users\".\"status\"")
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
    }

    #[test]
    fn regex_filter_without_literal_is_rejected() {
        let filter = Filter::new(text_field(), FilterOp::Regex, None);
        let mut builder = FragmentBuilder::new(SqlDialect::Postgres, true);
        let error = builder
            .filter_clause(&filter)
            .map_err(|error| error.error_code());

        assert_eq!(error, Err("adapter_unsupported"));
    }
}
