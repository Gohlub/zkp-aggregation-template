# SP1 Proof Aggregator Template 
## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)

## Brief Overview of the Project Structure
The structure of the project loosely follows the conventions of the [SP1 template](https://github.com/succinctlabs/sp1-project-template/tree/main), and in this convention, we have two primary components: `program` and `script`.

- `program/`: The program we want to generate a proof for. In our case, we have two programs: `fibonacci/`, which is a simple program that computes the nth Fibonacci number, for which we generate a bundle of core proofs, and `program/`, the program that aggregates the proofs.

- `script/`: The script that will build the program, run I/O operations, and facilitate the proof generation process.

## Project Components
- `fibonacci/`: The program that computes the nth Fibonacci number. This module includes both the script (which is used to generate the core proof) and the core fibonacci program.

- `program/`: This is the program that will aggregate the proofs. It reads the strean of incoming proofs from the zkVM (in the form of verification keys and public values), verifies the proofs, and bundles the verification keys and public values into a Merkle Tree whose root is then commited to the 'public output' of the program. 

- `script/`: The script builds the elf files for the `fibonacci/` and `program/` modules, writes the inputs, runs them through the zkVM, and then collects the proofs into an appropriate format. Important note here is that the script does not aggregate the proofs, it only prepares them for aggregation: it reads the 'public output' of the top-level `program/`, which is the root of the Merkle Tree, and prepares it for the on-chain contract. It also captures the verification keys and public values for the aggregated proof, bundling it with the root to be put on-chain for later verification.

- `contracts/`: The Solidity contracts for the on-chain verification of the aggregated proof with a Merkle Tree root.

- `elf/`: The elf file in which the top-level program script is compiled to.

### Build the Program

To build the `fibonacci/` and `program/` programs, run the following command. Keep in mind that the top-level script will handle building the programs for you, so you don't need to manually build the programs.

To build the `program/` program, run the following command:
```sh
cd program
cargo prove build
```

If you, however, want to generate a local compressed proof of the fibonacci program for benchmarking purposes, you can do that by running the following command:

```sh
cd fibonacci/script
cargo run --release -- --prove
```
If you are on Linux/MacOS, you can pass the above function call as an argument to `time` to see how long it takes to generate the proof:

```sh
time cargo run --release -- --prove
```
### Execute the Program

To run the program without generating a proof:

```sh
cd script
cargo run --release -- --execute
```

You can run the above command to test the program you are producing proofs for.

### Generate a Core Proof

To generate a core proof for your program:

```sh
cd script
cargo run --release -- --prove
```

### Generate an EVM-Compatible Proof (Unpreferred Method)

> [!WARNING]
> You will need at least 128GB RAM to generate a Groth16 or PLONK proof.

To generate a proof that is small enough to be verified on-chain and verifiable by the EVM:

```sh
cd script
cargo run --release --bin evm_aggregate -- --system groth16
```

this will generate a Groth16 proof. If you want to generate a PLONK proof, run the following command:

```sh
cargo run --release --bin evm_aggregate -- --system plonk
```

These commands will also generate fixtures that can be used to test the verification of SP1 zkVM proofs
inside Solidity. The fixtures found in `contracts/src/fixtures/` are for the produced by the top-level aggregator script. More details on the fixtures can be found in the README.md file in the `contracts/` directory.

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command:

```sh
cargo prove vkey --elf elf/riscv32im-succinct-zkvm-elf
```

## Using the Prover Network

We highly recommend using the Succinct prover network for any non-trivial programs or benchmarking purposes. For more information, see the [setup guide](https://docs.succinct.xyz/generating-proofs/prover-network.html).

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `SP1_PRIVATE_KEY`
environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run the following
command:

```sh
SP1_PROVER=network SP1_PRIVATE_KEY=... cargo run --release --bin evm
```

