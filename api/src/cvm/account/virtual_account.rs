
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

impl VirtualAccount {
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
        utils::hash(self.pack().as_ref())
    }

    /// Pack this VirtualAccount into a byte array
    pub fn pack(&self) -> Vec<u8> {
        // The first byte is the variant, followed by the data

        let mut bytes = vec![0u8; self.get_size()];
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