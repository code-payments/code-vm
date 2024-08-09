use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct TokenPool {
    pub vault: Pubkey,
    pub vault_bump: u8,
}

impl TokenPool {
    pub const LEN: usize = 
        32 +                         // vault
        1  ;                         // vault_bump
}
