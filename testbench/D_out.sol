// SPDX-License-Identifier: MIT AND (LicenseRef-VPL WITH AGPL-3.0-only)
pragma solidity 0.8.15;

import { OwnableUpgradeable } from
  "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";

import { ISemver } from "interfaces/universal/ISemver.sol";

import { SafeCall } from "src/libraries/SafeCall.sol";

/**
  An enum representing the status of a DA challenge.

  @param Uninitialized TODO
  @param Active TODO
  @param Resolved TODO
  @param Expired TODO
*/
enum ChallengeStatus {
  Uninitialized,
  Active,
  Resolved,
  Expired
}

/**
  An enum representing known commitment types.

  @param Keccak256 TODO
*/
enum CommitmentType {
  Keccak256
}

/**
  A struct representing a single DA challenge.

  @param challenger The address that initiated the challenge.
  @param lockedBond The amount of ETH bond that was locked by the challenger.
  @param startBlock The block number at which the challenge was initiated.
  @param resolvedBlock The block number at which the challenge was resolved.
*/
struct Challenge {
  address challenger;
  uint256 lockedBond;
  uint256 startBlock;
  uint256 resolvedBlock;
}

/**
  Compute the expected commitment for a given blob of data.

  @param _data The blob of data to compute a commitment for.

  @return _ The commitment for the given blob of data.
*/
function computeCommitmentKeccak256 (
  bytes memory _data
) pure returns (bytes memory) {
  return bytes.concat(
    bytes1(uint8(CommitmentType.Keccak256)), keccak256(_data)
  );
}

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title DataAvailabilityChallenge
  @author TODO
  @custom:terry "Is this too much voodoo for the next ten centuries?"
  @custom:proxied true

  This contract enables data availability of a data commitment at a given block
  number to be challenged. To challenge a commitment, the challenger must first
  post a bond (bondSize). Challenging a commitment is only possible within a
  certain block interval (challengeWindow) after the commitment was made. If the
  challenge is not resolved within a certain block interval (resolveWindow), the
  challenge can be expired. If a challenge is expired, the challenger's bond is
  unlocked and the challenged commitment is added to the chain of expired
  challenges.

  @custom:date June 18th, 2025.
