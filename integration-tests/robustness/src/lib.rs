//! Pure contract checks shared by generated tests and fuzz targets.

use restqs::{
    Field, FieldCatalog, FieldRef, Filter, FilterOp, Parser, ParserConfig, ParserLimits, RqsQuery,
    RqsResult, RqsValue, ValueKind,
    adapters::sqlx::{SqlDialect, SqlxAdapter, SqlxColumnMap, SqlxQueryParts},
};

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

pub fn generous_limits() -> ParserLimits {
    ParserLimits {
        max_query_bytes: 65536,
        max_parameters: 256,
        max_value_bytes: 16384,
        max_list_items: 128,
        max_limit: 1000,
    }
}

pub fn limits_from_seed(seed: &[u8]) -> ParserLimits {
    ParserLimits {
        max_query_bytes: usize::from(seed.first().copied().unwrap_or(0)) * 16,
        max_parameters: usize::from(seed.get(1).copied().unwrap_or(0)) % 33,
        max_value_bytes: usize::from(seed.get(2).copied().unwrap_or(0)) * 4,
        max_list_items: usize::from(seed.get(3).copied().unwrap_or(0)) % 33,
        max_limit: u64::from(seed.get(4).copied().unwrap_or(0)),
    }
}

pub fn parse(raw: &str, limits: ParserLimits) -> RqsResult<RqsQuery> {
    let catalog = catalog()?;
    Parser::with_config(&catalog, ParserConfig::with_limits(limits)).parse(raw)
}

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

pub fn text_query(value: &str) -> String {
    format!("text={}", encode(&format!("str({value})")))
}

pub fn decoded_text_matches(value: &str) -> bool {
    let Ok(query) = parse(&text_query(value), generous_limits()) else {
        return false;
    };
    query.filters().first().and_then(Filter::value) == Some(&RqsValue::Text(value.to_owned()))
}

fn resolved_field(field: &FieldRef, catalog: &FieldCatalog) -> bool {
    catalog.get(field.public_name()).is_some_and(|allowed| {
        allowed.value_kind() == field.value_kind()
            && allowed.regex_allowed() == field.regex_allowed()
    })
}

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

pub fn operators_are_valid(query: &RqsQuery) -> bool {
    query.filters().iter().all(valid_filter)
}

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
