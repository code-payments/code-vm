use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum TimelockState {
    Unknown = 0,
    Unlocked,
    WaitingForTimeout
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct UnlockStateAccount {
    pub vm: Pubkey,
    pub owner: Pubkey,
    pub address: Pubkey,
    pub unlock_at: i64,
    pub bump: u8,
    pub state: u8,

    _padding: [u8; 6],
}

impl UnlockStateAccount {
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

    pub fn is_unlocked(&self) -> bool {
        self.state == TimelockState::Unlocked as u8
    }

    pub fn is_waiting(&self) -> bool {
        self.state == TimelockState::WaitingForTimeout as u8
    }

}