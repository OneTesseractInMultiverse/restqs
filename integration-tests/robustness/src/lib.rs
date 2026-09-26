//! Pure contract checks shared by generated tests and fuzz targets.

use restqs::{
    Field, FieldCatalog, FieldRef, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsQuery,
    RqsResult, RqsValue, ValueKind,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
};

/// Authorize all supported value kinds plus one opt-in regex field for generated contract checks.
pub fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_integer("id")?
        .allow_integer("age")?
        .allow_date("day")?
        .allow_datetime("created")?
        .allow_uuid("uid")?
        .allow_text("text")?
        .allow_boolean("flag")?
        .allow_float("score")?
        .allow(Field::new("name", ValueKind::Text)?.allow_regex())
}

/// Provide bounded input budgets large enough for generated encoded values and mixed plans.
pub fn generous_limits() -> ParserLimits {
    ParserLimits {
        max_query_bytes: 65536,
        max_parameters: 256,
        max_value_bytes: 16384,
        max_list_items: 128,
        max_limit: 1000,
    }
}

/// Derive small deterministic budgets from up to five bytes, using zero for absent bytes to
/// exercise boundaries.
pub fn limits_from_seed(seed: &[u8]) -> ParserLimits {
    ParserLimits {
        max_query_bytes: usize::from(seed.first().copied().unwrap_or(0)) * 16,
        max_parameters: usize::from(seed.get(1).copied().unwrap_or(0)) % 33,
        max_value_bytes: usize::from(seed.get(2).copied().unwrap_or(0)) * 4,
        max_list_items: usize::from(seed.get(3).copied().unwrap_or(0)) % 33,
        max_limit: u64::from(seed.get(4).copied().unwrap_or(0)),
    }
}

/// Parse generated input with the shared catalog and an explicit budget, preserving all parser
/// errors.
pub fn parse(raw: &str, limits: ParserLimits) -> RqsResult<RqsQuery> {
    let catalog = catalog()?;
    Parser::with_config(&catalog, ParserConfig::with_limits(limits)).parse(raw)
}

/// Percent-encode every UTF-8 byte so arbitrary Unicode and delimiters can be supplied as one
/// value.
pub fn encode(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    value
        .bytes()
        .flat_map(|byte| {
            [
                '%',
                char::from(HEX[usize::from(byte >> 4)]),
                char::from(HEX[usize::from(byte & 15)]),
            ]
        })
        .collect()
}

/// Wrap arbitrary text in str(...) before encoding, preventing null, list, or regex
/// reinterpretation.
pub fn text_query(value: &str) -> String {
    format!("text={}", encode(&format!("str({value})")))
}

/// Check that encoded, wrapped text parses to the exact original string under the fixture budget.
pub fn decoded_text_matches(value: &str) -> bool {
    let Ok(query) = parse(&text_query(value), generous_limits()) else {
        return false;
    };
    query.filters().first().and_then(Filter::value) == Some(&RqsValue::Text(value.to_owned()))
}

/// Compare a plan field's name, type, and regex permission with the original authorized
/// definition.
fn resolved_field(field: &FieldRef, catalog: &FieldCatalog) -> bool {
    catalog.get(field.public_name()).is_some_and(|allowed| {
        allowed.value_kind() == field.value_kind()
            && allowed.regex_allowed() == field.regex_allowed()
    })
}

/// Check that every filter, sort, and projection field preserves authorized catalog metadata.
pub fn fields_are_resolved(query: &RqsQuery) -> bool {
    let Ok(catalog) = catalog() else {
        return false;
    };
    query
        .filters()
        .iter()
        .map(Filter::field)
        .chain(query.sort().iter().map(|term| term.field()))
        .chain(query.projection().fields().iter())
        .all(|field| resolved_field(field, &catalog))
}

/// Check the operand variant against the catalog kind, allowing null and requiring finite floats.
fn scalar_matches(value: &RqsValue, kind: ValueKind) -> bool {
    match (value, kind) {
        (RqsValue::Null, _) => true,
        (RqsValue::Integer(_), ValueKind::Integer)
        | (RqsValue::Text(_), ValueKind::Text)
        | (RqsValue::Boolean(_), ValueKind::Boolean)
        | (RqsValue::Date(_), ValueKind::Date)
        | (RqsValue::DateTime(_), ValueKind::DateTime)
        | (RqsValue::Uuid(_), ValueKind::Uuid) => true,
        (RqsValue::Float(value), ValueKind::Float) => value.is_finite(),
        _ => false,
    }
}

