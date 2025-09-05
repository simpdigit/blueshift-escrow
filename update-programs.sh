#!/bin/bash

solana program dump TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA ./tests/elf/token.so -u mainnet-beta
solana program dump TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb ./tests/elf/token_2022.so -u mainnet-beta
solana program dump ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL ./tests/elf/associated_token.so -u mainnet-beta
solana slot -u mainnet-beta | xargs -I {} sed -i '' 's|//! Last updated at mainnet-beta slot height: .*|//! Last updated at mainnet-beta slot height: {}|' ./tests/lib.rs