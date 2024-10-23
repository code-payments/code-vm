use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct TokenPool {
    pub vault: Pubkey,
    pub vault_bump: u8,
}
