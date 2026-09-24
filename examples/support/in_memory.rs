//! A deliberately small non-SQL consumer of a logical query plan.

use restqs::{FilterOp, RqsError, RqsQuery, RqsResult, RqsValue};

/// An application record stored in memory.
pub struct Person {
    /// Display name returned by the example.
    pub name: &'static str,
    /// Age in years, with no database column metadata.
    pub age: i64,
}

/// Select names using a single logical `age>=N` predicate.
///
/// This example rejects every other plan shape, including sorting, projection,
/// and pagination. It is not a general query engine.
pub fn eligible_names(query: &RqsQuery, people: &[Person]) -> RqsResult<Vec<&'static str>> {
    let age = minimum_age(query)?;
    Ok(people
        .iter()
        .filter(|person| person.age >= age)
        .map(|person| person.name)
        .collect())
}

fn minimum_age(query: &RqsQuery) -> RqsResult<i64> {
    let unsupported = RqsError::AdapterUnsupported {
        feature: "in-memory example query shape",
    };
    if !query.sort().is_empty()
        || !query.projection().is_empty()
        || query.pagination().limit().is_some()
        || query.pagination().offset().is_some()
    {
        return Err(unsupported);
    }
    match query.filters() {
        [filter] if filter.field().public_name() == "age" && filter.op() == FilterOp::Gte => {
            match filter.value() {
                Some(RqsValue::Integer(age)) => Ok(*age),
                _ => Err(unsupported),
            }
        }
        _ => Err(unsupported),
    }
}
