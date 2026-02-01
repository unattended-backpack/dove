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

/// Check if a tag is a param/return tag that should have wrapping applied
fn is_param_or_return_tag(tag: &str) -> bool {
    tag.starts_with("@param ")
        || tag.starts_with("@return ")
        || tag.starts_with("@custom:param ")
        || tag.starts_with("@custom:return ")
}

/// Format a NatSpec tag (like @param or @return) with word wrapping
/// The tag prefix and parameter name stay on the first line, description words can wrap
/// with continuation indentation (2 spaces)
/// Only applies to @param/@return tags - other tags are returned as plain text
fn format_tag_with_wrapping(tag: &str) -> Vec<IRElement> {
    // Only apply wrapping to @param and @return tags (which have name + description)
    if !is_param_or_return_tag(tag) {
        return vec![IRElement::text(tag)];
    }

    // Parse the tag: "@tag name description..."
    // We need to find where the description starts (after the second word for @param/@return)
    let parts: Vec<&str> = tag.splitn(3, ' ').collect();

    if parts.len() < 3 {
        // No description to wrap, just return as text
        return vec![IRElement::text(tag)];
    }

    // parts[0] = "@param" or "@custom:param" etc.
    // parts[1] = parameter name
    // parts[2] = description (may contain multiple words)
    let prefix = format!("{} {}", parts[0], parts[1]);
    let description = parts[2];

    let words: Vec<&str> = description.split_whitespace().collect();
    if words.is_empty() {
        return vec![IRElement::text(&prefix)];
    }

    let mut inner = vec![IRElement::text(&prefix)];

    // Add description words with soft line breaks that include continuation indent
    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            inner.push(IRElement::text(" "));
        } else {
            // SoftLineBreak with continuation indent (2 spaces)
            inner.push(IRElement::SoftLineBreakWithContinuation);
        }
        inner.push(IRElement::text(*word));
    }

    // Wrap in a Group so the printer uses fill logic for word wrapping
    vec![IRElement::group(inner)]
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

    // Parse content line by line, combining continuation lines with their tags
    // First pass: collect lines and identify which are tag continuations
    let lines: Vec<&str> = content.lines().collect();
    let mut combined_lines: Vec<(bool, String)> = Vec::new(); // (is_tag, content)

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.is_empty() {
            // Blank line
            combined_lines.push((false, String::new()));
            i += 1;
        } else if trimmed.starts_with("@") {
            // Start of a tag - collect continuation lines
            let mut tag_text = trimmed.to_string();
            i += 1;

            // Check for continuation lines (non-empty, not starting with @, not after blank)
            while i < lines.len() {
                let next_trimmed = lines[i].trim();
                if next_trimmed.is_empty() || next_trimmed.starts_with("@") {
                    break;
                }
                // This is a continuation line - append it to the tag
                tag_text.push(' ');
                tag_text.push_str(next_trimmed);
                i += 1;
            }
            combined_lines.push((true, tag_text));
        } else {
            // Description text
            combined_lines.push((false, trimmed.to_string()));
            i += 1;
        }
    }

    // Second pass: build IR from combined lines
    let mut content_lines = vec![IRElement::HardLineBreak];
    for (is_tag, text) in combined_lines {
        if text.is_empty() {
            // Blank line
            content_lines.push(IRElement::HardLineBreak);
        } else if is_tag {
            // Tag line - use wrapping for long tags
            content_lines.extend(format_tag_with_wrapping(&text));
            content_lines.push(IRElement::HardLineBreak);
        } else {
            // Description text - allow wrapping
            content_lines.push(IRElement::group(text_with_word_breaks(&text)));
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

/// Build IR for a commented element with leading and trailing comments.
/// Trailing line comments are combined with leading line comments into a single
/// block comment so that related comments stay together.
pub fn with_comments<F>(
    leading: &[Comment],
    trailing: &[Comment],
    build_element: F,
) -> Vec<IRElement>
where
    F: FnOnce() -> Vec<IRElement>,
{
    let mut ir = Vec::new();

    // Combine leading and trailing line comments together so they form a single
    // block comment. Non-line comments (block/doc) are kept separate.
    let mut leading_line_comments: Vec<Comment> = Vec::new();
    let mut other_leading: Vec<Comment> = Vec::new();

    for comment in leading {
        if matches!(comment, Comment::Line(_, _)) {
            leading_line_comments.push(comment.clone());
        } else {
            // Process any accumulated line comments first
            if !leading_line_comments.is_empty() {
                ir.extend(build_leading_comments(&leading_line_comments));
                leading_line_comments.clear();
            }
            other_leading.push(comment.clone());
        }
    }

    // Collect trailing line comments
    let trailing_line_comments: Vec<Comment> = trailing
        .iter()
        .filter(|c| matches!(c, Comment::Line(_, _)))
        .cloned()
        .collect();

    // Process non-line leading comments
    ir.extend(build_leading_comments(&other_leading));

    // If there are leading line comments, combine them with trailing line comments
    if !leading_line_comments.is_empty() {
        let mut combined = leading_line_comments;
        combined.extend(trailing_line_comments);
        ir.extend(build_leading_comments(&combined));
    } else if !trailing_line_comments.is_empty() {
        // No leading line comments to combine with - add blank line before trailing
        ir.push(IRElement::HardLineBreak);
        ir.extend(build_leading_comments(&trailing_line_comments));
    }

    // Build the element
    ir.extend(build_element());

    ir
}

/// Build IR for a commented element with leading and trailing comments, filtering out SPDX.
/// Trailing line comments are combined with leading line comments into a single
/// block comment so that related comments stay together.
pub fn with_comments_no_spdx<F>(
    leading: &[Comment],
    trailing: &[Comment],
    build_element: F,
) -> Vec<IRElement>
where
    F: FnOnce() -> Vec<IRElement>,
{
    let mut ir = Vec::new();

    // Combine leading and trailing line comments together so they form a single
    // block comment. Non-line comments (block/doc) are kept separate. Filter SPDX.
    let mut leading_line_comments: Vec<Comment> = Vec::new();
    let mut other_leading: Vec<Comment> = Vec::new();

    for comment in leading {
        if is_spdx_comment(comment) {
            continue;
        }
        if matches!(comment, Comment::Line(_, _)) {
            leading_line_comments.push(comment.clone());
        } else {
            // Process any accumulated line comments first
            if !leading_line_comments.is_empty() {
                ir.extend(build_leading_comments_without_spdx(&leading_line_comments));
                leading_line_comments.clear();
            }
            other_leading.push(comment.clone());
        }
    }

    // Collect trailing line comments (filtering SPDX)
    let trailing_line_comments: Vec<Comment> = trailing
        .iter()
        .filter(|c| !is_spdx_comment(c) && matches!(c, Comment::Line(_, _)))
        .cloned()
        .collect();

    // Process non-line leading comments
    ir.extend(build_leading_comments_without_spdx(&other_leading));

    // If there are leading line comments, combine them with trailing line comments
    if !leading_line_comments.is_empty() {
        let mut combined = leading_line_comments;
        combined.extend(trailing_line_comments);
        ir.extend(build_leading_comments_without_spdx(&combined));
    } else if !trailing_line_comments.is_empty() {
        // No leading line comments to combine with - add blank line before trailing
        ir.push(IRElement::HardLineBreak);
        ir.extend(build_leading_comments_without_spdx(&trailing_line_comments));
    }

    // Build the element
    ir.extend(build_element());

    ir
}
