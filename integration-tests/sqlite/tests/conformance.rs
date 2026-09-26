//! Conformance contract checks; each test owns one assertion and helpers return setup/results.

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
    sqlx::query("INSERT INTO users VALUES (30, 'Cleo', 'active', 20, 1), (10, 'Ada', 'active', 40, 1), (40, 'Dee', NULL, 20, 1), (20, 'Bob', 'pending', 30, 0)")
        .execute(pool).await?;
    Ok(())
}

#[test]
fn integer_equality() -> TestResult {
    let ids: Vec<_> = users("age=30")?.into_iter().map(|row| row.0).collect();
    assert_eq!(ids, vec![20]);
    Ok(())
}

#[test]
fn ordered_comparison() -> TestResult {
    let ids: Vec<_> = users("age>=30&sort=id")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![10, 20]);
    Ok(())
}

#[test]
fn null_equality() -> TestResult {
    let ids: Vec<_> = users("status=null")?.into_iter().map(|row| row.0).collect();
    assert_eq!(ids, vec![40]);
    Ok(())
}

#[test]
fn null_inequality() -> TestResult {
    let ids: Vec<_> = users("status!=null&sort=id")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![10, 20, 30]);
    Ok(())
}

#[test]
fn list_expansion() -> TestResult {
    let ids: Vec<_> = users("id=in(30,10)&sort=id")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![10, 30]);
    Ok(())
}

#[test]
fn list_exclusion() -> TestResult {
    let ids: Vec<_> = users("id!=in(10,30)&sort=id")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![20, 40]);
    Ok(())
}

#[test]
fn boolean_binding() -> TestResult {
    let ids: Vec<_> = users("active=false")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![20]);
    Ok(())
}

#[test]
fn text_binding() -> TestResult {
    let ids: Vec<_> = users("name=Ada")?.into_iter().map(|row| row.0).collect();
    assert_eq!(ids, vec![10]);
    Ok(())
}

#[test]
fn bound_injection_payload() -> TestResult {
    let ids: Vec<_> = users("name=str(x'%20OR%201=1--)")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![]);
    Ok(())
}

#[test]
fn ordering_and_pagination() -> TestResult {
    let ids: Vec<_> = users("sort=id&limit=2&skip=1")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![20, 30]);
    Ok(())
}

#[test]
fn offset_without_explicit_limit() -> TestResult {
    let ids: Vec<_> = users("sort=id&skip=3")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![40]);
    Ok(())
}

#[test]
fn zero_limit() -> TestResult {
    let ids: Vec<_> = users("limit=0")?.into_iter().map(|row| row.0).collect();
    assert_eq!(ids, vec![]);
    Ok(())
}

#[test]
fn fixed_projection() -> TestResult {
    let ids: Vec<_> = users("fields=name,id&sort=-id&limit=1")?
        .into_iter()
        .map(|row| row.0)
        .collect();
    assert_eq!(ids, vec![40]);
    Ok(())
}

#[test]
fn rejects_partial_projection() {
    assert!(users("fields=status").is_err());
}

#[test]
fn rejects_ordered_null() {
    assert!(users("age>null").is_err());
}

#[test]
fn rejects_excessive_offset() {
    assert!(users("skip=10001").is_err());
}

#[test]
fn rejects_regex_without_execution() {
    assert!(users("name=/Ada/").is_err());
}

/// Run the fixed-response repository against enough rows to exercise its default result cap.
async fn capped_fixture() -> TestResult<Vec<(i64, String)>> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    let result = seed_many_and_query(&pool).await;
    pool.close().await;
    result
}

/// Populate a larger isolated users table, then execute the requested query through the normal
/// repository.
async fn seed_many_and_query(pool: &SqlitePool) -> TestResult<Vec<(i64, String)>> {
    seed(pool).await?;
    sqlx::query("WITH RECURSIVE ids(id) AS (SELECT 101 UNION ALL SELECT id + 1 FROM ids WHERE id < 130) INSERT INTO users SELECT id, 'extra', 'active', 18, 1 FROM ids").execute(pool).await?;
    list_sqlite_users(pool, "sort=id").await
}

#[test]
fn omitted_limit_caps_actual_database_results() -> TestResult {
    let rows = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(capped_fixture())?;
    assert_eq!(rows.len(), 25);
    Ok(())
}
