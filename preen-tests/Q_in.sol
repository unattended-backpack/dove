// SPDX-License-Identifier: MIT
pragma solidity 0.8.15;

/// @title Quail
/// @notice A fixture for converting named return values into explicit locals.
///
///         This second paragraph exists to demonstrate how paragraph breaks
///         survive (or do not survive) header regeneration.
/// @dev Named returns assigned via compound assignment, assigned inside
///      unchecked blocks, and assigned as tuples must all convert into
///      explicitly declared, correctly scoped, and explicitly returned
///      locals.
contract Quail {
    /// @notice The running total of all skimmed excess.
    uint256 public skimmed;

    /// @notice Sums an array of values.
    function sum(uint256[] calldata values) external pure returns (uint256 total) {
        for (uint256 i = 0; i < values.length; i++) {
            total += values[i];
        }
    }

    /// @notice Skims the excess of a balance above a floor.
    function skim(uint256 balance, uint256 floor) external returns (uint256 excess) {
        if (balance <= floor) {
            return 0;
        }
        unchecked {
            excess = balance - floor;
        }
        skimmed += excess;
    }

    /// @notice Splits an amount into a half and a remainder.
    function split(uint256 amount) external pure returns (uint256 half, uint256 remainder) {
        if (amount > 1) {
            half = amount / 2;
            remainder = amount - half;
        }
    }

    /// @notice Remembers the prior skim total while updating it.
    function record(uint256 amount) external returns (uint256 previous) {
        previous = skimmed;
        skimmed = amount;
    }

    /// @notice Finds the last value in an array.
    function lastOf(uint256[] calldata values) external pure returns (uint256 last) {
        for (uint256 i = 0; i < values.length; i++) {
            last = values[i];
        }
    }

    /// @notice Doubles a value.
    function double(uint256 value) external pure returns (uint256 doubled) {
        doubled = value * 2;
    }

    /// @notice Caps a value: anything at or below the ceiling reports zero.
    function cap(uint256 value, uint256 ceiling) external pure returns (uint256 capped) {
        if (value > ceiling) {
            capped = ceiling;
        }
    }

    /// @notice Computes an unchecked difference.
    function diff(uint256 a, uint256 b) external pure returns (uint256 d) {
        unchecked {
            d = a - b;
        }
    }
}
