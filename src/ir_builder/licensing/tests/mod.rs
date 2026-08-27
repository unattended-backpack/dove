use super::*;

#[test]
fn test_generate_spdx_header() {
    assert_eq!(
        generate_spdx_header(),
        "// SPDX-License-Identifier: LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only"
    );
}
