//! Shared identifier syntax checks.

/// Require one or more valid ASCII identifier segments separated by single dots.
pub(crate) fn is_dotted_identifier(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_identifier)
}

/// Require an ASCII letter or underscore first, followed only by ASCII letters, digits, or
/// underscores.
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
