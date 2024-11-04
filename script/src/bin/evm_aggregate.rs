use alloy_sol_types::sol;
use alloy_sol_types::SolValue;
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use sp1_sdk::{
    HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey,
};
use std::path::PathBuf;

/// The ELF files
pub const FIBONACCI_ELF: &[u8] = include_bytes!("../../../elf/fibonacci-elf");
pub const AGGREGATOR_ELF: &[u8] = include_bytes!("../../../elf/aggregator-elf");

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct EVMArgs {
    #[clap(long, value_enum, default_value = "groth16")]
    system: ProofSystem,
    #[clap(long, default_value = "3")]
    num_proofs: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ProofSystem {
    Plonk,
    Groth16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SP1AggregateProofFixture {
    verification_key: String,
    public_values: String,
    proof: String,
}

// Define the Solidity struct that matches our contract
sol! {
    struct AggregateProof {
        bytes32 verification_key;    // aggregator's VK
        bytes32 merkle_root;         // public value (root)
        bytes proof;                 // aggregate proof
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OffChainData {
    verification_keys: Vec<String>,
    public_values: Vec<String>,
    leaf_indices: Vec<usize>,
}

struct AggregationInput {
    proof: SP1ProofWithPublicValues,
    vk: SP1VerifyingKey,
}

fn words_to_bytes_le(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

// Add this after the sol! macro definition
#[derive(Debug, Serialize)]
struct SerializableAggregateProof {
    verification_key: String,
    merkle_root: String,
    proof: String,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    let args = EVMArgs::parse();
    let client = ProverClient::new();

    // Setup both programs
    let (aggregator_pk, aggregator_vk) = client.setup(AGGREGATOR_ELF);
    let (fibonacci_pk, fibonacci_vk) = client.setup(FIBONACCI_ELF);

    // Generate proofs and collect them as AggregationInputs
    let mut inputs = Vec::new();

    for i in 1..=args.num_proofs {
        let proof =
            tracing::info_span!("generate fibonacci proof n={n}", n = i * 10).in_scope(|| {
                let mut stdin = SP1Stdin::new();
                stdin.write(&(i * 10));
                client
                    .prove(&fibonacci_pk, stdin)
                    .compressed()
                    .run()
                    .expect("proving failed")
            });

        inputs.push(AggregationInput {
            proof,
            vk: fibonacci_vk.clone(),
        });
    }
    // Generate aggregate proof
    let (aggregate_proof, vks, pub_vals) =
        tracing::info_span!("aggregate the proofs").in_scope(|| {
            let mut aggregate_stdin = SP1Stdin::new();

            // Write verification keys
            let vks: Vec<_> = inputs.iter().map(|input| input.vk.hash_u32()).collect();
            aggregate_stdin.write(&vks);

            // Write public values
            let pub_vals: Vec<_> = inputs
                .iter()
                .map(|input| input.proof.public_values.to_vec())
                .collect();
            aggregate_stdin.write(&pub_vals);

            // Use inputs (consumes it)
            for input in inputs {
                let SP1Proof::Compressed(proof) = input.proof.proof else {
                    panic!()
                };
                aggregate_stdin.write_proof(*proof, input.vk.vk);
            }

            let aggregate_proof = match args.system {
                ProofSystem::Plonk => client.prove(&aggregator_pk, aggregate_stdin).plonk().run(),
                ProofSystem::Groth16 => client
                    .prove(&aggregator_pk, aggregate_stdin)
                    .groth16()
                    .run(),
            }
            .expect("aggregate proving failed");

            (aggregate_proof, vks, pub_vals)
        });

    save_proof_data(
        &aggregate_proof,
        &aggregator_vk,
        &vks,
        &pub_vals,
        args.system,
    );
}

fn save_proof_data(
    aggregate_proof: &SP1ProofWithPublicValues,
    aggregator_vk: &SP1VerifyingKey,
    verification_keys: &[[u32; 8]],
    public_values: &[Vec<u8>],
    system: ProofSystem,
) {
    // Create the on-chain data using Alloy Sol type
    let on_chain = AggregateProof {
        verification_key: aggregator_vk.hash_bytes().into(),
        merkle_root: aggregate_proof.public_values.as_slice().try_into().unwrap(),
        proof: aggregate_proof.bytes().into(),
    };

    // Create serializable version for raw JSON
    let serializable = SerializableAggregateProof {
        verification_key: format!("0x{}", hex::encode(aggregator_vk.hash_bytes())),
        merkle_root: format!("0x{}", hex::encode(&aggregate_proof.public_values)),
        proof: format!("0x{}", hex::encode(aggregate_proof.bytes())),
    };

    let off_chain = OffChainData {
        verification_keys: verification_keys
            .iter()
            .map(|vk| format!("0x{}", hex::encode(words_to_bytes_le(vk))))
            .collect(),
        public_values: public_values
            .iter()
            .map(|pv| format!("0x{}", hex::encode(pv)))
            .collect(),
        leaf_indices: (0..verification_keys.len()).collect(),
    };

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/src/fixtures");
    std::fs::create_dir_all(&fixture_path).expect("failed to create fixture path");

    // Save the ABI-encoded version (for contract)
    std::fs::write(
        fixture_path.join(format!("{:?}-onchain-abi.json", system).to_lowercase()),
        serde_json::to_string_pretty(&on_chain.abi_encode()).unwrap(),
    )
    .expect("failed to write on-chain ABI data");

    // Save the serializable version
    std::fs::write(
        fixture_path.join(format!("{:?}-onchain.json", system).to_lowercase()),
        serde_json::to_string_pretty(&serializable).unwrap(),
    )
    .expect("failed to write on-chain data");

    std::fs::write(
        fixture_path.join(format!("{:?}-offchain.json", system).to_lowercase()),
        serde_json::to_string_pretty(&off_chain).unwrap(),
    )
    .expect("failed to write off-chain data");
}
