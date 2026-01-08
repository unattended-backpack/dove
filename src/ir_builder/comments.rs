//! Comment handling utilities for IR builder

use super::ir::IRElement;
use crate::ir_builder::text_with_word_breaks;
use solang_parser::pt::Comment;

/// Check if a comment is an SPDX license identifier
pub fn is_spdx_comment(comment: &Comment) -> bool {
    let text = match comment {
        Comment::Line(_, text) => text,
        Comment::Block(_, text) => text,
        Comment::DocLine(_, text) => text,
        Comment::DocBlock(_, text) => text,
    };

    // Remove comment prefixes before checking
    let trimmed = text.trim();
    let without_prefix = if trimmed.starts_with("//") {
        trimmed[2..].trim()
    } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
        trimmed[2..trimmed.len() - 2].trim()
    } else {
        trimmed
    };

    without_prefix.starts_with("SPDX-License-Identifier:")
}

/// Check if a comment is an import section header (e.g., "// Contracts", "// Interfaces")
pub fn is_import_section_comment(comment: &Comment) -> bool {
    let text = match comment {
        Comment::Line(_, text) => text,
        Comment::Block(_, text) => text,
        Comment::DocLine(_, text) => text,
        Comment::DocBlock(_, text) => text,
    };

    // Remove comment prefixes before checking
    let trimmed = text.trim();
    let without_prefix = if trimmed.starts_with("//") {
        trimmed[2..].trim()
    } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
        trimmed[2..trimmed.len() - 2].trim()
    } else {
        trimmed
    };

    // Common import section headers that should be dropped when reorganizing imports
    matches!(
        without_prefix,
        "Contracts"
            | "Interfaces"
            | "Libraries"
            | "Utils"
            | "Utilities"
            | "Local"
            | "External"
            | "Internal"
            | "Core"
            | "Helpers"
            | "Types"
            | "Imports"
            | "Dependencies"
            | "Modules"
    )
}

/// Build IR for a single comment using the Comment IR element.
/// The printer will decide how to render it based on line length and indentation.
pub fn build_comment(comment: &Comment) -> IRElement {
    build_comment_internal(comment, false)
}

fn build_comment_internal(comment: &Comment, strip_notice: bool) -> IRElement {
    match comment {
        Comment::Line(_, content) => {
            // Strip // prefix and whitespace
            let text = content
                .trim()
                .strip_prefix("//")
                .unwrap_or(content.trim())
                .trim();
            let text = if strip_notice {
                text.strip_prefix("@notice ").unwrap_or(text)
            } else {
                text
            };
            IRElement::comment(text, false)
        }
        Comment::Block(_, content) => {
            // Strip /* */ and whitespace
            let text = content.trim();
            let text = text.strip_prefix("/*").unwrap_or(text);
            let text = text.strip_suffix("*/").unwrap_or(text);
            let text = text.trim();
            let text = if strip_notice {
                text.strip_prefix("@notice ").unwrap_or(text)
            } else {
                text
            };
            IRElement::comment(text, false)
        }
        Comment::DocLine(_, content) => {
            // Strip /// prefix and whitespace
            let text = content
                .trim()
                .strip_prefix("///")
                .unwrap_or(content.trim())
                .trim();
            let text = if strip_notice {
                text.strip_prefix("@notice ").unwrap_or(text)
            } else {
                text
            };
            IRElement::comment(text, true)
        }
        Comment::DocBlock(_, content) => {
            // Strip /** */ and whitespace
            let text = content.trim();
            let text = text.strip_prefix("/**").unwrap_or(text);
            let text = text.strip_suffix("*/").unwrap_or(text);
            let text = text.trim();
            let text = if strip_notice {
                text.strip_prefix("@notice ").unwrap_or(text)
            } else {
                text
            };
            IRElement::comment(text, true)
        }
    }
}

/// Build structured IR for a NatSpec block comment
pub fn build_natspec_comment_ir(content: &str) -> Vec<IRElement> {
    let mut ir = vec![];

    // Remove comment delimiters if present
    let content = content.trim();
    let content = if content.starts_with("/**") && content.ends_with("*/") {
        content
            .strip_prefix("/**")
            .unwrap()
            .strip_suffix("*/")
            .unwrap()
            .trim()
    } else {
        content
    };

    // Start comment block
    ir.push(IRElement::text("/**"));

    // Parse content line by line - HardLineBreak at start ensures first line gets indented
    let mut content_lines = vec![IRElement::HardLineBreak];
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Blank line
            content_lines.push(IRElement::HardLineBreak);
        } else if trimmed.starts_with("@") {
            // Tag line - don't wrap
            content_lines.push(IRElement::text(trimmed));
            content_lines.push(IRElement::HardLineBreak);
        } else {
            // Description text - allow wrapping
            content_lines.push(IRElement::group(text_with_word_breaks(trimmed)));
            content_lines.push(IRElement::HardLineBreak);
        }
    }

    // Remove trailing line break
    if !content_lines.is_empty() {
        content_lines.pop();
    }

    // Wrap content in indent
    ir.push(IRElement::indent(content_lines));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Build IR for leading comments.
