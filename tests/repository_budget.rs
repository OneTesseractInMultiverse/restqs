//! Repository budget checks using explicit, single-assertion cases.
#![cfg(feature = "sqlx")]

#[path = "../examples/support/budget.rs"]
pub mod budget;
#[path = "../examples/support/pagination.rs"]
pub mod pagination;
#[path = "../examples/support/users.rs"]
pub mod users;

use budget::QueryBudget;
use pagination::SqlStatement;
use restqs::{Parser, ParserConfig, ParserLimits, RqsValue};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Build a repository statement with the supplied request and application result budget.
fn statement(raw: &str, policy: QueryBudget, postgres: bool) -> TestResult<SqlStatement> {
    let catalog = users::users_catalog()?;
    let parser = Parser::with_config(
        &catalog,
        ParserConfig::with_limits(ParserLimits {
            max_limit: u64::MAX,
            ..ParserLimits::default()
        }),
    );
    let query = parser.parse(raw)?;
    if postgres {
        users::postgres_users_statement_with_budget(&query, policy)
    } else {
        users::sqlite_users_statement_with_budget(&query, policy)
    }
}

#[test]
fn postgres_omitted_limit_binds_the_public_default() -> TestResult {
    assert_eq!(
        statement("", QueryBudget::default(), true)?.binds,
        vec![RqsValue::Integer(25)]
    );
    Ok(())
}

#[test]
fn sqlite_omitted_limit_binds_the_public_default() -> TestResult {
    assert_eq!(
        statement("", QueryBudget::default(), false)?.binds,
        vec![RqsValue::Integer(25)]
    );
    Ok(())
}

#[test]
fn zero_limit_is_preserved() -> TestResult {
    assert_eq!(
        statement("limit=0", QueryBudget::default(), true)?.binds,
        vec![RqsValue::Integer(0)]
    );
    Ok(())
}

#[test]
fn inclusive_result_boundary_is_accepted() -> TestResult {
    assert_eq!(
        statement("limit=100", QueryBudget::default(), true)?.binds,
        vec![RqsValue::Integer(100)]
    );
    Ok(())
}

#[test]
fn repository_rejects_limits_above_its_budget() {
    assert_eq!(
        statement("limit=101", QueryBudget::default(), true).map_err(|e| e.to_string()),
        Err("request exceeds repository result limit".to_owned())
    );
}

#[test]
fn inclusive_offset_boundary_is_accepted() -> TestResult {
    assert_eq!(
        statement("skip=10000", QueryBudget::default(), true)?.binds,
        vec![RqsValue::Integer(25), RqsValue::Integer(10000)]
    );
    Ok(())
}

#[test]
fn offset_above_the_boundary_is_rejected() {
    assert_eq!(
        statement("skip=10001", QueryBudget::default(), false).map_err(|e| e.to_string()),
        Err("request exceeds repository offset limit".to_owned())
    );
}

#[test]
fn public_policy_rejects_maximum_unsigned_offset() {
    assert!(statement("skip=18446744073709551615", QueryBudget::default(), true).is_err());
}

#[test]
fn custom_budget_sets_a_different_default() -> TestResult {
    assert_eq!(
        statement("", QueryBudget::bounded(10, 20, 30)?, false)?.binds,
        vec![RqsValue::Integer(10)]
    );
    Ok(())
}

#[test]
fn custom_budget_rejects_its_own_offset_boundary() -> TestResult {
    assert!(statement("skip=31", QueryBudget::bounded(10, 20, 30)?, true).is_err());
    Ok(())
}

#[test]
fn zero_default_is_invalid_configuration() {
    assert!(QueryBudget::bounded(0, 100, 10000).is_err());
}

#[test]
fn default_above_maximum_is_invalid_configuration() {
    assert!(QueryBudget::bounded(101, 100, 10000).is_err());
}

#[test]
fn maximum_limit_must_fit_database_integer_range() {
    assert!(QueryBudget::bounded(25, u64::MAX, 10000).is_err());
}

#[test]
fn maximum_offset_must_fit_database_integer_range() {
    assert!(QueryBudget::bounded(25, 100, u64::MAX).is_err());
}

#[test]
fn internal_opt_in_preserves_an_omitted_limit() -> TestResult {
    assert_eq!(
        statement("", QueryBudget::unbounded_internal(), true)?.sql,
        r#"SELECT "users"."id", "users"."name" FROM users"#
    );
    Ok(())
}

#[test]
fn internal_opt_in_preserves_large_representable_offsets() -> TestResult {
    assert_eq!(
        statement(
            "skip=9223372036854775807",
            QueryBudget::unbounded_internal(),
            false
        )?
        .binds,
        vec![RqsValue::Integer(i64::MAX)]
    );
    Ok(())
}

#[test]
fn internal_opt_in_still_rejects_out_of_range_offsets() {
    assert!(
        statement(
            "skip=9223372036854775808",
            QueryBudget::unbounded_internal(),
            false
        )
        .is_err()
    );
}

#[test]
fn internal_opt_in_still_rejects_out_of_range_limits() {
    assert!(
        statement(
            "limit=18446744073709551615",
            QueryBudget::unbounded_internal(),
            true
        )
        .is_err()
    );
}
