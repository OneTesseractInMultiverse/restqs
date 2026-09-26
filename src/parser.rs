//! RQS parser orchestration.

mod filter_policy;
mod parameter_policy;

use std::collections::BTreeSet;

use self::{
    filter_policy::{FilterKey, filter_key, validate_new_filter},
    parameter_policy::{
        Parameter, classify_parameter, control_key, validate_control_size, validate_new_control,
    },
};
use crate::{
    FieldCatalog, FieldRef, Filter, FilterOp, ParserLimits, Projection, RqsError, RqsQuery,
    RqsResult, SortTerm, catalog::validate_public_name, filter::build_value_filter,
    parameter::decode_parameters, sort::split_sort_token,
};

/// Parser configuration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParserConfig {
    /// Input budgets applied during decoding and typed-plan construction.
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

/// Parse a raw URL query component with default input budgets.
///
/// Pass the text after `?`, without the leading delimiter or prior URL decoding.
/// Empty input returns an empty plan. The catalog must come from trusted application
/// configuration; it authorizes fields, not tenants or individual database rows.
///
/// # Errors
///
/// Returns [`RqsError`] for malformed encoding/syntax, unauthorized fields,
/// incompatible values/operators, duplicate predicates/controls, or exceeded
/// [`ParserLimits`]. See [`Parser::parse`] for decoding and validation order.
pub fn parse(query: &str, catalog: &FieldCatalog) -> RqsResult<RqsQuery> {
    Parser::new(catalog).parse(query)
}

