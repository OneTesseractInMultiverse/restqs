//! REST Query Syntax parser for REST API filtering plans.
//!
//! RestQS parses RQS query strings into typed plans. The core parser does not
//! emit SQL. Database and ORM translation lives in adapters, so application
//! code can keep parsing, authorization, and persistence concerns separate.
//!
//! A service starts with a [`FieldCatalog`]. The catalog maps public query
//! fields to trusted database columns and value kinds. The parser checks every
//! requested field against that catalog and returns [`RqsQuery`].
//!
//! # Basic Parsing
//!
//! ```
//! use restqs::{FieldCatalog, parse};
//!
//! let catalog = FieldCatalog::new()
//!     .allow_integer("age", "users.age")?
//!     .allow_text("status", "users.status")?;
//!
//! let query = parse("age>=18&status=in(active,pending)", &catalog)?;
//!
//! assert_eq!(query.filters().len(), 2);
//! # Ok::<(), restqs::RqsError>(())
//! ```
//!
//! # Safe Defaults
//!
//! RestQS treats query strings as untrusted input. User field names never
//! become database identifiers. Adapters receive trusted catalog columns and
//! typed [`RqsValue`] values.
//!
//! The default [`ParserLimits`] cap raw query length at 8 KiB, parameter count
//! at 128, decoded value length at 2 KiB, list item count at 100, and `limit=`
//! at 100. The value byte limit covers filter values and `sort`, `fields`,
//! `limit`, and `skip` controls, including wrappers, regex delimiters and flags,
//! and complete lists. Existence filters have no value to limit.
//!
//! # Adapter Boundary
//!
//! The `sqlx` feature exposes SQLx-oriented fragment generation through
//! [`adapters::sqlx`]. The adapter returns SQL fragments and bind values. The
//! host repository still owns the final SQL statement, connection, transaction,
//! and result mapping.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::todo)]
#![deny(clippy::unwrap_used)]
#![deny(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod adapters;
mod catalog;
mod error;
mod filter;
mod identifier;
mod limits;
mod pagination;
mod parameter;
mod parser;
mod projection;
mod query;
mod sort;
mod value;

pub use catalog::{Field, FieldCatalog, FieldRef, ValueKind};
pub use error::{RqsError, RqsResult};
pub use filter::{Filter, FilterOp, RegexLiteral};
pub use limits::ParserLimits;
pub use pagination::Pagination;
pub use parser::{Parser, ParserConfig, parse};
pub use projection::Projection;
pub use query::RqsQuery;
pub use sort::{SortDirection, SortTerm};
pub use value::RqsValue;
