
use steel::*;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::{utils, types::Hash};
use super::{
    VirtualDurableNonce,
    VirtualTimelockAccount,
    VirtualRelayAccount,
};


#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, PartialEq, Debug)]
pub enum VirtualAccount {
    Nonce(VirtualDurableNonce),
    Timelock(VirtualTimelockAccount),
    Relay(VirtualRelayAccount),
}

const fn max3(a: usize, b: usize, c: usize) -> usize {
    let m = if a > b { a } else { b };
    if m > c { m } else { c }
}

impl VirtualAccount {
    /// The largest packed size of any variant: one variant byte plus the
    /// biggest inner account. Derived from the variants themselves so it keeps
    /// tracking them, and small enough to sit on the stack.
    pub const MAX_PACKED_LEN: usize = 1 + max3(
        VirtualDurableNonce::LEN,
        VirtualTimelockAccount::LEN,
        VirtualRelayAccount::LEN,
    );

    /// Get the size of this enum
    pub fn get_size(&self) -> usize {
        1 + (match self {
            VirtualAccount::Nonce(_) => VirtualDurableNonce::LEN,
            VirtualAccount::Timelock(_) => VirtualTimelockAccount::LEN,
            VirtualAccount::Relay(_) => VirtualRelayAccount::LEN,
        })
    }

    pub fn is_timelock(&self) -> bool {
        matches!(self, VirtualAccount::Timelock(_))
    }

    pub fn is_relay(&self) -> bool {
        matches!(self, VirtualAccount::Relay(_))
    }

    pub fn is_nonce(&self) -> bool {
        matches!(self, VirtualAccount::Nonce(_))
    }

    /// Get the hash of this VirtualAccount
    pub fn get_hash(&self) -> Hash {
        let mut bytes = [0u8; Self::MAX_PACKED_LEN];
        let size = self.pack_into(&mut bytes).unwrap();
        utils::hash(&bytes[..size])
    }

    /// Pack this VirtualAccount into a caller provided buffer, returning the
    /// number of bytes written.
    ///
    /// The buffer may be larger than needed (callers reuse a MAX_PACKED_LEN
    /// one), so anything past the returned length is left untouched and must
    /// not be read.
    pub fn pack_into(&self, out: &mut [u8]) -> Result<usize, ProgramError> {
        // The first byte is the variant, followed by the data

        let size = self.get_size();
        let bytes = out.get_mut(..size).ok_or(ProgramError::AccountDataTooSmall)?;

        bytes[0] = match self {
            VirtualAccount::Nonce(_) => 0,
            VirtualAccount::Timelock(_) => 1,
            VirtualAccount::Relay(_) => 2,
        };

        match self {
            VirtualAccount::Nonce(account) => {
                account.pack(&mut bytes[1..]).unwrap();
            },
            VirtualAccount::Timelock(account) => {
                account.pack(&mut bytes[1..]).unwrap();
            },
            VirtualAccount::Relay(account) => {
                account.pack(&mut bytes[1..]).unwrap();
            },
        }

        Ok(size)
    }

    /// Pack this VirtualAccount into a newly allocated byte array. On-chain
    /// callers should prefer pack_into with a stack buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; self.get_size()];
        self.pack_into(&mut bytes).unwrap();
        bytes
    }

    /// Unpack a VirtualAccount from a byte array
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // The first byte is the variant, followed by the data

        if input.len() < 1 {
            return Err(ProgramError::InvalidAccountData);
        }

        let variant = input[0];
        let data = &input[1..];
        let size = get_varient_size(variant);

        if data.len() < size {
            return Err(ProgramError::InvalidAccountData);
        } 

        match variant {
            0 => Ok(VirtualAccount::Nonce(
                VirtualDurableNonce::unpack(&data).unwrap()
            )),
            1 => Ok(VirtualAccount::Timelock(
                VirtualTimelockAccount::unpack(&data).unwrap()
            )),
            2 => Ok(VirtualAccount::Relay(
                VirtualRelayAccount::unpack(&data).unwrap()
            )),
            _ => Err(ProgramError::InvalidAccountData)
        }
    }
}

impl VirtualAccount {
    pub fn into_inner_nonce(self) -> Option<VirtualDurableNonce> {
        if let VirtualAccount::Nonce(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    pub fn into_inner_timelock(self) -> Option<VirtualTimelockAccount> {
        if let VirtualAccount::Timelock(inner) = self {
            Some(inner)
        } else {
            None
        }
    }

    pub fn into_inner_relay(self) -> Option<VirtualRelayAccount> {
        if let VirtualAccount::Relay(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
}

fn get_varient_size(variant: u8) -> usize {
    match variant {
        0 => VirtualDurableNonce::LEN,
        1 => VirtualTimelockAccount::LEN,
        2 => VirtualRelayAccount::LEN,
        _ => 0,
    }
}