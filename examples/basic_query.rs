#![allow(missing_docs)]

use restqs::{FieldCatalog, RqsError, parse};

fn main() -> Result<(), RqsError> {
    let catalog = FieldCatalog::new()
        .allow_integer("age", "users.age")?
        .allow_text("status", "users.status")?;

    let query = parse("age>=18&status=in(active,pending)&limit=25", &catalog)?;

    println!("{query:#?}");
    Ok(())
}
