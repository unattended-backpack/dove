use crate::ir_builder::tests::utils::{
    assert_ir_sequences_match, build_ir_from_source, grouped_text, param_doc, read_preen_test,
};
use crate::ir_builder::ir::IRElement;

/// Helper to build a struct IR block
fn struct_ir(
    name: &str,
    natspec_content: Vec<IRElement>,
    fields: Vec<(&str, &str)>,
) -> Vec<IRElement> {
    let mut ir = vec![
        IRElement::text("/**"),
        IRElement::indent(natspec_content),
        IRElement::HardLineBreak,
        IRElement::text("*/"),
        IRElement::HardLineBreak,
        IRElement::text("struct"),
        IRElement::text(" "),
        IRElement::text(name),
        IRElement::text(" "),
        IRElement::text("{"),
    ];

    for (ty, field_name) in fields {
        ir.push(IRElement::HardLineBreak);
        ir.push(IRElement::indent(vec![
            IRElement::text(ty),
            IRElement::text(" "),
            IRElement::text(field_name),
            IRElement::text(";"),
        ]));
    }

    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("}"));
    ir
}

#[test]
fn test_contract_b() {
    let source = read_preen_test("B_in.sol");
    let ir = build_ir_from_source(&source);

    // Build expected contract body contents (all inside a single Indent)
    let mut body_contents = vec![
        IRElement::HardLineBreak, // Initial newline after {
    ];

    // Slot struct
    body_contents.push(IRElement::HardLineBreak);
    body_contents.extend(struct_ir(
        "Slot",
        vec![
            IRElement::HardLineBreak,
            grouped_text("Represents a storage slot key value pair."),
            IRElement::HardLineBreak,
            IRElement::HardLineBreak,
            param_doc("@param key The storage slot key."),
            IRElement::HardLineBreak,
            param_doc("@param value The storage slot value."),
        ],
        vec![("bytes32", "key"), ("bytes32", "value")],
    ));

    // ProvenWithdrawal struct
    body_contents.push(IRElement::HardLineBreak);
    body_contents.push(IRElement::HardLineBreak);
    body_contents.extend(struct_ir(
        "ProvenWithdrawal",
        vec![
            IRElement::HardLineBreak,
            grouped_text("Represents a proven withdrawal."),
            IRElement::HardLineBreak,
            IRElement::HardLineBreak,
            param_doc("@param disputeGameProxy Game that the withdrawal was proven against. This is a really long description."),
            IRElement::HardLineBreak,
            param_doc("@param timestamp Timestamp at which the withdrawal was proven."),
        ],
        vec![("IDisputeGame", "disputeGameProxy"), ("uint64", "timestamp")],
    ));

    // Cat struct
    body_contents.push(IRElement::HardLineBreak);
    body_contents.push(IRElement::HardLineBreak);
    body_contents.extend(struct_ir(
        "Cat",
        vec![
            IRElement::HardLineBreak,
            IRElement::text("TODO"),
            IRElement::HardLineBreak,
            IRElement::HardLineBreak,
            IRElement::text("@param name TODO"),
            IRElement::HardLineBreak,
            IRElement::text("@param color TODO"),
        ],
        vec![("string", "name"), ("string", "color")],
    ));

    // Animal struct
    body_contents.push(IRElement::HardLineBreak);
    body_contents.push(IRElement::HardLineBreak);
    body_contents.extend(struct_ir(
        "Animal",
        vec![
            IRElement::HardLineBreak,
            grouped_text("This struct represents an animal."),
            IRElement::HardLineBreak,
            IRElement::HardLineBreak,
            param_doc("@param name The name of the animal."),
            IRElement::HardLineBreak,
            param_doc("@param color The color of the animal."),
            IRElement::HardLineBreak,
            IRElement::text("@param bite TODO"),
        ],
        vec![("string", "name"), ("string", "color"), ("string", "bite")],
    ));

    // State variable
    body_contents.push(IRElement::HardLineBreak);
    body_contents.push(IRElement::HardLineBreak);
    body_contents.push(IRElement::text("/// TODO"));
    body_contents.push(IRElement::HardLineBreak);
    body_contents.push(IRElement::text("uint256"));
    body_contents.push(IRElement::text(" "));
    body_contents.push(IRElement::text("testValue"));
    body_contents.push(IRElement::text(";"));

    let expected = vec![
        // SPDX License
        IRElement::text("// SPDX-License-Identifier: LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only"),
        IRElement::HardLineBreak,
        // Pragma
        IRElement::text("pragma"),
        IRElement::text(" "),
        IRElement::text("solidity"),
        IRElement::text(" "),
        IRElement::text("0.8.15"),
        IRElement::text(";"),
        IRElement::HardLineBreak,
        IRElement::HardLineBreak,
        // Contract-level comment
        IRElement::text("/**"),
        IRElement::indent(vec![
            IRElement::HardLineBreak,
            IRElement::text("@custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM"),
            IRElement::HardLineBreak,
            IRElement::text("@title A testing contract."),
            IRElement::HardLineBreak,
            IRElement::text("@author Tim Clancy <tim-clancy.eth>"),
            IRElement::HardLineBreak,
            IRElement::text("@custom:terry \"How is twenty times four 80?!\""),
            IRElement::HardLineBreak,
            IRElement::HardLineBreak,
            grouped_text("This contract has a very long description on a single line here. It really keeps running on and on and on. The formatter should make this one look beautiful."),
            IRElement::HardLineBreak,
            IRElement::HardLineBreak,
            IRElement::text("@custom:date June 3rd, 2025."),
        ]),
        IRElement::HardLineBreak,
        IRElement::text("*/"),
        IRElement::HardLineBreak,
        IRElement::text("contract"),
        IRElement::text(" "),
        IRElement::text("Test"),
        IRElement::text(" "),
        IRElement::text("{"),
        // Contract body - all in single Indent
        IRElement::indent(body_contents),
        IRElement::HardLineBreak,
        IRElement::text("}"),
    ];

    assert_ir_sequences_match(&expected, &ir);
}
