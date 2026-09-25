#![allow(missing_docs)]

use restqs::{FieldCatalog, RqsError, RqsResult, SortDirection, parse};

fn catalog() -> RqsResult<FieldCatalog> {
    FieldCatalog::new()
        .allow_text("status")?
        .allow_integer("age")?
        .allow_text("profile.name")
}

#[test]
fn mixed_sort_terms_preserve_field_identity_direction_and_order() -> RqsResult<()> {
    let query = parse("sort=status,-age,%2Bprofile.name", &catalog()?)?;
    let terms = query
        .sort()
        .iter()
        .map(|term| (term.field().public_name(), term.direction()))
        .collect::<Vec<_>>();

    assert_eq!(
        terms,
        vec![
            ("status", SortDirection::Asc),
            ("age", SortDirection::Desc),
            ("profile.name", SortDirection::Asc),
        ]
    );
    Ok(())
}

#[test]
fn descending_prefix_without_a_field_is_invalid() -> RqsResult<()> {
    let result = parse("sort=-", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidFieldName {
            field: String::new()
        })
    );
    Ok(())
}

#[test]
fn encoded_ascending_prefix_without_a_field_is_invalid() -> RqsResult<()> {
    let result = parse("sort=%2B", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidFieldName {
            field: String::new()
        })
    );
    Ok(())
}

#[test]
fn bare_sort_field_still_requires_catalog_authorization() -> RqsResult<()> {
    let result = parse("sort=secret", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::UnknownField {
            field: "secret".to_owned()
        })
    );
    Ok(())
}

#[test]
fn descending_sort_field_still_requires_catalog_authorization() -> RqsResult<()> {
    let result = parse("sort=-secret", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::UnknownField {
            field: "secret".to_owned()
        })
    );
    Ok(())
}

#[test]
fn encoded_ascending_sort_field_still_requires_catalog_authorization() -> RqsResult<()> {
    let result = parse("sort=%2Bsecret", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::UnknownField {
            field: "secret".to_owned()
        })
    );
    Ok(())
}

#[test]
fn repeated_sort_prefix_is_not_stripped_twice() -> RqsResult<()> {
    let result = parse("sort=--age", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidFieldName {
            field: "-age".to_owned()
        })
    );
    Ok(())
}

#[test]
fn a_raw_plus_keeps_form_decoding_semantics() -> RqsResult<()> {
    let result = parse("sort=+age", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidFieldName {
            field: " age".to_owned()
        })
    );
    Ok(())
}

#[test]
fn multibyte_field_after_a_sort_prefix_is_rejected_without_panicking() -> RqsResult<()> {
    let result = parse("sort=-%C3%A9", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidFieldName {
            field: "é".to_owned()
        })
    );
    Ok(())
}

#[test]
fn a_leading_empty_sort_term_is_invalid() -> RqsResult<()> {
    let result = parse("sort=,age", &catalog()?);

    assert_eq!(
        result,
        Err(RqsError::InvalidFieldName {
            field: String::new()
        })
    );
    Ok(())
}
