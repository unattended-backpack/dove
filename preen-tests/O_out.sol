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
  bytes32 public constant TYPED_DATA_SIGN_TYPEHASH =
    keccak256(
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
        keccak256(
          "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
        ), keccak256("MockERC7739Signer"), keccak256("1"), block.chainid,
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
    bytes32 _wrappedHash =
      keccak256(
        abi.encodePacked(
          "\x19\x01", DOMAIN_SEPARATOR,
          keccak256(
            abi.encode(TYPED_DATA_SIGN_TYPEHASH, _hash, bytes1(0x00), "")
          )
        )
      );
    if (
      SignatureCheckerLib.isValidSignatureNow(owner, _wrappedHash, _signature)
    ) {
      return ERC1271_MAGIC_VALUE;
    }
    return 0xffffffff;
  }

  /**
    Get the hash that the owner must sign for a given application hash. This is
    a helper for tests to construct valid signatures.

    @param _appHash The application's typed data hash.

    @return _ The hash the owner should sign.
  */
  function getWrappedHash (
    bytes32 _appHash
  ) external view returns (bytes32) {
    return keccak256(
      abi.encodePacked(
        "\x19\x01", DOMAIN_SEPARATOR,
        keccak256(
          abi.encode(TYPED_DATA_SIGN_TYPEHASH, _appHash, bytes1(0x00), "")
        )
      )
    );
  }

  /// An ERC-7739 signer rejects signatures over the unwrapped hash.
  function test_transferWithAuthorization_erc7739Signer_unwrappedHash_reverts ()
    public {

    // Create a smart contract signer that uses nested EIP-712.
    address _signerAddress = address(new MockERC7739Signer(alice));

    // Give tokens to the signer contract.
    token.mint(_signerAddress, 100 ether);
    bytes32 _nonce = bytes32(uint256(51));

    // Sign the application hash directly (without wrapping) - this should fail.
    bytes memory _signature =
      _signTransferAuthorization(
        ALICE_PK, _signerAddress, bob, 100 ether, block.timestamp - 1,
        block.timestamp + 1 hours, _nonce
      );
    vm.expectRevert(IERC3009.InvalidSignature.selector);
    token.transferWithAuthorization(
      _signerAddress, bob, 100 ether, block.timestamp - 1,
      block.timestamp + 1 hours, _nonce, _signature
    );
  }

  /// A transfer exactly at validBefore timestamp is rejected.
  function test_transferWithAuthorization_exactlyAtValidBefore_reverts ()
    public {

    // Warp forward to avoid underflow when computing validAfter.
    vm.warp(2 hours);
    uint256 _amount = 100 ether;
    bytes32 _nonce = bytes32(uint256(63));
    uint256 _validAfter = block.timestamp - 1 hours;
    uint256 _validBefore = block.timestamp;
    bytes memory _signature =
      _signTransferAuthorization(
        ALICE_PK, alice, bob, _amount, _validAfter, _validBefore, _nonce
      );

    // ERC-3009 requires block.timestamp < validBefore (strict inequality).
    vm.expectRevert(IERC3009.AuthorizationExpired.selector);
    token.transferWithAuthorization(
      alice, bob, _amount, _validAfter, _validBefore, _nonce, _signature
    );
  }

  /**
    Call `onApprovalReceived` on `_spender` and verify it returns the expected
    selector.

    @param _spender The address that was approved.
    @param _value The amount of tokens approved.
    @param _data Additional data to pass to the spender.
  */
  function _checkOnApprovalReceived (
    address _spender,
    uint256 _value,
    bytes memory _data
  ) private {
    if (_spender.code.length == 0) {
      revert ERC1363EOAReceiver();
    }

    // Revert if the target could not handle the approval and bubble up errors.
    try IERC1363Spender(_spender).onApprovalReceived(
      msg.sender, _value, _data
    ) returns (bytes4 _retval) {
      if (_retval != IERC1363Spender.onApprovalReceived.selector) {
        revert ERC1363InvalidSpender();
      }
    } catch (bytes memory _reason) {
      if (_reason.length == 0) {
        revert ERC1363InvalidSpender();
      } else {
        assembly ("memory-safe") {
          revert(add(_reason, 0x20), mload(_reason))
        }
      }
    }
  }

  /**
    @custom:preserve

    convertToShares() returns correct share amount.

    With initial state:
    - totalAssets = 1 gwei (1e9 wei WETH)
    - totalSupply = 1 billion SIGIL (1e27 wei)
    - decimalsOffset = 18

    Formula (Solady ERC4626 with offset):
    shares = assets * (totalSupply + 10^offset) / (totalAssets + 1)

    For 1 WETH (1e18 wei):
    shares = 1e18 * (1e27 + 1e18) / (1e9 + 1)
           ≈ 1e18 * 1e27 / 1e9
           = 1e45 / 1e9 = 1e36 shares

    This is 1 billion times the total supply because 1 WETH is 1 billion times
    the initial backing amount (1 gwei).
  */
  function test_convertToShares () public view {
    uint256 _assets = 1 ether;
    uint256 _shares = token.convertToShares(_assets);

    // 1 WETH should convert to ~1e36 shares (1B times total supply).
    uint256 _expectedShares =
      _assets * (TOTAL_SUPPLY + 1e18) / (INIT_WETH_AMOUNT + 1);
    assertEq(_shares, _expectedShares);

    // Sanity check: 1 WETH = 1e9 gwei, so 1e9 times the total supply.
    assertApproxEqRel(_shares, TOTAL_SUPPLY * 1e9, 0.001e18);
  }

  /*
    @custom:preserve

    This fallback routes unrecognized calls to a query contract specified as
    the first argument in the calldata. This allows external callers to interact
    with query contracts as if their view functions lived directly on this
    contract.

    The query contract address is extracted from the first ABI-encoded argument.
    Query contract functions should accept the query contract address as their
    first parameter and ignore it, since it is only used for routing.

    function myQuery (
      address,
      address _user
    ) external view returns (uint256);

    Callers can then invoke their queries like so.
 
    IMyQuery(address(this)).myQuery(queryAddr, user);
  */
  fallback () external {
    address _query = abi.decode(msg.data[4:], (address));
    (bool _innerSuccess, bytes memory _innerResult) = delegateview(
      _query, msg.data
    );
    assembly ("memory-safe") {
      let _ptr := add(_innerResult, 0x20)
      let _len := mload(_innerResult)
      if iszero(_innerSuccess) {
        revert(_ptr, _len)
      }
      return(_ptr, _len)
    }

    /*
      Allow for rounding differences due to ERC4626 virtual shares/assets math.
      The _decimalsOffset of 18 introduces rounding at extreme ratios. 0.1%
      tolerance
    */
    assertApproxEqRel(_received, _expectedAssets, 0.001e18);
  }

  /**
    Return whether this contract supports a given interface.

    @param _interfaceId The interface identifier to check.

    @return _ Whether the interface is supported.
  */
  function supportsInterface (
    bytes4 _interfaceId
  ) public view override(
    ERC1363, ERC2612, BurnableERC3009, ERC5805, BurnOnlyERC4626
  ) returns (
    bool
  ) {
    return ERC1363.supportsInterface(_interfaceId)
    || ERC2612.supportsInterface(_interfaceId)
    || BurnableERC3009.supportsInterface(_interfaceId)
    || ERC5805.supportsInterface(_interfaceId)
    || BurnOnlyERC4626.supportsInterface(_interfaceId);
  }
}
