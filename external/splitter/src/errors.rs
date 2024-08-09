use solana_program::{
    decode_error::DecodeError, msg, program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum SplitterError {
    #[error("Invalid pool state for this instruction")]
    InvalidPoolState = 6000,
    #[error("Invalid commitment state for this instruction")]
    InvalidCommitmentState = 6001,
    #[error("Invalid recent root value")]
    InvalidRecentRoot = 6002,
    #[error("Invalid token account")]
    InvalidVaultAccount = 6003,
    #[error("Insufficient vault funds")]
    InsufficientVaultBalance = 6004,
    #[error("Invalid authority")]
    InvalidAuthority = 6005,
    #[error("Invalid vault owner")]
    InvalidVaultOwner = 6006,
    #[error("Merkle tree full")]
    MerkleTreeFull = 6007,
    #[error("Invalid merkle tree depth")]
    InvalidMerkleTreeDepth = 6008,
    #[error("Proof already verified")]
    ProofAlreadyVerified = 6009,
    #[error("Proof not verified")]
    ProofNotVerified = 6010,
    #[error("Invalid proof size")]
    InvalidProofSize = 6011,
    #[error("Invalid proof")]
    InvalidProof = 6012,
}
impl From<SplitterError> for ProgramError {
    fn from(e: SplitterError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for SplitterError {
    fn type_of() -> &'static str {
        "SplitterError"
    }
}
impl PrintProgramError for SplitterError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError
            + num_traits::FromPrimitive,
    {
        msg!(& self.to_string());
    }
}
