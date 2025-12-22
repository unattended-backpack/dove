//! Shared utilities for collected IR builder tests

use crate::ir_builder::build_ir;
use crate::collector::collect_source_unit;
use crate::ir_builder::ir::IRElement;
use solang_parser::parse;
use std::fs;
use std::path::Path;

/// Read a testbench file by name
pub fn read_testbench(filename: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testbench")
        .join(filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

/// Parse Solidity code and build IR
pub fn build_ir_from_source(source: &str) -> Vec<IRElement> {
    let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
    let collected = collect_source_unit(&source_unit, &comments, source);
    build_ir(&collected)
}

/// Assert that two IR sequences match, with helpful diff output on failure
pub fn assert_ir_sequences_match(expected: &[IRElement], actual: &[IRElement]) {
    if expected != actual {
        // Find first difference
        for i in 0..expected.len().min(actual.len()) {
            if expected[i] != actual[i] {
                println!("First difference at index {}:", i);
                println!("Expected: {:?}", expected[i]);
                println!("Actual:   {:?}", actual[i]);

                // Print some context
                let start = i.saturating_sub(2);
                let end = (i + 3).min(expected.len()).min(actual.len());

                println!("\nContext (expected):");
                for j in start..end {
                    println!("  [{}] {:?}", j, expected[j]);
                }

                println!("\nContext (actual):");
                for j in start..end {
                    println!("  [{}] {:?}", j, actual[j]);
                }
                break;
            }
        }

        if expected.len() != actual.len() {
            println!(
                "\nLength mismatch: expected {} elements, got {}",
                expected.len(),
                actual.len()
            );
        }

        panic!("IR sequences do not match!");
    }
}

/// Creates a grouped IR element containing text with SoftLineBreaks between words
///
/// This function takes a string and converts it into a `IRElement::group(...)`
/// containing individual `IRElement::text(...)` entries for each word, with
/// `IRElement::SoftLineBreak` between them.
///
/// # Example
/// ```
/// let ir = grouped_text("Hello world foo");
/// // Returns: IRElement::group(vec![
/// //     IRElement::text("Hello"),
/// //     IRElement::SoftLineBreak,
/// //     IRElement::text("world"),
/// //     IRElement::SoftLineBreak,
/// //     IRElement::text("foo"),
/// // ])
/// ```
pub fn grouped_text(text: &str) -> IRElement {
    IRElement::group(text_with_breaks(text))
}

/// Creates a vector of IR elements containing text with SoftLineBreaks between words
///
/// This is similar to `grouped_text` but returns the raw vector without wrapping
/// it in a group. Useful when you need to combine the result with other elements
/// before grouping.
///
/// # Example
/// ```
/// let elements = text_with_breaks("Hello world");
/// // Returns: vec![
/// //     IRElement::text("Hello"),
/// //     IRElement::SoftLineBreak,
/// //     IRElement::text("world"),
/// // ]
/// ```
pub fn text_with_breaks(text: &str) -> Vec<IRElement> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut elements = Vec::new();

    for (i, word) in words.into_iter().enumerate() {
        if i > 0 {
            elements.push(IRElement::SoftLineBreak);
        }
        elements.push(IRElement::text(word));
    }

    elements
}

/// Creates a grouped IR element for @param documentation lines
///
/// This function takes a parameter documentation string and creates the proper
/// IR structure where the @param tag and parameter name are kept together,
/// followed by the description wrapped in an Indent for continuation indentation.
///
/// # Example
/// ```
/// let ir = param_doc("@param foo The foo parameter");
/// // Returns: IRElement::group(vec![
/// //     IRElement::text("@param foo"),
/// //     IRElement::indent(vec![
/// //         IRElement::SoftLineBreak,
/// //         IRElement::text("The"),
/// //         IRElement::SoftLineBreak,
/// //         IRElement::text("foo"),
/// //         IRElement::SoftLineBreak,
/// //         IRElement::text("parameter"),
/// //     ])
/// // ])
/// ```
pub fn param_doc(text: &str) -> IRElement {
    // Split the text to separate @param tag + name from description
    let parts: Vec<&str> = text.splitn(3, ' ').collect();

    if parts.len() < 3 || !parts[0].starts_with("@param") {
        // Fallback to regular grouped_text if not a proper @param line
        return grouped_text(text);
    }

    // Build continuation with SoftLineBreak and description words
    let mut continuation = vec![IRElement::SoftLineBreak];
    let description_words: Vec<&str> = parts[2].split_whitespace().collect();
    for (i, word) in description_words.into_iter().enumerate() {
        if i > 0 {
            continuation.push(IRElement::SoftLineBreak);
        }
        continuation.push(IRElement::text(word));
    }

    // Wrap continuation in Indent for extra indentation on line wrap
    let elements = vec![
        IRElement::text(format!("{} {}", parts[0], parts[1])),
        IRElement::indent(continuation),
    ];

    IRElement::group(elements)
}
