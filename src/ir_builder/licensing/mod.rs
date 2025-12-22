//! SPDX License Management Module
//!
//! This module handles all SPDX license identifier processing for Solidity files,
//! including detection of existing licenses, intelligent merging with default
//! licenses, and generation of proper SPDX header comments.
//!
//! Key responsibilities:
//! - Generate SPDX license headers with intelligent merging
//! - Detect existing SPDX license comments in source files
//! - Parse license identifiers from comment text
//! - Provide compliance utilities for legal requirements

#[cfg(test)]
mod tests;

use solang_parser::pt::Comment;

/// The default license identifier used when no existing license is found
const DEFAULT_LICENSE: &str = "LicenseRef-VPL WITH AGPL-3.0-only";

/// Generate the correct SPDX license header
///
/// This function intelligently merges existing license identifiers with the default
/// license to ensure compliance while preserving existing licensing terms.
///
/// # Arguments
///
/// * `comments` - All comments from the source file to scan for existing licenses
///
/// # Returns
///
/// A properly formatted SPDX license header comment
///
/// # License Merging Logic
///
/// - If no existing license: Use default license
/// - If existing license matches default: Use default as-is
/// - If existing license contains default: Use existing unchanged
/// - Otherwise: Combine using "AND" operator per SPDX specification
pub fn generate_spdx_header(comments: &[Comment]) -> String {
    // Look for existing SPDX header in comments
    if let Some(existing_license) = find_existing_spdx_license(comments) {
        if existing_license == DEFAULT_LICENSE {
            // Already has the correct license
            format!("// SPDX-License-Identifier: {}", DEFAULT_LICENSE)
        } else if existing_license.contains(DEFAULT_LICENSE) {
            // Default license already included in existing license, use as-is
            format!("// SPDX-License-Identifier: {}", existing_license)
        } else {
            // Combine with existing license using AND
            format!(
                "// SPDX-License-Identifier: {} AND ({})",
                existing_license, DEFAULT_LICENSE
            )
        }
    } else {
        // No existing license, use default
        format!("// SPDX-License-Identifier: {}", DEFAULT_LICENSE)
    }
}

/// Find existing SPDX license identifier in comments
///
/// Scans through all comments in the source file looking for SPDX license
/// identifier patterns and returns the first one found.
///
/// # Arguments
///
/// * `comments` - All comments from the source file to scan
///
/// # Returns
///
/// The license identifier string if found, None otherwise
pub fn find_existing_spdx_license(comments: &[Comment]) -> Option<String> {
    for comment in comments {
        let comment_text = match comment {
            Comment::Line(_, text) => text,
            Comment::Block(_, text) => text,
            Comment::DocLine(_, text) => text,
            Comment::DocBlock(_, text) => text,
        };

        // Check if this is an SPDX header comment
        if let Some(license) = extract_spdx_license(comment_text) {
            return Some(license);
        }
    }
    None
}

/// Extract license identifier from SPDX comment text
///
/// Parses a comment string to extract the license identifier portion
/// from an SPDX-License-Identifier comment.
///
/// # Arguments
///
/// * `comment_text` - The raw comment text to parse
///
/// # Returns
///
/// The license identifier string if the comment contains valid SPDX syntax
///
/// # Supported Formats
///
/// - `// SPDX-License-Identifier: MIT`
/// - `/* SPDX-License-Identifier: Apache-2.0 */`
/// - `SPDX-License-Identifier: GPL-3.0-or-later` (without comment markers)
pub fn extract_spdx_license(comment_text: &str) -> Option<String> {
    // Look for "SPDX-License-Identifier:" pattern
    let trimmed = comment_text.trim();

    // Handle both // and /* comment styles
    let content = if trimmed.starts_with("//") {
        trimmed.strip_prefix("//").unwrap_or(trimmed).trim()
    } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
        trimmed
            .strip_prefix("/*")
            .unwrap_or(trimmed)
            .strip_suffix("*/")
            .unwrap_or(trimmed)
            .trim()
    } else {
        trimmed
    };

    // Check for SPDX-License-Identifier prefix
    if content.starts_with("SPDX-License-Identifier:") {
        let license_part = content
            .strip_prefix("SPDX-License-Identifier:")
            .unwrap_or("")
            .trim();
        if !license_part.is_empty() {
            return Some(license_part.to_string());
        }
    }

    None
}
