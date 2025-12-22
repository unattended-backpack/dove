// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title MulticallDelegate
/// @notice Generic multicall contract for EIP-7702 delegation
/// @dev EOAs can delegate to this contract to gain batched call capability.
///      When an EOA authorizes this contract via 7702, calls to the EOA
///      execute this code with address(this) = EOA, preserving msg.sender
///      semantics for all subcalls.
contract MulticallDelegate {
    struct Call {
        uint256 value;
        address target;
        bytes data;
    }

    /// @notice Execute multiple calls in a single transaction
    /// @dev Only the delegating EOA can call this (msg.sender must equal address(this))
    /// @param calls Array of calls to execute
    /// @return results Array of return data from each call
    function multicall(Call[] calldata calls) external returns (bytes[] memory results) {
        require(msg.sender == address(this), "MulticallDelegate: unauthorized");
        results = new bytes[](calls.length);
        for (uint256 i = 0; i < calls.length; i++) {
            (bool success, bytes memory result) = calls[i].target.call{value: calls[i].value}(calls[i].data);
            require(success, "MulticallDelegate: call failed");
            results[i] = result;
        }
    }

    /// @notice Execute multiple calls, allowing failures
    /// @dev Only the delegating EOA can call this (msg.sender must equal address(this))
    /// @param calls Array of calls to execute
    /// @return successes Array of success flags for each call
    /// @return results Array of return data from each call
    function tryMulticall(Call[] calldata calls)
        external
        returns (bool[] memory successes, bytes[] memory results)
    {
        require(msg.sender == address(this), "MulticallDelegate: unauthorized");
        successes = new bool[](calls.length);
        results = new bytes[](calls.length);
        for (uint256 i = 0; i < calls.length; i++) {
            (successes[i], results[i]) = calls[i].target.call{value: calls[i].value}(calls[i].data);
        }
    }
}
