#![allow(missing_docs)]

use restqs::{
    Field, FieldCatalog, Parser, ParserConfig, ParserLimits, RegexLiteral, RqsResult, RqsValue,
    ValueKind, parse,
};

fn regex_catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new().allow(Field::new("email", "users.email", ValueKind::Text)?.allow_regex())
}

#[test]
fn parser_preserves_empty_regex_flags() -> RqsResult<()> {
    let query = parse("email=/admin/", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].regex_literal().map(RegexLiteral::flags),
        Some("")
    );
    Ok(())
}

#[test]
fn parser_preserves_case_insensitive_flag() -> RqsResult<()> {
    let query = parse("email=/admin/i", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].regex_literal().map(RegexLiteral::flags),
        Some("i")
    );
    Ok(())
}

#[test]
fn parser_preserves_multiline_flag() -> RqsResult<()> {
    let query = parse("email=/admin/m", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].regex_literal().map(RegexLiteral::flags),
        Some("m")
    );
    Ok(())
}

#[test]
fn parser_preserves_dotall_flag() -> RqsResult<()> {
    let query = parse("email=/a.b/s", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].regex_literal().map(RegexLiteral::flags),
        Some("s")
    );
    Ok(())
}

#[test]
fn parser_preserves_extended_flag() -> RqsResult<()> {
    let query = parse("email=/a b/x", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].regex_literal().map(RegexLiteral::flags),
        Some("x")
    );
    Ok(())
}

#[test]
fn parser_preserves_combined_flags_in_input_order() -> RqsResult<()> {
    let query = parse("email=/admin/xsmi", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].regex_literal().map(RegexLiteral::flags),
        Some("xsmi")
    );
    Ok(())
}

