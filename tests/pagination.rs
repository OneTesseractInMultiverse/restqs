#![allow(missing_docs)]

#[test]
fn pagination_new_sets_limit() {
    let pagination = restqs::Pagination::new(Some(10), Some(20));

    assert_eq!(pagination.limit(), Some(10));
}
