use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum SpriteManagerError {
    /// 0 - Derived Key Invalid
    #[error("Derived Key Invalid")]
    DerivedKeyInvalid,

    /// 1 - Already initialized
    #[error("Already initialized")]
    AlreadyInitialized,

    /// 2 - The data failed to serialize
    #[error("Failed to serialize")]
    FailedToSerialize,

    /// 3 - Account data borrow failed
    #[error("Failed to borrow account data")]
    FailedToBorrowAccountData,

    /// 4 - Incorrect account owner
    #[error("Incorrect account owner")]
    IncorrectOwner,

    /// 5 - Data type mismatch
    #[error("Data type mismatch")]
    DataTypeMismatch,

    /// 6 - NumericalOverflowError
    #[error("NumericalOverflowError")]
    NumericalOverflow,
}

impl PrintProgramError for SpriteManagerError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<SpriteManagerError> for ProgramError {
    fn from(e: SpriteManagerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for SpriteManagerError {
    fn type_of() -> &'static str {
        "Error Thingy"
    }
}
