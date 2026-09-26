#![allow(missing_docs)]

#[path = "../examples/support/in_memory.rs"]
mod in_memory;

use in_memory::{Person, eligible_names};
use restqs::{Field, FieldCatalog, RqsResult, RqsValue, ValueKind, parse};

#[test]
fn logical_catalog_produces_typed_values_without_storage_metadata() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("age>=18", &catalog)?;

    assert_eq!(query.filters()[0].value(), Some(&RqsValue::Integer(18)));
    Ok(())
}

#[test]
fn logical_field_reference_preserves_name() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("profile.status")?;
    let query = parse("profile.status=active", &catalog)?;

    assert_eq!(query.filters()[0].field().public_name(), "profile.status");
    Ok(())
}

#[test]
fn logical_field_reference_preserves_value_kind() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("age>=18", &catalog)?;

    assert_eq!(query.filters()[0].field().value_kind(), ValueKind::Integer);
    Ok(())
}

#[test]
fn logical_field_reference_preserves_regex_permission() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow(Field::new("name", ValueKind::Text)?.allow_regex())?;
    let query = parse("name=/sam/i", &catalog)?;

    assert!(query.filters()[0].field().regex_allowed());
    Ok(())
}

#[test]
fn logical_sort_keeps_public_identity() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("sort=-age", &catalog)?;

    assert_eq!(query.sort()[0].field().public_name(), "age");
    Ok(())
}

#[test]
fn logical_projection_keeps_public_identity() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("fields=age", &catalog)?;

    assert_eq!(query.projection().fields()[0].public_name(), "age");
    Ok(())
}

#[test]
fn in_memory_consumer_uses_the_authorized_typed_plan() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("age>=18", &catalog)?;
    let people = [
        Person {
            name: "Alex",
            age: 17,
        },
        Person {
            name: "Sam",
            age: 18,
        },
    ];

    assert_eq!(eligible_names(&query, &people)?, vec!["Sam"]);
    Ok(())
}

#[test]
fn in_memory_consumer_rejects_unsupported_operator() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("age=18", &catalog)?;

    assert!(eligible_names(&query, &[]).is_err());
    Ok(())
}

#[test]
fn in_memory_consumer_rejects_non_integer_value() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_text("age")?;
    let query = parse("age>=adult", &catalog)?;

    assert!(eligible_names(&query, &[]).is_err());
    Ok(())
}

#[test]
fn in_memory_consumer_rejects_pagination() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("age>=18&limit=1", &catalog)?;

    assert!(eligible_names(&query, &[]).is_err());
    Ok(())
}
