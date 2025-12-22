use super::*;
use solang_parser::pt::{Comment, Loc};

#[test]
fn test_extract_spdx_license_line_comment() {
    let comment_text = "// SPDX-License-Identifier: MIT";
    assert_eq!(extract_spdx_license(comment_text), Some("MIT".to_string()));
}

#[test]
fn test_extract_spdx_license_block_comment() {
    let comment_text = "/* SPDX-License-Identifier: Apache-2.0 */";
    assert_eq!(
        extract_spdx_license(comment_text),
        Some("Apache-2.0".to_string())
    );
}

#[test]
fn test_extract_spdx_license_no_prefix() {
    let comment_text = "SPDX-License-Identifier: GPL-3.0-or-later";
    assert_eq!(
        extract_spdx_license(comment_text),
        Some("GPL-3.0-or-later".to_string())
    );
}

#[test]
fn test_extract_spdx_license_invalid() {
    let comment_text = "// This is not an SPDX comment";
    assert_eq!(extract_spdx_license(comment_text), None);
}

#[test]
fn test_find_existing_spdx_license() {
    let comments = vec![
        Comment::Line(Loc::Builtin, "// Some regular comment".to_string()),
        Comment::Line(Loc::Builtin, "// SPDX-License-Identifier: MIT".to_string()),
        Comment::Line(Loc::Builtin, "// Another comment".to_string()),
    ];

    assert_eq!(
        find_existing_spdx_license(&comments),
        Some("MIT".to_string())
    );
}

#[test]
fn test_generate_spdx_header_no_existing() {
    let comments = vec![Comment::Line(
        Loc::Builtin,
        "// Some regular comment".to_string(),
    )];

    let header = generate_spdx_header(&comments);
    assert!(header.contains("LicenseRef-VPL WITH AGPL-3.0-only"));
}

#[test]
fn test_generate_spdx_header_with_existing() {
    let comments = vec![Comment::Line(
        Loc::Builtin,
        "// SPDX-License-Identifier: MIT".to_string(),
    )];

    let header = generate_spdx_header(&comments);
    assert!(header.contains("MIT AND"));
    assert!(header.contains("LicenseRef-VPL WITH AGPL-3.0-only"));
}