#[test]
fn parser_rejects_unknown_regex_flag() -> RqsResult<()> {
    let result = parse("email=/admin/z", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_unknown_flag_after_known_flag() -> RqsResult<()> {
    let result = parse("email=/admin/iz", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_uppercase_regex_flag() -> RqsResult<()> {
    let result = parse("email=/admin/I", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_unicode_regex_flag() -> RqsResult<()> {
    let result = parse("email=/admin/é", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_whitespace_in_regex_flags() -> RqsResult<()> {
    let result = parse("email=/admin/i%20m", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_validates_percent_decoded_flags() -> RqsResult<()> {
    let result =
        parse("email=%2Fadmin%2F%7A", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_duplicate_case_insensitive_flag() -> RqsResult<()> {
    let result = parse("email=/admin/ii", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_duplicate_multiline_flag() -> RqsResult<()> {
    let result = parse("email=/admin/mm", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_duplicate_dotall_flag() -> RqsResult<()> {
    let result = parse("email=/a.b/ss", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_duplicate_extended_flag() -> RqsResult<()> {
    let result = parse("email=/a b/xx", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_rejects_nonadjacent_duplicate_flags() -> RqsResult<()> {
    let result = parse("email=/admin/imsi", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_regex_flags"));
    Ok(())
}

#[test]
fn parser_checks_size_before_regex_flags() -> RqsResult<()> {
    let catalog = regex_catalog()?;
    let config = ParserConfig::with_limits(ParserLimits {
        max_value_bytes: 3,
        ..ParserLimits::default()
    });
    let result = Parser::with_config(&catalog, config)
        .parse("email=/admin/z")
        .map_err(|error| error.error_code());

    assert_eq!(result, Err("value_too_large"));
    Ok(())
}

#[test]
fn parser_checks_operator_before_regex_flags() -> RqsResult<()> {
    let result = parse("email!=/admin/z", &regex_catalog()?).map_err(|error| error.error_code());

    assert_eq!(result, Err("invalid_operator"));
    Ok(())
}

#[test]
fn parser_checks_field_permission_before_regex_flags() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("email", "users.email")?;
    let result = parse("email=/admin/z", &catalog).map_err(|error| error.error_code());

    assert_eq!(result, Err("regex_disabled"));
    Ok(())
}

#[test]
fn regex_flag_error_message_excludes_input() -> RqsResult<()> {
    let result = parse(
        "email=/private-pattern/z%0Aprivate-flags",
        &regex_catalog()?,
    )
    .map_err(|error| error.to_string());

    assert_eq!(
        result,
        Err("regex flags must be unique letters from i, m, s, x".to_owned())
    );
    Ok(())
}

#[test]
fn explicit_text_bypasses_regex_flag_validation() -> RqsResult<()> {
    let query = parse("email=str(/admin/z)", &regex_catalog()?)?;

    assert_eq!(
        query.filters()[0].value(),
        Some(&RqsValue::Text("/admin/z".to_owned()))
    );
    Ok(())
}

#[cfg(feature = "sqlx")]
mod sqlx {
    use super::*;
    use restqs::RqsError;
    use restqs::adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxQueryParts};

    fn build_regex(input: &str, dialect: SqlDialect) -> RqsResult<SqlxQueryParts> {
        let query = parse(input, &regex_catalog()?)?;
        SqlxAdapter::new(dialect).allow_regex().build(&query)
    }

    #[test]
    fn postgres_rejects_multiline_flag() {
        let result = build_regex("email=/admin/m", SqlDialect::Postgres);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "postgres regex flags"
            })
        );
    }

    #[test]
    fn postgres_rejects_dotall_flag() {
        let result = build_regex("email=/a.b/s", SqlDialect::Postgres);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "postgres regex flags"
            })
        );
    }

    #[test]
    fn postgres_rejects_extended_flag() {
        let result = build_regex("email=/a b/x", SqlDialect::Postgres);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "postgres regex flags"
            })
        );
    }

    #[test]
    fn postgres_rejects_supported_flag_combined_with_unsupported_flag() {
        let result = build_regex("email=/admin/im", SqlDialect::Postgres);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "postgres regex flags"
            })
        );
    }

    #[test]
    fn mysql_rejects_case_insensitive_flag() {
        let result = build_regex("email=/admin/i", SqlDialect::MySql);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "mysql regex flags"
            })
        );
    }

    #[test]
    fn mysql_rejects_multiline_flag() {
        let result = build_regex("email=/admin/m", SqlDialect::MySql);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "mysql regex flags"
            })
        );
    }

    #[test]
    fn mysql_rejects_dotall_flag() {
        let result = build_regex("email=/a.b/s", SqlDialect::MySql);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "mysql regex flags"
            })
        );
    }

    #[test]
    fn mysql_rejects_extended_flag() {
        let result = build_regex("email=/a b/x", SqlDialect::MySql);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "mysql regex flags"
            })
        );
    }

    #[test]
    fn sqlite_rejects_case_insensitive_flag() {
        let result = build_regex("email=/admin/i", SqlDialect::Sqlite);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "sqlite regex"
            })
        );
    }

    #[test]
    fn sqlite_rejects_multiline_flag() {
        let result = build_regex("email=/admin/m", SqlDialect::Sqlite);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "sqlite regex"
            })
        );
    }

    #[test]
    fn sqlite_rejects_dotall_flag() {
        let result = build_regex("email=/a.b/s", SqlDialect::Sqlite);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "sqlite regex"
            })
        );
    }

    #[test]
    fn sqlite_rejects_extended_flag() {
        let result = build_regex("email=/a b/x", SqlDialect::Sqlite);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported {
                feature: "sqlite regex"
            })
        );
    }

    #[test]
    fn adapter_permission_precedes_dialect_flag_support() -> RqsResult<()> {
        let query = parse("email=/a.b/s", &regex_catalog()?)?;
        let result = SqlxAdapter::new(SqlDialect::Postgres).build(&query);

        assert_eq!(
            result,
            Err(RqsError::AdapterUnsupported { feature: "regex" })
        );
        Ok(())
    }

    #[test]
    fn postgres_case_insensitive_pattern_remains_bound() -> RqsResult<()> {
        let parts = build_regex("email=/^admin' OR 1=1--$/i", SqlDialect::Postgres)?;

        assert_eq!(
            parts.binds,
            vec![RqsValue::Text("^admin' OR 1=1--$".to_owned())]
        );
        Ok(())
    }

    #[test]
    fn postgres_case_insensitive_sql_contains_only_a_placeholder() -> RqsResult<()> {
        let parts = build_regex("email=/^admin' OR 1=1--$/i", SqlDialect::Postgres)?;

        assert_eq!(
            parts.where_clause.as_deref(),
            Some("\"users\".\"email\" ~* $1")
        );
        Ok(())
    }
}
