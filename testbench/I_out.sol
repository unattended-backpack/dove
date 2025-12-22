// SPDX-License-Identifier: MIT AND (LicenseRef-VPL WITH AGPL-3.0-only)
pragma solidity 0.8.15;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";

import { IGovernanceToken } from "interfaces/governance/IGovernanceToken.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title MintManager
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  Set as `owner` of the governance token and responsible for the token inflation
  schedule. Contract acts as the token "mint manager" with permission to the
  `mint` function only. Currently permitted to mint once per year of up to 2% of
  the total token supply. Upgradable to allow changes in the inflation schedule.

  @custom:date TODO.
*/
contract MintManager is
  Ownable {

  /// The GovernanceToken that the MintManager can mint tokens
  IGovernanceToken public immutable governanceToken;

  /**
    The amount of tokens that can be minted per year. The value is a fixed point
    number with 4 decimals. 2%
  */
  uint256 public constant MINT_CAP = 20;

  /// The number of decimals for the MINT_CAP.
  uint256 public constant DENOMINATOR = 1000;

  /**
    The amount of time that must pass before the MINT_CAP number of tokens can
    be minted again.
  */
  uint256 public constant MINT_PERIOD = 365 days;

  /// Tracks the time of last mint.
  uint256 public mintPermittedAfter;

  /**
    Constructs the MintManager contract.

    @param _upgrader The owner of this contract.
    @param _governanceToken The governance token this contract can mint tokens
      of.
  */
  constructor (
    address _upgrader,
    address _governanceToken
  ) {
    transferOwnership(_upgrader);
    governanceToken = IGovernanceToken(_governanceToken);
  }

  /**
    Only the token owner is allowed to mint a certain amount of the governance
    token per year.

    @param _account The account receiving minted tokens.
    @param _amount The amount of tokens to mint.
  */
  function mint (
    address _account,
    uint256 _amount
  ) public onlyOwner {
    if (mintPermittedAfter > 0) {
      require(
        mintPermittedAfter <= block.timestamp,
        "MintManager: minting not permitted yet"
      );
      require(
        _amount <= (governanceToken.totalSupply() * MINT_CAP) / DENOMINATOR,
        "MintManager: mint amount exceeds cap"
      );
    }
    mintPermittedAfter = block.timestamp + MINT_PERIOD;
    governanceToken.mint(_account, _amount);
  }

  /**
    Upgrade the owner of the governance token to a new MintManager.

    @param _newMintManager The MintManager to upgrade to.
  */
  function upgrade (
    address _newMintManager
  ) public onlyOwner {
    require(
      _newMintManager != address(0),
      "MintManager: mint manager cannot be the zero address"
    );
    governanceToken.transferOwnership(_newMintManager);
  }
}