/// RQS parser bound to one allowlist catalog.
///
/// The parser borrows trusted configuration and keeps no per-request state.
/// Returned plans own their field metadata and outlive the parser and catalog.
/// The parser performs no I/O and never executes a query.
pub struct Parser<'a> {
    /// Borrowed authorization catalog; resulting plans own their resolved field metadata.
    catalog: &'a FieldCatalog,
    /// Parser input-budget configuration, independent of repository execution budgets.
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

    /// Parse a raw URL query component into an owned database-neutral plan.
    ///
    /// Split on raw ampersands before decoding each component exactly once.
    /// Empty components are ignored, `+` decodes to a space, and percent escapes
    /// must form valid UTF-8. Encode a literal plus as `%2B`.
    /// Filters, sort terms, and projected fields preserve request order.
    ///
    /// Filter operators are recognized at the field boundary after decoding.
    /// Later comparison characters remain part of the value.
    /// Each of `sort`, `fields`, `limit`, and `skip` may appear only once after
    /// decoding, including empty values. Repeats return [`RqsError::DuplicateControl`]
    /// after the value-size check and before interpreting the repeated value.
    ///
    /// # Errors
    ///
    /// Raw size and parameter-count limits precede interpretation. Field syntax
    /// is checked before catalog lookup. Typed filter validation precedes duplicate
    /// field/operator rejection. The first encountered error aborts the whole parse;
    /// no partial plan is returned. Omitted pagination remains unspecified and must
    /// be bounded by the executing application.
    pub fn parse(&self, query: &str) -> RqsResult<RqsQuery> {
        let parameters = decode_parameters(query, self.config.limits())?;
        let mut output = RqsQuery::new();
        let mut seen_filters = BTreeSet::new();
        let mut seen_controls = BTreeSet::new();
        for parameter in parameters {
            self.apply_parameter(
                &parameter,
                &mut output,
                &mut seen_filters,
                &mut seen_controls,
            )?;
        }
        Ok(output)
    }

    /// Classify one decoded component, validate control size and uniqueness, then dispatch its
    /// plan update. Record control ownership only after the update succeeds.
    fn apply_parameter(
        &self,
        parameter: &str,
        output: &mut RqsQuery,
        seen_filters: &mut BTreeSet<FilterKey>,
        seen_controls: &mut BTreeSet<&'static str>,
    ) -> RqsResult<()> {
        let parameter = classify_parameter(parameter)?;
        validate_control_size(parameter, self.config.limits().max_value_bytes)?;
        let key = control_key(parameter);
        validate_new_control(key, seen_controls)?;
        match parameter {
            Parameter::Sort(value) => self.apply_sort(value, output),
            Parameter::Projection(value) => self.apply_projection(value, output),
            Parameter::Limit(value) => self.apply_limit(value, output),
            Parameter::Offset(value) => self.apply_offset(value, output),
            Parameter::Filter(value) => self.apply_filter(value, output, seen_filters),
        }?;
        seen_controls.extend(key);
        Ok(())
    }

    /// Parse and validate a filter, reject an existing normalized identity, then append it in
    /// input order.
    fn apply_filter(
        &self,
        parameter: &str,
        output: &mut RqsQuery,
        seen_filters: &mut BTreeSet<FilterKey>,
    ) -> RqsResult<()> {
        let filter = self.parse_filter(parameter)?;
        let key = filter_key(&filter);
        validate_new_filter(&key, seen_filters)?;
        seen_filters.insert(key);
        output.push_filter(filter);
        Ok(())
    }

    /// Resolve a comma-separated sort list in order; an empty control produces no sort terms.
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

    /// Resolve one signed sort token against the catalog and construct its logical sort node.
    fn parse_sort_term(&self, item: &str) -> RqsResult<SortTerm> {
        let (field_name, direction) = split_sort_token(item);
        let field = self.resolve_field(field_name)?;
        Ok(SortTerm::new(field, direction))
    }

    /// Resolve requested fields in input order; an empty control leaves projection unspecified.
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

    /// Parse an unsigned limit, enforce the configured maximum, and store the explicit value.
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

    /// Parse and store an unsigned offset without imposing an application execution budget.
    fn apply_offset(&self, value: &str, output: &mut RqsQuery) -> RqsResult<()> {
        let offset = parse_pagination_value("skip", value)?;
        output.pagination_mut().set_offset(offset);
        Ok(())
    }

    /// Resolve existence or comparison syntax into an authorized, typed filter node.
    fn parse_filter(&self, parameter: &str) -> RqsResult<Filter> {
        let (field_name, op, value) = split_filter(parameter)?;
        let field = self.resolve_field(field_name)?;
        match op {
            FilterOp::Exists | FilterOp::NotExists => Ok(Filter::new(field, op, None)),
            _ => build_value_filter(field, op, value, self.config.limits()),
        }
    }

    /// Validate public-name syntax and clone authorized metadata; reject fields absent from the
    /// catalog.
    fn resolve_field(&self, field_name: &str) -> RqsResult<FieldRef> {
        validate_public_name(field_name)?;
        self.catalog
            .get(field_name)
            .map(crate::Field::to_ref)
            .ok_or_else(|| RqsError::UnknownField {
                field: field_name.to_owned(),
            })
    }
}

/// Split at the first comparison character and consume the longest supported operator there;
/// preserve later operator characters as value data.
fn split_filter(parameter: &str) -> RqsResult<(&str, FilterOp, &str)> {
    if let Some(field) = parameter.strip_prefix('!') {
        return Ok((field, FilterOp::NotExists, ""));
    }

    let Some(boundary) = parameter.find(['!', '>', '<', '=']) else {
        return Ok((parameter, FilterOp::Exists, ""));
    };
    let (field, expression) = parameter.split_at(boundary);
    if field.is_empty() {
        return Err(RqsError::InvalidOperator);
    }

    for (token, op) in [
        (">=", FilterOp::Gte),
        ("<=", FilterOp::Lte),
        ("!=", FilterOp::Ne),
        (">", FilterOp::Gt),
        ("<", FilterOp::Lt),
        ("=", FilterOp::Eq),
    ] {
        if let Some(value) = expression.strip_prefix(token) {
            return Ok((field, op, value));
        }
    }

    Err(RqsError::InvalidOperator)
}

/// Parse an unsigned decimal control, treating empty input as zero and distinguishing negative
/// values from malformed numbers.
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
