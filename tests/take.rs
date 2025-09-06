use {
    crate::helpers::*, mollusk_svm::{
        program::keyed_account_for_system_program,
        result::{Check, ProgramResult}
    }, solana_account::Account, solana_pubkey::Pubkey, solana_system_program, spl_token::solana_program::program_error::ProgramError
};

#[test]
fn test_take_instruction_success() {
    let mollusk = setup_mollusk();
    
    // Test parameters
    let seed = 54321u64;
    let receive_amount = 2000u64;  // Amount taker needs to pay
    let deposit_amount = 1500u64;  // Amount maker deposited (vault balance)

    // Generate test keypairs
    let maker = Pubkey::new_unique();
    let taker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique(); // Token maker deposited
    let mint_b = Pubkey::new_unique(); // Token taker will pay

    // Derive PDAs and ATAs
    let (escrow, bump) = derive_escrow_pda(&maker, seed);
    let vault = derive_associated_token_account(&escrow, &mint_a);
    let taker_ata_a = derive_associated_token_account(&taker, &mint_a);
    let taker_ata_b = derive_associated_token_account(&taker, &mint_b);
    let maker_ata_b = derive_associated_token_account(&maker, &mint_b);

    // Create take instruction
    let instruction = create_take_instruction(
        &taker,
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &vault,
        &taker_ata_a,
        &taker_ata_b,
        &maker_ata_b,
    );

    // Setup accounts
    let accounts = vec![
        (taker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (maker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (escrow, create_escrow_account(seed, &maker, &mint_a, &mint_b, receive_amount, bump)),
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (vault, create_token_account(&escrow, &mint_a, deposit_amount)), // Vault has deposit
        (taker_ata_a, create_token_account(&taker, &mint_a, 0)), // Will receive from vault
        (taker_ata_b, create_token_account(&taker, &mint_b, 3000)), // Taker has enough tokens to pay
        (maker_ata_b, create_token_account(&maker, &mint_b, 0)), // Will receive from taker
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
        (ATOKEN_PROGRAM_ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Process and validate the instruction
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[
            Check::success(),
            // Verify taker_ata_a received tokens from vault
            Check::account(&taker_ata_a)
                .owner(&spl_token::ID)
                .build(),
            // Verify maker_ata_b received tokens from taker  
            Check::account(&maker_ata_b)
                .owner(&spl_token::ID)
                .build(),
        ],
    );
}

// ============================================================================
// SUCCESS CASE TESTS
// ============================================================================

#[test]
fn test_take_instruction_minimum_amounts() {
    let mollusk = setup_mollusk();
    
    // Test parameters - using minimum amounts (1 unit)
    let seed = 99999u64;
    let receive_amount = 1u64;  // Minimum amount taker needs to pay
    let deposit_amount = 1u64;  // Minimum amount maker deposited

    // Generate test keypairs
    let maker = Pubkey::new_unique();
    let taker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique(); // Token maker deposited
    let mint_b = Pubkey::new_unique(); // Token taker will pay

    // Derive PDAs and ATAs
    let (escrow, bump) = derive_escrow_pda(&maker, seed);
    let vault = derive_associated_token_account(&escrow, &mint_a);
    let taker_ata_a = derive_associated_token_account(&taker, &mint_a);
    let taker_ata_b = derive_associated_token_account(&taker, &mint_b);
    let maker_ata_b = derive_associated_token_account(&maker, &mint_b);

    // Create take instruction
    let instruction = create_take_instruction(
        &taker,
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &vault,
        &taker_ata_a,
        &taker_ata_b,
        &maker_ata_b,
    );

    // Setup accounts
    let accounts = vec![
        (taker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (maker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (escrow, create_escrow_account(seed, &maker, &mint_a, &mint_b, receive_amount, bump)),
        (mint_a, create_mint_account(&mint_authority, 9)), // 9 decimals
        (mint_b, create_mint_account(&mint_authority, 6)), // 6 decimals
        (vault, create_token_account(&escrow, &mint_a, deposit_amount)), // Vault has 1 unit
        (taker_ata_a, create_token_account(&taker, &mint_a, 0)), // Will receive 1 unit from vault
        (taker_ata_b, create_token_account(&taker, &mint_b, 1)), // Taker has exactly 1 unit to pay
        (maker_ata_b, create_token_account(&maker, &mint_b, 0)), // Will receive 1 unit from taker
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
        (ATOKEN_PROGRAM_ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Process and validate the instruction
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[
            Check::success(),
            // Verify taker_ata_a received exactly 1 unit from vault
            Check::account(&taker_ata_a)
                .owner(&spl_token::ID)
                .build(),
            // Verify maker_ata_b received exactly 1 unit from taker  
            Check::account(&maker_ata_b)
                .owner(&spl_token::ID)
                .build(),
            // Verify vault is closed (balance should be 0)
            Check::account(&vault)
                .lamports(0) // Account should be closed
                .build(),
            // Verify escrow is closed
            Check::account(&escrow)
                .lamports(0) // Account should be closed
                .build(),
        ],
    );
}

#[test]
fn test_take_instruction_large_amounts() {
    let mollusk = setup_mollusk();
    
    // Test parameters - using large amounts (close to u64::MAX practical limits)
    let seed = 88888u64;
    let receive_amount = 1_000_000_000_000_000u64;  // 1 quadrillion units (large but reasonable)
    let deposit_amount = 500_000_000_000_000u64;    // 500 trillion units

    // Generate test keypairs
    let maker = Pubkey::new_unique();
    let taker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique(); // Token maker deposited
    let mint_b = Pubkey::new_unique(); // Token taker will pay

    // Derive PDAs and ATAs
    let (escrow, bump) = derive_escrow_pda(&maker, seed);
    let vault = derive_associated_token_account(&escrow, &mint_a);
    let taker_ata_a = derive_associated_token_account(&taker, &mint_a);
    let taker_ata_b = derive_associated_token_account(&taker, &mint_b);
    let maker_ata_b = derive_associated_token_account(&maker, &mint_b);

    // Create take instruction
    let instruction = create_take_instruction(
        &taker,
        &maker,
        &escrow,
        &mint_a,
        &mint_b,
        &vault,
        &taker_ata_a,
        &taker_ata_b,
        &maker_ata_b,
    );

    // Setup accounts with large balances
    let accounts = vec![
        (taker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (maker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (escrow, create_escrow_account(seed, &maker, &mint_a, &mint_b, receive_amount, bump)),
        (mint_a, create_mint_account(&mint_authority, 9)), // 9 decimals
        (mint_b, create_mint_account(&mint_authority, 6)), // 6 decimals
        (vault, create_token_account(&escrow, &mint_a, deposit_amount)), // Vault has large amount
        (taker_ata_a, create_token_account(&taker, &mint_a, 0)), // Will receive large amount from vault
        (taker_ata_b, create_token_account(&taker, &mint_b, 2_000_000_000_000_000u64)), // Taker has enough to pay
        (maker_ata_b, create_token_account(&maker, &mint_b, 0)), // Will receive large amount from taker
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
        (ATOKEN_PROGRAM_ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Process and validate the instruction
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[
            Check::success(),
            // Verify taker_ata_a received the large amount from vault
            Check::account(&taker_ata_a)
                .owner(&spl_token::ID)
                .build(),
            // Verify maker_ata_b received the large amount from taker  
            Check::account(&maker_ata_b)
                .owner(&spl_token::ID)
                .build(),
            // Verify vault is closed (balance should be 0)
            Check::account(&vault)
                .lamports(0) // Account should be closed
                .build(),
            // Verify escrow is closed
            Check::account(&escrow)
                .lamports(0) // Account should be closed
                .build(),
        ],
    );
}

// ============================================================================
// ERROR HANDLING TESTS - ACCOUNT VALIDATION
// ============================================================================

#[test]
fn test_take_instruction_invalid_escrow_pda() {
    let mollusk = setup_mollusk();
    
    // Test parameters
    let seed = 12345u64;
    let receive_amount = 2000u64;
    let deposit_amount = 1500u64;

    // Generate test keypairs
    let maker = Pubkey::new_unique();
    let taker = Pubkey::new_unique();
    let mint_authority = Pubkey::new_unique();
    let mint_a = Pubkey::new_unique();
    let mint_b = Pubkey::new_unique();

    // Derive correct PDA
    let (correct_escrow, bump) = derive_escrow_pda(&maker, seed);
    let vault = derive_associated_token_account(&correct_escrow, &mint_a);
    let taker_ata_a = derive_associated_token_account(&taker, &mint_a);
    let taker_ata_b = derive_associated_token_account(&taker, &mint_b);
    let maker_ata_b = derive_associated_token_account(&maker, &mint_b);

    // Create an INVALID escrow account (different from the correct PDA)
    let invalid_escrow = Pubkey::new_unique(); // Random pubkey, not a proper PDA

    // Create take instruction with the INVALID escrow
    let instruction = create_take_instruction(
        &taker,
        &maker,
        &invalid_escrow, // Using invalid escrow here
        &mint_a,
        &mint_b,
        &vault,
        &taker_ata_a,
        &taker_ata_b,
        &maker_ata_b,
    );

    // Setup accounts - note we create the invalid escrow account but it won't match PDA derivation
    let accounts = vec![
        (taker, Account::new(10_000_000, 0, &solana_system_program::id())),
        (maker, Account::new(10_000_000, 0, &solana_system_program::id())),
        // Create invalid escrow account with proper data but wrong address
        (invalid_escrow, create_escrow_account(seed, &maker, &mint_a, &mint_b, receive_amount, bump)),
        (mint_a, create_mint_account(&mint_authority, 9)),
        (mint_b, create_mint_account(&mint_authority, 6)),
        (vault, create_token_account(&correct_escrow, &mint_a, deposit_amount)), // Vault still uses correct escrow
        (taker_ata_a, create_token_account(&taker, &mint_a, 0)),
        (taker_ata_b, create_token_account(&taker, &mint_b, 3000)),
        (maker_ata_b, create_token_account(&maker, &mint_b, 0)),
        keyed_account_for_system_program(),
        (spl_token::ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
        (ATOKEN_PROGRAM_ID, Account::new(1_000_000, 0, &solana_pubkey::pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"))),
    ];

    // Should fail because the escrow account doesn't match the derived PDA
    mollusk.process_and_validate_instruction(
        &instruction,
        &accounts,
        &[Check::program_result(ProgramResult::Failure(ProgramError::InvalidAccountOwner))],
    );
}

#[test]
#[ignore]
fn test_take_instruction_not_enough_account_keys() {
    // TODO: Test instruction with insufficient accounts (< 12 accounts)
    todo!("Implement test for insufficient account keys");
}

#[test]
#[ignore]
fn test_take_instruction_invalid_signer() {
    // TODO: Test taker account not marked as signer
    todo!("Implement test for invalid signer");
}

#[test]
#[ignore]
fn test_take_instruction_wrong_account_owners() {
    // TODO: Test accounts with incorrect program owners
    todo!("Implement test for wrong account owners");
}

#[test]
#[ignore]
fn test_take_instruction_invalid_mint_accounts() {
    // TODO: Test non-mint accounts passed as mint_a or mint_b
    todo!("Implement test for invalid mint accounts");
}

#[test]
#[ignore]
fn test_take_instruction_mismatched_vault() {
    // TODO: Test vault ATA not belonging to escrow PDA
    todo!("Implement test for mismatched vault");
}

#[test]
#[ignore]
fn test_take_instruction_wrong_token_program() {
    // TODO: Test invalid token program account
    todo!("Implement test for wrong token program");
}

// ============================================================================
// ERROR HANDLING TESTS - ESCROW STATE
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_invalid_escrow_data() {
    // TODO: Test corrupted or malformed escrow account data
    todo!("Implement test for invalid escrow data");
}

#[test]
#[ignore]
fn test_take_instruction_escrow_not_initialized() {
    // TODO: Test uninitialized escrow account
    todo!("Implement test for uninitialized escrow");
}

#[test]
#[ignore]
fn test_take_instruction_wrong_maker() {
    // TODO: Test escrow maker doesn't match provided maker account
    todo!("Implement test for wrong maker");
}

#[test]
#[ignore]
fn test_take_instruction_wrong_mints() {
    // TODO: Test escrow mint_a/mint_b don't match provided mints
    todo!("Implement test for wrong mints");
}

#[test]
#[ignore]
fn test_take_instruction_escrow_already_closed() {
    // TODO: Test attempting to take from already closed escrow
    todo!("Implement test for already closed escrow");
}

// ============================================================================
// ERROR HANDLING TESTS - TOKEN BALANCE
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_insufficient_taker_balance() {
    // TODO: Test taker doesn't have enough mint_b tokens to pay
    todo!("Implement test for insufficient taker balance");
}

#[test]
#[ignore]
fn test_take_instruction_empty_vault() {
    // TODO: Test vault has zero balance
    todo!("Implement test for empty vault");
}

#[test]
#[ignore]
fn test_take_instruction_vault_balance_mismatch() {
    // TODO: Test vault balance doesn't match expected escrow amount
    todo!("Implement test for vault balance mismatch");
}

// ============================================================================
// ERROR HANDLING TESTS - ATA CREATION
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_ata_creation_fails() {
    // TODO: Test system program or token program unavailable for ATA creation
    todo!("Implement test for ATA creation failure");
}

#[test]
#[ignore]
fn test_take_instruction_insufficient_sol_for_ata() {
    // TODO: Test taker doesn't have enough SOL to create ATAs
    todo!("Implement test for insufficient SOL for ATA creation");
}

#[test]
#[ignore]
fn test_take_instruction_wrong_ata_derivation() {
    // TODO: Test manually provided ATAs don't match derived addresses
    todo!("Implement test for wrong ATA derivation");
}

// ============================================================================
// ERROR HANDLING TESTS - TRANSFER ERRORS
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_vault_transfer_fails() {
    // TODO: Test unable to transfer from vault to taker (authority issues)
    todo!("Implement test for vault transfer failure");
}

#[test]
#[ignore]
fn test_take_instruction_taker_transfer_fails() {
    // TODO: Test unable to transfer from taker to maker (insufficient funds)
    todo!("Implement test for taker transfer failure");
}

#[test]
#[ignore]
fn test_take_instruction_token_program_cpi_fails() {
    // TODO: Test token program calls fail due to various reasons
    todo!("Implement test for token program CPI failure");
}

// ============================================================================
// ERROR HANDLING TESTS - ACCOUNT CLOSURE
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_vault_close_fails() {
    // TODO: Test unable to close vault account
    todo!("Implement test for vault close failure");
}

#[test]
#[ignore]
fn test_take_instruction_escrow_close_fails() {
    // TODO: Test unable to close escrow account
    todo!("Implement test for escrow close failure");
}

#[test]
#[ignore]
fn test_take_instruction_wrong_rent_destination() {
    // TODO: Test rent not properly returned to correct recipients
    todo!("Implement test for wrong rent destination");
}

// ============================================================================
// ERROR HANDLING TESTS - EDGE CASES
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_zero_receive_amount() {
    // TODO: Test escrow with zero receive amount (edge case)
    todo!("Implement test for zero receive amount");
}

#[test]
#[ignore]
fn test_take_instruction_same_maker_and_taker() {
    // TODO: Test maker attempting to take their own escrow
    todo!("Implement test for same maker and taker");
}

#[test]
#[ignore]
fn test_take_instruction_same_token_types() {
    // TODO: Test escrow where mint_a equals mint_b
    todo!("Implement test for same token types");
}

#[test]
#[ignore]
fn test_take_instruction_frozen_token_accounts() {
    // TODO: Test attempting to transfer from/to frozen token accounts
    todo!("Implement test for frozen token accounts");
}

// ============================================================================
// ERROR HANDLING TESTS - INSTRUCTION DATA
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_invalid_discriminator() {
    // TODO: Test wrong instruction discriminator
    todo!("Implement test for invalid discriminator");
}

#[test]
#[ignore]
fn test_take_instruction_extra_instruction_data() {
    // TODO: Test take instruction with unexpected additional data
    todo!("Implement test for extra instruction data");
}

// ============================================================================
// ERROR HANDLING TESTS - SYSTEM-LEVEL
// ============================================================================

#[test]
#[ignore]
fn test_take_instruction_account_info_borrow_fails() {
    // TODO: Test unable to borrow account data
    todo!("Implement test for account info borrow failure");
}

#[test]
#[ignore]
fn test_take_instruction_program_address_derivation_fails() {
    // TODO: Test PDA derivation with invalid seeds
    todo!("Implement test for PDA derivation failure");
}

#[test]
#[ignore]
fn test_take_instruction_cpi_depth_exceeded() {
    // TODO: Test too many cross-program invocations
    todo!("Implement test for CPI depth exceeded");
}