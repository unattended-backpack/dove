//! Comment collection utilities

use crate::collector::utils::{get_loc_end, get_loc_start, group_comments_by_blank_lines, has_blank_line_between};
use solang_parser::pt::Comment;
use solang_parser::helpers::CodeLocation;

/// Partition comments into those before a position and those after
pub fn partition_comments_before(
    comments: &[Comment],
    position: usize,
) -> (Vec<Comment>, Vec<Comment>) {
    let mut before = Vec::new();
    let mut after = Vec::new();

    for comment in comments {
        let comment_end = get_loc_end(&comment.loc());
        if comment_end <= position {
            before.push(comment.clone());
        } else {
            after.push(comment.clone());
        }
    }

    (before, after)
}

/// Find the last group of comments before a position (separated by blank lines)
pub fn take_last_comment_group_before(
    comments: &[Comment],
    source: &str,
    position: usize,
) -> (Vec<Comment>, Vec<Comment>) {
    let (before_comments, after_comments) = partition_comments_before(comments, position);

    if before_comments.is_empty() {
        return (vec![], after_comments);
    }

    // Group the before_comments by blank lines
    let groups = group_comments_by_blank_lines(&before_comments, source);

    if groups.is_empty() {
        return (vec![], after_comments);
    }

    // Take the last group as leading comments for the element
    let last_group = groups.last().unwrap().clone();

    // Check if there's a blank line between the last comment group and the element
    let last_comment = last_group.last().unwrap();
    let last_comment_end = get_loc_end(&last_comment.loc());

    // If there's a blank line between the comment group and the element,
    // treat it as a standalone comment group
    if has_blank_line_between(source, last_comment_end, position) {
        // All comments remain as standalone
        let mut remaining = before_comments;
        remaining.extend(after_comments);
        return (vec![], remaining);
    }

    // Calculate remaining before_comments (all except the last group)
    let last_group_start = get_loc_start(&last_group[0].loc());
    let mut remaining_before = Vec::new();
    for comment in &before_comments {
        if get_loc_start(&comment.loc()) < last_group_start {
            remaining_before.push(comment.clone());
        }
    }

    // Combine remaining_before with after_comments
    let mut remaining = remaining_before;
    remaining.extend(after_comments);

    (last_group, remaining)
}
/// Find comments on the same line after an element
pub fn find_trailing_comments(comments: &[Comment], source: &str, element_end: usize) -> Vec<Comment> {
    comments
        .iter()
        .filter(|comment| {
            let start = get_loc_start(&comment.loc());
            if start >= element_end {
                // Check if comment is on the same line as the element
                !source[element_end..start].contains('\n')
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

/// Remove comments from a collection based on their start positions
pub fn remove_comments_by_position(comments: &mut Vec<Comment>, positions_to_remove: &[usize]) {
    let positions: std::collections::HashSet<usize> = positions_to_remove.iter().cloned().collect();
    comments.retain(|c| !positions.contains(&get_loc_start(&c.loc())));
}

/// Common pattern for collecting comments for an element
pub fn collect_element_comments(
    element_start: usize,
    element_end: usize,
    remaining_comments: &mut Vec<Comment>,
    source: &str,
) -> (Vec<Comment>, Vec<Comment>) {
    // Take the last comment group before this element
    let (leading_comments, rest) =
        take_last_comment_group_before(remaining_comments, source, element_start);
    *remaining_comments = rest;

    // Find trailing comments on the same line
    let trailing_comments = find_trailing_comments(remaining_comments, source, element_end);

    // Remove trailing comments from remaining
    let trailing_positions: Vec<usize> = trailing_comments
        .iter()
        .map(|c| get_loc_start(&c.loc()))
        .collect();
    remove_comments_by_position(remaining_comments, &trailing_positions);

    (leading_comments, trailing_comments)
}

/// Separate comments into internal comments and comments after an element
pub fn separate_internal_and_after_comments(
    remaining_comments: Vec<Comment>,
    trailing_comments: &[Comment],
    element_start: usize,
    element_end: usize,
) -> (Vec<Comment>, Vec<Comment>, Vec<Comment>) {
    let mut internal_comments = Vec::new();
    let mut after_element = Vec::new();
    let trailing_starts: std::collections::HashSet<usize> = trailing_comments
        .iter()
        .map(|c| get_loc_start(&c.loc()))
        .collect();

    for comment in remaining_comments {
        let comment_start = get_loc_start(&comment.loc());
        if trailing_starts.contains(&comment_start) {
            // Skip - already in trailing_comments
        } else if comment_start >= element_start && comment_start < element_end {
            internal_comments.push(comment);
        } else {
            after_element.push(comment);
        }
    }

    (internal_comments, after_element, trailing_comments.to_vec())
}