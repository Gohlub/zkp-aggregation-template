// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {SP1AggregateVerifier} from "../src/SP1AggregateVerifier.sol";
import {SP1VerifierGateway} from "@sp1-contracts/SP1VerifierGateway.sol";

struct SP1ProofFixtureJson {
    bytes32 verification_key;
    bytes32 merkle_root;
    bytes proof;
}

contract SP1AggregateVerifierTest is Test {
    using stdJson for string;

    address verifier;
    SP1AggregateVerifier public aggregateVerifier;

    function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/src/fixtures/groth16-onchain-abi.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1ProofFixtureJson));
    }

    function setUp() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        verifier = address(new SP1VerifierGateway(address(1)));
        aggregateVerifier = new SP1AggregateVerifier(
            verifier, 
            fixture.verification_key
        );
    }

    function test_ValidAggregateProof() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        vm.mockCall(
            verifier, 
            abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector), 
            abi.encode(true)
        );

        aggregateVerifier.verifyAggregateProofAndUpdateRoot(
            abi.encode(fixture.merkle_root),
            fixture.proof,
            fixture.merkle_root
        );

        assert(aggregateVerifier.merkleRoot() == fixture.merkle_root);
    }

    function testFail_InvalidAggregateProof() public {
        SP1ProofFixtureJson memory fixture = loadFixture();
        
        bytes memory fakeProof = new bytes(fixture.proof.length);

        aggregateVerifier.verifyAggregateProofAndUpdateRoot(
            abi.encode(fixture.merkle_root),
            fakeProof,
            fixture.merkle_root
        );
    }
}
