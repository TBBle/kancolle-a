//! Module implementing a Serde (De)serialiser for wikiwiki.jp tables
//! SUPER WIP and incomplete.

// Mostly cadged from https://serde.rs/data-format.html and looking at serde_json

pub mod error;
pub use error::{Error, Result};

pub mod de;
pub use de::{from_str, Deserializer};

/*
from_bytes, from_reader, to_bytes, to_string, to_writer, Serializer,
*/

#[cfg(test)]
mod tests;
