//! Calendar and timestamp validation without normalization.

/// Check an ASCII `YYYY-MM-DD` Gregorian date, including leap years and year zero, without
/// normalization.
pub(crate) fn is_valid_date(raw: &str) -> bool {
    if raw.len() != 10 || !raw.is_ascii() {
        return false;
    }
    if raw.as_bytes()[4] != b'-' || raw.as_bytes()[7] != b'-' {
        return false;
    }
    let Some(year) = decimal(&raw[..4]) else {
        return false;
    };
    let Some(month) = decimal(&raw[5..7]) else {
        return false;
    };
    let Some(day) = decimal(&raw[8..10]) else {
        return false;
    };
    day >= 1 && day <= days_in_month(year, month)
}

/// Require a valid date, `T` or `t`, and a valid clock with an explicit UTC marker or numeric
/// offset.
pub(crate) fn is_valid_datetime(raw: &str) -> bool {
    let Some((date, time)) = raw.split_once(['T', 't']) else {
        return false;
    };
    is_valid_date(date) && is_valid_time(time)
}

/// Compute the Gregorian month length; invalid month numbers have zero valid days.
fn days_in_month(year: u16, month: u16) -> u16 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Apply the Gregorian four-year rule with century and four-century exceptions.
fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Check ASCII `HH:MM:SS` and its offset suffix; reject leap seconds and out-of-range clock
/// fields.
fn is_valid_time(raw: &str) -> bool {
    if raw.len() < 9 || !raw.is_ascii() {
        return false;
    }
    if raw.as_bytes()[2] != b':' || raw.as_bytes()[5] != b':' {
        return false;
    }
    let Some(hour) = decimal(&raw[..2]) else {
        return false;
    };
    let Some(minute) = decimal(&raw[3..5]) else {
        return false;
    };
    let Some(second) = decimal(&raw[6..8]) else {
        return false;
    };
    hour < 24 && minute < 60 && second < 60 && is_valid_time_suffix(&raw[8..])
}

/// Consume optional fractional seconds with at least one digit, then require a valid offset.
fn is_valid_time_suffix(raw: &str) -> bool {
    let offset = if let Some(fraction) = raw.strip_prefix('.') {
        let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        &fraction[digits..]
    } else {
        raw
    };
    is_valid_offset(offset)
}

/// Accept `Z`/`z` or signed `HH:MM` with hours below 24 and minutes below 60, preserving negative
/// zero.
fn is_valid_offset(raw: &str) -> bool {
    if matches!(raw, "Z" | "z") {
        return true;
    }
    if raw.len() != 6 || !raw.is_ascii() {
        return false;
    }
    if !matches!(raw.as_bytes()[0], b'+' | b'-') || raw.as_bytes()[3] != b':' {
        return false;
    }
    let Some(hour) = decimal(&raw[1..3]) else {
        return false;
    };
    let Some(minute) = decimal(&raw[4..6]) else {
        return false;
    };
    hour < 24 && minute < 60
}

/// Parse an ASCII-only unsigned decimal component into u16; empty or overflowing input returns
/// None.
fn decimal(raw: &str) -> Option<u16> {
    if raw.bytes().all(|byte| byte.is_ascii_digit()) {
        raw.parse().ok()
    } else {
        None
    }
}
