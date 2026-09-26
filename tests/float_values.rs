//! Float values checks using explicit, single-assertion cases.

use restqs::{FieldCatalog, Filter, RqsError, RqsQuery, RqsResult, RqsValue, parse};

/// Parse one float score predicate using the normal catalog conversion path.
fn parse_score(raw: &str) -> RqsResult<RqsQuery> {
    let catalog = FieldCatalog::new().allow_float("score")?;
    parse(raw, &catalog)
}

/// Extract the parsed float bit pattern so signed zero and subnormal behavior can be checked
/// exactly.
fn float_bits(query: &RqsQuery) -> Option<u64> {
    match query.filters().first()?.value()? {
        RqsValue::Float(value) => Some(value.to_bits()),
        _ => None,
    }
}

#[test]
fn float_scalar_rejects_nan() {
    let result = parse_score("score=NaN").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_positive_infinity() {
    let result = parse_score("score=inf").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_negative_infinity() {
    let result = parse_score("score=-inf").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_positive_overflow() {
    let result = parse_score("score=1e999").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_negative_overflow() {
    let result = parse_score("score=-1e999").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_wrapper_rejects_nan() {
    let result = parse_score("score=float(NaN)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_wrapper_rejects_positive_infinity() {
    let result = parse_score("score=float(inf)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_wrapper_rejects_negative_infinity() {
    let result = parse_score("score=float(-inf)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_wrapper_rejects_positive_overflow() {
    let result = parse_score("score=float(1e999)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_wrapper_rejects_negative_overflow() {
    let result = parse_score("score=float(-1e999)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_list_rejects_nan() {
    let result = parse_score("score=in(1,NaN)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_list_rejects_positive_infinity() {
    let result = parse_score("score=in(inf,1)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_list_rejects_negative_infinity() {
    let result = parse_score("score=in(1,-inf)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_list_rejects_positive_overflow() {
    let result = parse_score("score=in(1,1e999)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_list_rejects_negative_overflow() {
    let result = parse_score("score=in(1,-1e999)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_list_alias_rejects_non_finite_items() {
    let result = parse_score("score=list(1,NaN)").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_encoded_positive_infinity() {
    let result = parse_score("score=%2Binf").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_full_infinity_spelling() {
    let result = parse_score("score=Infinity").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_scalar_rejects_case_variant_nan() {
    let result = parse_score("score=nAn").map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_value"));
}

#[test]
fn float_accepts_maximum_finite_value() -> RqsResult<()> {
    let query = parse_score("score=1.7976931348623157e308")?;

    assert_eq!(float_bits(&query), Some((f64::MAX).to_bits()));
    Ok(())
}

#[test]
fn float_accepts_minimum_finite_value() -> RqsResult<()> {
    let query = parse_score("score=-1.7976931348623157e308")?;

    assert_eq!(float_bits(&query), Some((f64::MIN).to_bits()));
    Ok(())
}

#[test]
fn float_accepts_minimum_positive_normal_value() -> RqsResult<()> {
    let query = parse_score("score=2.2250738585072014e-308")?;

    assert_eq!(float_bits(&query), Some((f64::MIN_POSITIVE).to_bits()));
    Ok(())
}

#[test]
fn float_accepts_smallest_positive_subnormal() -> RqsResult<()> {
    let query = parse_score("score=5e-324")?;

    assert_eq!(float_bits(&query), Some((f64::from_bits(1)).to_bits()));
    Ok(())
}

#[test]
fn float_preserves_positive_zero() -> RqsResult<()> {
    let query = parse_score("score=0.0")?;

    assert_eq!(float_bits(&query), Some((0.0_f64).to_bits()));
    Ok(())
}

#[test]
fn float_preserves_negative_zero() -> RqsResult<()> {
    let query = parse_score("score=-0.0")?;

    assert_eq!(float_bits(&query), Some((-0.0_f64).to_bits()));
    Ok(())
}

#[test]
fn float_positive_underflow_remains_positive_zero() -> RqsResult<()> {
    let query = parse_score("score=1e-999")?;

    assert_eq!(float_bits(&query), Some((0.0_f64).to_bits()));
    Ok(())
}

#[test]
fn float_negative_underflow_remains_negative_zero() -> RqsResult<()> {
    let query = parse_score("score=-1e-999")?;

    assert_eq!(float_bits(&query), Some((-0.0_f64).to_bits()));
    Ok(())
}

#[test]
fn float_wrapper_accepts_maximum_finite_value() -> RqsResult<()> {
    let query = parse_score("score=float(1.7976931348623157e308)")?;

    assert_eq!(float_bits(&query), Some((f64::MAX).to_bits()));
    Ok(())
}

#[test]
fn float_list_preserves_finite_boundaries() -> RqsResult<()> {
    let query = parse_score("score=in(-1.7976931348623157e308,5e-324,1.7976931348623157e308)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::List(vec![
            RqsValue::Float(f64::MIN),
            RqsValue::Float(f64::from_bits(1)),
            RqsValue::Float(f64::MAX),
        ]))
    );
    Ok(())
}

#[test]
fn non_finite_float_error_keeps_field_and_type_metadata() {
    assert_eq!(
        parse_score("score=NaN"),
        Err(RqsError::InvalidValue {
            field: "score".to_owned(),
            expected: "float",
        })
    );
}

#[test]
fn float_field_keeps_explicit_null_behavior() -> RqsResult<()> {
    let query = parse_score("score=null")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Null)
    );
    Ok(())
}

#[test]
fn text_field_preserves_special_float_spelling_as_text() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("label")?;
    let query = parse("label=str(NaN)", &catalog)?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Text("NaN".to_owned()))
    );
    Ok(())
}
