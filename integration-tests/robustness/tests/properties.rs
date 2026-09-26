//! Properties contract checks; each test owns one assertion and helpers return setup/results.

use proptest::{
    prelude::*,
    test_runner::{Config, FileFailurePersistence, RngSeed, TestCaseError, TestError, TestRunner},
};
use restqs::adapters::sqlx::SqlDialect;
use restqs::{ParserLimits, RqsError};
use restqs_robustness::*;

/// Generate up to 255 arbitrary Unicode scalar values, including query delimiters and non-ASCII
/// input.
fn text() -> impl Strategy<Value = String> {
    proptest::collection::vec(any::<char>(), 0..256).prop_map(|chars| chars.into_iter().collect())
}

/// Load proptest configuration, then apply reproducible defaults and the checked-in regression
/// path.
fn config() -> Config {
    deterministic_config(Config::default())
}

/// Preserve explicit seeds, replace random seeds with a fixed default, and cap shrinking at 4096
/// iterations.
fn deterministic_config(mut config: Config) -> Config {
    if matches!(config.rng_seed, RngSeed::Random) {
        config.rng_seed = RngSeed::Fixed(0x525153);
    }
    config.max_shrink_iters = 4096;
    config.failure_persistence = Some(Box::new(FileFailurePersistence::Direct(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/regressions/seeds.txt"
    ))));
    config
}

/// Run one boolean contract across generated cases, returning proptest's minimized failure to the
/// test assertion.
fn check<S: Strategy>(
    strategy: S,
    property: impl Fn(S::Value) -> bool,
) -> Result<(), TestError<S::Value>> {
    TestRunner::new(config()).run(&strategy, |value| {
        if property(value) {
            Ok(())
        } else {
            Err(TestCaseError::fail("contract violation"))
        }
    })
}

/// Generate nonempty integer lists and arbitrary text/age values in a valid mixed query shape.
fn mixed() -> impl Strategy<Value = String> {
    (
        text(),
        proptest::collection::vec(any::<i64>(), 1..16),
        any::<i64>(),
    )
        .prop_map(|(text, values, age)| mixed_query(&text, &values, age))
}

