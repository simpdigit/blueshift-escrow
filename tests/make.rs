use {
    mollusk_svm::{
        program::keyed_account_for_system_program,
        result::{Check, ProgramResult}
    }, 
    solana_account::Account, 
    solana_pubkey::Pubkey, 
    solana_system_program, 
    spl_token::solana_program::program_error::ProgramError,
    crate::helpers::*
};



#[test]
fn test_make_instruction_success() {
    let mollusk = setup_mollusk();
    
    // Test parameters
    let seed = 12345u64;
    let receive_amount = 1000u64;
    let deposit_amount = 500u64;

    // Generate test keypairs
    let maker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();

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
        (vault, create_token_account(&escrow, &mint_a, 1000)), // Vault has tokens
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Process and validate the instruction
    mollusk.process_and_validate_instruction(
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
    );
}


#[test]
fn test_make_instruction_zero_amount_fails() {
    let mollusk = setup_mollusk();

    let seed = 12345u64;
    let receive_amount = 1000u64;
    let deposit_amount = 0u64; // Invalid: zero amount. The program must handle this and return Failure(InvalidInstructionData)

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
        (maker_ata_a, create_token_account(&maker, &mint_a, 1000)), // Maker has tokens
        (vault, create_token_account(&escrow, &mint_a, 0)), // Vault does not have tokens
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Should fail due to zero amount
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        // Example error log:
        // [2025-09-05T12:10:57.503782000Z DEBUG solana_runtime::message_processor::stable_log] Program 22222222222222222222222222222222222222222222 failed: invalid instruction data
        &[Check::program_result(ProgramResult::Failure(ProgramError::InvalidInstructionData))],
    );
}


#[test]
fn test_make_instruction_insufficient_balance_fails() {
    let mollusk = setup_mollusk();

    let seed = 12345u64;
    let receive_amount = 1000u64;
    let deposit_amount = 2000u64; // Invalid: The Maker only has 1000 tokens, but wants to deposit 2000, see below.

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
        (maker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (escrow, Account::new(0, 0, &solana_system_program::id())),
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (maker_ata_a, create_token_account(&maker, &mint_a, 1000)), // Invalid: The Maker only has 1000 tokens, but wants to deposit 2000.
        (vault, create_token_account(&escrow, &mint_a, 1000)), 
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Should fail due to insufficient balance
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        // Example error log:
        // [2025-09-05T12:07:32.562774000Z DEBUG solana_runtime::message_processor::stable_log] Program log: Error: insufficient funds
        &[Check::program_result(ProgramResult::Failure(ProgramError::Custom(1)))],
        
    );
}

/* 
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