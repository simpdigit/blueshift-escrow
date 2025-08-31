use {
    num_derive::FromPrimitive,
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror::Error,
};
 
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PinocchioError {
    // 0
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    #[error("Account is not a signer")]
    NotSigner,
    #[error("Account owner is invalid")]
    InvalidOwner,
    #[error("Account is not owned by the program")]
    NotProgramOwner,
    #[error("Account data is invalid")]
    #[from(PinocchioError::InvalidAccountData)]
    InvalidAccountData,
    #[error("Account Address is invalid")]
    InvalidAddress,
}

impl From<PinocchioError> for ProgramError {
    fn from(e: PinocchioError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl TryFrom<u32> for PinocchioError {
    type Error = ProgramError;
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(PinocchioError::NotRentExempt),
            1 => Ok(PinocchioError::NotSigner),
            2 => Ok(PinocchioError::InvalidOwner),
            3 => Ok(PinocchioError::NotProgramOwner),
            4 => Ok(PinocchioError::InvalidAccountData),
            5 => Ok(PinocchioError::InvalidAddress),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

impl ToStr for PinocchioError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            PinocchioError::NotRentExempt => "Error: Lamport balance below rent-exempt threshold",
            PinocchioError::NotSigner => "Error: Account is not a signer",
            PinocchioError::InvalidOwner => "Error: Account owner is invalid",
            PinocchioError::NotProgramOwner => "Error: Account is not owned by the program",
            PinocchioError::InvalidAccountData => "Error: Account data is invalid",
            PinocchioError::InvalidAddress => "Error: Account Address is invalid",
        }
    }
}