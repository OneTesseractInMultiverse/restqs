#![allow(missing_docs)]

use restqs::{FieldCatalog, Filter, RqsError, RqsQuery, RqsResult, RqsValue, parse};

fn parse_temporal(input: &str) -> RqsResult<RqsQuery> {
    let catalog = FieldCatalog::new()
        .allow_date("day", "events.day")?
        .allow_datetime("timestamp", "events.timestamp")?;
    parse(input, &catalog)
}

#[test]
fn date_accepts_leap_day() -> RqsResult<()> {
    let query = parse_temporal("day=2024-02-29")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("2024-02-29".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_leap_century() -> RqsResult<()> {
    let query = parse_temporal("day=2000-02-29")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("2000-02-29".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_common_century_last_february_day() -> RqsResult<()> {
    let query = parse_temporal("day=1900-02-28")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("1900-02-28".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_year_zero() -> RqsResult<()> {
    let query = parse_temporal("day=0000-02-29")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("0000-02-29".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_maximum_year() -> RqsResult<()> {
    let query = parse_temporal("day=9999-12-31")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("9999-12-31".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_minimum_month_and_day() -> RqsResult<()> {
    let query = parse_temporal("day=0001-01-01")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("0001-01-01".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_thirty_day_month_end() -> RqsResult<()> {
    let query = parse_temporal("day=2026-04-30")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("2026-04-30".to_owned()))
    );
    Ok(())
}

#[test]
fn date_accepts_thirty_one_day_month_end() -> RqsResult<()> {
    let query = parse_temporal("day=2026-01-31")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("2026-01-31".to_owned()))
    );
    Ok(())
}

#[test]
fn date_rejects_common_year_leap_day() {
    let result = parse_temporal("day=2025-02-29");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_non_leap_century() {
    let result = parse_temporal("day=1900-02-29");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_day_after_leap_day() {
    let result = parse_temporal("day=2024-02-30");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_thirty_day_month_overflow() {
    let result = parse_temporal("day=2026-04-31");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_thirty_one_day_month_overflow() {
    let result = parse_temporal("day=2026-01-32");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_zero_day() {
    let result = parse_temporal("day=2026-01-00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_zero_month() {
    let result = parse_temporal("day=2026-00-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_month_thirteen() {
    let result = parse_temporal("day=2026-13-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_impossible_month_and_day() {
    let result = parse_temporal("day=2026-99-99");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_nondigit_year() {
    let result = parse_temporal("day=20a6-01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_nondigit_month() {
    let result = parse_temporal("day=2026-0a-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_nondigit_day() {
    let result = parse_temporal("day=2026-01-0a");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_first_separator() {
    let result = parse_temporal("day=2026%2F01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_second_separator() {
    let result = parse_temporal("day=2026-01%2F01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_short_month() {
    let result = parse_temporal("day=2026-1-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_expanded_year() {
    let result = parse_temporal("day=10000-01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_signed_year() {
    let result = parse_temporal("day=-001-01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_unicode_digits() {
    let result = parse_temporal("day=%EF%BC%92%EF%BC%90%EF%BC%92%EF%BC%96-01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_non_ascii_at_expected_byte_length() {
    let result = parse_temporal("day=2%C3%A96-01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date",
        })
    );
}

#[test]
fn date_rejects_trailing_text() {
    let result = parse_temporal("day=2026-01-01extra");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn date_rejects_surrounding_whitespace() {
    let result = parse_temporal("day=%202026-01-01%20");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn datetime_accepts_negative_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-09-16T12%3A00%3A00-06%3A00")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-09-16T12:00:00-06:00".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_accepts_positive_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-09-16T12%3A00%3A00%2B05%3A30")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-09-16T12:00:00+05:30".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_preserves_unknown_local_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-09-16T12%3A00%3A00-00%3A00")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-09-16T12:00:00-00:00".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_accepts_midnight() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T00%3A00%3A00Z")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-01-01T00:00:00Z".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_accepts_last_second_of_day() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T23%3A59%3A59Z")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-01-01T23:59:59Z".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_accepts_leap_day() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2000-02-29T12%3A00%3A00Z")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2000-02-29T12:00:00Z".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_preserves_lowercase_designators() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01t12%3A00%3A00z")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-01-01t12:00:00z".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_accepts_single_fraction_digit() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.1Z")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-01-01T12:00:00.1Z".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_preserves_long_fraction() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.123456789012345000Z")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime(
            "2026-01-01T12:00:00.123456789012345000Z".to_owned()
        ))
    );
    Ok(())
}

#[test]
fn datetime_accepts_fraction_with_negative_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.0001-06%3A00")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime(
            "2026-01-01T12:00:00.0001-06:00".to_owned()
        ))
    );
    Ok(())
}

#[test]
fn datetime_accepts_maximum_positive_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B23%3A59")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-01-01T12:00:00+23:59".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_accepts_maximum_negative_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=2026-01-01T12%3A00%3A00-23%3A59")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime("2026-01-01T12:00:00-23:59".to_owned()))
    );
    Ok(())
}

#[test]
fn datetime_rejects_impossible_components() {
    let result = parse_temporal("timestamp=2026-99-99T99%3A99%3A99Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_invalid_calendar_day() {
    let result = parse_temporal("timestamp=2025-02-29T12%3A00%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_hour_twenty_four() {
    let result = parse_temporal("timestamp=2026-01-01T24%3A00%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_minute_sixty() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A60%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_leap_second() {
    let result = parse_temporal("timestamp=2016-12-31T23%3A59%3A60Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_positive_offset_hour_overflow() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B24%3A00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_negative_offset_hour_overflow() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00-24%3A00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_positive_offset_minute_overflow() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B01%3A60");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_negative_offset_minute_overflow() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00-01%3A60");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_missing_offset() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_date_only() {
    let result = parse_temporal("timestamp=2026-01-01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_space_separator() {
    let result = parse_temporal("timestamp=2026-01-01%2012%3A00%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_short_clock() {
    let result = parse_temporal("timestamp=2026-01-01T1%3A00%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_first_time_separator() {
    let result = parse_temporal("timestamp=2026-01-01T12-00%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_second_time_separator() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00-00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_nondigit_hour() {
    let result = parse_temporal("timestamp=2026-01-01Tab%3A00%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_nondigit_minute() {
    let result = parse_temporal("timestamp=2026-01-01T12%3Aab%3A00Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_nondigit_second() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3AabZ");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_empty_fraction() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_nondigit_fraction() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.xZ");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_repeated_fraction_separator() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.1.2Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_comma_fraction() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2C5Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_unicode_fraction() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.%EF%BC%91Z");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_offset_without_colon() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B0600");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_offset_without_minutes() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B06");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_offset_seconds() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B06%3A00%3A01");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_nondigit_offset_hour() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2Bab%3A00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_nondigit_offset_minute() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B06%3Aab");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_wrong_offset_separator() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00%2B06-00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_missing_offset_sign() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00x06%3A00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_trailing_text() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00Zextra");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_multiple_offsets() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00Z%2B06%3A00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_fraction_without_offset() {
    let result = parse_temporal("timestamp=2026-01-01T12%3A00%3A00.123");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_rejects_unescaped_plus_in_query() {
    let result = parse_temporal("timestamp=2026-01-01T12:00:00+06:00");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn date_wrapper_accepts_leap_day() -> RqsResult<()> {
    let query = parse_temporal("day=date(2024-02-29)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::Date("2024-02-29".to_owned()))
    );
    Ok(())
}

#[test]
fn date_wrapper_rejects_invalid_calendar_day() {
    let result = parse_temporal("day=date(2025-02-29)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn datetime_wrapper_preserves_fraction_and_offset() -> RqsResult<()> {
    let query = parse_temporal("timestamp=datetime(2026-01-01T12:00:00.123-06:00)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::DateTime(
            "2026-01-01T12:00:00.123-06:00".to_owned()
        ))
    );
    Ok(())
}

#[test]
fn datetime_wrapper_rejects_invalid_calendar_day() {
    let result = parse_temporal("timestamp=datetime(2025-02-29T12:00:00Z)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_wrapper_rejects_invalid_time() {
    let result = parse_temporal("timestamp=datetime(2026-01-01T24:00:00Z)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_wrapper_rejects_invalid_offset() {
    let result = parse_temporal("timestamp=datetime(2026-01-01T12:00:00-24:00)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn date_list_preserves_valid_items() -> RqsResult<()> {
    let query = parse_temporal("day=in(2024-02-29,2026-04-30)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::List(vec![
            RqsValue::Date("2024-02-29".to_owned()),
            RqsValue::Date("2026-04-30".to_owned())
        ]))
    );
    Ok(())
}

#[test]
fn date_list_rejects_invalid_item() {
    let result = parse_temporal("day=in(2024-02-29,2025-02-29)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "day".to_owned(),
            expected: "date"
        })
    );
}

#[test]
fn datetime_list_preserves_offsets() -> RqsResult<()> {
    let query =
        parse_temporal("timestamp=list(2026-01-01T00:00:00-06:00,2026-01-01T12:00:00%2B05:30)")?;

    assert_eq!(
        query.filters().first().and_then(Filter::value),
        Some(&RqsValue::List(vec![
            RqsValue::DateTime("2026-01-01T00:00:00-06:00".to_owned()),
            RqsValue::DateTime("2026-01-01T12:00:00+05:30".to_owned())
        ]))
    );
    Ok(())
}

#[test]
fn datetime_list_rejects_invalid_calendar_day() {
    let result = parse_temporal("timestamp=in(2026-01-01T12:00:00Z,2025-02-29T12:00:00Z)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_list_rejects_invalid_time() {
    let result = parse_temporal("timestamp=list(2026-01-01T12:00:00Z,2026-01-01T12:60:00Z)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}

#[test]
fn datetime_list_rejects_invalid_offset() {
    let result = parse_temporal("timestamp=in(2026-01-01T12:00:00Z,2026-01-01T12:00:00-06:60)");

    assert_eq!(
        result,
        Err(RqsError::InvalidValue {
            field: "timestamp".to_owned(),
            expected: "datetime"
        })
    );
}
