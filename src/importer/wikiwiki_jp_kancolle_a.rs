//! Module for importers for https://wikiwiki.jp/kancolle-a/ resources

// For refactoring reasons, import everything.
// TODO: Import less stuff?

pub mod serde_table;

mod kansen_table;
pub use kansen_table::*;
