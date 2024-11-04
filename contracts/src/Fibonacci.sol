// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/// @title SP1 Merkle Root Verifier
/// @notice This contract verifies SP1 proofs and manages a merkle root.
/// @dev Developed by Uncentered Systems, based on the Succinct Labs SP1 Fibonacci example contract
contract Fibonacci is Ownable {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;

    /// @notice The verification key for the fibonacci program.
    bytes32 public fibonacciProgramVKey;

    /// @notice The verification key for the aggregate program
    bytes32 public aggregateProgramVKey;

    /// @notice The current merkle root of all verified proofs
    bytes32 public merkleRoot;

    event MerkleRootUpdated(bytes32 oldRoot, bytes32 newRoot);

    constructor(
        address _verifier, 
        bytes32 _fibonacciProgramVKey,
        bytes32 _aggregateProgramVKey
    ) Ownable(msg.sender) {
        verifier = _verifier;
        fibonacciProgramVKey = _fibonacciProgramVKey;
        aggregateProgramVKey = _aggregateProgramVKey;
    }

    /// @notice The entrypoint for verifying the proof of a fibonacci number.
    /// @param _proofBytes The encoded proof.
    /// @param _publicValues The encoded public values.
    function verifyFibonacciProof(bytes calldata _publicValues, bytes calldata _proofBytes)
        public
        view
        returns (uint32, uint32, uint32)
    {
        ISP1Verifier(verifier).verifyProof(fibonacciProgramVKey, _publicValues, _proofBytes);
        PublicValuesStruct memory publicValues = abi.decode(_publicValues, (PublicValuesStruct));
        return (publicValues.n, publicValues.a, publicValues.b);
    }

    /// @notice Updates the merkle root. Only callable by owner.
    /// @param newRoot The new merkle root to set
    function updateMerkleRoot(bytes32 newRoot) public onlyOwner {
        bytes32 oldRoot = merkleRoot;
        merkleRoot = newRoot;
        emit MerkleRootUpdated(oldRoot, newRoot);
    }

    /// @notice Verifies an aggregate proof and updates the merkle root
    /// @param _publicValues The encoded public values array
    /// @param _proofBytes The encoded aggregate proof
    /// @param _newRoot The new merkle root to set after verification
    function verifyAggregateProofAndUpdateRoot(
        bytes calldata _publicValues,
        bytes calldata _proofBytes,
        bytes32 _newRoot
    ) public onlyOwner {
        // First verify the proof
        ISP1Verifier(verifier).verifyProof(
            aggregateProgramVKey,
            _publicValues,
            _proofBytes
        );
        
        // Decode and verify the public values match
        AggregateProofPublicValues memory publicValues = abi.decode(_publicValues, (AggregateProofPublicValues));
        require(_newRoot == publicValues.merkleRoot, "New root must match proof public values");
        
        // If verification succeeds, update the root
        bytes32 oldRoot = merkleRoot;
        merkleRoot = _newRoot;
        emit MerkleRootUpdated(oldRoot, _newRoot);
    }
}
