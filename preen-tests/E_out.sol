// SPDX-License-Identifier: MIT AND (LicenseRef-VPL WITH AGPL-3.0-only)
pragma solidity 0.8.15;

import { Initializable } from
  "@openzeppelin/contracts/proxy/utils/Initializable.sol";
import { Math } from "@openzeppelin/contracts/utils/math/Math.sol";

import { Arithmetic } from "src/libraries/Arithmetic.sol";
import { Burn } from "src/libraries/Burn.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title ResourceMetering
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"
  @custom:upgradeable

  ResourceMetering implements an EIP-1559 style resource metering system where
  pricing updates automatically based on current demand.

  @custom:date June 20th, 2025.
*/
abstract contract ResourceMetering is
  Initializable {

  /**
    Represents the various parameters that control the way in which resources
    are metered. Corresponds to the EIP-1559 resource metering system.

    @param prevBaseFee Base fee from the previous block(s).
    @param prevBoughtGas Amount of gas bought so far in the current block.
    @param prevBlockNum Last block number that the base fee was updated.
  */
  struct ResourceParams {
    uint128 prevBaseFee;
    uint64 prevBoughtGas;
    uint64 prevBlockNum;
  }

  /**
    Represents the configuration for the EIP-1559 based curve for the deposit
    gas market. These values should be set with care as it is possible to set
    them in a way that breaks the deposit gas market. The target resource limit
    is defined as maxResourceLimit / elasticityMultiplier. This struct was
    designed to fit within a single word. There is additional space for
    additions in the future.

    @param maxResourceLimit Represents the maximum amount of deposit gas that
      can be purchased per block.
    @param elasticityMultiplier Determines the target resource limit along with
      the resource limit.
    @param baseFeeMaxChangeDenominator Determines max change on fee per block.
    @param minimumBaseFee The min deposit base fee, it is clamped to this value.
    @param systemTxMaxGas The amount of gas supplied to the system transaction.
      This should be set to the same number that the op-node sets as the gas
      limit for the system transaction.
    @param maximumBaseFee The max deposit base fee, it is clamped to this value.
  */
  struct ResourceConfig {
    uint32 maxResourceLimit;
    uint8 elasticityMultiplier;
    uint8 baseFeeMaxChangeDenominator;
    uint32 minimumBaseFee;
    uint32 systemTxMaxGas;
    uint128 maximumBaseFee;
  }

  /// Error returned when too much gas resource is consumed.
  error OutOfGas ();

  /// EIP-1559 style gas parameters.
  ResourceParams public params;

  /**
    Reserve extra slots (to a total of 50) in the storage layout for future
    upgrades.
  */
  uint256[48] private __gap;

  /**
    Meters access to a function based an amount of a requested resource.

    @param _amount Amount of the resource requested.
  */
  modifier metered (
    uint64 _amount
  ) {

    // Record initial gas amount so we can refund for it later.
    uint256 _initialGas = gasleft();

    // Run the underlying function.
    _;

    // Run the metering function.
    _metered(_amount, _initialGas);
  }

  /**
    An internal function that holds all of the logic for metering a resource.

    @param _amount Amount of the resource requested.
    @param _initialGas The amount of gas before any modifier execution.
  */
  function _metered (
    uint64 _amount,
    uint256 _initialGas
  ) internal {

    // Update block number and base fee if necessary.
    uint256 _blockDiff = block.number - params.prevBlockNum;
    ResourceConfig memory _config = _resourceConfig();
    int256 _targetResourceLimit =
      int256(uint256(_config.maxResourceLimit)) / int256(
        uint256(_config.elasticityMultiplier)
      );
    if (_blockDiff > 0) {

      /*
        Handle updating EIP-1559 style gas parameters. We use EIP-1559 to
        restrict the rate at which deposits can be created and therefore limit
        the potential for deposits to spam the L2 system. Fee scheme is very
        similar to EIP-1559 with minor changes.
      */
      int256 _gasUsedDelta =
        int256(uint256(params.prevBoughtGas)) - _targetResourceLimit;
      int256 _baseFeeDelta =
        (int256(uint256(params.prevBaseFee)) * _gasUsedDelta) / (
          _targetResourceLimit * int256(
            uint256(_config.baseFeeMaxChangeDenominator)
          )
        );

      /*
        Update base fee by adding the base fee delta and clamp the resulting
        value between min and max.
      */
      int256 _newBaseFee = Arithmetic.clamp({
        _value: int256(uint256(params.prevBaseFee)) + _baseFeeDelta,
        _min: int256(uint256(_config.minimumBaseFee)),
        _max: int256(uint256(_config.maximumBaseFee))
      });

      /*
        If we skipped more than one block, we also need to account for every
        empty block. Empty block means there was no demand for deposits in that
        block, so we should reflect this lack of demand in the fee.
      */
      if (_blockDiff > 1) {

        /*
          Update the base fee by repeatedly applying the exponent
          1-(1/change_denominator) blockDiff - 1 times. Simulates multiple empty
          blocks. Clamp the resulting value between min and max.
        */
        _newBaseFee = Arithmetic.clamp({
          _value: Arithmetic.cdexp({
            _coefficient: _newBaseFee,
            _denominator: int256(uint256(_config.baseFeeMaxChangeDenominator)),
            _exponent: int256(_blockDiff - 1)
          }),
          _min: int256(uint256(_config.minimumBaseFee)),
          _max: int256(uint256(_config.maximumBaseFee))
        });
      }

      // Update new base fee, reset bought gas, and update block number.
      params.prevBaseFee = uint128(uint256(_newBaseFee));
      params.prevBoughtGas = 0;
      params.prevBlockNum = uint64(block.number);
    }

    // Make sure we can actually buy the resource amount requested by the user.
    params.prevBoughtGas += _amount;
    if (
      int256(uint256(params.prevBoughtGas)) > int256(
        uint256(_config.maxResourceLimit)
      )
    ) {
      revert OutOfGas();
    }

    // Determine the amount of ETH to be paid.
    uint256 _resourceCost = uint256(_amount) * uint256(params.prevBaseFee);

    /*
      We currently charge for this ETH amount as an L1 gas burn, so we convert
      the ETH amount into gas by dividing by the L1 base fee. We assume a
      minimum base fee of 1 gwei to avoid division by zero for L1s that don't
      support 1559 or to avoid excessive gas burns during periods of extremely
      low L1 demand. One-day average gas fee hasn't dipped below 1 gwei during
      any 1 day period in the last 5 years, so should be fine.
    */
    uint256 _gasCost = _resourceCost / Math.max(block.basefee, 1 gwei);

    /*
      Give the user a refund based on the amount of gas they used to do all of
      the work up to this point. Since we're at the end of the modifier, this
      should be pretty accurate. Acts effectively like a dynamic stipend (with a
      minimum value).
    */
    uint256 _usedGas = _initialGas - gasleft();
    if (_gasCost > _usedGas) {
      Burn.gas(_gasCost - _usedGas);
    }
  }

  /**
    Adds an amount of L2 gas consumed to the prev bought gas params. This is
    meant to be used when L2 system transactions are generated from L1.

    @param _amount Amount of the L2 gas resource requested.
  */
  function useGas (
    uint32 _amount
  ) internal {
    params.prevBoughtGas += uint64(_amount);
  }

  /**
    Virtual function that returns the resource config. Contracts that inherit
    this contract must implement this function.

    @return _ ResourceConfig
  */
  function _resourceConfig () internal virtual returns (ResourceConfig memory);

  /**
    Sets initial resource parameter values. This function must either be called
    by the initializer function of an upgradeable child contract.
  */
  function __ResourceMetering_init () internal onlyInitializing {
    if (params.prevBlockNum == 0) {
      params = ResourceParams({
        prevBaseFee: 1 gwei,
        prevBoughtGas: 0,
        prevBlockNum: uint64(block.number)
      });
    }
  }
}
