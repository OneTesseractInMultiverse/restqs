//! Repository SQL assembly for a fixed `(id, name)` user response.

use restqs::{
    FieldCatalog, Projection, RqsError, RqsQuery, RqsResult,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
};

use super::budget::QueryBudget;
use super::pagination::{SqlStatement, append_postgres_pagination, append_sqlite_pagination};

type RepositoryResult = Result<SqlStatement, Box<dyn std::error::Error>>;

/// Authorize the fixed response fields and the endpoint's filter/sort fields.
pub fn users_catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("id")?
        .allow_text("name")?
        .allow_text("status")?
        .allow_integer("age")?
        .allow_boolean("active")
}

/// Assemble PostgreSQL SQL after validating the fixed response projection.
///
/// The plan must be authorized with [`users_catalog`]. Omitted/empty projection
/// or exactly `id,name` in either order is supported. Other selections return
/// `adapter_unsupported` before SQL execution. Rows always contain `id` and `name`.
/// Default policy caps omitted limits at 25, explicit limits at 100, and offsets at 10,000.
pub fn postgres_users_statement(query: &RqsQuery) -> RepositoryResult {
    postgres_users_statement_with_budget(query, QueryBudget::default())
}

/// Assemble a user statement with an explicit application-owned result policy.
pub fn postgres_users_statement_with_budget(
    query: &RqsQuery,
    budget: QueryBudget,
) -> RepositoryResult {
    validate_users_projection(query.projection())?;
    let parts = SqlxAdapter::new(SqlDialect::Postgres, users_columns()?).build(query)?;
    let parts = budget.apply(parts)?;
    let sql = users_select_sql(&parts);
    Ok(append_postgres_pagination(&sql, &parts)?)
}

/// Assemble SQLite SQL using the same fixed response and default budgets as PostgreSQL.
pub fn sqlite_users_statement(query: &RqsQuery) -> RepositoryResult {
    sqlite_users_statement_with_budget(query, QueryBudget::default())
}

/// Assemble a user statement with an explicit application-owned result policy.
pub fn sqlite_users_statement_with_budget(
    query: &RqsQuery,
    budget: QueryBudget,
) -> RepositoryResult {
    validate_users_projection(query.projection())?;
    let parts = SqlxAdapter::new(SqlDialect::Sqlite, users_columns()?).build(query)?;
    let parts = budget.apply(parts)?;
    let sql = users_select_sql(&parts);
    Ok(append_sqlite_pagination(&sql, &parts)?)
}

fn validate_users_projection(projection: &Projection) -> RqsResult<()> {
    let fields = projection.fields();
    if fields.is_empty()
        || (fields.len() == 2
            && fields.iter().any(|field| field.public_name() == "id")
            && fields.iter().any(|field| field.public_name() == "name"))
    {
        Ok(())
    } else {
        Err(RqsError::AdapterUnsupported {
            feature: "users projection other than id and name",
        })
    }
}

fn users_columns() -> RqsResult<SqlxColumnMap> {
    SqlxColumnMap::new()
        .map("id", "users.id")?
        .map("name", "users.name")?
        .map("status", "users.status")?
        .map("age", "users.age")?
        .map("active", "users.active")
}

fn users_select_sql(parts: &SqlxQueryParts) -> String {
    let mut sql = r#"SELECT "users"."id", "users"."name" FROM users"#.to_owned();
    if let Some(where_clause) = &parts.where_clause {
        sql.push_str(" WHERE ");
        sql.push_str(where_clause);
    }
    if let Some(order_by) = &parts.order_by {
        sql.push_str(" ORDER BY ");
        sql.push_str(order_by);
    }
    sql
}
