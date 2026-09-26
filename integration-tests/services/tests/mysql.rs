//! Mysql contract checks; each test owns one assertion and helpers return setup/results.

use restqs_service_tests::mysql;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Return deterministic user rows whose ages, names, statuses, and activity distinguish query
/// behavior.
fn users(raw: &str, regex: bool) -> TestResult<Vec<i64>> {
    let rows = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(query_fixture(raw, regex))?;
    Ok(rows.into_iter().map(|row| row.0).collect())
}

/// Create an isolated runtime and database connection, then execute one query against seeded
/// fixture data.
async fn query_fixture(raw: &str, regex: bool) -> TestResult<Vec<(i64, String)>> {
    let url = std::env::var("RESTQS_MYSQL_URL")?;
    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let result = seed_and_query(&pool, raw, regex).await;
    pool.close().await;
    result
}

/// Seed the isolated users table and invoke the repository path selected by the test.
async fn seed_and_query(
    pool: &MySqlPool,
    raw: &str,
    regex: bool,
) -> TestResult<Vec<(i64, String)>> {
    seed(pool).await?;
    let _ = regex;
    mysql::list_users(pool, raw).await
}

/// Create and populate a fixture users table using trusted test data and bound values.
async fn seed(pool: &MySqlPool) -> TestResult {
    sqlx::query("CREATE TEMPORARY TABLE users (id BIGINT PRIMARY KEY, name VARCHAR(80) NOT NULL, status VARCHAR(20), age BIGINT, active BOOLEAN)")
        .execute(pool).await?;
    sqlx::query("INSERT INTO users VALUES (30, 'Cleo', 'active', 20, TRUE), (10, 'Ada', 'active', 40, TRUE), (40, 'Dee', NULL, 20, TRUE), (20, 'Bob', 'pending', 30, FALSE)")
        .execute(pool).await?;
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn integer_equality() -> TestResult {
    assert_eq!(users("age=30", false)?, vec![20]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn ordered_comparison() -> TestResult {
    assert_eq!(users("age>=30&sort=id", false)?, vec![10, 20]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn null_equality() -> TestResult {
    assert_eq!(users("status=null", false)?, vec![40]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn null_inequality() -> TestResult {
    assert_eq!(users("status!=null&sort=id", false)?, vec![10, 20, 30]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn list_expansion() -> TestResult {
    assert_eq!(users("id=in(30,10)&sort=id", false)?, vec![10, 30]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn list_exclusion() -> TestResult {
    assert_eq!(users("id!=in(10,30)&sort=id", false)?, vec![20, 40]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn boolean_binding() -> TestResult {
    assert_eq!(users("active=false", false)?, vec![20]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn text_binding() -> TestResult {
    assert_eq!(users("name=Ada", false)?, vec![10]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn bound_injection_payload() -> TestResult {
    assert_eq!(users("name=str(x'%20OR%201=1--)", false)?, vec![]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn ordering_and_pagination() -> TestResult {
    assert_eq!(users("sort=id&limit=2&skip=1", false)?, vec![20, 30]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn offset_without_explicit_limit() -> TestResult {
    assert_eq!(users("sort=id&skip=3", false)?, vec![40]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn zero_limit() -> TestResult {
    assert_eq!(users("limit=0", false)?, vec![]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn fixed_projection() -> TestResult {
    assert_eq!(users("fields=name,id&sort=-id&limit=1", false)?, vec![40]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn rejects_partial_projection() {
    assert!(users("fields=status", false).is_err());
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn rejects_ordered_null() {
    assert!(users("age>null", false).is_err());
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn rejects_excessive_offset() {
    assert!(users("skip=10001", false).is_err());
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn supported_regex_matches_rows() -> TestResult {
    assert_eq!(users("name=/^A/", true)?, vec![10]);
    Ok(())
}

#[test]
#[ignore = "requires RESTQS_MYSQL_URL"]
fn unsupported_regex_flags_are_rejected() {
    assert!(users("name=/^A/i", true).is_err());
}
