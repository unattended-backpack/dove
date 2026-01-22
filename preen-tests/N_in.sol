// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.26;

import { MockERC1271Signer } from "./MockERC1271Signer.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title A factory for deploying mock ERC-1271 signers.
  @author Tim Clancy <tim-clancy.eth>
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  This factory is used for testing ERC-6492 counterfactual signatures. It
  deploys `MockERC1271Signer` contracts to deterministic addresses.

  @custom:date January 21st, 2026.
*/
contract MockERC1271SignerFactory {

  /**
    Deploy a new `MockERC1271Signer` with the given owner.

    @param _owner The owner of the new signer contract.

    @return _ The address of the deployed signer.
  */
  function deploy (
    address _owner
  ) external returns (address) {
    return address(new MockERC1271Signer(_owner));
  }

  /**
    Compute the address where a signer would be deployed for a given owner using
    CREATE2.

    @param _owner The owner of the signer.
    @param _salt The salt for CREATE2.

    @return _ The predicted address.
  */
  function computeAddress (
    address _owner,
    bytes32 _salt
  ) external view returns (address) {
    bytes memory _bytecode =
      abi.encodePacked(type(MockERC1271Signer).creationCode, abi.encode(_owner));
    bytes32 _hash =
      keccak256(
        abi.encodePacked(
          bytes1(0xff), address(this), _salt, keccak256(_bytecode)
        )
      );
    return address(uint160(uint256(_hash)));
  }

  /**
    Deploy a new `MockERC1271Signer` with the given owner using CREATE2.

    @param _owner The owner of the new signer contract.
    @param _salt The salt for CREATE2.

    @return _ The address of the deployed signer.
  */
  function deployWithSalt (
    address _owner,
    bytes32 _salt
  ) external returns (address) {
    return address(new MockERC1271Signer{ salt: _salt }(_owner));
  }
}

