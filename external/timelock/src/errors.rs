use solana_program::{
    decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum TimelockError {
    #[error("Invalid timelock state for this instruction")]
    InvalidTimeLockState = 6000,
    #[error("Invalid timelock duration provided")]
    InvalidTimeLockDuration = 6001,
    #[error("Invalid vault account")]
    InvalidVaultAccount = 6002,
    #[error("The timelock period has not yet been reached")]
    InsufficientTimeElapsed = 6003,
    #[error("Insufficient vault funds")]
    InsufficientVaultBalance = 6004,
    #[error("Invalid time authority")]
    InvalidTimeAuthority = 6005,
    #[error("Invalid vault owner")]
    InvalidVaultOwner = 6006,
    #[error("Invalid close authority")]
    InvalidCloseAuthority = 6007,
    #[error("Invalid token balance. Token balance must be zero.")]
    NonZeroTokenBalance = 6008,
    #[error("Invalid dust burn.")]
    InvalidDustBurn = 6009,
    #[error("Invalid token mint.")]
    InvalidTokenMint = 6010,
}
impl From<TimelockError> for ProgramError {
    fn from(e: TimelockError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TimelockError {
    fn type_of() -> &'static str {
        "TimelockError"
    }
}
impl PrintProgramError for TimelockError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError
            + num_traits::FromPrimitive,
    {
        msg!(& self.to_string());
    }
}
