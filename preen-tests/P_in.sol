// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity ^0.8.0;

interface IThing {
    function read() external view returns (uint256);
}

/// @notice Regression demo for two preen bugs.
contract P {
    /// Bug 1: `for (...) <stmt>;` — a `for` loop whose single-statement
    /// body has no braces. dove silently DROPS the body, replacing it
    /// with `{}`. The output compiles but does the wrong thing.
    ///
    /// Note that the analogous shapes for `while` and `if` are handled
    /// correctly:
    ///   while (cond) stmt;   → preserved verbatim
    ///   if (cond) stmt;      → wrapped to `{ stmt; }`
    /// Only `for` is broken.
    function sumTo(uint256 n) external pure returns (uint256) {
        uint256 acc = 0;
        for (uint256 i = 0; i < n; i++) acc += i;
        return acc;
    }

    /// Bug 2: dove's underscore-prefix rename pass rewrites the
    /// `try`-returns binding from `v` to `_v` in the *binding position*,
    /// but does not rewrite references to `v` inside the try-block. The
    /// output references an undefined identifier and won't compile.
    function tryRead(IThing t) external view returns (uint256) {
        try t.read() returns (uint256 v) {
            return v;
        } catch {
            return 0;
        }
    }
}
