//! Pure classification and size policy for decoded parameters.

use crate::{RqsError, RqsResult, limits::validate_value_size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Parameter<'a> {
    Sort(&'a str),
    Projection(&'a str),
    Limit(&'a str),
    Offset(&'a str),
    Filter(&'a str),
}

pub(super) fn classify_parameter(parameter: &str) -> RqsResult<Parameter<'_>> {
    if parameter.starts_with("$text=") {
        return Err(RqsError::TextSearchUnsupported);
    }
    if let Some(value) = parameter.strip_prefix("sort=") {
        return Ok(Parameter::Sort(value));
    }
    if let Some(value) = parameter.strip_prefix("fields=") {
        return Ok(Parameter::Projection(value));
    }
    if let Some(value) = parameter.strip_prefix("limit=") {
        return Ok(Parameter::Limit(value));
    }
    if let Some(value) = parameter.strip_prefix("skip=") {
        return Ok(Parameter::Offset(value));
    }
    Ok(Parameter::Filter(parameter))
}

pub(super) fn validate_control_size(parameter: Parameter<'_>, max_bytes: usize) -> RqsResult<()> {
    match parameter {
        Parameter::Sort(value) => validate_value_size("sort", value, max_bytes),
        Parameter::Projection(value) => validate_value_size("fields", value, max_bytes),
        Parameter::Limit(value) => validate_value_size("limit", value, max_bytes),
        Parameter::Offset(value) => validate_value_size("skip", value, max_bytes),
        // Filters validate their values after syntax checks and field resolution.
        Parameter::Filter(_) => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_classification_preserves_the_decoded_value() {
        assert_eq!(
            classify_parameter("sort=+age,-status"),
            Ok(Parameter::Sort("+age,-status"))
        );
    }

    #[test]
    fn projection_classification_preserves_an_empty_value() {
        assert_eq!(classify_parameter("fields="), Ok(Parameter::Projection("")));
    }

    #[test]
    fn limit_classification_leaves_numeric_validation_to_the_parser() {
        assert_eq!(classify_parameter("limit=bad"), Ok(Parameter::Limit("bad")));
    }

    #[test]
    fn offset_classification_leaves_negative_validation_to_the_parser() {
        assert_eq!(classify_parameter("skip=-1"), Ok(Parameter::Offset("-1")));
    }

    #[test]
    fn filter_classification_preserves_the_whole_expression() {
        assert_eq!(
            classify_parameter("age>=18"),
            Ok(Parameter::Filter("age>=18"))
        );
    }

    #[test]
    fn text_search_classification_rejects_even_an_empty_value() {
        assert_eq!(
            classify_parameter("$text="),
            Err(RqsError::TextSearchUnsupported)
        );
    }

    #[test]
    fn control_name_without_equals_remains_a_filter() {
        assert_eq!(classify_parameter("sort"), Ok(Parameter::Filter("sort")));
    }

    #[test]
    fn similar_field_name_is_not_a_control() {
        assert_eq!(
            classify_parameter("sorting=age"),
            Ok(Parameter::Filter("sorting=age"))
        );
    }

    #[test]
    fn control_matching_remains_case_sensitive() {
        assert_eq!(
            classify_parameter("Sort=age"),
            Ok(Parameter::Filter("Sort=age"))
        );
    }

    #[test]
    fn control_classification_does_not_decode_input_a_second_time() {
        assert_eq!(
            classify_parameter("sort%3Dage"),
            Ok(Parameter::Filter("sort%3Dage"))
        );
    }
}
