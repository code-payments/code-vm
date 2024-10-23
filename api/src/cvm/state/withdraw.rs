use steel::*;
use crate::types::Hash;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct WithdrawReceiptAccount {
    pub unlock_pda: Pubkey,
    pub nonce: Hash,
    pub amount: u64,
    pub bump: u8,

    _padding: [u8; 7],
}