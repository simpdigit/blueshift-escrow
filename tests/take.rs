use {
    crate::helpers::*, mollusk_svm::{
        program::keyed_account_for_system_program,
        result::Check
    }, solana_account::Account, solana_pubkey::Pubkey, solana_system_program
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