#[test]
fn arbitrary_unicode_and_limits_do_not_panic() {
    let result = check((text(), any::<[u8; 5]>()), |(raw, seed)| {
        let _ = parse(&raw, limits_from_seed(&seed));
        true
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn percent_encoding_round_trips_arbitrary_unicode() {
    let result = check(text(), |value| decoded_text_matches(&value));
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn successful_fields_match_catalog_metadata() {
    let result = check(prop_oneof![text(), mixed()], |raw| {
        match parse(&raw, generous_limits()) {
            Ok(query) => fields_are_resolved(&query),
            Err(_) => true,
        }
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn successful_filters_have_valid_operator_value_pairs() {
    let result = check(prop_oneof![text(), mixed()], |raw| {
        match parse(&raw, generous_limits()) {
            Ok(query) => operators_are_valid(&query),
            Err(_) => true,
        }
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn raw_query_limit_rejects_one_byte_over_budget() {
    let result = check(text(), |value| {
        let raw = text_query(&value);
        let limits = ParserLimits {
            max_query_bytes: raw.len() - 1,
            ..generous_limits()
        };
        matches!(parse(&raw, limits), Err(RqsError::QueryTooLarge { .. }))
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn raw_query_limit_accepts_the_exact_byte_boundary() {
    let result = check(text(), |value| {
        let raw = text_query(&value);
        parse(
            &raw,
            ParserLimits {
                max_query_bytes: raw.len(),
                ..generous_limits()
            },
        )
        .is_ok()
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn nonempty_parameter_count_is_bounded_before_interpretation() {
    let result = check(1usize..128, |count| {
        let raw = vec!["id"; count].join("&&");
        matches!(
            parse(
                &raw,
                ParserLimits {
                    max_parameters: count - 1,
                    ..generous_limits()
                }
            ),
            Err(RqsError::TooManyParameters { .. })
        )
    });
    assert!(result.is_ok(), "{result:?}");
}

/// Construct the selected scalar, matching cast, or list input for temporal conversion
/// properties.
fn value_path(kind: u8, count: usize) -> (&'static str, String) {
    match kind {
        0 => ("text", "é".repeat(count + 1)),
        1 => ("text", format!("str({})", "é".repeat(count))),
        2 => ("name", format!("/{}/", "x".repeat(count))),
        3 => ("id", format!("in({})", vec!["1"; count + 1].join(","))),
        4 => ("sort", vec!["age"; count + 1].join(",")),
        5 => ("fields", vec!["text"; count + 1].join(",")),
        6 => ("limit", "1".repeat(count + 1)),
        _ => ("skip", "1".repeat(count + 1)),
    }
}

#[test]
fn decoded_value_limit_covers_scalars_wrappers_regex_lists_and_controls() {
    let result = check((0u8..8, 0usize..64), |(kind, count)| {
        let (field, value) = value_path(kind, count);
        let raw = format!("{field}={}", encode(&value));
        matches!(
            parse(
                &raw,
                ParserLimits {
                    max_value_bytes: value.len() - 1,
                    ..generous_limits()
                }
            ),
            Err(RqsError::ValueTooLarge { .. })
        )
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn exact_decoded_value_boundary_is_not_rejected_as_oversized() {
    let result = check((0u8..8, 0usize..64), |(kind, count)| {
        let (field, value) = value_path(kind, count);
        let raw = format!("{field}={}", encode(&value));
        !matches!(
            parse(
                &raw,
                ParserLimits {
                    max_value_bytes: value.len(),
                    ..generous_limits()
                }
            ),
            Err(RqsError::ValueTooLarge { .. })
        )
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn list_item_limits_precede_operator_compatibility() {
    let result = check(
        (
            1usize..64,
            prop::sample::select(vec!["=", "!=", ">", ">=", "<", "<="]),
        ),
        |(count, op)| {
            let raw = format!("id{op}in({})", vec!["1"; count].join(","));
            matches!(
                parse(
                    &raw,
                    ParserLimits {
                        max_list_items: count - 1,
                        ..generous_limits()
                    }
                ),
                Err(RqsError::TooManyListItems { .. })
            )
        },
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn list_item_boundary_accepts_membership_lists() {
    let result = check(0usize..64, |count| {
        let raw = format!("id=in({})", vec!["1"; count].join(","));
        parse(
            &raw,
            ParserLimits {
                max_list_items: count,
                ..generous_limits()
            },
        )
        .is_ok()
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn mixed_filters_preserve_complete_bind_sequences() {
    let result = check(mixed(), |raw| {
        let Ok(query) = parse(&raw, generous_limits()) else {
            return false;
        };
        [SqlDialect::Postgres, SqlDialect::MySql, SqlDialect::Sqlite]
            .into_iter()
            .all(|dialect| {
                build(&query, dialect, 1).is_ok_and(|parts| parts.binds == expected_binds(&query))
            })
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn mixed_filters_preserve_placeholder_correspondence_after_existing_binds() {
    let result = check((mixed(), 1usize..128), |(raw, first)| {
        let Ok(query) = parse(&raw, generous_limits()) else {
            return false;
        };
        [SqlDialect::Postgres, SqlDialect::MySql, SqlDialect::Sqlite]
            .into_iter()
            .all(|dialect| {
                build(&query, dialect, first)
                    .is_ok_and(|parts| placeholders_match(&parts, dialect, first))
            })
    });
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn changing_value_text_cannot_change_sql_structure() {
    let result = check(text(), |value| value_cannot_change_sql(&value));
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn malformed_percent_bytes_are_rejected() {
    let result = check(
        prop::sample::select(vec!["%", "%0", "%GG", "%FF", "%C3%28", "%E2%82"]),
        |suffix| {
            matches!(
                parse(&format!("text={suffix}"), generous_limits()),
                Err(RqsError::InvalidEncoding)
            )
        },
    );
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn regex_binds_follow_null_predicates() {
    let result = check(("[a-z]{0,64}", 1usize..128), |(pattern, first)| {
        let raw = format!("text=null&name=/{pattern}/i&age>=18");
        let Ok(query) = parse(&raw, generous_limits()) else {
            return false;
        };
        build(&query, SqlDialect::Postgres, first)
            .is_ok_and(|parts| placeholders_match(&parts, SqlDialect::Postgres, first))
    });
    assert!(result.is_ok(), "{result:?}");
}
