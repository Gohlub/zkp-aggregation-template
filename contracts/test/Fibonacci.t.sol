// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {Fibonacci} from "../src/Fibonacci.sol";
import {SP1VerifierGateway} from "@sp1-contracts/SP1VerifierGateway.sol";

struct SP1ProofFixtureJson {
    bytes32 verification_key;
    bytes32 merkle_root;
    bytes proof;
}

contract FibonacciTest is Test {
    using stdJson for string;

    address verifier;
    Fibonacci public fibonacci;

    function loadFixture() public view returns (SP1ProofFixtureJson memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, "/src/fixtures/groth16-onchain.json");
        string memory json = vm.readFile(path);
        bytes memory jsonBytes = json.parseRaw(".");
        return abi.decode(jsonBytes, (SP1ProofFixtureJson));
    }

    function setUp() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        verifier = address(new SP1VerifierGateway(address(1)));
        fibonacci = new Fibonacci(
            verifier, 
            fixture.verification_key,
            fixture.verification_key  // Using same key for both in test
        );
    }

    function test_ValidAggregateProof() public {
        SP1ProofFixtureJson memory fixture = loadFixture();

        vm.mockCall(
            verifier, 
            abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector), 
            abi.encode(true)
        );

        fibonacci.verifyAggregateProofAndUpdateRoot(
            abi.encode(fixture.merkle_root),
            fixture.proof,
            fixture.merkle_root
        );

        assert(fibonacci.merkleRoot() == fixture.merkle_root);
    }

    function testFail_InvalidAggregateProof() public {
        SP1ProofFixtureJson memory fixture = loadFixture();
        
        bytes memory fakeProof = new bytes(fixture.proof.length);

        fibonacci.verifyAggregateProofAndUpdateRoot(
            abi.encode(fixture.merkle_root),
            fakeProof,
            fixture.merkle_root
        );
    }
}
