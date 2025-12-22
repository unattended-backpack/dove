//! Collection pass for the two-pass formatting system.
//!
//! This module implements the first pass of the formatter which collects and organizes
//! source unit elements with their associated comments. The collect_source_unit function
//! is the main entry point that transforms raw solang-parser AST into our intermediate
//! representation (IR) defined in the model module.

// Re-export the main entry point
pub use source_unit::collect_source_unit;

// Re-export commonly used utilities
pub use utils::{get_loc_end, get_loc_range, get_loc_start, group_comments_by_blank_lines};

pub mod model;

mod comments;
mod contracts;
mod functions;
mod source_unit;
mod statements;
mod types;
mod utils;
mod yul;

// Re-export test modules
#[cfg(test)]
pub mod test_builder;

#[cfg(test)]
pub mod tests;
