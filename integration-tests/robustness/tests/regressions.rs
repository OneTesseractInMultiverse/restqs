use restqs::{ParserLimits, RqsError, RqsValue};
use restqs_robustness::{generous_limits, parse};

#[test]
fn operator_tokens_remain_inside_value_text() -> Result<(), RqsError> {
    let query = parse(
        include_str!("../corpus/operator-in-value.rqs"),
        generous_limits(),
    )?;
    assert_eq!(
        query.filters()[0].value(),
        Some(&RqsValue::Text("a>=b".to_owned()))
    );
    Ok(())
}

#[test]
fn invalid_encoding_regression() {
    assert!(matches!(
        parse(
            include_str!("../corpus/invalid-encoding.rqs"),
            generous_limits()
        ),
        Err(RqsError::InvalidEncoding)
    ));
}

#[test]
fn ordered_list_regression() {
    assert!(matches!(
        parse(
            include_str!("../corpus/ordered-list.rqs"),
            generous_limits()
        ),
        Err(RqsError::InvalidOperator)
    ));
}

#[test]
fn non_finite_regression() {
    assert!(matches!(
        parse(include_str!("../corpus/non-finite.rqs"), generous_limits()),
        Err(RqsError::InvalidValue { .. })
    ));
}

#[test]
fn regex_flags_regression() {
    assert!(matches!(
        parse(include_str!("../corpus/regex-flags.rqs"), generous_limits()),
        Err(RqsError::InvalidRegexFlags)
    ));
}

#[test]
fn duplicate_control_regression() {
    assert!(matches!(
        parse(
            include_str!("../corpus/duplicate-control.rqs"),
            generous_limits()
        ),
        Err(RqsError::DuplicateControl { .. })
    ));
}

#[test]
fn control_size_limit_regression() {
    let limits = ParserLimits {
        max_value_bytes: 6,
        ..generous_limits()
    };
    assert!(matches!(
        parse(include_str!("../corpus/oversized-control.rqs"), limits),
        Err(RqsError::ValueTooLarge { .. })
    ));
}
