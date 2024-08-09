use anchor_lang::prelude::*;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::types::Hash;
use crate::utils;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct VirtualTimelockAccount {
    pub owner: Pubkey,
    pub nonce: Hash,

    pub token_bump: u8,
    pub unlock_bump: u8,
    pub withdraw_bump: u8,

    pub balance: u64,
    pub bump: u8,
}

impl VirtualTimelockAccount {
    pub const LEN: usize = // 76 bytes
        32 + // owner
        32 + // nonce
        1 +  // token_bump
        1 +  // unlock_bump
        1 +  // withdraw_bump
        8 +  // balance
        1;   // bump

    pub fn get_timelock_address(&self, mint: Pubkey, authority: Pubkey, lock_duration: u8) -> Pubkey {
        utils::create_virtual_timelock_address(
            mint,
            authority,
            self.owner,
            lock_duration,
            self.bump
        )
    }

    pub fn get_token_address(&self, timelock: Pubkey) -> Pubkey {
        utils::create_virtual_timelock_vault_address(
            timelock,
            self.token_bump
        )
    }

    pub fn get_unlock_address(&self, timelock: Pubkey, vm: Pubkey) -> Pubkey {
        utils::create_unlock_address(
            self.owner,
            timelock,
            vm,
            self.unlock_bump
        )
    }

    pub fn get_withdraw_receipt_address(&self, unlock_pda: Pubkey, vm: Pubkey) -> Pubkey {
        utils::create_withdraw_receipt_address(
            unlock_pda, 
            self.nonce, 
            vm, 
            self.withdraw_bump
        )
    }

    pub fn pack<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        BorshSerialize::serialize(self, &mut writer)
    }

    pub fn unpack(buf: &[u8]) -> std::io::Result<Self> {
        let data = &buf[..VirtualTimelockAccount::LEN];
        BorshDeserialize::try_from_slice(data)
    }

}