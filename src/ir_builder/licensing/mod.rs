//! SPDX License Management Module
//!
//! This module handles SPDX license identifier processing for Solidity files.
//! Dove enforces a single license policy: every formatted file receives the
//! canonical license header, replacing whatever license was present before.

#[cfg(test)]
mod tests;

/// The license identifier applied to every formatted file
const LICENSE: &str = "LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only";

/// Generate the correct SPDX license header
///
/// Any license already present in the source is discarded; the canonical
/// license is always applied.
///
/// # Returns
///
/// A properly formatted SPDX license header comment
pub fn generate_spdx_header() -> String {
    format!("// SPDX-License-Identifier: {}", LICENSE)
}
