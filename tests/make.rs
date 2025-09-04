use {
    //blueshift_escrow::Escrow, 
    crate::associated_token, mollusk_svm::{program::{keyed_account_for_bpf_loader_v3_program, keyed_account_for_system_program}, result::Check, Mollusk}, solana_account::{Account, WritableAccount}, solana_instruction::{AccountMeta, Instruction}, solana_pubkey::Pubkey, solana_system_program, spl_token::solana_program::program_pack::Pack
    
};

const PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("22222222222222222222222222222222222222222222");
// Helper function to create a Make instruction
fn create_make_instruction(
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
            AccountMeta::new(*escrow, true),             // escrow (PDA)
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
fn create_mint_account(mint_authority: &Pubkey, decimals: u8) -> Account {
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
fn create_token_account(owner: &Pubkey, mint: &Pubkey, amount: u64) -> Account {
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
fn derive_escrow_pda(maker: &Pubkey, seed: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &seed.to_le_bytes()],
        &PROGRAM_ID,
    )
}

// Helper function to derive associated token account
fn derive_associated_token_account(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(owner, mint)
}

#[test]
fn test_make_instruction_success() {
    std::env::set_var("SBF_OUT_DIR", "../target/deploy");  
    let mut mollusk = Mollusk::default();
    mollusk.add_program(&PROGRAM_ID, "blueshift_escrow", &mollusk_svm::program::loader_keys::LOADER_V3);
    //let mut mollusk = Mollusk::new(&PROGRAM_ID, "blueshift_escrow");
    mollusk.add_program(&spl_token::ID, "../../tests/elf/token", &mollusk_svm::program::loader_keys::LOADER_V3);
    mollusk.add_program(&associated_token::ID, "../../tests/elf/associated_token", &mollusk_svm::program::loader_keys::LOADER_V3);
    mollusk.add_program(&spl_token_2022::ID, "../../tests/elf/token_2022", &mollusk_svm::program::loader_keys::LOADER_V3);
    // Test parameters
    let seed = 12345u64;
    let receive_amount = 1000u64;
    let deposit_amount = 500u64;

    // Generate test keypairs
    let maker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let empty: Pubkey = Pubkey::default();

    // Derive PDAs
    let (escrow, _bump) = derive_escrow_pda(&maker, seed);
    let maker_ata_a = derive_associated_token_account(&maker, &mint_a);
    let vault = derive_associated_token_account(&escrow, &mint_a);

    // Create instruction
    let instruction = create_make_instruction(
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &maker_ata_a,
        &vault,
        seed,
        receive_amount,
        deposit_amount,
    );

    // Setup accounts
    let accounts = vec![
        (maker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (escrow, Account::new(0, 0, &solana_system_program::id())), // Will be initialized
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (maker_ata_a, create_token_account(&maker, &mint_a, 1000)), // Maker has tokens
        (vault, create_token_account(&escrow, &mint_a, 1000)), // Maker has tokens
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
        (empty, Account::new(1_000_000, 0, &solana_system_program::id())), // Rent sysvar placeholder
        //(spl_token_2022::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
        //(spl_associated_token_account::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111")))
    ];

    // Process and validate the instruction
    /*mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[
            Check::success(),
            // Verify escrow account was created and initialized
            Check::account(&escrow)
                .owner(&PROGRAM_ID)
                .build(),
            // Verify vault was created
            Check::account(&vault)
                .owner(&spl_token::ID)
                .build(),
        ],
    );*/
    let instruction_result= mollusk.process_instruction(&instruction, &accounts);
    assert!(instruction_result.program_result.is_ok(), "Instruction failed: {:?}", instruction_result.program_result);
}

/* 
//#[test]
fn test_make_instruction_zero_amount_fails() {
    let mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/blueshift_escrow");

    let seed = 12345u64;
    let receive_amount = 1000u64;
    let deposit_amount = 0u64; // Invalid: zero amount

    let maker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();

    let (escrow, _bump) = derive_escrow_pda(&maker, seed);
    let maker_ata_a = derive_associated_token_account(&maker, &mint_a);
    let vault = derive_associated_token_account(&escrow, &mint_a);

    let instruction = create_make_instruction(
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &maker_ata_a,
        &vault,
        seed,
        receive_amount,
        deposit_amount,
    );

    let accounts = vec![
        (maker, Account::new(10_000_000, 0, &PROGRAM_ID)),
        (escrow, Account::new(0, 0, &PROGRAM_ID)),
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (maker_ata_a, create_token_account(&maker, &mint_a, 1000)),
        (vault, Account::new(0, 0, &PROGRAM_ID)),
    ];

    // Should fail due to zero amount
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(solana_instruction::InstructionError::InvalidInstructionData)],
    );
}

//#[test]
fn test_make_instruction_insufficient_balance_fails() {
    let mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/blueshift_escrow");

    let seed = 12345u64;
    let receive_amount = 1000u64;
    let deposit_amount = 2000u64; // More than available balance

    let maker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();

    let (escrow, _bump) = derive_escrow_pda(&maker, seed);
    let maker_ata_a = derive_associated_token_account(&maker, &mint_a);
    let vault = derive_associated_token_account(&escrow, &mint_a);

    let instruction = create_make_instruction(
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &maker_ata_a,
        &vault,
        seed,
        receive_amount,
        deposit_amount,
    );

    let accounts = vec![
        (maker, Account::new(10_000_000, 0, &PROGRAM_ID)),
        (escrow, Account::new(0, 0, &PROGRAM_ID)),
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (maker_ata_a, create_token_account(&maker, &mint_a, 1000)), // Only 1000 tokens
        (vault, Account::new(0, 0, &PROGRAM_ID)),
    ];

    // Should fail due to insufficient balance
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(solana_instruction::InstructionError::InsufficientFunds)],
    );
}

//#[test]
fn test_make_instruction_invalid_instruction_data() {
    let mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/blueshift_escrow");

    let maker = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();
    let seed = 12345u64;

    let (escrow, _bump) = derive_escrow_pda(&maker, seed);
    let maker_ata_a = derive_associated_token_account(&maker, &mint_a);
    let vault = derive_associated_token_account(&escrow, &mint_a);

    // Create instruction with invalid data (missing fields)
    let mut invalid_instruction_data = vec![0u8]; // Make discriminator only
    // Missing seed, receive, and amount fields

    let instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(maker, true),
            AccountMeta::new(escrow, false),
            AccountMeta::new_readonly(mint_a, false),
            AccountMeta::new_readonly(mint_b, false),
            AccountMeta::new(maker_ata_a, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(PROGRAM_ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
        ],
        data: invalid_instruction_data,
    };

    let accounts = vec![
        (maker, Account::new(10_000_000, 0, &PROGRAM_ID)),
    ];

    // Should fail due to invalid instruction data
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(solana_instruction::InstructionError::InvalidInstructionData)],
    );
}

//#[test]
fn test_make_instruction_not_enough_accounts() {
    let mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/blueshift_escrow");

    let maker = Pubkey::new_unique();

    // Create instruction with insufficient accounts
    let mut instruction_data = vec![0u8];
    instruction_data.extend_from_slice(&12345u64.to_le_bytes());
    instruction_data.extend_from_slice(&1000u64.to_le_bytes());
    instruction_data.extend_from_slice(&500u64.to_le_bytes());

    let instruction = Instruction {
        program_id: PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(maker, true), // Only one account instead of required 8+
        ],
        data: instruction_data,
    };

    let accounts = vec![
        (maker, Account::new(10_000_000, 0, &PROGRAM_ID)),
    ];

    // Should fail due to not enough account keys
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::err(solana_instruction::InstructionError::NotEnoughAccountKeys)],
    );
}

//#[test]
fn test_make_instruction_escrow_data_validation() {
    let mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/blueshift_escrow");

    let seed = 98765u64;
    let receive_amount = 2000u64;
    let deposit_amount = 750u64;

    let maker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();

    let (escrow, _bump) = derive_escrow_pda(&maker, seed);
    let maker_ata_a = derive_associated_token_account(&maker, &mint_a);
    let vault = derive_associated_token_account(&escrow, &mint_a);

    let instruction = create_make_instruction(
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &maker_ata_a,
        &vault,
        seed,
        receive_amount,
        deposit_amount,
    );

    let accounts = vec![
        (maker, Account::new(10_000_000, 0, &PROGRAM_ID)),
        (escrow, Account::new(0, 0, &PROGRAM_ID)),
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (maker_ata_a, create_token_account(&maker, &mint_a, 1000)),
        (vault, Account::new(0, 0, &PROGRAM_ID)),
    ];

    let result = mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[
            Check::success(),
            Check::account(&escrow)
                .data(Escrow::LEN)
                .owner(&PROGRAM_ID),
        ],
    );

    // Additional validation: Check that the escrow data was set correctly
    // Note: In a real test, you might want to extract and validate the escrow data
    // to ensure seed, maker, mints, and receive amount are correctly stored
}
    */