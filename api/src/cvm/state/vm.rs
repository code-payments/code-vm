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
        // hashv streams its chunks, so hashing the account keys in place is
        // byte-for-byte identical to hashing a concatenated buffer, minus the
        // allocation and the copy of every key.
        // poh + opcode + accounts + data. Every handler destructures an
        // exact-length account slice before reaching here, and the largest
        // takes 14, so 24 leaves headroom without paying to initialise a
        // needlessly large array. Anything bigger falls back below.
        const MAX_PARTS: usize = 24;
        let ix_byte = [ix as u8];

        if accounts.len() + 3 <= MAX_PARTS {
            let mut parts: [&[u8]; MAX_PARTS] = [&[]; MAX_PARTS];
            let mut n = 0;

            parts[n] = self.poh.as_ref();
            n += 1;
            parts[n] = &ix_byte;
            n += 1;
            for account in accounts {
                parts[n] = account.key.as_ref();
                n += 1;
            }
            parts[n] = data;
            n += 1;

            self.poh = utils::hashv(&parts[..n]);
        } else {
            let mut message = Vec::new();
            for account in accounts {
                message.extend_from_slice(account.key.as_ref());
            }
            message.extend_from_slice(data);

            self.poh = utils::hashv(&[
                self.poh.as_ref(),
                &ix_byte,
                &message
            ]);
        }

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

