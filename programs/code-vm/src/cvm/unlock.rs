use anchor_lang::prelude::*;
use crate::types::Hash;

#[account]
pub struct UnlockStateAccount {
    pub vm: Pubkey,        // The VM this account is part of
    pub bump: u8,

    pub owner: Pubkey,     // The owner of the address
    pub address: Pubkey,   // The address to unlock

    pub state: TimelockState,
    pub unlock_at: Option<i64>,
}

impl UnlockStateAccount {
    pub const LEN: usize = 
        8 +                  // anchor (discriminator)
        32 +                 // vm
        1 +                  // bump
        32 +                 // owner
        32 +                 // address
        1  +                 // state
        8  + 1;              // unlock_at + optional flag
}

#[account]
pub struct WithdrawReceiptAccount {
    pub unlock_pda: Pubkey, 
    pub nonce: Hash,
    pub amount: u64,
    pub bump: u8,
}

impl WithdrawReceiptAccount {
    pub const LEN: usize = 
        8 +                  // anchor (discriminator)
        32 +                 // unlock_pda
        32 +                 // nonce
        8  +                 // amount
        1;                   // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum TimelockState {
    Unknown = 0,
    Unlocked,
    WaitingForTimeout,
    Locked,
}