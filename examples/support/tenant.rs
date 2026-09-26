//! Repository-owned composition with a mandatory tenant predicate.

use restqs::{
    RqsQuery, RqsValue,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
};

use super::pagination::{SqlStatement, append_postgres_pagination};

/// Build a tenant-scoped statement from an authorized query plan.
///
/// `tenant_id` comes from the authenticated application context, never from RQS.
/// The caller supplies trusted column mappings and binds `statement.binds` in order.
/// Projection remains dynamic; the caller must decode the selected columns.
pub fn tenant_users_statement(
    query: &RqsQuery,
    columns: SqlxColumnMap,
    tenant_id: i64,
) -> Result<SqlStatement, Box<dyn std::error::Error>> {
    let parts = SqlxAdapter::new(SqlDialect::Postgres, columns).build_with_bind_start(query, 2)?;
    let sql = tenant_select_sql(&parts);
    let combined = prepend_tenant_bind(parts, tenant_id);
    Ok(append_postgres_pagination(&sql, &combined)?)
}

fn tenant_select_sql(parts: &SqlxQueryParts) -> String {
    let projection = if parts.projection.is_empty() {
        r#""users"."id", "users"."status""#.to_owned()
    } else {
        parts.projection.join(", ")
    };
    let mut sql = format!(r#"SELECT {projection} FROM users WHERE "users"."tenant_id" = $1"#);
    if let Some(clause) = &parts.where_clause {
        sql.push_str(" AND (");
        sql.push_str(clause);
        sql.push(')');
    }
    if let Some(order_by) = &parts.order_by {
        sql.push_str(" ORDER BY ");
        sql.push_str(order_by);
    }
    sql
}

fn prepend_tenant_bind(mut parts: SqlxQueryParts, tenant_id: i64) -> SqlxQueryParts {
    parts.binds.insert(0, RqsValue::Integer(tenant_id));
    parts
}