/// Check operator/operand invariants, including regex permissions, unique flags, and homogeneous
/// membership lists.
fn valid_filter(filter: &Filter) -> bool {
    match filter.op() {
        FilterOp::Exists | FilterOp::NotExists => {
            filter.value().is_none() && filter.regex_literal().is_none()
        }
        FilterOp::Regex => {
            filter.value().is_none()
                && filter.field().regex_allowed()
                && filter.regex_literal().is_some_and(|regex| {
                    let mut flags = std::collections::BTreeSet::new();
                    regex
                        .flags()
                        .chars()
                        .all(|flag| "imsx".contains(flag) && flags.insert(flag))
                })
        }
        FilterOp::In | FilterOp::NotIn => {
            filter.regex_literal().is_none()
                && matches!(filter.value(), Some(RqsValue::List(values)) if values.iter().all(|v| scalar_matches(v, filter.field().value_kind())))
        }
        _ => {
            filter.regex_literal().is_none()
                && filter
                    .value()
                    .is_some_and(|value| scalar_matches(value, filter.field().value_kind()))
        }
    }
}

/// Check that every successfully parsed filter satisfies the normalized operator/operand
/// contract.
pub fn operators_are_valid(query: &RqsQuery) -> bool {
    query.filters().iter().all(valid_filter)
}

/// Translate a generated plan with explicit records-table mappings and regex opt-in for the
/// selected dialect.
pub fn build(query: &RqsQuery, dialect: SqlDialect, first: usize) -> RqsResult<SqlxQueryParts> {
    let columns = SqlxColumnMap::new()
        .map("id", "records.id")?
        .map("age", "records.age")?
        .map("text", "records.text")?
        .map("name", "records.name")?
        .map("day", "records.day")?
        .map("created", "records.created")?
        .map("uid", "records.uid")?
        .map("flag", "records.flag")?
        .map("score", "records.score")?;
    SqlxAdapter::new(dialect, columns)
        .allow_regex()
        .build_with_bind_start(query, first)
}

/// Compute the bind oracle in input order, flattening lists, binding regex patterns, and skipping
/// null/existence predicates.
pub fn expected_binds(query: &RqsQuery) -> Vec<RqsValue> {
    query
        .filters()
        .iter()
        .flat_map(|filter| {
            if let Some(regex) = filter.regex_literal() {
                return vec![RqsValue::Text(regex.pattern().to_owned())];
            }
            match filter.value() {
                Some(RqsValue::List(values)) => values.clone(),
                Some(RqsValue::Null) | None => Vec::new(),
                Some(value) => vec![value.clone()],
            }
        })
        .collect()
}

/// Compare SQL placeholder order with generated binds; callers supply bounded positions that
/// cannot overflow usize.
pub fn placeholders_match(parts: &SqlxQueryParts, dialect: SqlDialect, first: usize) -> bool {
    let sql = parts.where_clause.as_deref().unwrap_or("");
    match dialect {
        SqlDialect::Postgres => {
            let positions: Vec<_> = sql
                .split('$')
                .skip(1)
                .map(|tail| {
                    tail.chars()
                        .take_while(char::is_ascii_digit)
                        .collect::<String>()
                        .parse::<usize>()
                })
                .collect();
            positions
                == (first..first + parts.binds.len())
                    .map(Ok)
                    .collect::<Vec<_>>()
        }
        _ => sql.matches('?').count() == parts.binds.len(),
    }
}

/// Check all dialects for ordered binds and matching placeholders, accepting explicit
/// unsupported-feature errors.
pub fn adapter_contract(query: &RqsQuery, first: usize) -> bool {
    [SqlDialect::Postgres, SqlDialect::MySql, SqlDialect::Sqlite]
        .into_iter()
        .all(|dialect| match build(query, dialect, first) {
            Ok(parts) => {
                parts.binds == expected_binds(query) && placeholders_match(&parts, dialect, first)
            }
            Err(restqs::RqsError::AdapterUnsupported { .. }) => true,
            Err(_) => false,
        })
}

/// Compare each dialect's predicate SQL with a fixed reference while changing only the bound text
/// value.
pub fn value_cannot_change_sql(value: &str) -> bool {
    let Ok(query) = parse(&text_query(value), generous_limits()) else {
        return false;
    };
    let Ok(reference) = parse(&text_query("reference"), generous_limits()) else {
        return false;
    };
    [SqlDialect::Postgres, SqlDialect::MySql, SqlDialect::Sqlite]
        .into_iter()
        .all(
            |dialect| match (build(&query, dialect, 1), build(&reference, dialect, 1)) {
                (Ok(actual), Ok(reference)) => actual.where_clause == reference.where_clause,
                _ => false,
            },
        )
}

/// Construct a mixed text, list, scalar, null, ordering, and projection query for cross-feature
/// properties.
pub fn mixed_query(value: &str, values: &[i64], age: i64) -> String {
    let list = values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}&id=in({list})&age>={age}&flag=null&sort=-id&fields=id,text",
        text_query(value)
    )
}

/// Check successful SQL translation and value/SQL separation for one bounded input.
pub fn adapter_input_contract(raw: &str, first: usize) -> bool {
    let translation = match parse(raw, generous_limits()) {
        Ok(query) => adapter_contract(&query, first),
        Err(_) => true,
    };
    translation && value_cannot_change_sql(raw)
}
