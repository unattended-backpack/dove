// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.26;

import { SignatureCheckerLib } from "solady/utils/SignatureCheckerLib.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title A mock ERC-7739 signer for testing.
  @author Tim Clancy <tim-clancy.eth>
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  This mock contract implements ERC-7739 nested EIP-712 signature validation.
  It wraps incoming hashes in its own domain before validating against the
  owner's signature.

  @custom:date January 21st, 2026.
*/
contract MockERC7739Signer {

  /// The ERC-1271 magic value returned when a signature is valid.
  bytes4 public constant ERC1271_MAGIC_VALUE = 0x1626ba7e;

  /// The EIP-712 typehash for the TypedDataSign wrapper.
  bytes32 public constant TYPED_DATA_SIGN_TYPEHASH = keccak256(
    "TypedDataSign(bytes32 contentsHash,bytes1 contentsDescriptionHash,string contentsDescription)"
  );

  /// The address of the owner whose signatures are considered valid.
  address public immutable owner;

  /// The cached domain separator for this signer.
  bytes32 public immutable DOMAIN_SEPARATOR;

  /**
    Construct a new mock ERC-7739 signer with a designated owner.

    @param _owner The address of the owner whose signatures will be validated.
  */
  constructor (
    address _owner
  ) {
    owner = _owner;
    DOMAIN_SEPARATOR = keccak256(
      abi.encode(
        keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
        keccak256("MockERC7739Signer"),
        keccak256("1"),
        block.chainid,
        address(this)
      )
    );
  }

  /**
    Validate a signature per ERC-1271 with ERC-7739 support. The signature
    should be the owner's signature over the nested hash structure.

    @param _hash The application's typed data hash.
    @param _signature The signature to validate.

    @return _ The ERC-1271 magic value if valid, otherwise `0xffffffff`.
  */
  function isValidSignature (
    bytes32 _hash,
    bytes memory _signature
  ) external view returns (bytes4) {

    // Wrap the application's hash in our own EIP-712 structure.
    bytes32 _wrappedHash = keccak256(
      abi.encodePacked(
        "\x19\x01",
        DOMAIN_SEPARATOR,
        keccak256(abi.encode(TYPED_DATA_SIGN_TYPEHASH, _hash, bytes1(0x00), ""))
      )
    );

    if (SignatureCheckerLib.isValidSignatureNow(owner, _wrappedHash, _signature)) {
      return ERC1271_MAGIC_VALUE;
    }
    return 0xffffffff;
  }

  /**
    Get the hash that the owner must sign for a given application hash.
    This is a helper for tests to construct valid signatures.

    @param _appHash The application's typed data hash.

    @return _ The hash the owner should sign.
  */
  function getWrappedHash (
    bytes32 _appHash
  ) external view returns (bytes32) {
    return keccak256(
      abi.encodePacked(
        "\x19\x01",
        DOMAIN_SEPARATOR,
        keccak256(abi.encode(TYPED_DATA_SIGN_TYPEHASH, _appHash, bytes1(0x00), ""))
      )
    );
  }
}
