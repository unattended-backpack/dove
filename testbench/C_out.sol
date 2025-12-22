// SPDX-License-Identifier: MIT AND (LicenseRef-VPL WITH AGPL-3.0-only)
pragma solidity 0.8.15;

import { Initializable } from
  "@openzeppelin/contracts/proxy/utils/Initializable.sol";

import { IETHLockbox } from "interfaces/L1/IETHLockbox.sol";
import { IOptimismPortal2 as IOptimismPortal } from
  "interfaces/L1/IOptimismPortal2.sol";
import { ISuperchainConfig } from "interfaces/L1/ISuperchainConfig.sol";
import { ISystemConfig } from "interfaces/L1/ISystemConfig.sol";
import { ISemver } from "interfaces/universal/ISemver.sol";

import { ProxyAdminOwnedBase } from "src/L1/ProxyAdminOwnedBase.sol";
import { Constants } from "src/libraries/Constants.sol";
import { ReinitializableBase } from "src/universal/ReinitializableBase.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title ETHLockbox
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"
  @custom:proxied true

  Manages ETH liquidity locking and unlocking for authorized OptimismPortals,
  enabling unified ETH liquidity management across chains in the superchain
  cluster.

  @custom:date June 18th, 2025.
*/
contract ETHLockbox is
  ProxyAdminOwnedBase,
  Initializable,
  ReinitializableBase,
  ISemver {

  /// Thrown when the lockbox is paused.
  error ETHLockbox_Paused ();

  /// Thrown when the caller is not authorized.
  error ETHLockbox_Unauthorized ();

  /**
    Thrown when the value to unlock is greater than the balance of the lockbox.
  */
  error ETHLockbox_InsufficientBalance ();

  /**
    Thrown when attempting to unlock ETH from the lockbox through a withdrawal
    transaction.
  */
  error ETHLockbox_NoWithdrawalTransactions ();

  /// Thrown when any authorized portal has a different SuperchainConfig.
  error ETHLockbox_DifferentSuperchainConfig ();

  /**
    Emitted when ETH is locked in the lockbox by an authorized portal.

    @param portal The address of the portal that locked the ETH.
    @param amount The amount of ETH locked.
  */
  event ETHLocked (
    IOptimismPortal indexed portal,
    uint256 amount
  );

  /**
    Emitted when ETH is unlocked from the lockbox by an authorized portal.

    @param portal The address of the portal that unlocked the ETH.
    @param amount The amount of ETH unlocked.
  */
  event ETHUnlocked (
    IOptimismPortal indexed portal,
    uint256 amount
  );

  /**
    Emitted when a portal is authorized to lock and unlock ETH.

    @param portal The address of the portal that was authorized.
  */
  event PortalAuthorized (
    IOptimismPortal indexed portal
  );

  /**
    Emitted when an ETH lockbox is authorized to migrate its liquidity to the
    current ETH lockbox.

    @param lockbox The address of the ETH lockbox that was authorized.
  */
  event LockboxAuthorized (
    IETHLockbox indexed lockbox
  );

  /**
    Emitted when ETH liquidity is migrated from the current ETH lockbox to
    another.

    @param lockbox The address of the ETH lockbox that was migrated.
    @param amount TODO
  */
  event LiquidityMigrated (
    IETHLockbox indexed lockbox,
    uint256 amount
  );

  /**
    Emitted when ETH liquidity is received during an authorized lockbox
    migration.

    @param lockbox The address of the ETH lockbox that received the liquidity.
    @param amount The amount of ETH received.
  */
  event LiquidityReceived (
    IETHLockbox indexed lockbox,
    uint256 amount
  );

  /// The address of the SystemConfig contract.
  ISystemConfig public systemConfig;

  /**
    Mapping of authorized portals.

    @param _portal The address of an `OptimismPortal`.

    @return _isAuthorized Whether or not the `portal` is authorized.
  */
  mapping (
    IOptimismPortal _portal => bool _isAuthorized
  ) public authorizedPortals;

  /**
    Mapping of authorized lockboxes.

    @param TODO
    @param TODO

    @return TODO
  */
  mapping (
    IETHLockbox TODO => mapping (
      address TODO => bool TODO
    )
  ) public authorizedLockboxes;

  /**
    Semantic version.

    @return _ TODO
  */
  function version () public view virtual returns (string memory) {
    return "1.2.0";
  }

  /// Constructs the ETHLockbox contract.
  constructor () ReinitializableBase(1) {
    _disableInitializers();
  }

  /**
    Initializer. Note: Multiple chains can share an ETHLockbox contract. In this
    case, all SystemConfig contracts will point to the same pause identifier
    (the lockbox itself). Therefore, it doesn't matter which SystemConfig is
    used here as long as it belongs to one of the chains that share the lockbox.

    @param _systemConfig The address of the SystemConfig contract.
    @param _portals The addresses of the portals to authorize.
  */
  function initialize (
    ISystemConfig _systemConfig,
    IOptimismPortal[] calldata _portals
  ) external reinitializer(initVersion()) {

    // Initialization transactions must come from the ProxyAdmin or its owner.
    _assertOnlyProxyAdminOrProxyAdminOwner();

    // Now perform initialization logic.
    systemConfig = _systemConfig;
    for (uint256 i; i < _portals.length; i++) {
      _authorizePortal(_portals[i]);
    }
  }

  /**
    Getter for the current paused status.

    @return _ TODO
  */
  function paused () public view returns (bool) {
    return systemConfig.paused();
  }

  /**
    Returns the SuperchainConfig contract.

    @return _ ISuperchainConfig The SuperchainConfig contract.
  */
  function superchainConfig () public view returns (ISuperchainConfig) {
    return systemConfig.superchainConfig();
  }

  /**
    Authorizes a portal to lock and unlock ETH.

    @param _portal The address of the portal to authorize.
  */
  function authorizePortal (
    IOptimismPortal _portal
  ) external {

    // Check that this transaction is coming from the ProxyAdmin owner.
    _assertOnlyProxyAdminOwner();

    // Authorize the portal.
    _authorizePortal(_portal);
  }

  /// Receives the ETH liquidity migrated from an authorized lockbox.
  function receiveLiquidity () external payable {

    // Check that the sender is authorized to trigger this function.
    IETHLockbox _sender = IETHLockbox(payable(msg.sender));
    if (!authorizedLockboxes[_sender]) {
      revert ETHLockbox_Unauthorized();
    }

    // Emit the event.
    emit LiquidityReceived(_sender, msg.value);
  }

  /**
    Locks ETH in the lockbox. Called by an authorized portal on a deposit to
    lock the ETH value.
  */
  function lockETH () external payable {

    // Check that the sender is authorized to trigger this function.
    IOptimismPortal _sender = IOptimismPortal(payable(msg.sender));
    if (!authorizedPortals[_sender]) {
      revert ETHLockbox_Unauthorized();
    }

    // Emit the event.
    emit ETHLocked(_sender, msg.value);
  }

  /**
    Unlocks ETH from the lockbox. Called by an authorized portal when finalizing
    a withdrawal that requires ETH. Cannot be called if the lockbox is paused.

    @param _value The amount of ETH to unlock.
  */
  function unlockETH (
    uint256 _value
  ) external {

    // Unlocks are blocked when paused, locks are not.
    if (paused()) {
      revert ETHLockbox_Paused();
    }

    // Check that the sender is authorized to trigger this function.
    IOptimismPortal _sender = IOptimismPortal(payable(msg.sender));
    if (!authorizedPortals[_sender]) {
      revert ETHLockbox_Unauthorized();
    }

    // Check that we have enough balance to process the unlock.
    if (_value > address(this).balance) {
      revert ETHLockbox_InsufficientBalance();
    }

    // Check that the sender is not executing a withdrawal transaction.
    if (_sender.l2Sender() != Constants.DEFAULT_L2_SENDER) {
      revert ETHLockbox_NoWithdrawalTransactions();
    }

    // Using donateETH to avoid triggering a deposit.
    _sender.donateETH{ value: _value }();

    // Emit the event.
    emit ETHUnlocked(_sender, _value);
  }

  /**
    Authorizes an ETH lockbox to migrate its liquidity to the current ETH
    lockbox. We allow this function to be called more than once for the same
    lockbox. A lockbox cannot be removed from the authorized list once added.

    @param _lockbox The address of the ETH lockbox to authorize.
  */
  function authorizeLockbox (
    IETHLockbox _lockbox
  ) external {

    // Check that this transaction is coming from the ProxyAdmin owner.
    _assertOnlyProxyAdminOwner();

    // Check that the lockbox has the same proxy admin owner.
    _assertSharedProxyAdminOwner(address(_lockbox));

    // Authorize the lockbox.
    authorizedLockboxes[_lockbox] = true;

    // Emit the event.
    emit LockboxAuthorized(_lockbox);
  }

  /**
    Migrates liquidity from the current ETH lockbox to another. Must be called
    atomically with `OptimismPortal.migrateToSuperRoots()` in the same
    transaction batch, or otherwise the OptimismPortal may not be able to unlock
    ETH from the ETHLockbox on finalized withdrawals.

    @param _lockbox The address of the ETH lockbox to migrate liquidity to.
  */
  function migrateLiquidity (
    IETHLockbox _lockbox
  ) external {

    // Check that this transaction is coming from the ProxyAdmin owner.
    _assertOnlyProxyAdminOwner();

    // Check that the lockbox has the same proxy admin owner.
    _assertSharedProxyAdminOwner(address(_lockbox));

    // Receive the liquidity.
    uint256 _balance = address(this).balance;
    IETHLockbox(_lockbox).receiveLiquidity{ value: _balance }();

    // Emit the event.
    emit LiquidityMigrated(_lockbox, _balance);
  }

  /**
    Authorizes a portal to lock and unlock ETH.

    @param _portal The address of the portal to authorize.
  */
  function _authorizePortal (
    IOptimismPortal _portal
  ) internal {

    // Check that the portal has the same proxy admin owner.
    _assertSharedProxyAdminOwner(address(_portal));

    // Check that the portal has the same superchain config.
    if (_portal.superchainConfig() != superchainConfig()) {
      revert ETHLockbox_DifferentSuperchainConfig();
    }

    // Authorize the portal.
    authorizedPortals[_portal] = true;

    // Emit the event.
    emit PortalAuthorized(_portal);
  }
}