/// Combines consecutive non-doc line comments into a single comment
/// so the printer can decide whether to use line or block format.
/// If @custom:preserve is found, outputs each line comment unchanged.
pub fn build_leading_comments(comments: &[Comment]) -> Vec<IRElement> {
    let mut ir = Vec::new();
    let mut i = 0;

    while i < comments.len() {
        // Check if this is a non-doc line comment that could be combined
        if let Comment::Line(_, _) = &comments[i] {
            // Collect consecutive non-doc line comments
            let mut line_texts: Vec<String> = Vec::new();
            let start_index = i;
            while i < comments.len() {
                if let Comment::Line(_, text) = &comments[i] {
                    let cleaned = text
                        .trim()
                        .strip_prefix("//")
                        .unwrap_or(text.trim())
                        .trim()
                        .to_string();
                    if !cleaned.is_empty() {
                        line_texts.push(cleaned);
                    }
                    i += 1;
                } else {
                    break;
                }
            }

            if !line_texts.is_empty() {
                // Check for @custom:preserve - if present, output each line unchanged
                let has_preserve = line_texts.iter().any(|l| {
                    l.trim() == "@custom:preserve" || l.starts_with("@custom:preserve ")
                });

                if has_preserve {
                    // Preserve mode - output each line comment as raw text
                    for j in start_index..i {
                        if let Comment::Line(_, text) = &comments[j] {
                            // Output the raw comment text (without extra processing)
                            let trimmed = text.trim();
                            ir.push(IRElement::text(trimmed));
                            ir.push(IRElement::HardLineBreak);
                        } else {
                            ir.push(build_comment(&comments[j]));
                            ir.push(IRElement::HardLineBreak);
                        }
                    }
                } else {
                    // Combine the text - the printer will decide the format
                    let combined = line_texts.join(" ");
                    ir.push(IRElement::comment(combined, false));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        } else {
            // Not a non-doc line comment, output as-is
            ir.push(build_comment(&comments[i]));
            ir.push(IRElement::HardLineBreak);
            i += 1;
        }
    }

    ir
}

/// Build IR for leading comments, filtering out SPDX comments.
/// Combines consecutive non-doc line comments into a single comment
/// so the printer can decide whether to use line or block format.
/// If @custom:preserve is found, outputs each line comment unchanged.
pub fn build_leading_comments_without_spdx(comments: &[Comment]) -> Vec<IRElement> {
    let mut ir = Vec::new();
    let mut i = 0;

    while i < comments.len() {
        // Skip SPDX comments
        if is_spdx_comment(&comments[i]) {
            i += 1;
            continue;
        }

        // Check if this is a non-doc line comment that could be combined
        if let Comment::Line(_, _) = &comments[i] {
            // Collect consecutive non-doc, non-SPDX line comments
            let mut line_texts: Vec<String> = Vec::new();
            let mut collected_indices: Vec<usize> = Vec::new();
            while i < comments.len() {
                if is_spdx_comment(&comments[i]) {
                    i += 1;
                    continue;
                }
                if let Comment::Line(_, text) = &comments[i] {
                    let cleaned = text
                        .trim()
                        .strip_prefix("//")
                        .unwrap_or(text.trim())
                        .trim()
                        .to_string();
                    if !cleaned.is_empty() {
                        line_texts.push(cleaned);
                        collected_indices.push(i);
                    }
                    i += 1;
                } else {
                    break;
                }
            }

            if !line_texts.is_empty() {
                // Check for @custom:preserve - if present, output each line unchanged
                let has_preserve = line_texts.iter().any(|l| {
                    l.trim() == "@custom:preserve" || l.starts_with("@custom:preserve ")
                });

                if has_preserve {
                    // Preserve mode - output each line comment as raw text
                    for &j in &collected_indices {
                        if let Comment::Line(_, text) = &comments[j] {
                            // Output the raw comment text (without extra processing)
                            let trimmed = text.trim();
                            ir.push(IRElement::text(trimmed));
                            ir.push(IRElement::HardLineBreak);
                        } else {
                            ir.push(build_comment(&comments[j]));
                            ir.push(IRElement::HardLineBreak);
                        }
                    }
                } else {
                    let combined = line_texts.join(" ");
                    ir.push(IRElement::comment(combined, false));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        } else {
            ir.push(build_comment(&comments[i]));
            ir.push(IRElement::HardLineBreak);
            i += 1;
        }
    }

    ir
}

/// Build IR for a commented element with leading and trailing comments
/// Note: Trailing comments are converted to leading comments (appear before the element)
/// and are preceded by a blank line per style rules.
pub fn with_comments<F>(
    leading: &[Comment],
    trailing: &[Comment],
    build_element: F,
) -> Vec<IRElement>
where
    F: FnOnce() -> Vec<IRElement>,
{
    let mut ir = Vec::new();

    // Add leading comments
    ir.extend(build_leading_comments(leading));

    // Trailing comments become leading comments (appear before the element)
    // Add a blank line before them per style rules
    if !trailing.is_empty() {
        ir.push(IRElement::HardLineBreak);
    }
    ir.extend(build_leading_comments(trailing));

    // Build the element
    ir.extend(build_element());

    ir
}

/// Build IR for a commented element with leading and trailing comments, filtering out SPDX
/// Note: Trailing comments are converted to leading comments (appear before the element)
/// and are preceded by a blank line per style rules.
pub fn with_comments_no_spdx<F>(
    leading: &[Comment],
    trailing: &[Comment],
    build_element: F,
) -> Vec<IRElement>
where
    F: FnOnce() -> Vec<IRElement>,
{
    let mut ir = Vec::new();

    // Add leading comments, filtering out SPDX
    ir.extend(build_leading_comments_without_spdx(leading));

    // Trailing comments become leading comments (appear before the element), filtering SPDX
    // Add a blank line before them per style rules (only if there are non-SPDX trailing comments)
    let has_non_spdx_trailing = trailing.iter().any(|c| !is_spdx_comment(c));
    if has_non_spdx_trailing {
        ir.push(IRElement::HardLineBreak);
    }
    ir.extend(build_leading_comments_without_spdx(trailing));

    // Build the element
    ir.extend(build_element());

    ir
}
