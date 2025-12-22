//! Utility functions for the collector module

use solang_parser::pt::{Comment, Loc};
use solang_parser::helpers::CodeLocation;

/// Converts Loc to (start, end) tuple
pub fn get_loc_range(loc: &Loc) -> (usize, usize) {
    match loc {
        Loc::File(_, start, end) => (*start, *end),
        _ => (0, 0),
    }
}

/// Gets start position from Loc
pub fn get_loc_start(loc: &Loc) -> usize {
    match loc {
        Loc::File(_, start, _) => *start,
        _ => 0,
    }
}

/// Gets end position from Loc
pub fn get_loc_end(loc: &Loc) -> usize {
    match loc {
        Loc::File(_, _, end) => *end,
        _ => 0,
    }
}

/// Checks for blank lines between two consecutive positions
pub fn has_blank_line_between(source: &str, start: usize, end: usize) -> bool {
    if start >= end || end > source.len() {
        return false;
    }

    let text_between = &source[start..end];

    // Count newlines - if there are 2 or more newlines, there's a blank line
    let newline_count = text_between.chars().filter(|&c| c == '\n').count();
    newline_count >= 2
}

/// Groups comments based on blank line separation
pub fn group_comments_by_blank_lines(comments: &[Comment], source: &str) -> Vec<Vec<Comment>> {
    let mut groups = Vec::new();
    let mut current_group = Vec::new();

    let mut sorted_comments: Vec<Comment> = comments.to_vec();
    sorted_comments.sort_by_key(|comment| get_loc_start(&comment.loc()));

    for (i, comment) in sorted_comments.iter().enumerate() {
        if i == 0 {
            current_group.push(comment.clone());
        } else {
            let prev_comment = &sorted_comments[i - 1];
            let prev_end = get_loc_end(&prev_comment.loc());
            let curr_start = get_loc_start(&comment.loc());

            // Check if there are blank lines between the previous and current comment
            if has_blank_line_between(source, prev_end, curr_start) {
                // Blank line found - finish current group and start new one
                if !current_group.is_empty() {
                    groups.push(current_group);
                    current_group = Vec::new();
                }
            }
            current_group.push(comment.clone());
        }
    }

    // Add the last group if it's not empty
    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}