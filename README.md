# Blueshift Escrow

A Solana program that implements a trustless escrow system for token exchanges between two parties.

## Overview

The Blueshift Escrow program enables secure peer-to-peer token swaps on Solana through an escrow mechanism. A maker deposits tokens they want to exchange and specifies what tokens they want to receive in return. A taker can then fulfill the escrow by providing the requested tokens and receiving the deposited tokens.

The program uses Program Derived Addresses (PDAs) to create secure vaults that hold the escrowed tokens until the exchange is completed or cancelled.

## Instructions

The program supports three main instructions:

### Make

Creates a new escrow and deposits tokens to be exchanged.

**What happens:**
- Creates an escrow account with a unique seed
- Creates a token vault (Associated Token Account) to hold the deposited tokens  
- Transfers the specified amount of tokens from the maker to the vault
- Stores escrow details including the maker, tokens involved, and amount requested

**Key parameters:**
- `seed`: Random seed for PDA derivation (u64)
- `receive`: Amount of token B requested in exchange (u64) 
- `amount`: Amount of token A being deposited (u64)

### Take

Completes the escrow exchange by providing the requested tokens.

**What happens:**
- Validates the escrow account and its parameters
- Transfers the escrowed tokens (token A) from the vault to the taker
- Transfers the requested tokens (token B) from the taker to the maker
- Closes the token vault and returns SOL rent to the maker
- Closes the escrow account and returns SOL rent to the taker

**Requirements:**
- Taker must have sufficient balance of the requested token (token B)
- Creates Associated Token Accounts for both parties if needed

### Refund

Allows the original maker to cancel the escrow and retrieve their deposited tokens.

**What happens:**
- Validates that only the original maker can call this instruction
- Transfers the escrowed tokens back from the vault to the maker
- Closes the token vault and returns SOL rent to the maker  
- Closes the escrow account and returns SOL rent to the maker

**Requirements:**
- Only the maker who created the escrow can call this instruction
- Creates maker's Associated Token Account if needed

## Program Architecture

The program is built using the Pinocchio framework for optimized Solana development and includes:

- **State**: Defines the `Escrow` struct that stores escrow metadata
- **Instructions**: Three instruction handlers (Make, Take, Refund)
- **Helpers**: Utility functions for account validation and initialization
- **Errors**: Custom error types for better error handling

## Binary Location

After building the program, the binary will be available at:
```
target/debug/blueshift_escrow.so
```

## Testing

The project uses a workspace structure with separate test configuration. To run the tests:

```bash
# Run all tests from the root directory
cargo test
```

**Note**: The test framework uses `mollusk-svm` for Solana program testing with all built-in programs enabled. Tests validate the Make, Take, and Refund instructions with various scenarios including success cases, error conditions, and edge cases.

## Building

To build the program for Solana deployment:

```bash
# Build the program using Solana's build system
cargo build-sbf
```

This command compiles the program into a `.so` binary optimized for the Solana runtime.

## Updating Token Program Binaries

The test suite requires up-to-date token program binaries. To update these binaries from mainnet-beta:

```bash
# Download latest token program binaries and update slot height
./update-programs.sh
```

This script downloads the following programs:
- Token Program (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
- Token-2022 Program (TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb)  
- Associated Token Program (ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL)

It also updates the slot height reference in the test files to ensure compatibility with the latest mainnet state.

## Program ID

The program uses a fixed Program ID defined in `src/lib.rs`:
```
22222222222222222222222222222222222222222222
```