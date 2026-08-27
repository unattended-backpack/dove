// SPDX-License-Identifier: LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only
pragma solidity 0.8.15;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { ERC20Burnable } from
  "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import { ERC20Permit, ERC20Votes } from
  "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title GovernanceToken
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"
  @custom:predeploy 0x4200000000000000000000000000000000000042

  The Optimism token used in governance and supporting voting and delegation.
  Implements EIP 2612 allowing signed approvals. Contract is "owned" by a
  `MintManager` instance with permission to the `mint` function only, for the
  purposes of enforcing the token inflation schedule.

  @custom:date June 25th, 2025.
*/
contract GovernanceToken is
  ERC20Burnable,
  ERC20Votes,
  Ownable {

  /// Constructs the GovernanceToken contract.
  constructor () ERC20("Optimism", "OP") ERC20Permit("Optimism") { }

  /**
    Allows the owner to mint tokens.

    @param _account The account receiving minted tokens.
    @param _amount The amount of tokens to mint.
  */
  function mint (
    address _account,
    uint256 _amount
  ) public onlyOwner {
    _mint(_account, _amount);
  }

  /**
    Callback called after a token transfer.

    @param _from The account sending tokens.
    @param _to The account receiving tokens.
    @param _amount The amount of tokens being transferred.
  */
  function _afterTokenTransfer (
    address _from,
    address _to,
    uint256 _amount
  ) internal override(ERC20, ERC20Votes) {
    super._afterTokenTransfer(_from, _to, _amount);
  }

  /**
    Internal mint function.

    @param _to The account receiving minted tokens.
    @param _amount The amount of tokens to mint.
  */
  function _mint (
    address _to,
    uint256 _amount
  ) internal override(ERC20, ERC20Votes) {
    super._mint(_to, _amount);
  }

  /**
    Internal burn function.

    @param _account The account that tokens will be burned from.
    @param _amount The amount of tokens that will be burned.
  */
  function _burn (
    address _account,
    uint256 _amount
  ) internal override(ERC20, ERC20Votes) {
    super._burn(_account, _amount);
  }
}
