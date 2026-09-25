//! Sort-token interpretation and sort terms in an RQS plan.

use crate::FieldRef;

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Split one decoded sort token without resolving or validating its field.
pub(crate) fn split_sort_token(token: &str) -> (&str, SortDirection) {
    if let Some(field) = token.strip_prefix('-') {
        return (field, SortDirection::Desc);
    }
    if let Some(field) = token.strip_prefix('+') {
        return (field, SortDirection::Asc);
    }
    (token, SortDirection::Asc)
}

/// One sort term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortTerm {
    field: FieldRef,
    direction: SortDirection,
}

impl SortTerm {
    /// Create a sort term.
    #[must_use]
    pub fn new(field: FieldRef, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Return the field.
    #[must_use]
    pub fn field(&self) -> &FieldRef {
        &self.field
    }

    /// Return the direction.
    #[must_use]
    pub fn direction(&self) -> SortDirection {
        self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::{SortDirection, split_sort_token};

    #[test]
    fn a_bare_token_keeps_its_field_and_defaults_to_ascending() {
        assert_eq!(
            split_sort_token("profile.name"),
            ("profile.name", SortDirection::Asc)
        );
    }

    #[test]
    fn a_descending_token_returns_its_unprefixed_field() {
        assert_eq!(split_sort_token("-age"), ("age", SortDirection::Desc));
    }

    #[test]
    fn an_ascending_token_returns_its_unprefixed_field() {
        assert_eq!(split_sort_token("+age"), ("age", SortDirection::Asc));
    }

    #[test]
    fn an_empty_token_is_left_for_field_validation() {
        assert_eq!(split_sort_token(""), ("", SortDirection::Asc));
    }

    #[test]
    fn only_the_first_prefix_is_consumed() {
        assert_eq!(split_sort_token("+-age"), ("-age", SortDirection::Asc));
    }

    #[test]
    fn a_multibyte_first_character_is_preserved() {
        assert_eq!(split_sort_token("éclair"), ("éclair", SortDirection::Asc));
    }
}