*/
contract DataAvailabilityChallenge is
  OwnableUpgradeable,
  ISemver {

  /**
    Error for when the provided resolver refund percentage exceeds 100%.

    @param invalidResolverRefundPercentage TODO
  */
  error InvalidResolverRefundPercentage (
    uint256 invalidResolverRefundPercentage
  );

  /**
    Error for when the challenger's bond is too low.

    @param balance TODO
    @param required TODO
  */
  error BondTooLow (
    uint256 balance,
    uint256 required
  );

  /**
    Error for when attempting to challenge a commitment that already has a
    challenge.
  */
  error ChallengeExists ();

  /// Error for when attempting to resolve a challenge that is not active.
  error ChallengeNotActive ();

  /**
    Error for when attempting to unlock a bond from a challenge that is not
    expired.
  */
  error ChallengeNotExpired ();

  /**
    Error for when attempting to challenge a commitment that is not in the
    challenge window.
  */
  error ChallengeWindowNotOpen ();

  /**
    Error for when the provided input data doesn't match the commitment.

    @param providedDataCommitment TODO
    @param expectedCommitment TODO
  */
  error InvalidInputData (
    bytes providedDataCommitment,
    bytes expectedCommitment
  );

  /// Error for when the call to withdraw a bond failed.
  error WithdrawalFailed ();

  /**
    Error for when a the type of a given commitment is unknown

    @param commitmentType TODO
  */
  error UnknownCommitmentType (
    uint8 commitmentType
  );

  /**
    Error for when the commitment length does not match the commitment type

    @param commitmentType TODO
    @param expectedLength TODO
    @param actualLength TODO
  */
  error InvalidCommitmentLength (
    uint8 commitmentType,
    uint256 expectedLength,
    uint256 actualLength
  );

  /**
    An event that is emitted when the status of a challenge changes.

    @param challengedBlockNumber The block number at which the commitment was
      made.
    @param challengedCommitment The commitment that is being challenged.
    @param status The new status of the challenge.
  */
  event ChallengeStatusChanged (
    uint256 indexed challengedBlockNumber,
    bytes challengedCommitment,
    ChallengeStatus status
  );

  /**
    An event that is emitted when the bond size required to initiate a challenge
    changes.

    @param challengeWindow TODO
  */
  event RequiredBondSizeChanged (
    uint256 challengeWindow
  );

  /**
    An event that is emitted when the percentage of the resolving cost to be
    refunded to the resolver changes.

    @param resolverRefundPercentage TODO
  */
  event ResolverRefundPercentageChanged (
    uint256 resolverRefundPercentage
  );

  /**
    An event that is emitted when a user's bond balance changes.

    @param account TODO
    @param balance TODO
  */
  event BalanceChanged (
    address account,
    uint256 balance
  );

  /**
    Semantic version.
    @custom:semver 1.0.1
  */
  string public constant version = "1.0.1";

  /**
    The fixed cost of resolving a challenge. The value is estimated by measuring
    the cost of resolving with `bytes(0)`
  */
  uint256 public constant fixedResolutionCost = 72925;

  /**
    The variable cost of resolving a callenge per byte scaled by the
    variableResolutionCostPrecision. upper limit; The value is estimated by
    measuring the cost of resolving with variable size data where each byte is
    non-zero.
  */
  uint256 public constant variableResolutionCost = 16640;

  /// The precision of the variable resolution cost.
  uint256 public constant variableResolutionCostPrecision = 1000;

  /// The block interval during which a commitment can be challenged.
  uint256 public challengeWindow;

  /// The block interval during which a challenge can be resolved.
  uint256 public resolveWindow;

  /// The amount required to post a challenge.
  uint256 public bondSize;

  /**
    The percentage of the resolving cost to be refunded to the resolver. There
    are no decimals, ie a value of 50 corresponds to 50%.
  */
  uint256 public resolverRefundPercentage;

  /**
    A mapping from addresses to their bond balance in the contract.

    @param TODO

    @return TODO
  */
  mapping (
    address TODO => uint256 TODO
  ) public balances;

  /**
    A mapping from challenged block numbers to challenged commitments to
    challenges.

    @param TODO
    @param TODO

    @return TODO
  */
  mapping (
    uint256 TODO => mapping (
      bytes TODO => Challenge TODO
    )
  ) internal challenges;

  /// Constructs the DataAvailabilityChallenge contract.
  constructor () OwnableUpgradeable() {
    _disableInitializers();
  }

  /**
    Initializes the contract.

    @param _owner The owner of the contract.
    @param _challengeWindow The block interval during which a commitment can be
      challenged.
    @param _resolveWindow The block interval during which a challenge can be
      resolved.
    @param _bondSize The amount required to post a challenge.
    @param _resolverRefundPercentage TODO
  */
  function initialize (
    address _owner,
    uint256 _challengeWindow,
    uint256 _resolveWindow,
    uint256 _bondSize,
    uint256 _resolverRefundPercentage
  ) external initializer {
    __Ownable_init();
    challengeWindow = _challengeWindow;
    resolveWindow = _resolveWindow;
    setBondSize(_bondSize);
    setResolverRefundPercentage(_resolverRefundPercentage);
    _transferOwnership(_owner);
  }

  /**
    Sets the bond size.

    @param _bondSize The amount required to post a challenge.
  */
  function setBondSize (
    uint256 _bondSize
  ) public onlyOwner {
    bondSize = _bondSize;
    emit RequiredBondSizeChanged(_bondSize);
  }

  /**
    Sets the percentage of the resolving cost to be refunded to the resolver.
    The function reverts if the provided percentage is above 100, since the
    refund logic assumes a value smaller or equal to 100%.

    @param _resolverRefundPercentage The percentage of the resolving cost to be
      refunded to the resolver.
  */
  function setResolverRefundPercentage (
    uint256 _resolverRefundPercentage
  ) public onlyOwner {
    if (_resolverRefundPercentage > 100) {
      revert InvalidResolverRefundPercentage(_resolverRefundPercentage);
    }
    resolverRefundPercentage = _resolverRefundPercentage;
  }

  /// Post a bond as prerequisite for challenging a commitment.
  receive () external payable {
    deposit();
  }

  /// Post a bond as prerequisite for challenging a commitment.
  function deposit () public payable {
    balances[msg.sender] += msg.value;
    emit BalanceChanged(msg.sender, balances[msg.sender]);
  }

  /// Withdraw a user's unlocked bond.
  function withdraw () external {

    // get caller's balance
    uint256 _balance = balances[msg.sender];

    // set caller's balance to 0
    balances[msg.sender] = 0;
    emit BalanceChanged(msg.sender, 0);

    // send caller's balance to caller
    bool _success = SafeCall.send(msg.sender, gasleft(), _balance);
    if (!_success) {
      revert WithdrawalFailed();
    }
  }

  /**
    Checks if the current block is within the challenge window for a given
    challenged block number.

    @param _challengedBlockNumber The block number at which the commitment was
      made.

    @return _ True if the current block is within the challenge window, false
      otherwise.
  */
  function _isInChallengeWindow (
    uint256 _challengedBlockNumber
  ) internal view returns (bool) {
    return (
      block.number >= _challengedBlockNumber
      && block.number <= _challengedBlockNumber + challengeWindow
    );
  }

  /**
    Checks if the current block is within the resolve window for a given
    challenge start block number.

    @param _challengeStartBlockNumber The block number at which the challenge
      was initiated.

    @return _ True if the current block is within the resolve window, false
      otherwise.
  */
  function _isInResolveWindow (
    uint256 _challengeStartBlockNumber
  ) internal view returns (bool) {
    return block.number <= _challengeStartBlockNumber + resolveWindow;
  }

  /**
    Returns a challenge for the given block number and commitment. Unlike with a
    public `challenges` mapping, we can return a Challenge struct instead of
    tuple.

    @param _challengedBlockNumber The block number at which the commitment was
      made.
    @param _challengedCommitment The commitment that is being challenged.

    @return _ The challenge struct.
  */
  function getChallenge (
    uint256 _challengedBlockNumber,
    bytes calldata _challengedCommitment
  ) public view returns (Challenge memory) {
    return challenges[_challengedBlockNumber][_challengedCommitment];
  }

  /**
    Returns the status of a challenge for a given challenged block number and
    challenged commitment.

    @param _challengedBlockNumber The block number at which the commitment was
      made.
    @param _challengedCommitment The commitment that is being challenged.

    @return _ The status of the challenge.
  */
  function getChallengeStatus (
    uint256 _challengedBlockNumber,
    bytes calldata _challengedCommitment
  ) public view returns (ChallengeStatus) {
    Challenge memory _challenge =
      challenges[_challengedBlockNumber][_challengedCommitment];

    // if the address is 0, the challenge is uninitialized
    if (_challenge.challenger == address(0)) {
      return ChallengeStatus.Uninitialized;
    }

    // if the challenge has a resolved block, it is resolved
    if (_challenge.resolvedBlock != 0) {
      return ChallengeStatus.Resolved;
    }

    // if the challenge's start block is in the resolve window, it is active
    if (_isInResolveWindow(_challenge.startBlock)) {
      return ChallengeStatus.Active;
    }

    /*
      if the challenge's start block is not in the resolve window, it is expired
    */
    return ChallengeStatus.Expired;
  }

  /**
    Extract the commitment type from a given commitment. The commitment type is
    located in the first byte of the commitment.

    @param _commitment The commitment from which to extract the commitment type.

    @return _ The commitment type of the given commitment.
  */
  function _getCommitmentType (
    bytes calldata _commitment
  ) internal pure returns (uint8) {
    return uint8(bytes1(_commitment));
  }

  /**
    Validate that a given commitment has a known type and the expected length
    for this type. The type of a commitment is stored in its first byte. The
    function reverts with `UnknownCommitmentType` if the type is not known and
    with `InvalidCommitmentLength` if the commitment has an unexpected length.

    @param _commitment The commitment for which to check the type.
  */
  function validateCommitment (
    bytes calldata _commitment
  ) public pure {
    uint8 _commitmentType = _getCommitmentType(_commitment);
    if (_commitmentType == uint8(CommitmentType.Keccak256)) {
      if (_commitment.length != 33) {
        revert InvalidCommitmentLength(
          uint8(CommitmentType.Keccak256), 33, _commitment.length
        );
      }
      return;
    }
    revert UnknownCommitmentType(_commitmentType);
  }

  /**
    Challenge a commitment at a given block number. The block number parameter
    is necessary for the contract to verify the challenge window, since the
    contract cannot access the block number of the commitment. The function
    reverts if the commitment type (first byte) is unknown, if the caller does
    not have a bond or if the challenge already exists.

    @param _challengedBlockNumber The block number at which the commitment was
      made.
    @param _challengedCommitment The commitment that is being challenged.
  */
  function challenge (
    uint256 _challengedBlockNumber,
    bytes calldata _challengedCommitment
  ) external payable {

    // require the commitment type to be known
    validateCommitment(_challengedCommitment);

    // deposit value sent with the transaction as bond
    deposit();

    // require the caller to have a bond
    if (balances[msg.sender] < bondSize) {
      revert BondTooLow(balances[msg.sender], bondSize);
    }

    // require the challenge status to be uninitialized
    if (
      getChallengeStatus(_challengedBlockNumber, _challengedCommitment) !=
      ChallengeStatus.Uninitialized
    ) {
      revert ChallengeExists();
    }

    // require the current block to be in the challenge window
    if (!_isInChallengeWindow(_challengedBlockNumber)) {
      revert ChallengeWindowNotOpen();
    }

    // reduce the caller's balance
    balances[msg.sender] -= bondSize;

    /*
      store the challenger's address, bond size, and start block of the
      challenge
    */
    challenges[_challengedBlockNumber][_challengedCommitment] = Challenge({
      challenger: msg.sender,
      lockedBond: bondSize,
      startBlock: block.number,
      resolvedBlock: 0
    });

    // emit an event to notify that the challenge status is now active
    emit ChallengeStatusChanged(
      _challengedBlockNumber, _challengedCommitment, ChallengeStatus.Active
    );
  }

  /**
    Resolve a challenge by providing the data corresponding to the challenged
    commitment. The function computes a commitment from the provided resolveData
    and verifies that it matches the challenged commitment. It reverts if the
    commitment type is unknown, if the data doesn't match the commitment, if the
    challenge is not active or if the resolve window is not open.

    @param _challengedBlockNumber The block number at which the commitment was
      made.
    @param _challengedCommitment The challenged commitment that is being
      resolved.
    @param _resolveData The pre-image data corresponding to the challenged
      commitment.
  */
  function resolve (
    uint256 _challengedBlockNumber,
    bytes calldata _challengedCommitment,
    bytes calldata _resolveData
  ) external {

    // require the commitment type to be known
    validateCommitment(_challengedCommitment);

    /*
      require the challenge to be active (started, not resolved, and resolve
      window still open)
    */
    if (
      getChallengeStatus(_challengedBlockNumber, _challengedCommitment) !=
      ChallengeStatus.Active
    ) {
      revert ChallengeNotActive();
    }

    // compute the commitment corresponding to the given resolveData
    uint8 _commitmentType = _getCommitmentType(_challengedCommitment);
    bytes memory _computedCommitment;
    if (_commitmentType == uint8(CommitmentType.Keccak256)) {
      _computedCommitment = computeCommitmentKeccak256(_resolveData);
    }

    /*
      require the provided input data to correspond to the challenged commitment
    */
    if (keccak256(_computedCommitment) != keccak256(_challengedCommitment)) {
      revert InvalidInputData(_computedCommitment, _challengedCommitment);
    }

    // store the block number at which the challenge was resolved
    Challenge storage _activeChallenge =
      challenges[_challengedBlockNumber][_challengedCommitment];
    _activeChallenge.resolvedBlock = block.number;

    // emit an event to notify that the challenge status is now resolved
    emit ChallengeStatusChanged(
      _challengedBlockNumber, _challengedCommitment, ChallengeStatus.Resolved
    );

    // distribute the bond among challenger, resolver and address(0)
    _distributeBond(_activeChallenge, _resolveData.length, msg.sender);
  }

  /**
    Distribute the bond of a resolved challenge among the resolver, challenger
    and address(0). The challenger is refunded the bond amount exceeding the
    resolution cost. The resolver is refunded a percentage of the resolution
    cost based on the `resolverRefundPercentage` state variable. The remaining
    bond is burned by sending it to address(0). The resolution cost is
    approximated based on a fixed cost and variable cost depending on the size
    of the pre-image. The real resolution cost might vary, because calldata is
    priced differently for zero and non-zero bytes. Computing the exact cost
    adds too much gas overhead to be worth the tradeoff.

    @param _resolvedChallenge The resolved challenge in storage.
    @param _preImageLength The size of the pre-image used to resolve the
      challenge.
    @param _resolver The address of the resolver.
  */
  function _distributeBond (
    Challenge storage _resolvedChallenge,
    uint256 _preImageLength,
    address _resolver
  ) internal {
    uint256 _lockedBond = _resolvedChallenge.lockedBond;
    address _challenger = _resolvedChallenge.challenger;

    /*
      approximate the cost of resolving a challenge with the provided pre-image
      size
    */
    uint256 _resolutionCost =
      (
        fixedResolutionCost + _preImageLength * variableResolutionCost /
        variableResolutionCostPrecision
      ) * block.basefee;

    // refund bond exceeding the resolution cost to the challenger
    if (_lockedBond > _resolutionCost) {
      balances[_challenger] += _lockedBond - _resolutionCost;
      _lockedBond = _resolutionCost;
      emit BalanceChanged(_challenger, balances[_challenger]);
    }

    /*
      refund a percentage of the resolution cost to the resolver (but not more
      than the locked bond)
    */
    uint256 _resolverRefund = _resolutionCost * resolverRefundPercentage / 100;
    if (_resolverRefund > _lockedBond) {
      _resolverRefund = _lockedBond;
    }

    if (_resolverRefund > 0) {
      balances[_resolver] += _resolverRefund;
      _lockedBond -= _resolverRefund;
      emit BalanceChanged(_resolver, balances[_resolver]);
    }

    // burn the remaining bond
    if (_lockedBond > 0) {
      payable(address(0)).transfer(_lockedBond);
    }
    _resolvedChallenge.lockedBond = 0;
  }

  /**
    Unlock the bond associated wth an expired challenge. The function reverts if
    the challenge is not expired. If the expiration is successful, the
    challenger's bond is unlocked.

    @param _challengedBlockNumber The block number at which the commitment was
      made.
    @param _challengedCommitment The commitment that is being challenged.
  */
  function unlockBond (
    uint256 _challengedBlockNumber,
    bytes calldata _challengedCommitment
  ) external {

    /*
      require the challenge to be active (started, not resolved, and in the
      resolve window)
    */
    if (
      getChallengeStatus(_challengedBlockNumber, _challengedCommitment) !=
      ChallengeStatus.Expired
    ) {
      revert ChallengeNotExpired();
    }

    // Unlock the bond associated with the challenge
    Challenge storage _expiredChallenge =
      challenges[_challengedBlockNumber][_challengedCommitment];
    balances[_expiredChallenge.challenger] += _expiredChallenge.lockedBond;
    _expiredChallenge.lockedBond = 0;

    // Emit balance update event
    emit BalanceChanged(
      _expiredChallenge.challenger, balances[_expiredChallenge.challenger]
    );
  }
}
