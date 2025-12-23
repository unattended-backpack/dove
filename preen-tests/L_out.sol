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

  // The precomputed hash of the `.eth` ENS node.
  bytes32 constant ETH_NODE =
    0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae;

  /// The precomputed hash of the `addr.reverse` ENS node.
  bytes32 constant ADDR_REVERSE_NODE =
    0x91d1777781884d03a6757a803996e38de2a42967fb37eeaca72729271025a9e2;

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

  /// Run this script.
  function run () external {

    // Read configuration from environment.
    string memory _mnemonic = vm.envString("MNEMONIC");
    string memory _namesRaw = vm.envString("NAMES");

    // Parse semicolon-separated names.
    string[] memory _names = Utility.splitString(_namesRaw, ";");
    console.log("ENS Registration Script");
    console.log("  Registry:", REGISTRY);
    console.log("  Resolver:", RESOLVER);
    console.log("  Accounts:", _names.length);

    // Register each account's name.
    for (uint32 i = 0; i < _names.length; i++) {
      string memory _name = _names[i];
      if (bytes(_name).length == 0) {
        continue;
      }

      // Broadcast a multicall registration from each account.
      uint256 _privateKey = vm.deriveKey(_mnemonic, i);
      address _account = vm.addr(_privateKey);
      console.log("");
      console.log("Registering:", string.concat(_name, ".eth"));
      console.log("  Account:", _account);
      IMulticallDelegate.Call[] memory _calls =
        _buildRegistrationCalls(_name, _account);
      vm.startBroadcast(_privateKey);
      IMulticallDelegate(_account).multicall(_calls);
      vm.stopBroadcast();
    }
    console.log("");
    console.log("ENS registration complete!");

    /*
      Begin building the registration calls.
      Start with forward resolution: name.eth -> address.
      1. Claim the name.eth subnode.
    */
    IMulticallDelegate.Call[] memory _calls = new IMulticallDelegate.Call[](6);
    _calls[0] = IMulticallDelegate.Call({
      value: 0,
      target: REGISTRY,
      data: abi.encodeCall(
        IENSRegistry.setSubnodeOwner, (ETH_NODE, _labelHash, _account)
      )
    });
  }
}
