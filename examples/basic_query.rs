//! Executable basic query example; failures are returned to the caller.

use restqs::{FieldCatalog, RqsError, parse};

/// Parse a bounded catalog-authorized query and print its typed plan; propagate configuration and
/// input errors.
fn main() -> Result<(), RqsError> {
    let catalog = FieldCatalog::new()
        .allow_integer("age")?
        .allow_text("status")?;

    let query = parse("age>=18&status=in(active,pending)&limit=25", &catalog)?;

    println!("{query:#?}");
    Ok(())
}
