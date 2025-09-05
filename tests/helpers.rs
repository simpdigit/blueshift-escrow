use {
    mollusk_svm::Mollusk, solana_account::{Account, WritableAccount}, solana_instruction::{AccountMeta, Instruction}, solana_pubkey::Pubkey, solana_system_program, spl_token::solana_program::program_pack::Pack
};

pub const PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("22222222222222222222222222222222222222222222");
pub const ATOKEN_PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
//const TOKEN_PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
//const TOKEN_2022_PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");


pub fn setup_mollusk() -> Mollusk {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");  
    let mut mollusk = Mollusk::default();
    mollusk.add_program(&PROGRAM_ID, "blueshift_escrow", &mollusk_svm::program::loader_keys::LOADER_V3);
    mollusk.add_program(&spl_token::ID, "../../tests/elf/token", &mollusk_svm::program::loader_keys::LOADER_V3);
    mollusk.add_program(&ATOKEN_PROGRAM_ID, "../../tests/elf/associated_token", &mollusk_svm::program::loader_keys::LOADER_V3);
    // Token 2022 is not needed, but if you want to test with it, uncomment below
    //mollusk.add_program(&spl_token_2022::ID, "../../tests/elf/token_2022", &mollusk_svm::program::loader_keys::LOADER_V3);
    
    mollusk
}


// Helper function to create a Make instruction
pub fn create_make_instruction(
    maker: &Pubkey,
    escrow: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    maker_ata_a: &Pubkey,
    vault: &Pubkey,
    seed: u64,
    receive: u64,
    amount: u64,
) -> Instruction {
    let mut instruction_data = vec![0u8]; // Make discriminator
    instruction_data.extend_from_slice(&seed.to_le_bytes());
    instruction_data.extend_from_slice(&receive.to_le_bytes());
    instruction_data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*maker, true),               // maker (signer)
            AccountMeta::new(*escrow, false),             // escrow (PDA)
            AccountMeta::new_readonly(*mint_a, false),    // mint_a
            AccountMeta::new_readonly(*mint_b, false),    // mint_b
            AccountMeta::new(*maker_ata_a, false),        // maker_ata_a
            AccountMeta::new(*vault, false),              // vault (ATA)
            AccountMeta::new_readonly(solana_system_program::id(), false), // system_program
            AccountMeta::new_readonly(spl_token::ID, false), // token_program
            AccountMeta::new_readonly(PROGRAM_ID, false), // rent sysvar (not used but placeholder)
        ],
        data: instruction_data,
    }
}

// Helper function to create a mint account
/// The function returns a fully initialized token mint account that can:
/// Issue new tokens (via the mint authority)
/// Track total supply
/// Define token precision (decimals)
/// Be used by other token accounts that hold this mint's tokens
pub fn create_mint_account(mint_authority: &Pubkey, decimals: u8) -> Account {
    // Step 1: Allocate Data Buffer
    // Creates a zero-filled byte vector with the exact size needed for a mint account
    //  spl_token::state::Mint::LEN is the fixed size (82 bytes) required for mint data
    let mut mint_data = vec![0u8; spl_token::state::Mint::LEN];

    // Step 2: Create Mint State Structure
    // Creates the mint's state with:
    // mint_authority: Who can mint/burn tokens 
    // supply: Total tokens in circulation 
    // decimals: Number of decimal places
    // is_initialized: Mint is ready to use
    // freeze_authority: Who can freeze accounts

    let mint = spl_token::state::Mint {
        mint_authority: Some(*mint_authority).into(),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };

    // Step 3: Serialize Mint Data
    // Converts the mint struct into binary format that Solana can understand
    // pack() method serializes the mint state into the byte buffer
    // .unwrap() panics if serialization fails (acceptable in tests)
    spl_token::state::Mint::pack(mint, &mut mint_data).unwrap();
    
    // Step 4: Create Solana Account
    // Creates a Solana account with:
    // 1_000_000 lamports: Rent-exempt balance
    // mint_data: The serialized mint information
    // spl_token::ID: Owner is the SPL Token program
    // false: Not executable (data account, not program)
    // 0: Rent epoch (when rent is next due)
    Account::create(1_000_000, mint_data, spl_token::ID, false, 0)
}

// Helper function to create a token account
///  The function returns a fully initialized token account that can:
///  - Hold tokens of a specific mint type
///  - Be transferred from/to by the owner
///  - Track the current balance of tokens
///  - Be used in token operations like transfers, burns, etc.

///  Key Differences from Mint Account:
///  - Token accounts hold tokens of a specific type and belong to a user
///  - Mint accounts define the token type itself and control issuance
///  - Token accounts reference a mint account to know what type of tokens they hold
///  - Token accounts need higher rent (2M lamports) vs mint accounts (1M lamports)
pub fn create_token_account(owner: &Pubkey, mint: &Pubkey, amount: u64) -> Account {
    //Step 1: Allocate Data Buffer
    // Creates a zero-filled byte vector with the exact size needed for a token account
    // spl_token::state::Account::LEN is the fixed size (165 bytes) required for token account data
    let mut account_data = vec![0u8; spl_token::state::Account::LEN];

    // Step 2: Create Token Account State Structure
    // - mint: Which token type this account holds (e.g., USDC, SOL)
    // - owner: Who can transfer/spend these tokens
    // - amount: How many tokens are currently in the account
    // - delegate: Optional account that can spend on behalf of owner (set to None)
    // - state: Account is ready to use (Initialized)
    // - is_native: Whether this is a native SOL account (set to None)
    // - delegated_amount: How many tokens the delegate can spend (0)
    // - close_authority: Who can close this account (set to None)
    let token_account = spl_token::state::Account {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: None.into(),
        state: spl_token::state::AccountState::Initialized,
        is_native: None.into(),
        delegated_amount: 0,
        close_authority: None.into(),
    };
    // Step 3: Serialize Token Account Dat
    spl_token::state::Account::pack(token_account, &mut account_data).unwrap();
    
    // Step 4: Create Solana Account
    // - 2_000_000 lamports: Rent-exempt balance (higher than mint because token accounts need more rent)
    // - account_data: The serialized token account information
    // - spl_token::ID: Owner is the SPL Token program
    // - false: Not executable (data account, not program)
    // - 0: Rent epoch (when rent is next due)
    Account::create(2_000_000, account_data, spl_token::ID, false, 0)
}

// Helper function to derive escrow PDA
pub fn derive_escrow_pda(maker: &Pubkey, seed: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &PROGRAM_ID,
    )
}

// Helper function to derive associated token account
pub fn derive_associated_token_account(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(owner, mint)
}