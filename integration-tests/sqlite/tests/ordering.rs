//! Ordering contract checks; each test owns one assertion and helpers return setup/results.

use restqs_sqlite_tests::list_sqlite_users;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Return deterministic user rows whose ages, names, statuses, and activity distinguish query
/// behavior.
fn users(raw: &str) -> TestResult<Vec<(i64, String)>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(query_fixture(raw))
}

/// Create an isolated runtime and database connection, then execute one query against seeded
/// fixture data.
async fn query_fixture(raw: &str) -> TestResult<Vec<(i64, String)>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let result = seed_and_query(&pool, raw).await;
    pool.close().await;
    result
}

/// Seed the isolated users table and invoke the repository path selected by the test.
async fn seed_and_query(pool: &SqlitePool, raw: &str) -> TestResult<Vec<(i64, String)>> {
    seed(pool).await?;
    list_sqlite_users(pool, raw).await
}

/// Create and populate a fixture users table using trusted test data and bound values.
async fn seed(pool: &SqlitePool) -> TestResult {
    sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, status TEXT, age INTEGER, active BOOLEAN)")
        .execute(pool).await?;
    sqlx::query("INSERT INTO users VALUES (30, 'Cleo', 'active', 20, 1), (10, 'Ada', 'active', 40, 1), (40, 'Dee', 'active', 20, 1), (20, 'Bob', 'pending', 30, 0)")
        .execute(pool).await?;
    Ok(())
}

#[test]
fn ascending_sort_returns_rows_in_requested_order() -> TestResult {
    assert_eq!(
        users("sort=age,id")?,
        vec![
            (30, "Cleo".to_owned()),
            (40, "Dee".to_owned()),
            (20, "Bob".to_owned()),
            (10, "Ada".to_owned())
        ]
    );
    Ok(())
}

#[test]
fn explicit_ascending_prefix_is_honored() -> TestResult {
    assert_eq!(
        users("sort=%2Bname")?,
        vec![
            (10, "Ada".to_owned()),
            (20, "Bob".to_owned()),
            (30, "Cleo".to_owned()),
            (40, "Dee".to_owned())
        ]
    );
    Ok(())
}

#[test]
fn descending_sort_returns_rows_in_requested_order() -> TestResult {
    assert_eq!(
        users("sort=-id")?,
        vec![
            (40, "Dee".to_owned()),
            (30, "Cleo".to_owned()),
            (20, "Bob".to_owned()),
            (10, "Ada".to_owned())
        ]
    );
    Ok(())
}

#[test]
fn multiple_sort_terms_resolve_ties_in_order() -> TestResult {
    assert_eq!(
        users("sort=age,-name")?,
        vec![
            (40, "Dee".to_owned()),
            (30, "Cleo".to_owned()),
            (20, "Bob".to_owned()),
            (10, "Ada".to_owned())
        ]
    );
    Ok(())
}

#[test]
fn sorting_precedes_limit_and_offset() -> TestResult {
    assert_eq!(
        users("sort=age,id&limit=2&skip=1")?,
        vec![(40, "Dee".to_owned()), (20, "Bob".to_owned())]
    );
    Ok(())
}

#[test]
fn offset_without_limit_preserves_sorted_order() -> TestResult {
    assert_eq!(
        users("sort=age,id&skip=1")?,
        vec![
            (40, "Dee".to_owned()),
            (20, "Bob".to_owned()),
            (10, "Ada".to_owned())
        ]
    );
    Ok(())
}

#[test]
fn filters_and_fixed_projection_preserve_order() -> TestResult {
    assert_eq!(
        users("status=active&fields=name,id&sort=-name&limit=2")?,
        vec![(40, "Dee".to_owned()), (30, "Cleo".to_owned())]
    );
    Ok(())
}

#[test]
fn query_without_sort_still_executes() -> TestResult {
    assert_eq!(users("id=20")?, vec![(20, "Bob".to_owned())]);
    Ok(())
}
