// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.33;

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title Test
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  TODO

  @custom:date TODO.
*/
contract Test {

  /**
    TODO

    @custom:param TODO

    @custom:return TODO
  */
  mapping (
    bytes32 TODO => address TODO
  ) private addresses;

  /**
    TODO

    @param _node TODO
  */
  modifier authorised (
    bytes32 _node
  ) {
    require(ens.owner(_node) == msg.sender, "Not authorized");
    _;
  }

  /**
    TODO

    @param _node TODO
    @param _addr TODO
  */
  function setAddr (
    bytes32 _node,
    address _addr
  ) public authorised(_node) {
    addresses[_node] = _addr;
    emit AddrChanged(_node, _addr);
  }

  /**
    EIP-165 interface detection

    @param _interfaceId TODO

    @return _ TODO
  */
  function supportsInterface (
    bytes4 _interfaceId
  ) public pure returns (bool) {

    // supportsInterface
    return _interfaceId == 0x3b3b57de || _interfaceId == 0x691f3431
    || _interfaceId == 0x01ffc9a7;
  }
}
