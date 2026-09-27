//! Executable SQLite repository used by the integration guide.

#[path = "../../../examples/support/budget.rs"]
pub mod budget;
#[path = "../../../examples/support/pagination.rs"]
pub mod pagination;
#[path = "../../../examples/support/users.rs"]
pub mod users;

use restqs::{RqsValue, parse};
use sqlx::{Row, SqlitePool};
use users::{sqlite_users_statement, users_catalog};

/// Parse, assemble, bind, execute, and decode the fixed user response.
pub async fn list_sqlite_users(
    pool: &SqlitePool,
    raw: &str,
) -> Result<Vec<(i64, String)>, Box<dyn std::error::Error>> {
    let query = parse(raw, &users_catalog()?)?;
    let statement = sqlite_users_statement(&query)?;
    // Only repository-built SQL reaches this boundary; request values remain binds.
    let mut query = sqlx::query(sqlx::AssertSqlSafe(statement.sql.as_str()));
    for value in &statement.binds {
        query = bind_sqlite_value(query, value)?;
    }
    let rows = query.fetch_all(pool).await?;
    Ok(decode_sqlite_users(rows)?)
}

/// Decode id and name from every SQLite row, propagating driver type or missing-column errors.
fn decode_sqlite_users(
    rows: Vec<sqlx::sqlite::SqliteRow>,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    rows.into_iter()
        .map(|row| Ok((row.try_get("id")?, row.try_get("name")?)))
        .collect()
}

/// Bind one flattened scalar, storing dates, timestamps, and UUIDs as text; reject nested list
/// values.
fn bind_sqlite_value<'query>(
    query: sqlx::query::Query<'query, sqlx::Sqlite, sqlx::sqlite::SqliteArguments>,
    value: &'query RqsValue,
) -> Result<sqlx::query::Query<'query, sqlx::Sqlite, sqlx::sqlite::SqliteArguments>, restqs::RqsError>
{
    match value {
        RqsValue::Null => Ok(query.bind(Option::<String>::None)),
        RqsValue::Boolean(value) => Ok(query.bind(*value)),
        RqsValue::Integer(value) => Ok(query.bind(*value)),
        RqsValue::Float(value) => Ok(query.bind(*value)),
        RqsValue::Text(value)
        | RqsValue::Date(value)
        | RqsValue::DateTime(value)
        | RqsValue::Uuid(value) => Ok(query.bind(value.as_str())),
        RqsValue::List(_) => Err(restqs::RqsError::AdapterUnsupported {
            feature: "nested list bind",
        }),
    }
}
