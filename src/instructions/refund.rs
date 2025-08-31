use pinocchio::{account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::create_program_address, ProgramResult};
use pinocchio_token::{instructions::{CloseAccount, Transfer}, state::TokenAccount};

use crate::{helpers::{AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, MintInterface, ProgramAccount, SignerAccount}, Escrow};

use pinocchio_log::log;

pub struct RefundAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for RefundAccounts<'a> {
  type Error = ProgramError;
 
  fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
    let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program, _] = accounts else {
      return Err(ProgramError::NotEnoughAccountKeys);
    };
    log!("RefundAccounts: checking maker_ata_a account");

    // Basic Accounts Checks
    SignerAccount::check(maker)?;
    log!("SignerAccount check passed");
    ProgramAccount::check(escrow)?;
    log!("ProgramAccount check passed");
    MintInterface::check(mint_a)?;
    log!("MintInterface check passed");
    //AssociatedTokenAccount::check(maker_ata_a, maker, mint_a, token_program)?;
    //log!("AssociatedTokenAccount check passed");
    
    log!("RefundAccounts: checking vault account");
    log!("Vault key: {}", vault.key());
    log!("Escrow key: {}", escrow.key());
    
    AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

    log!("RefundAccounts: returning accounts");
    // Return the accounts
    Ok(Self {
      maker,
      escrow,
      mint_a,
      vault,
      maker_ata_a,
      system_program,
      token_program,
    })
  }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Refund<'a> {
    type Error = ProgramError;
  
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        log!("Refund: try_from started");
        let accounts = RefundAccounts::try_from(accounts)?;

        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
            )?;

        Ok(Self {
            accounts,
        })
    }
}

impl<'a> Refund<'a> {
  pub const DISCRIMINATOR: &'a u8 = &2;
  
  pub fn process(&mut self) -> ProgramResult {
    // Populate the escrow account
    log!("Refund: processing refund");
    let data = self.accounts.escrow.try_borrow_data()?;
    let escrow = Escrow::load(&data)?;
    
    // Check if the escrow is valid
    let escrow_key = create_program_address(&[b"escrow", self.accounts.maker.key(), &escrow.seed.to_le_bytes(), &escrow.bump], &crate::ID)?;
    if &escrow_key != self.accounts.escrow.key() {
      return Err(ProgramError::InvalidAccountOwner);
    }
    
    let seed_binding = escrow.seed.to_le_bytes();
    let bump_binding = escrow.bump;

    let escrow_seeds = [
      Seed::from(b"escrow"),
      Seed::from(self.accounts.maker.key().as_ref()),
      Seed::from(&seed_binding),
      Seed::from(&bump_binding),
    ];
    
    let signer = Signer::from(&escrow_seeds);
    // get the amount to be refunded from the vault
    let amount = TokenAccount::from_account_info(self.accounts.vault)?.amount();
    
    // Transfer tokens from vault to maker
    Transfer {
      from: self.accounts.vault,
      to: self.accounts.maker_ata_a,
      authority: self.accounts.escrow,
      amount,
    }.invoke_signed(&[signer.clone()])?;

    // Close the Vault
    CloseAccount {
      account: self.accounts.vault,
      destination: self.accounts.maker,
      authority: self.accounts.escrow,
    }.invoke_signed(&[signer.clone()])?;
    
    // Close the Escrow
    drop(data);
    ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;

    Ok(())
    }
}