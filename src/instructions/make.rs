use pinocchio::{account_info::AccountInfo, instruction::Seed, program_error::ProgramError, pubkey::find_program_address, ProgramResult};
use pinocchio_log::log;
use pinocchio_token::instructions::Transfer;

use crate::{helpers::{AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, MintInterface, ProgramAccount, ProgramAccountInit, SignerAccount}, Escrow};


pub struct MakeAccounts<'a> {
  pub maker: &'a AccountInfo,
  pub escrow: &'a AccountInfo,
  pub mint_a: &'a AccountInfo,
  pub mint_b: &'a AccountInfo,
  pub maker_ata_a: &'a AccountInfo,
  pub vault: &'a AccountInfo,
  pub system_program: &'a AccountInfo,
  pub token_program: &'a AccountInfo,
}
 
impl<'a> TryFrom<&'a [AccountInfo]> for MakeAccounts<'a> {
  type Error = ProgramError;
 
  fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
    log!("maker: {}", accounts[0].key());
    log!("escrow: {}", accounts[1].key());
    log!("mint_a: {}", accounts[2].key());
    log!("mint_b: {}", accounts[3].key());
    log!("maker_ata_a: {}", accounts[4].key());
    log!("vault: {}", accounts[5].key());
    log!("system_program: {}", accounts[6].key());
    log!("token_program: {}", accounts[7].key());

    let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] = accounts else {  
      return Err(ProgramError::NotEnoughAccountKeys);
    };
 
    log!("All accounts provided");
    // Basic Accounts Checks
    SignerAccount::check(maker)?;
    MintInterface::check(mint_a)?;
    MintInterface::check(mint_b)?;
    AssociatedTokenAccount::check(maker_ata_a, maker, mint_a, token_program)?;
 
    // Return the accounts
    Ok(Self {
      maker,
      escrow,
      mint_a,
      mint_b,
      maker_ata_a,
      vault,
      system_program,
      token_program,
    })
  }
}

pub struct MakeInstructionData {
  pub seed: u64,
  pub receive: u64,
  pub amount: u64,
}
 
impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
  type Error = ProgramError;
 
  fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
    if data.len() != size_of::<u64>() * 3 {
      return Err(ProgramError::InvalidInstructionData);
    }
 
    let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let receive = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());
 
    // Instruction Checks
    if amount == 0 {
      return Err(ProgramError::InvalidInstructionData);
    }
    log!("Instruction data checks passed");
    Ok(Self {
      seed,
      receive,
      amount,
    })
  }
}

pub struct Make<'a> {
  pub accounts: MakeAccounts<'a>,
  pub instruction_data: MakeInstructionData,
  pub bump: u8,
}
 
impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Make<'a> {
  type Error = ProgramError;
  
  fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
    let accounts = MakeAccounts::try_from(accounts)?;
    let instruction_data = MakeInstructionData::try_from(data)?;
    log!("All accounts and instruction data parsed");
    // Initialize the Accounts needed
    let (_, bump) = find_program_address(&[b"escrow", accounts.maker.key(), &instruction_data.seed.to_le_bytes()], &crate::ID);
 
    let seed_binding = instruction_data.seed.to_le_bytes();
    let bump_binding = [bump];
    let escrow_seeds = [
      Seed::from(b"escrow"),
      Seed::from(accounts.maker.key().as_ref()),
      Seed::from(&seed_binding),
      Seed::from(&bump_binding),
    ];
            
    ProgramAccount::init::<Escrow>(
      accounts.maker,
      accounts.escrow,
      &escrow_seeds,
      Escrow::LEN
    )?;
    log!("Escrow account initialized");
    // Initialize the vault
    
    AssociatedTokenAccount::init_if_needed(
      accounts.vault,
      accounts.mint_a,
      accounts.maker,
      accounts.escrow,
      accounts.system_program,
      accounts.token_program,
    )?;
    log!("Vault account initialized");


    log!("All accounts initialized");
    Ok(Self {
      accounts,
      instruction_data,
      bump,
    })
  }
}

impl<'a> Make<'a> {
  pub const DISCRIMINATOR: &'a u8 = &0;
  
  pub fn process(&mut self) -> ProgramResult {
    // Populate the escrow account
    let mut data = self.accounts.escrow.try_borrow_mut_data()?;
    let escrow = Escrow::load_mut(data.as_mut())?;
    
    escrow.set_inner(
      self.instruction_data.seed,
      *self.accounts.maker.key(),
      *self.accounts.mint_a.key(),
      *self.accounts.mint_b.key(),
      self.instruction_data.receive,
      [self.bump],
    );
 
    // Transfer tokens to vault
    Transfer {
      from: self.accounts.maker_ata_a,
      to: self.accounts.vault,
      authority: self.accounts.maker,
      amount: self.instruction_data.amount
    }.invoke()?;
 
    Ok(())
  }
}