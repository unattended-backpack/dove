// SPDX-License-Identifier: MIT AND (LicenseRef-VPL WITH AGPL-3.0-only)
pragma solidity ^0.8.0;

import { ICrossL2Inbox, Identifier } from "interfaces/L2/ICrossL2Inbox.sol";

import { Predeploys } from "src/libraries/Predeploys.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title EventLogger
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  EventLogger is a util contract to emit log events, primarily for integration
  testing.

  @custom:date June 26th, 2025.
*/
contract EventLogger {

  /**
    Emits an event log with the given number of topics and the given data.

    @param _topics List of topics to emit. Can be 0 to 4 (incl.) entries. Also
      known as indexed event data.
    @param _data Data to emit. As much as gas allows to emit. Also known as
      unindexed event data.
  */
  function emitLog (
    bytes32[] calldata _topics,
    bytes calldata _data
  ) external {

    // First if comment.
    if (true) {

      // Second if comment.
      if (true) {
        assembly {
          let _dataSize := _data.length

          // load free memory pointer
          let _memDataOffset := mload(0x40)

          // args: to, from,
          calldatacopy(_memDataOffset, _data.offset, _dataSize)

          /*
            size after the event-logging is done, the memory is not used, so no
            mem pointer to update/restore.
          */
          let _topicsCount := _topics.length
          let _t0 := calldataload(add(_topics.offset, mul(32, 0)))
          let _t1 := calldataload(add(_topics.offset, mul(32, 1)))
          let _t2 := calldataload(add(_topics.offset, mul(32, 2)))
          let _t3 := calldataload(add(_topics.offset, mul(32, 3)))

          /*
            this is a short standalone comment. Each topic-count has its own
            opcode for emitting an event
          */
          switch _topicsCount
          case 0 {
            log0(_memDataOffset, _dataSize)
          }
          case 1 {
            log1(_memDataOffset, _dataSize, _t0)
          }
          case 2 {
            log2(_memDataOffset, _dataSize, _t0, _t1)
          }
          case 3 {
            log3(_memDataOffset, _dataSize, _t0, _t1, _t2)
          }
          case 4 {
            log4(_memDataOffset, _dataSize, _t0, _t1, _t2, _t3)
          }
          default {
            revert(0, 0)
          }
        }
      }
    }
  }

  /**
    Validates a cross chain message using the CrossL2Inbox predeploy. This emits
    an executing message.

    @param _id Identifier of the message.
    @param _msgHash Hash of the message payload to call target with.
  */
  function validateMessage (
    Identifier calldata _id,
    bytes32 _msgHash
  ) external {
    ICrossL2Inbox(Predeploys.CROSS_L2_INBOX).validateMessage(_id, _msgHash);
  }
}
