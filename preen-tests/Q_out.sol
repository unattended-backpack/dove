// SPDX-License-Identifier: LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only
pragma solidity 0.8.15;

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title Quail
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  A fixture for converting named return values into explicit locals. This second
  paragraph exists to demonstrate how paragraph breaks survive (or do not
  survive) header regeneration. Named returns assigned via compound assignment,
  assigned inside unchecked blocks, and assigned as tuples must all convert into
  explicitly declared, correctly scoped, and explicitly returned locals.

  @custom:date TODO.
*/
contract Quail {

  /// The running total of all skimmed excess.
  uint256 public skimmed;

  /**
    Sums an array of values.

    @param _values TODO

    @return _ TODO
  */
  function sum (
    uint256[] calldata _values
  ) external pure returns (uint256) {
    uint256 _totalOutput;
    for (uint256 i = 0; i < _values.length; i++) {
      _totalOutput += _values[i];
    }
    return _totalOutput;
  }

  /**
    Skims the excess of a balance above a floor.

    @param _balance TODO
    @param _floor TODO

    @return _ TODO
  */
  function skim (
    uint256 _balance,
    uint256 _floor
  ) external returns (uint256) {
    uint256 _excessOutput;
    if (_balance <= _floor) {
      return 0;
    }
    unchecked {
      _excessOutput = _balance - _floor;
    }
    skimmed += _excessOutput;
    return _excessOutput;
  }

  /**
    Splits an amount into a half and a remainder.

    @param _amount TODO

    @return _ TODO
  */
  function split (
    uint256 _amount
  ) external pure returns (uint256, uint256) {
    uint256 _halfOutput;
    uint256 _remainderOutput;
    if (_amount > 1) {
      _halfOutput = _amount / 2;
      _remainderOutput = _amount - _halfOutput;
    }
    return (_halfOutput, _remainderOutput);
  }

  /**
    Remembers the prior skim total while updating it.

    @param _amount TODO

    @return _ TODO
  */
  function record (
    uint256 _amount
  ) external returns (uint256) {
    uint256 _previous = skimmed;
    skimmed = _amount;
    return _previous;
  }

  /**
    Finds the last value in an array.

    @param _values TODO

    @return _ TODO
  */
  function lastOf (
    uint256[] calldata _values
  ) external pure returns (uint256) {
    uint256 _lastOutput;
    for (uint256 i = 0; i < _values.length; i++) {
      _lastOutput = _values[i];
    }
    return _lastOutput;
  }

  /**
    Doubles a value.

    @param _value TODO

    @return _ TODO
  */
  function double (
    uint256 _value
  ) external pure returns (uint256) {
    return _value * 2;
  }

  /**
    Caps a value: anything at or below the ceiling reports zero.

    @param _value TODO
    @param _ceiling TODO

    @return _ TODO
  */
  function cap (
    uint256 _value,
    uint256 _ceiling
  ) external pure returns (uint256) {
    if (_value > _ceiling) {
      return _ceiling;
    }
  }

  /**
    Computes an unchecked difference.

    @param _a TODO
    @param _b TODO

    @return _ TODO
  */
  function diff (
    uint256 _a,
    uint256 _b
  ) external pure returns (uint256) {
    unchecked {
      return _a - _b;
    }
  }
}
