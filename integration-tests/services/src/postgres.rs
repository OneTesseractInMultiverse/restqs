//! SQLx Postgres binding and fixed-response decoding.
use crate::{RepositoryResult, pagination::SqlStatement};
use restqs::RqsValue;
use sqlx::{PgPool, Row};

/// Execute the documented fixed-response repository.
pub async fn list_users(pool: &PgPool, raw: &str) -> RepositoryResult<Vec<(i64, String)>> {
    let query = restqs::parse(raw, &crate::users::users_catalog()?)?;
    let statement = crate::users::postgres_users_statement(&query)?;
    execute(pool, statement).await
}

/// Execute regex with the fixture's explicitly authorized regex catalog.
pub async fn regex_users(pool: &PgPool, raw: &str) -> RepositoryResult<Vec<(i64, String)>> {
    let statement = crate::postgres_regex_statement(raw)?;
    execute(pool, statement).await
}

async fn execute(pool: &PgPool, statement: SqlStatement) -> RepositoryResult<Vec<(i64, String)>> {
    // Only fixed repository SQL, authorized mapped identifiers, and placeholders
    // reach this private executor. Request values remain separate bound data.
    let mut query = sqlx::query(sqlx::AssertSqlSafe(statement.sql.as_str()));
    for value in &statement.binds {
        query = bind_value(query, value)?;
    }
    let rows = query.fetch_all(pool).await?;
    Ok(decode_users(rows)?)
}

fn decode_users(rows: Vec<sqlx::postgres::PgRow>) -> Result<Vec<(i64, String)>, sqlx::Error> {
    rows.into_iter()
        .map(|row| Ok((row.try_get("id")?, row.try_get("name")?)))
        .collect()
}

fn bind_value<'query>(
    query: sqlx::query::Query<'query, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: &'query RqsValue,
) -> Result<sqlx::query::Query<'query, sqlx::Postgres, sqlx::postgres::PgArguments>, restqs::RqsError>
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
