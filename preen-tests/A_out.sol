// SPDX-License-Identifier: LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only
pragma solidity 0.8.15;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";

import { IAnchorStateRegistry } from
  "interfaces/dispute/IAnchorStateRegistry.sol";
import { IResourceMetering } from "interfaces/L1/IResourceMetering.sol";
import { ISemver } from "interfaces/universal/ISemver.sol";

import { GameStatus, GameType } from "src/dispute/lib/Types.sol";
import { ResourceMetering } from "src/L1/ResourceMetering.sol";
import { Storage } from "src/libraries/Storage.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title StorageSetter
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  A simple contract that allows setting arbitrary storage slots. WARNING: this
  contract is not safe to be called by untrusted parties. It is only meant as an
  intermediate step during upgrades.

  @custom:date June 18th, 2025.
*/
contract StorageSetter is
  ISemver {

  /**
    Represents a storage slot key value pair.

    @param key TODO
    @param value TODO
  */
  struct Slot {
    bytes32 key;
    bytes32 value;
  }

  /**
    Semantic version.
    @custom:semver 1.2.1-beta.4
  */
  string public constant version = "1.2.1-beta.4";

  /**
    Stores a bytes32 `_value` at `_slot`. Any storage slots that are packed
    should be set through this interface.

    @param _slot TODO
    @param _value TODO
  */
  function setBytes32 (
    bytes32 _slot,
    bytes32 _value
  ) public {

    // This is an example of a postfix comment that ought to be moved.
    Storage.setBytes32(_slot, _value);
  }

  /**
    Stores a bytes32 value at each key in `_slots`.

    @param _slots TODO
  */
  function setBytes32 (
    Slot[] calldata _slots
  ) public {
    uint256 _length = _slots.length;
    for (uint256 i; i < _length; i++) {
      Storage.setBytes32(_slots[i].key, _slots[i].value);
    }
  }

  /**
    Retrieves a bytes32 value from `_slot`.

    @param _slot TODO

    @return _ TODO
  */
  function getBytes32 (
    bytes32 _slot
  ) external view returns (bytes32) {
    return Storage.getBytes32(_slot);
  }

  /**
    Stores a uint256 `_value` at `_slot`.

    @param _slot TODO
    @param _value TODO
  */
  function setUint (
    bytes32 _slot,
    uint256 _value
  ) public {
    Storage.setUint(_slot, _value);
  }

  /**
    Retrieves a uint256 value from `_slot`.

    @param _slot TODO

    @return _ TODO
  */
  function getUint (
    bytes32 _slot
  ) external view returns (uint256) {
    return Storage.getUint(_slot);
  }

  /**
    Stores an address `_value` at `_slot`.

    @param _slot TODO
    @param _address TODO
  */
  function setAddress (
    bytes32 _slot,
    address _address
  ) public {
    Storage.setAddress(_slot, _address);
  }

  /**
    Retrieves an address value from `_slot`.

    @param _slot TODO

    @return _ TODO
  */
  function getAddress (
    bytes32 _slot
  ) external view returns (address) {
    return Storage.getAddress(_slot);
  }

  /**
    Stores a bool `_value` at `_slot`.

    @param _slot TODO
    @param _value TODO
  */
  function setBool (
    bytes32 _slot,
    bool _value
  ) public {
    Storage.setBool(_slot, _value);
  }

  /**
    Retrieves a bool value from `_slot`.

    @param _slot TODO

    @return _ TODO
  */
  function getBool (
    bytes32 _slot
  ) external view returns (bool) {
    return Storage.getBool(_slot);
  }

  /**
    TODO

    @return _ TODO
  */
  function resourceConfig () internal view returns (
    ResourceMetering.ResourceConfig memory
  ) {
    IResourceMetering.ResourceConfig memory _config;

    // This goes away when we're all done.
    ResourceMetering.ResourceConfig memory _configOutput;
    assembly ("memory-safe") {
      _configOutput := _config
    }
    return _configOutput;
  }
}
