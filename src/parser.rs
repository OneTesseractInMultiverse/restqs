//! RQS parser orchestration.

use std::collections::BTreeSet;

use crate::{
    FieldCatalog, FieldRef, Filter, FilterOp, ParserLimits, Projection, RqsError, RqsQuery,
    RqsResult, SortDirection, SortTerm, filter::build_value_filter, parameter::decode_parameters,
};

/// Parser configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParserConfig {
    limits: ParserLimits,
}

impl ParserConfig {
    /// Create a config with custom limits.
    #[must_use]
    pub fn with_limits(limits: ParserLimits) -> Self {
        Self { limits }
    }

    /// Return parser limits.
    #[must_use]
    pub fn limits(&self) -> ParserLimits {
        self.limits
    }
}

/// Parse an RQS string with default config.
pub fn parse(query: &str, catalog: &FieldCatalog) -> RqsResult<RqsQuery> {
    Parser::new(catalog).parse(query)
}

/// RQS parser bound to one allowlist catalog.
pub struct Parser<'a> {
    catalog: &'a FieldCatalog,
    config: ParserConfig,
}

impl<'a> Parser<'a> {
    /// Create a parser with default config.
    #[must_use]
    pub fn new(catalog: &'a FieldCatalog) -> Self {
        Self {
            catalog,
            config: ParserConfig::default(),
        }
    }

    /// Create a parser with explicit config.
    #[must_use]
    pub fn with_config(catalog: &'a FieldCatalog, config: ParserConfig) -> Self {
        Self { catalog, config }
    }

    /// Parse an RQS string into a database-neutral plan.
    pub fn parse(&self, query: &str) -> RqsResult<RqsQuery> {
        let parameters = decode_parameters(query, self.config.limits())?;
        let mut output = RqsQuery::new();
        let mut seen_filters = BTreeSet::new();
        for parameter in parameters {
            self.apply_parameter(&parameter, &mut output, &mut seen_filters)?;
        }
        Ok(output)
    }

    fn apply_parameter(
        &self,
        parameter: &str,
        output: &mut RqsQuery,
        seen_filters: &mut BTreeSet<(String, &'static str)>,
    ) -> RqsResult<()> {
        if parameter.starts_with("$text=") {
            return Err(RqsError::TextSearchUnsupported);
        }
        if let Some(value) = parameter.strip_prefix("sort=") {
            return self.apply_sort(value, output);
        }
        if let Some(value) = parameter.strip_prefix("fields=") {
            return self.apply_projection(value, output);
        }
        if let Some(value) = parameter.strip_prefix("limit=") {
            return self.apply_limit(value, output);
        }
        if let Some(value) = parameter.strip_prefix("skip=") {
            return self.apply_offset(value, output);
        }
        let filter = self.parse_filter(parameter)?;
        let key = (filter.field().public_name().to_owned(), filter.op().token());
        if !seen_filters.insert(key.clone()) {
            return Err(RqsError::DuplicateFilter {
                field: key.0,
                operator: key.1,
            });
        }
        output.push_filter(filter);
        Ok(())
    }

    fn apply_sort(&self, value: &str, output: &mut RqsQuery) -> RqsResult<()> {
        if value.is_empty() {
            output.set_sort(Vec::new());
            return Ok(());
        }

        let sort = value
            .split(',')
            .map(|item| self.parse_sort_term(item))
            .collect::<RqsResult<Vec<_>>>()?;
        output.set_sort(sort);
        Ok(())
    }

    fn parse_sort_term(&self, item: &str) -> RqsResult<SortTerm> {
        let (direction, field_name) = match item.as_bytes().first() {
            Some(b'-') => (SortDirection::Desc, &item[1..]),
            Some(b'+') => (SortDirection::Asc, &item[1..]),
            _ => (SortDirection::Asc, item),
        };
        Ok(SortTerm::new(self.resolve_field(field_name)?, direction))
    }

    fn apply_projection(&self, value: &str, output: &mut RqsQuery) -> RqsResult<()> {
        if value.is_empty() {
            output.set_projection(Projection::default());
            return Ok(());
        }

        let fields = value
            .split(',')
            .map(|field| self.resolve_field(field))
            .collect::<RqsResult<Vec<_>>>()?;
        output.set_projection(Projection::new(fields));
        Ok(())
    }

    fn apply_limit(&self, value: &str, output: &mut RqsQuery) -> RqsResult<()> {
        let limit = parse_pagination_value("limit", value)?;
        if limit > self.config.limits().max_limit {
            return Err(RqsError::LimitTooLarge {
                max_limit: self.config.limits().max_limit,
            });
        }
        output.pagination_mut().set_limit(limit);
        Ok(())
    }

    fn apply_offset(&self, value: &str, output: &mut RqsQuery) -> RqsResult<()> {
        let offset = parse_pagination_value("skip", value)?;
        output.pagination_mut().set_offset(offset);
        Ok(())
    }

    fn parse_filter(&self, parameter: &str) -> RqsResult<Filter> {
        let (field_name, op, value) = split_filter(parameter)?;
        let field = self.resolve_field(field_name)?;
        match op {
            FilterOp::Exists | FilterOp::NotExists => Ok(Filter::new(field, op, None)),
            _ => build_value_filter(field, op, value, self.config.limits()),
        }
    }

    fn resolve_field(&self, field_name: &str) -> RqsResult<FieldRef> {
        if field_name.is_empty() {
            return Err(RqsError::InvalidFieldName {
                field: field_name.to_owned(),
            });
        }
        self.catalog
            .get(field_name)
            .map(crate::Field::to_ref)
            .ok_or_else(|| RqsError::UnknownField {
                field: field_name.to_owned(),
            })
    }
}

fn split_filter(parameter: &str) -> RqsResult<(&str, FilterOp, &str)> {
    if let Some(field) = parameter.strip_prefix('!') {
        return Ok((field, FilterOp::NotExists, ""));
    }

    for (token, op) in [
        (">=", FilterOp::Gte),
        ("<=", FilterOp::Lte),
        ("!=", FilterOp::Ne),
        (">", FilterOp::Gt),
        ("<", FilterOp::Lt),
        ("=", FilterOp::Eq),
    ] {
        if let Some((field, value)) = parameter.split_once(token) {
            if field.is_empty() {
                return Err(RqsError::InvalidOperator);
            }
            return Ok((field, op, value));
        }
    }

    Ok((parameter, FilterOp::Exists, ""))
}

fn parse_pagination_value(parameter: &'static str, value: &str) -> RqsResult<u64> {
    if value.is_empty() {
        return Ok(0);
    }
    if value.starts_with('-') {
        return Err(RqsError::NegativePagination { parameter });
    }
    value
        .parse::<u64>()
        .map_err(|_| RqsError::InvalidPagination { parameter })
}
