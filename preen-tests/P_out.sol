// SPDX-License-Identifier: LicenseRef-(SEPPUKU WITH VPL) WITH AGPL-3.0-only
pragma solidity ^0.8.0;

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title IThing
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  TODO

  @custom:date TODO.
*/
interface IThing {

  /**
    TODO

    @return _ TODO
  */
  function read () external view returns (uint256);
}

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title P
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  Regression demo for two preen bugs.

  @custom:date TODO.
*/
contract P {

  /**
    Bug 1: `for (...) <stmt>;` — a `for` loop whose single-statement body has no
    braces. dove silently DROPS the body, replacing it with `{}`. The output
    compiles but does the wrong thing. Note that the analogous shapes for
    `while` and `if` are handled correctly: while (cond) stmt; → preserved
    verbatim if (cond) stmt; → wrapped to `{ stmt; }` Only `for` is broken.

    @param _n TODO

    @return _ TODO
  */
  function sumTo (
    uint256 _n
  ) external pure returns (uint256) {
    uint256 _acc = 0;
    for (uint256 i = 0; i < _n; i++) {
      _acc += i;
    }
    return _acc;
  }

  /**
    Bug 2: dove's underscore-prefix rename pass rewrites the `try`-returns
    binding from `v` to `_v` in the *binding position*, but does not rewrite
    references to `v` inside the try-block. The output references an undefined
    identifier and won't compile.

    @param _t TODO

    @return _ TODO
  */
  function tryRead (
    IThing _t
  ) external view returns (uint256) {
    try _t.read() returns (uint256 _v) {
      return _v;
    } catch {
      return 0;
    }
  }
}
