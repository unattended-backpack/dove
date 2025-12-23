pragma solidity 0.8.15;

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title A testing contract.
  @author Tim Clancy <tim-clancy.eth>
  @custom:terry "How is twenty times four 80?!"

  This contract has a very long description on a single line here. It really keeps running on and on and on. The formatter should make this one look beautiful.

  @custom:date June 3rd, 2025.
*/
contract Test { uint256 testValue;

  /**
    Represents a storage slot key value pair.

          @param key The storage slot key.
    @param value The storage slot value.
  */
  struct Slot {
    bytes32 key;
    bytes32 value;
  }

  /// @notice Represents a proven withdrawal.
  /// @custom:field timestamp        Timestamp at which the withdrawal was
  /// proven.
  /// @custom:field disputeGameProxy Game that the withdrawal was proven
  /// against. This is a really long
  /// description.
  struct ProvenWithdrawal {
    IDisputeGame disputeGameProxy;
    uint64 timestamp;
  }

  struct Cat {
    string name;
    string color;
  }

  /// This struct represents an animal.
  /// @param name The name of the animal.
  /// @custom:field color The color of the animal.
  struct Animal {
    string name;
    string color;
    string bite;
  }
}
