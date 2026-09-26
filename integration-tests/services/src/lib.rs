//! Executable PostgreSQL and MySQL repository conformance fixtures.

#[path = "../../../examples/support/budget.rs"]
pub mod budget;
#[path = "../../../examples/support/pagination.rs"]
pub mod pagination;
#[path = "../../../examples/support/users.rs"]
pub mod users;

pub mod mysql;
pub mod postgres;

use budget::QueryBudget;
use pagination::{SqlStatement, append_postgres_pagination};
use restqs::{
    Field, FieldCatalog, RqsError, RqsValue, ValueKind,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
    parse,
};

/// Result returned by the fixture repositories.
pub type RepositoryResult<T> = Result<T, Box<dyn std::error::Error>>;

// This separate catalog explicitly opts the fixture into regex execution.
/// Authorize the users fixture fields with regex permitted only for name.
fn regex_catalog() -> restqs::RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("id")?
        .allow(Field::new("name", ValueKind::Text)?.allow_regex())?
        .allow_text("status")?
        .allow_integer("age")?
        .allow_boolean("active")
}

/// Map fixture logical fields explicitly to trusted users-table columns.
fn columns() -> restqs::RqsResult<SqlxColumnMap> {
    SqlxColumnMap::new()
        .map("id", "users.id")?
        .map("name", "users.name")?
        .map("status", "users.status")?
        .map("age", "users.age")?
        .map("active", "users.active")
}

/// Reject selections that cannot be decoded as the fixture's fixed id/name response.
fn validate_projection(query: &restqs::RqsQuery) -> restqs::RqsResult<()> {
    let fields = query.projection().fields();
    if fields.is_empty()
        || (fields.len() == 2
            && fields.iter().any(|f| f.public_name() == "id")
            && fields.iter().any(|f| f.public_name() == "name"))
    {
        Ok(())
    } else {
        Err(RqsError::AdapterUnsupported {
            feature: "users projection other than id and name",
        })
    }
}

/// Parse, validate the response shape, translate with regex opt-in, and apply the default
/// repository result budget.
fn filter_parts(raw: &str, dialect: SqlDialect) -> RepositoryResult<SqlxQueryParts> {
    let query = parse(raw, &regex_catalog()?)?;
    validate_projection(&query)?;
    let parts = SqlxAdapter::new(dialect, columns()?)
        .allow_regex()
        .build(&query)?;
    Ok(QueryBudget::default().apply(parts)?)
}

/// Combine fixed response SQL with mapped filter and ordering fragments before pagination.
fn select_sql(parts: &SqlxQueryParts) -> String {
    let mut sql = "SELECT users.id, users.name FROM users".to_owned();
    if let Some(clause) = &parts.where_clause {
        sql.push_str(" WHERE ");
        sql.push_str(clause);
    }
    if let Some(clause) = &parts.order_by {
        sql.push_str(" ORDER BY ");
        sql.push_str(clause);
    }
    sql
}

/// Build bounded PostgreSQL regex SQL and append pagination after filter binds.
fn postgres_regex_statement(raw: &str) -> RepositoryResult<SqlStatement> {
    let parts = filter_parts(raw, SqlDialect::Postgres)?;
    Ok(append_postgres_pagination(&select_sql(&parts), &parts)?)
}

/// Build bounded MySQL fragments, then assemble fixed response SQL and positional pagination
/// binds.
fn mysql_statement(raw: &str) -> RepositoryResult<SqlStatement> {
    let parts = filter_parts(raw, SqlDialect::MySql)?;
    let sql = select_sql(&parts);
    mysql_pagination(sql, parts)
}

/// Require an effective limit and append signed limit/offset binds after filter values; reject
/// numeric overflow.
fn mysql_pagination(mut sql: String, parts: SqlxQueryParts) -> RepositoryResult<SqlStatement> {
    // The bounded policy always supplies a limit, including offset-only input.
    let limit = parts.limit.ok_or(RqsError::AdapterUnsupported {
        feature: "mysql offset without limit",
    })?;
    let limit = i64::try_from(limit)?;
    let offset = parts.offset.map(i64::try_from).transpose()?;
    let mut binds = parts.binds;
    sql.push_str(" LIMIT ?");
    binds.push(RqsValue::Integer(limit));
    if let Some(offset) = offset {
        sql.push_str(" OFFSET ?");
        binds.push(RqsValue::Integer(offset));
    }
    Ok(SqlStatement { sql, binds })
}
