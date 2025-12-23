// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.33;
contract Test {
  modifier authorised (bytes32 node) {
    require(ens.owner(node) == msg.sender, "Not authorized");
    _;
  }
  function setAddr (bytes32 node, address _addr) public authorised(node) {
    addresses[node] = _addr;
    emit AddrChanged(node, _addr);
  }
}

