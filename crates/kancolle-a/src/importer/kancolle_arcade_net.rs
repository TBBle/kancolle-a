//! Module for importers for https://kancolle-arcade.net/ac/api/ resources

// For refactoring reasons, import everything.
// TODO: Import less stuff?

mod tc_book;
pub use tc_book::*;

mod blueprint_list;
pub use blueprint_list::*;

mod character_list;
pub use character_list::*;

mod place;
pub use place::districts::*;
pub use place::places::*;

mod kekkonkakkokari;
pub use kekkonkakkokari::kanmusu_list::*;

#[cfg(not(target_family = "wasm"))]
mod api_client;
#[cfg(not(target_family = "wasm"))]
pub use api_client::*;
