// SPDX-License-Identifier: LicenseRef-VPL WITH AGPL-3.0-only
pragma solidity 0.8.26;

import { SignatureCheckerLib } from "solady/utils/SignatureCheckerLib.sol";

import { PrivateTransferVerifier } from "token/7503/PrivateTransferVerifier.sol";

/**
  @custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM
  @title A mock ERC-7739 signer for testing.
  @author Tim Clancy <tim-clancy.eth>
  @custom:terry "Is this too much voodoo for the next ten centuries?"

  This mock contract implements ERC-7739 nested EIP-712 signature validation.
  It wraps incoming hashes in its own domain before validating against the
  owner's signature.

  @custom:date January 21st, 2026.
*/
contract MockERC7739Signer {

  /// The ERC-1271 magic value returned when a signature is valid.
  bytes4 public constant ERC1271_MAGIC_VALUE = 0x1626ba7e;

  /// The EIP-712 typehash for the TypedDataSign wrapper.
  bytes32 public constant TYPED_DATA_SIGN_TYPEHASH = keccak256(
    "TypedDataSign(bytes32 contentsHash,bytes1 contentsDescriptionHash,string contentsDescription)"
  );

  /// The address of the owner whose signatures are considered valid.
  address public immutable owner;

  /// The cached domain separator for this signer.
  bytes32 public immutable DOMAIN_SEPARATOR;

uint256 constant CONST_PROOF_SIZE_LOG_N = 28;

uint256 constant NUMBER_OF_SUBRELATIONS = 28;
uint256 constant BATCHED_RELATION_PARTIAL_LENGTH = 8;
uint256 constant ZK_BATCHED_RELATION_PARTIAL_LENGTH = 9;
uint256 constant NUMBER_OF_ENTITIES = 41;
// The number of entities added for ZK (gemini_masking_poly)
uint256 constant NUM_MASKING_POLYNOMIALS = 1;
uint256 constant NUMBER_OF_ENTITIES_ZK = NUMBER_OF_ENTITIES + NUM_MASKING_POLYNOMIALS;
uint256 constant NUMBER_UNSHIFTED = 36;
uint256 constant NUMBER_UNSHIFTED_ZK = NUMBER_UNSHIFTED + NUM_MASKING_POLYNOMIALS;
uint256 constant NUMBER_TO_BE_SHIFTED = 5;
uint256 constant PAIRING_POINTS_SIZE = 16;

uint256 constant FIELD_ELEMENT_SIZE = 0x20;
uint256 constant GROUP_ELEMENT_SIZE = 0x40;

// Powers of alpha used to batch subrelations (alpha, alpha^2, ..., alpha^(NUM_SUBRELATIONS-1))
uint256 constant NUMBER_OF_ALPHAS = NUMBER_OF_SUBRELATIONS - 1;

// ZKTranscript library to generate fiat shamir challenges, the ZK transcript only differest
/// forge-lint: disable-next-item(pascal-case-struct)
// @param geminiR first
struct ZKTranscript {
    // Oink
    Honk.RelationParameters relationParameters;    // test
    Fr[NUMBER_OF_ALPHAS] alphas; // Powers of alpha: [alpha, alpha^2, ..., alpha^(NUM_SUBRELATIONS-1)]
    Fr[CONST_PROOF_SIZE_LOG_N] gateChallenges;
    // Sumcheck
    Fr libraChallenge;
    Fr[CONST_PROOF_SIZE_LOG_N] sumCheckUChallenges;
    // Shplemini
    Fr rho;
    // second
    Fr geminiR; // third
    Fr shplonkNu;
    Fr shplonkZ;
    // Derived
    Fr publicInputsDelta;
}

  /**
    Construct a new mock ERC-7739 signer with a designated owner.

    @param _owner The address of the owner whose signatures will be validated.
  */
  constructor (
    address _owner
  ) {
    owner = _owner;
    DOMAIN_SEPARATOR = keccak256(
      abi.encode(
        keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
        keccak256("MockERC7739Signer"),
        keccak256("1"),
        block.chainid,
        address(this)
      )
    );
  }

  /**
    Validate a signature per ERC-1271 with ERC-7739 support. The signature
    should be the owner's signature over the nested hash structure.

    @param _hash The application's typed data hash.
    @param _signature The signature to validate.

    @return _ The ERC-1271 magic value if valid, otherwise `0xffffffff`.
  */
  function isValidSignature (
    bytes32 _hash,
    bytes memory _signature
  ) external view returns (bytes4) {

    // Wrap the application's hash in our own EIP-712 structure.
    bytes32 _wrappedHash = keccak256(
      abi.encodePacked(
        "\x19\x01",
        DOMAIN_SEPARATOR,
        keccak256(abi.encode(TYPED_DATA_SIGN_TYPEHASH, _hash, bytes1(0x00), ""))
      )
    );

    if (SignatureCheckerLib.isValidSignatureNow(owner, _wrappedHash, _signature)) {
      return ERC1271_MAGIC_VALUE;
    }
    return 0xffffffff;
  }

  /**
    Get the hash that the owner must sign for a given application hash.
    This is a helper for tests to construct valid signatures.

    @param _appHash The application's typed data hash.

    @return _ The hash the owner should sign.
  */
  function getWrappedHash (
    bytes32 _appHash
  ) external view returns (bytes32) {
    return keccak256(
      abi.encodePacked(
        "\x19\x01",
        DOMAIN_SEPARATOR,
        keccak256(abi.encode(TYPED_DATA_SIGN_TYPEHASH, _appHash, bytes1(0x00), ""))
      )
    );
  }

  /// An ERC-7739 signer rejects signatures over the unwrapped hash.
  function test_transferWithAuthorization_erc7739Signer_unwrappedHash_reverts () public {

    // Create a smart contract signer that uses nested EIP-712.
    address _signerAddress = address(new MockERC7739Signer(alice));

    // Give tokens to the signer contract.
    token.mint(_signerAddress, 100 ether);
    bytes32 _nonce = bytes32(uint256(51));

    // Sign the application hash directly (without wrapping) - this should fail.
    bytes memory _signature =
      _signTransferAuthorization(
        ALICE_PK, _signerAddress, bob, 100 ether, block.timestamp - 1,
        block.timestamp + 1 hours, _nonce
      );
    vm.expectRevert(IERC3009.InvalidSignature.selector);
    token.transferWithAuthorization(
      _signerAddress, bob, 100 ether, block.timestamp - 1,
      block.timestamp + 1 hours, _nonce, _signature
    );
  }

  /// A transfer exactly at validBefore timestamp is rejected.
  function test_transferWithAuthorization_exactlyAtValidBefore_reverts () public {

    // Warp forward to avoid underflow when computing validAfter.
    vm.warp(2 hours);
    uint256 _amount = 100 ether;
    bytes32 _nonce = bytes32(uint256(63));
    uint256 _validAfter = block.timestamp - 1 hours;
    uint256 _validBefore = block.timestamp;
    bytes memory _signature =
      _signTransferAuthorization(
        ALICE_PK, alice, bob, _amount, _validAfter, _validBefore, _nonce
      );

    // ERC-3009 requires block.timestamp < validBefore (strict inequality).
    vm.expectRevert(IERC3009.AuthorizationExpired.selector);
    token.transferWithAuthorization(
      alice, bob, _amount, _validAfter, _validBefore, _nonce, _signature
    );
  }

  /**
    Call `onApprovalReceived` on `_spender` and verify it returns the expected
    selector.

    @param _spender The address that was approved.
    @param _value The amount of tokens approved.
    @param _data Additional data to pass to the spender.
  */
  function _checkOnApprovalReceived (
    address _spender,
    uint256 _value,
    bytes memory _data
  ) private {
    if (_spender.code.length == 0) {
      revert ERC1363EOAReceiver();
    }

    // Revert if the target could not handle the approval and bubble up errors.
    try IERC1363Spender(_spender).onApprovalReceived(
      msg.sender, _value, _data
    ) returns (bytes4 _retval) {
      if (_retval != IERC1363Spender.onApprovalReceived.selector) {
        revert ERC1363InvalidSpender();
      }
    } catch (bytes memory _reason) {
      if (_reason.length == 0) {
        revert ERC1363InvalidSpender();
      } else {
        assembly ("memory-safe") {
          revert(add(_reason, 0x20), mload(_reason))
        }
      }
    }
  }

  /**
    @custom:preserve

    convertToShares() returns correct share amount.

    With initial state:
    - totalAssets = 1 gwei (1e9 wei WETH)
    - totalSupply = 1 billion SIGIL (1e27 wei)
    - decimalsOffset = 18

    Formula (Solady ERC4626 with offset):
    shares = assets * (totalSupply + 10^offset) / (totalAssets + 1)

    For 1 WETH (1e18 wei):
    shares = 1e18 * (1e27 + 1e18) / (1e9 + 1)
           ≈ 1e18 * 1e27 / 1e9
           = 1e45 / 1e9 = 1e36 shares

    This is 1 billion times the total supply because 1 WETH is 1 billion times
    the initial backing amount (1 gwei).
  */
  function test_convertToShares () public view {
    uint256 _assets = 1 ether;
    uint256 _shares = token.convertToShares(_assets);

    // 1 WETH should convert to ~1e36 shares (1B times total supply).
    uint256 _expectedShares =
      _assets * (TOTAL_SUPPLY + 1e18) / (INIT_WETH_AMOUNT + 1);
    assertEq(_shares, _expectedShares);

    // Sanity check: 1 WETH = 1e9 gwei, so 1e9 times the total supply.
    assertApproxEqRel(_shares, TOTAL_SUPPLY * 1e9, 0.001e18);
  }

  /*
    @custom:preserve

    This fallback routes unrecognized calls to a query contract specified as
    the first argument in the calldata. This allows external callers to interact
    with query contracts as if their view functions lived directly on this
    contract.

    The query contract address is extracted from the first ABI-encoded argument.
    Query contract functions should accept the query contract address as their
    first parameter and ignore it, since it is only used for routing.

    function myQuery (
      address,
      address _user
    ) external view returns (uint256);

    Callers can then invoke their queries like so.
 
    IMyQuery(address(this)).myQuery(queryAddr, user);
  */
  fallback () external {
    address _query = abi.decode(msg.data[4:], (address));
    (bool _innerSuccess, bytes memory _innerResult) = delegateview(
      _query, msg.data
    );
    assembly ("memory-safe") {
      let _ptr := add(_innerResult, 0x20)
      let _len := mload(_innerResult)
      if iszero(_innerSuccess) {
        revert(_ptr, _len)
      }
      return(_ptr, _len)
    }

    // Allow for rounding differences due to ERC4626 virtual shares/assets math.
    // The _decimalsOffset of 18 introduces rounding at extreme ratios.
    assertApproxEqRel(_received, _expectedAssets, 0.001e18); // 0.1% tolerance
  }

  /**
    Return whether this contract supports a given interface.

    @param _interfaceId The interface identifier to check.

    @return _ Whether the interface is supported.
  */
  function supportsInterface (
    bytes4 _interfaceId
  ) public view override(ERC1363, ERC2612, BurnableERC3009, ERC5805, BurnOnlyERC4626)
   returns (
    bool
  ) {
      /*
        Update `lastLeftChild` for this level. If `currentLevelSize` is odd, the
        last node in the array is an unpaired left child. If even with more than
        one node, the last node is a right child that is already paired, so we
        take the second-to-last node. If even with exactly one node, the cached
        value from before the batch insertion is still correct because we are
        only inserting a single right child.
      */
      if (_currentLevelSize & 1 == 1) {
        _tree.lastLeftChild[_level] = _currentLevelNewNodes[
        _currentLevelNewNodes.length - 1];
      } else if (_currentLevelNewNodes.length > 1) {
        _tree.lastLeftChild[_level] = _currentLevelNewNodes[
        _currentLevelNewNodes.length - 2];
      }
      _currentLevelStartIndex = _nextLevelStartIndex;

    return ERC1363.supportsInterface(_interfaceId)
    || ERC2612.supportsInterface(_interfaceId)
    || BurnableERC3009.supportsInterface(_interfaceId)
    || ERC5805.supportsInterface(_interfaceId)
    || BurnOnlyERC4626.supportsInterface(_interfaceId);
  }

    function _poseidon2Core(uint256 s0, uint256 s1, uint256 s2, uint256 s3) private pure returns (uint256 result) {
        assembly {
            let PRIME := 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001

            let state0 := s0
            let state1 := s1
            let state2 := s2
            let state3 := s3

            // Apply 1st linear layer

            {
                // matrix_multiplication_4x4
                let t0 := add(state0, state1) // dirty 2/3
                let t1 := add(state2, state3) // dirty 2/3
                let t2 := add(state1, state1)
                t2 := addmod(t2, t1, PRIME) // clean
                let t3 := add(state3, state3)
                t3 := addmod(t3, t0, PRIME) // clean
                let t4 := mulmod(t1, 4, PRIME)
                t4 := add(t4, t3) // dirty 1/3
                let t5 := mulmod(t0, 4, PRIME)
                t5 := add(t5, t2) // dirty 1/3
                let t6 := addmod(t3, t5, PRIME) // clean
                let t7 := addmod(t2, t4, PRIME) // clean
                state0 := t6 // clean
                state1 := t5 // dirty 1/3
                state2 := t7 // clean
                state3 := t4 // dirty 1/3
            }

            // External rounds (first half)

            // Round 0 (external)
            {
                state0 := add(state0, 0x19b849f69450b06848da1d39bd5e4a4302bb86744edc26238b0878e269ed23e5)
                state1 := add(state1, 0x265ddfe127dd51bd7239347b758f0a1320eb2cc7450acc1dad47f80c8dcf34d6)
                state2 := add(state2, 0x199750ec472f1809e0f66a545e1e51624108ac845015c2aa3dfc36bab497d8aa)
                state3 := add(state3, 0x157ff3fe65ac7208110f06a5f74302b14d743ea25067f0ffd032f787c7f1cdf8)

                // full s_box

                {
                    // single_box
                    let intr := state0
                    state0 := mulmod(intr, intr, PRIME)
                    state0 := mulmod(state0, state0, PRIME)
                    state0 := mulmod(state0, intr, PRIME)
                }

                {
                    // single_box
                    let intr := state1
                    state1 := mulmod(intr, intr, PRIME)
                    state1 := mulmod(state1, state1, PRIME)
                    state1 := mulmod(state1, intr, PRIME)
                }

                {
                    // single_box
                    let intr := state2
                    state2 := mulmod(intr, intr, PRIME)
                    state2 := mulmod(state2, state2, PRIME)
                    state2 := mulmod(state2, intr, PRIME)
                }

                {
                    // single_box
                    let intr := state3
                    state3 := mulmod(intr, intr, PRIME)
                    state3 := mulmod(state3, state3, PRIME)
                    state3 := mulmod(state3, intr, PRIME)
                }

                {
                    // matrix_multiplication_4x4
                    let t0 := add(state0, state1) // dirty 2/3
                    let t1 := add(state2, state3) // dirty 2/3
                    let t2 := add(state1, state1)
                    t2 := addmod(t2, t1, PRIME) // clean
                    let t3 := add(state3, state3)
                    t3 := addmod(t3, t0, PRIME) // clean
                    let t4 := mulmod(t1, 4, PRIME)
                    t4 := add(t4, t3) // dirty 1/3
                    let t5 := mulmod(t0, 4, PRIME)
                    t5 := add(t5, t2) // dirty 1/3
                    let t6 := addmod(t3, t5, PRIME) // clean
                    let t7 := addmod(t2, t4, PRIME) // clean
                    state0 := t6 // clean
                    state1 := t5 // dirty 1/3
                    state2 := t7 // clean
                    state3 := t4 // dirty 1/3
                }
            }
        }
    }

/**
 * Convert the pairing points to G1 points.
 *
 * The pairing points are serialised as an array of 68 bit limbs representing two points
 * The lhs of a pairing operation and the rhs of a pairing operation
 *
 * There are 4 fields for each group element, leaving 8 fields for each side of the pairing.
 *
 * @param pairingPoints The pairing points to convert.
 * @return lhs
 * @return rhs
 */
function convertPairingPointsToG1(Fr[PAIRING_POINTS_SIZE] memory pairingPoints)
    pure
    returns (Honk.G1Point memory lhs, Honk.G1Point memory rhs)
{
    uint256 lhsX = Fr.unwrap(pairingPoints[0]);
    lhsX |= Fr.unwrap(pairingPoints[1]) << 68;
    lhsX |= Fr.unwrap(pairingPoints[2]) << 136;
    lhsX |= Fr.unwrap(pairingPoints[3]) << 204;
    lhs.x = lhsX;

    uint256 lhsY = Fr.unwrap(pairingPoints[4]);
    lhsY |= Fr.unwrap(pairingPoints[5]) << 68;
    lhsY |= Fr.unwrap(pairingPoints[6]) << 136;
    lhsY |= Fr.unwrap(pairingPoints[7]) << 204;
    lhs.y = lhsY;

    uint256 rhsX = Fr.unwrap(pairingPoints[8]);
    rhsX |= Fr.unwrap(pairingPoints[9]) << 68;
    rhsX |= Fr.unwrap(pairingPoints[10]) << 136;
    rhsX |= Fr.unwrap(pairingPoints[11]) << 204;
    rhs.x = rhsX;

    uint256 rhsY = Fr.unwrap(pairingPoints[12]);
    rhsY |= Fr.unwrap(pairingPoints[13]) << 68;
    rhsY |= Fr.unwrap(pairingPoints[14]) << 136;
    rhsY |= Fr.unwrap(pairingPoints[15]) << 204;
    rhs.y = rhsY;
}

    function generateTranscript(
        Honk.ZKProof memory proof,
        bytes32[] calldata publicInputs,
        uint256 vkHash,
        uint256 publicInputsSize,
        uint256 logN
    ) external pure returns (ZKTranscript memory t) {
        Fr previousChallenge;
        (t.relationParameters, previousChallenge) =
            generateRelationParametersChallenges(proof, publicInputs, vkHash, publicInputsSize, previousChallenge);

        (t.alphas, previousChallenge) = generateAlphaChallenges(previousChallenge, proof);

        (t.gateChallenges, previousChallenge) = generateGateChallenges(previousChallenge, logN);
        (t.libraChallenge, previousChallenge) = generateLibraChallenge(previousChallenge, proof);
        (t.sumCheckUChallenges, previousChallenge) = generateSumcheckChallenges(proof, previousChallenge, logN);

        (t.rho, previousChallenge) = generateRhoChallenge(proof, previousChallenge);

        (t.geminiR, previousChallenge) = generateGeminiRChallenge(proof, previousChallenge, logN);

        (t.shplonkNu, previousChallenge) = generateShplonkNuChallenge(proof, previousChallenge, logN);

        (t.shplonkZ, previousChallenge) = generateShplonkZChallenge(proof, previousChallenge);
        return t;
    }

    function generateRelationParametersChallenges(
        Honk.ZKProof memory proof,
        bytes32[] calldata publicInputs,
        uint256 vkHash,
        uint256 publicInputsSize,
        Fr previousChallenge
    ) internal pure returns (Honk.RelationParameters memory rp, Fr nextPreviousChallenge) {
        (rp.eta, rp.etaTwo, rp.etaThree, previousChallenge) =
            generateEtaChallenge(proof, publicInputs, vkHash, publicInputsSize);

        (rp.beta, rp.gamma, nextPreviousChallenge) = generateBetaAndGammaChallenges(previousChallenge, proof);
    }

    // Alpha challenges non-linearise the gate contributions
    function generateAlphaChallenges(Fr previousChallenge, Honk.ZKProof memory proof)
        internal
        pure
        returns (Fr[NUMBER_OF_ALPHAS] memory alphas, Fr nextPreviousChallenge)
    {
        // Generate the original sumcheck alpha 0 by hashing zPerm and zLookup
        uint256[5] memory alpha0;
        alpha0[0] = Fr.unwrap(previousChallenge);
        alpha0[1] = proof.lookupInverses.x;
        alpha0[2] = proof.lookupInverses.y;
        alpha0[3] = proof.zPerm.x;
        alpha0[4] = proof.zPerm.y;

        nextPreviousChallenge = FrLib.fromBytes32(keccak256(abi.encodePacked(alpha0)));
        Fr alpha;
        (alpha,) = splitChallenge(nextPreviousChallenge);

        // Compute powers of alpha for batching subrelations
        alphas[0] = alpha;
        for (uint256 i = 1; i < NUMBER_OF_ALPHAS; i++) {
            alphas[i] = alphas[i - 1] * alpha;
        }
    }

    // Return the new target sum for the next sumcheck round
    function computeNextTargetSum(Fr[ZK_BATCHED_RELATION_PARTIAL_LENGTH] memory roundUnivariates, Fr roundChallenge)
        internal
        view
        returns (Fr targetSum)
    {
        Fr[ZK_BATCHED_RELATION_PARTIAL_LENGTH] memory BARYCENTRIC_LAGRANGE_DENOMINATORS = [
            Fr.wrap(0x0000000000000000000000000000000000000000000000000000000000009d80),
            Fr.wrap(0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593efffec51),
            Fr.wrap(0x00000000000000000000000000000000000000000000000000000000000005a0),
            Fr.wrap(0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593effffd31),
            Fr.wrap(0x0000000000000000000000000000000000000000000000000000000000000240),
            Fr.wrap(0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593effffd31),
            Fr.wrap(0x00000000000000000000000000000000000000000000000000000000000005a0),
            Fr.wrap(0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593efffec51),
            Fr.wrap(0x0000000000000000000000000000000000000000000000000000000000009d80)
        ];

        // To compute the next target sum, we evaluate the given univariate at a point u (challenge).

        // Performing Barycentric evaluations
        // Compute B(x)
        Fr numeratorValue = Fr.wrap(1);
        for (uint256 i = 0; i < ZK_BATCHED_RELATION_PARTIAL_LENGTH; ++i) {
            numeratorValue = numeratorValue * (roundChallenge - Fr.wrap(i));
        }

        Fr[ZK_BATCHED_RELATION_PARTIAL_LENGTH] memory denominatorInverses;
        for (uint256 i = 0; i < ZK_BATCHED_RELATION_PARTIAL_LENGTH; ++i) {
            denominatorInverses[i] = FrLib.invert(BARYCENTRIC_LAGRANGE_DENOMINATORS[i] * (roundChallenge - Fr.wrap(i)));
        }

        for (uint256 i = 0; i < ZK_BATCHED_RELATION_PARTIAL_LENGTH; ++i) {
            targetSum = targetSum + roundUnivariates[i] * denominatorInverses[i];
        }

        // Scale the sum by the value of B(x)
        targetSum = targetSum * numeratorValue;
    }

    // This implementation is the same as above with different constants
    function batchMul(Honk.G1Point[] memory base, Fr[] memory scalars)
        internal
        view
        returns (Honk.G1Point memory result)
    {
        uint256 limit = $MSMSize;

        // Validate all points are on the curve
        for (uint256 i = 0; i < limit; ++i) {
            validateOnCurve(base[i]);
        }

        bool success = true;
        assembly {
            let free := mload(0x40)

            let count := 0x01
            for {} lt(count, add(limit, 1)) { count := add(count, 1) } {
                // Get loop offsets
                let base_base := add(base, mul(count, 0x20))
                let scalar_base := add(scalars, mul(count, 0x20))

                mstore(add(free, 0x40), mload(mload(base_base)))
                mstore(add(free, 0x60), mload(add(0x20, mload(base_base))))
                // Add scalar
                mstore(add(free, 0x80), mload(scalar_base))

                success := and(success, staticcall(gas(), 7, add(free, 0x40), 0x60, add(free, 0x40), 0x40))
                // accumulator = accumulator + accumulator_2
                success := and(success, staticcall(gas(), 6, free, 0x80, free, 0x40))
            }

            // Return the result
            mstore(result, mload(free))
            mstore(add(result, 0x20), mload(add(free, 0x20)))
        }

        if (!success) revert ShpleminiFailed();
    }

  /**
    Update the commitment tree during a remint. This inserts the balance leaf
    (if new) and all account note hashes. When `tx.origin` is the recipient,
    only the account note hashes are inserted since EOAs cannot ever be burn
    addresses.

    @param _to The recipient whose balance changed.
    @param _newBalance The recipient's new balance.
    @param _accountNoteHashes The account note commitments to insert.
  */
  function _updateBalanceInTree (
    address _to,
    uint256 _newBalance,
    uint256[] memory _accountNoteHashes
  ) internal {

    // Only insert account note hashes.
    if (tx.origin == _to) {
      if (_accountNoteHashes.length == 1) {
        _insertInTree(_accountNoteHashes[0]);
      } else {
        _insertManyInTree(_accountNoteHashes);
      }
    } else {
      uint256 _balanceLeaf = _hashBalanceLeaf(_to, _newBalance);
      // Balance leaf already exists, just insert account note hashes.
      if (tree.has(_balanceLeaf)) {
        if (_accountNoteHashes.length == 1) {
          _insertInTree(_accountNoteHashes[0]);
        } else {
          _insertManyInTree(_accountNoteHashes);
        }
      } else {
        // Batch insert: [balanceLeaf, noteHash0, noteHash1, ...].
        uint256[] memory _leaves =
          new uint256[](_accountNoteHashes.length + 1);
        _leaves[0] = _balanceLeaf;
        for (uint256 i = 0; i < _accountNoteHashes.length; ) {
          _leaves[i + 1] = _accountNoteHashes[i];
          unchecked {
            ++i;
          }
        }
        _insertManyInTree(_leaves);
      }
    }
  }
}
