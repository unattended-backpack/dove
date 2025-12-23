// SPDX-License-Identifier: MIT AND (LicenseRef-VPL WITH AGPL-3.0-only)
pragma solidity ^0.8.20;

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title MulticallDelegate
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  Generic multicall contract for EIP-7702 delegation EOAs can delegate to this
  contract to gain batched call capability. When an EOA authorizes this contract
  via 7702, calls to the EOA execute this code with address(this) = EOA,
  preserving msg.sender semantics for all subcalls.

  @custom:date TODO.
*/
contract MulticallDelegate {

  /**
    TODO

    @param value TODO
    @param target TODO
    @param data TODO
  */
  struct Call {
    uint256 value;
    address target;
    bytes data;
  }

  /**
    Execute multiple calls in a single transaction Only the delegating EOA can
    call this (msg.sender must equal address(this))

    @param _calls Array of calls to execute

    @return _ Array of return data from each call
  */
  function multicall (
    Call[] calldata _calls
  ) external returns (bytes[] memory) {
    require(msg.sender == address(this), "MulticallDelegate: unauthorized");
    bytes[] memory _results = new bytes[](_calls.length);
    for (uint256 i = 0; i < _calls.length; i++) {
      (bool _success, bytes memory _result) = _calls[i].target.call{
        value: _calls[i].value
      }(
        _calls[i].data
      );
      require(_success, "MulticallDelegate: call failed");
      _results[i] = _result;
    }
    return _results;
  }

  /**
    Execute multiple calls, allowing failures Only the delegating EOA can call
    this (msg.sender must equal address(this))

    @param _calls Array of calls to execute

    @return _ Array of success flags for each call
    @return _ Array of return data from each call
  */
  function tryMulticall (
    Call[] calldata _calls
  ) external returns (bool[] memory, bytes[] memory) {
    require(msg.sender == address(this), "MulticallDelegate: unauthorized");
    bool[] memory _successes = new bool[](_calls.length);
    bytes[] memory _results = new bytes[](_calls.length);
    for (uint256 i = 0; i < _calls.length; i++) {
      (_successes[i], _results[i]) = _calls[i].target.call{
        value: _calls[i].value
      }(
        _calls[i].data
      );
    }
    return (_successes, _results);
  }
}
