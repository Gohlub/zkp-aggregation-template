//! A simple program that aggregates the proofs of multiple programs proven with the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);
use rs_merkle::{algorithms::Sha256 as MerkleSha256, Hasher, MerkleTree};
use sha2::{Digest, Sha256};

pub fn words_to_bytes_le(words: &[u32; 8]) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    for i in 0..8 {
        let word_bytes = words[i].to_le_bytes();
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&word_bytes);
    }
    bytes
}

/// Encode a list of vkeys and committed values into a single byte array using a merkle tree
pub fn commit_proof_pairs(vkeys: &[[u32; 8]], committed_values: &[Vec<u8>]) -> Vec<u8> {
    assert_eq!(vkeys.len(), committed_values.len());

    // Prepare the leaves by concatenating vkey and value, then hashing
    let leaves: Vec<[u8; 32]> = vkeys
        .iter()
        .zip(committed_values.iter())
        .map(|(vkey, value)| {
            let concat = [&words_to_bytes_le(vkey)[..], value].concat();
            MerkleSha256::hash(&concat)
        })
        .collect();

    // Create the Merkle tree and get the root
    let merkle_tree = MerkleTree::<MerkleSha256>::from_leaves(&leaves);
    merkle_tree
        .root()
        .expect("Tree should have a root with valid leaves")
        .to_vec()
}

pub fn main() {
    // Read the verification keys.
    let vkeys = sp1_zkvm::io::read::<Vec<[u32; 8]>>();

    // Read the public values.
    let public_values = sp1_zkvm::io::read::<Vec<Vec<u8>>>();

    // Verify the proofs.
    assert_eq!(vkeys.len(), public_values.len());
    for i in 0..vkeys.len() {
        let vkey = &vkeys[i];
        let public_values = &public_values[i];
        let public_values_digest = Sha256::digest(public_values);
        sp1_zkvm::lib::verify::verify_sp1_proof(vkey, &public_values_digest.into());
    }

    // Only commit the root
    let root = commit_proof_pairs(&vkeys, &public_values);
    sp1_zkvm::io::commit_slice(&root);
}
