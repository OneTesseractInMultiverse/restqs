#![allow(missing_docs)]

#[path = "support/in_memory.rs"]
mod in_memory;

use in_memory::{Person, eligible_names};
use restqs::{FieldCatalog, RqsResult, parse};

fn main() -> RqsResult<()> {
    let catalog = FieldCatalog::new().allow_integer("age")?;
    let query = parse("age>=18", &catalog)?;
    let people = [
        Person {
            name: "Alex",
            age: 17,
        },
        Person {
            name: "Sam",
            age: 21,
        },
    ];
    let names = eligible_names(&query, &people)?;
    println!("{names:?}");
    Ok(())
}
