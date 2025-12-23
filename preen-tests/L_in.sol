// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.33;
contract Test {
  mapping (bytes32 => address) private addresses;
  modifier authorised (bytes32 node) {
    require(ens.owner(node) == msg.sender, "Not authorized");
    _;
  }
  function setAddr (bytes32 node, address _addr) public authorised(node) {
    addresses[node] = _addr;
    emit AddrChanged(node, _addr);
  }

  // EIP-165 interface detection
  function supportsInterface (
    bytes4 _interfaceId
  ) public pure returns (bool) {

    // supportsInterface
    return _interfaceId == 0x3b3b57de || _interfaceId == 0x691f3431
    || _interfaceId == 0x01ffc9a7;
  }
}

