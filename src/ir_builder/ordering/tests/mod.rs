use super::*;
use solang_parser::pt::{Identifier, Loc};

#[test]
fn test_compare_versions() {
    assert_eq!(compare_versions(&[1, 0, 0], &[0, 9, 9]), 1);
    assert_eq!(compare_versions(&[0, 8, 0], &[0, 8, 1]), -1);
    assert_eq!(compare_versions(&[1, 2, 3], &[1, 2, 3]), 0);
}

#[test]
fn test_format_identifier_path() {
    let path = IdentifierPath {
        loc: Loc::Builtin,
        identifiers: vec![
            Identifier {
                loc: Loc::Builtin,
                name: "contracts".to_string(),
            },
            Identifier {
                loc: Loc::Builtin,
                name: "MyContract".to_string(),
            },
        ],
    };
    assert_eq!(format_identifier_path(&path), "contracts.MyContract");
}

#[test]
fn test_get_import_path_string() {
    let path = ImportPath::Filename(StringLiteral {
        loc: Loc::Builtin,
        unicode: false,
        string: "@openzeppelin/contracts/token/ERC20/ERC20.sol".to_string(),
    });
    let import = Import::Plain(path, Loc::Builtin);
    assert_eq!(
        get_import_path_string(&import),
        "@openzeppelin/contracts/token/ERC20/ERC20.sol"
    );
}
