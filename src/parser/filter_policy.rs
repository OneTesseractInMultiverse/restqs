//! Pure filter identity and duplicate-rejection policy.

use std::collections::BTreeSet;

use crate::{Filter, RqsError, RqsResult};

pub(super) type FilterKey = (String, &'static str);

pub(super) fn filter_key(filter: &Filter) -> FilterKey {
    (filter.field().public_name().to_owned(), filter.op().token())
}

pub(super) fn validate_new_filter(key: &FilterKey, seen: &BTreeSet<FilterKey>) -> RqsResult<()> {
    if seen.contains(key) {
        Err(RqsError::DuplicateFilter {
            field: key.0.clone(),
            operator: key.1,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Field, FilterOp, RqsValue, ValueKind};

    #[test]
    fn filter_identity_ignores_its_value() -> RqsResult<()> {
        let field = Field::new("age", ValueKind::Integer)?.to_ref();
        let first = Filter::new(field.clone(), FilterOp::Gte, Some(RqsValue::Integer(18)));
        let second = Filter::new(field, FilterOp::Gte, Some(RqsValue::Integer(21)));

        assert_eq!(filter_key(&first), filter_key(&second));
        Ok(())
    }

    #[test]
    fn duplicate_policy_reports_the_conflicting_identity() {
        let key = ("age".to_owned(), ">=");
        let seen = BTreeSet::from([key.clone()]);

        assert_eq!(
            validate_new_filter(&key, &seen),
            Err(RqsError::DuplicateFilter {
                field: "age".to_owned(),
                operator: ">=",
            })
        );
    }

    #[test]
    fn duplicate_policy_accepts_a_different_operator() {
        let seen = BTreeSet::from([("age".to_owned(), ">=")]);

        assert_eq!(validate_new_filter(&("age".to_owned(), "<"), &seen), Ok(()));
    }
}
