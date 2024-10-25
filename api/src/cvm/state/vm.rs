use steel::*;

use crate::{
    cvm::TokenPool, 
    instruction::CodeInstruction, 
    types::Hash, 
    utils
};

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct CodeVmAccount {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub slot: u64,
    pub poh: Hash,
    pub omnibus: TokenPool,
    pub lock_duration: u8,  // in days
    pub bump: u8,

    _padding: [u8; 5],
}

impl CodeVmAccount {
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn unpack(data: &[u8]) -> Self {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data).unwrap().clone()
    }

    pub fn unpack_mut(data: &mut [u8]) -> &mut Self {
        let data = &mut data[..Self::get_size()];
        Self::try_from_bytes_mut(data).unwrap()
    }

    pub fn advance_slot(&mut self) {
        self.slot += 1;
    }

    pub fn advance_poh(
        &mut self,
        ix: CodeInstruction,
        accounts: &[AccountInfo],
        data: &[u8],
    ) {
        let mut message = Vec::new();
        for account in accounts {
            message.extend_from_slice(account.key.as_ref());
        }
        message.extend_from_slice(data);

        self.poh = utils::hashv(&[
            self.poh.as_ref(),
            &[ix as u8],
            &message
        ]);

        self.advance_slot();
    }

    #[inline]
    pub fn get_authority(&self) -> Pubkey {
        self.authority
    }

    #[inline]
    pub fn get_mint(&self) -> Pubkey {
        self.mint
    }

    #[inline]
    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    #[inline]
    pub fn get_omnibus_bump(&self) -> u8 {
        self.omnibus.vault_bump
    }

    #[inline]
    pub fn get_lock_duration(&self) -> u8 {
        self.lock_duration
    }

    #[inline]
    pub fn get_current_poh(&self) -> Hash {
        self.poh
    }

    #[inline]
    pub fn get_current_slot(&self) -> u64 {
        self.slot
    }

}

