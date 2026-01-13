// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.33;

import { ITest20 } from "./interfaces/ITest20.sol";
import { ERC20 } from "solady/tokens/ERC20.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title Test20
  @custom:blame Tim Clancy <tim-clancy.eth>
  @custom:terry "Is this too much voodoo for the next ten centuries?"
  @custom:preserve

  A test ERC-20 token with a public faucet mint function. Anyone can mint any
  amount of tokens to themselves for testing purposes.

  @custom:date January 4th, 2026.
*/
contract Test20 is
  ITest20,
  ERC20 {

  /**
    An enum for representing the decoded message sender portion of the contract
    creation salt. This is used for checking for permissioned deploy protection.

    @param MsgSender The salt contains the message sender.
    @param ZeroAddress The salt contains the 0x0...0 zero address.
    @param Random The salt contains random bytes.
  */
  enum SenderBytes {
    MsgSender,
    ZeroAddress,
    Random
  }

  /// A constant.
  uint128 constant AUCTION_SUPPLY = 500_000000_000000000000000000;

  /**
    Return the name of the token.

    @return _ The name of the token.
  */
  function name () public pure override(ITest20, ERC20) returns (
    string memory
  ) {
    return "Test20";
  }

  /**
    Returns the symbol of the token.

    @return _ The symbol of the token.
  */
  function symbol () public pure override(ITest20, ERC20) returns (
    string memory
  ) {

    // @custom:preserve
    // |                      | ↓ ptr ...  ↓ ptr + 0x0B (start) ...  ↓ ptr + 0x20 ...  ↓ ptr + 0x40 ...   |
    // |----------------------|---------------------------------------------------------------------------|
    // | initCodeHash         |                                                        CCCCCCCCCCCCC...CC |
    // | salt                 |                                      BBBBBBBBBBBBB...BB                   |
    // | deployer             | 000000...0000AAAAAAAAAAAAAAAAAAA...AA                                     |
    // | 0xFF                 |            FF                                                             |
    // |----------------------|---------------------------------------------------------------------------|
    // | memory               | 000000...00FFAAAAAAAAAAAAAAAAAAA...AABBBBBBBBBBBBB...BBCCCCCCCCCCCCC...CC |
    // | keccak256(start, 85) |            ↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑↑ |
    return "TEST";
  }

  /**
    Mint `_amount` tokens to the caller. This is a faucet function for testing.

    @param _amount The amount of tokens to mint.
  */
  function mint (
    uint256 _amount
  ) external {
    _mint(msg.sender, _amount);
  }

  /**
    Parse a given salt to decode settings for permissioned deployer protection
    and cross-chain redeploy protection.

    @param _salt The 32-byte value used to create the contract address.

    @return _ A tuple of (SenderBytes, RedeployProtectionFlag) containing the
      decoded protection parts of the `_salt`.
  */
  function _parseSalt (
    bytes32 _salt
  ) internal view returns (SenderBytes, RedeployProtectionFlag) {

    // The salt is protected to the caller with cross-chain use disallowed.
    if (address(bytes20(_salt)) == msg.sender && bytes1(_salt[20]) == hex"01") {
      return (SenderBytes.MsgSender, RedeployProtectionFlag.True);

    // The salt is protected to the caller with cross-chain use allowed.
    } else if (
      address(bytes20(_salt)) == msg.sender && bytes1(_salt[20]) == hex"00"
    ) {
      return (SenderBytes.MsgSender, RedeployProtectionFlag.False);

    // The salt is protected to the caller with invalid cross-chain settings.
    } else if (address(bytes20(_salt)) == msg.sender) {
      return (SenderBytes.MsgSender, RedeployProtectionFlag.Unspecified);

    // The salt is unprotected with cross-chain use disallowed.
    } else if (
      address(bytes20(_salt)) == address(0) && bytes1(_salt[20]) == hex"01"
    ) {
      return (SenderBytes.ZeroAddress, RedeployProtectionFlag.True);

    // The salt is unprotected with cross-chain use allowed.
    } else if (
      address(bytes20(_salt)) == address(0) && bytes1(_salt[20]) == hex"00"
    ) {
      return (SenderBytes.ZeroAddress, RedeployProtectionFlag.False);

    // The salt is unprotected with invalid cross-chain settings.
    } else if (address(bytes20(_salt)) == address(0)) {
      return (SenderBytes.ZeroAddress, RedeployProtectionFlag.Unspecified);

    /*
      The salt is unprotected with more randomness than the zero address alone
      and cross-chain use disallowed.
    */
    } else if (bytes1(_salt[20]) == hex"01") {
      return (SenderBytes.Random, RedeployProtectionFlag.True);

    /*
      The sale is unprotected with more randomness than the zero address alone
      and cross-chain use is allowed.
    */
    } else if (bytes1(_salt[20]) == hex"00") {
      return (SenderBytes.Random, RedeployProtectionFlag.False);

    // The salt has invalid cross-chain settings.
    } else {
      return (SenderBytes.Random, RedeployProtectionFlag.Unspecified);
    }
  }
}
