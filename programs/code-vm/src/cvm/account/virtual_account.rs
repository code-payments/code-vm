use anchor_lang::prelude::*;

use crate::error::CodeVmError;
use crate::{utils, Hash};
use super::{
    VirtualDurableNonce,
    VirtualTimelockAccount,
    VirtualRelayAccount,
};


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
pub enum VirtualAccount {
    Nonce(VirtualDurableNonce),
    Timelock(VirtualTimelockAccount),
    Relay(VirtualRelayAccount),
}

impl VirtualAccount {
    /// The maximum size that any variant of this enum can be (Relay is the largest)
    pub const MAX_SIZE: usize = 1 + VirtualRelayAccount::LEN;

    /// Get the size of this enum
    pub fn get_size(&self) -> usize {
        1 + (match self {
            VirtualAccount::Nonce(_) => VirtualDurableNonce::LEN,
            VirtualAccount::Timelock(_) => VirtualTimelockAccount::LEN,
            VirtualAccount::Relay(_) => VirtualRelayAccount::LEN,
        })
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
    pub fn unpack(input: &[u8]) -> Result<Self> {
        // The first byte is the variant, followed by the data

        if input.len() < 1 {
            return Err(CodeVmError::InvalidVirtualAccount.into());
        }

        let variant = input[0];
        let data = &input[1..];

        match variant {
            0 => {
                if data.len() < VirtualDurableNonce::LEN {
                    return Err(CodeVmError::InvalidVirtualAccount.into());
                }

                let account = VirtualDurableNonce::unpack(&data).unwrap();
                Ok(VirtualAccount::Nonce(account))
            },
            1 => {
                if data.len() < VirtualTimelockAccount::LEN {
                    return Err(CodeVmError::InvalidVirtualAccount.into());
                }

                let account = VirtualTimelockAccount::unpack(data).unwrap();
                Ok(VirtualAccount::Timelock(account))
            },
            2 => {
                if data.len() < VirtualRelayAccount::LEN {
                    return Err(CodeVmError::InvalidVirtualAccount.into());
                }

                let account = VirtualRelayAccount::unpack(data).unwrap();
                Ok(VirtualAccount::Relay(account))
            },
            _ => Err(CodeVmError::InvalidVirtualAccount.into())
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