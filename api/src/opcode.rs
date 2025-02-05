use crate::types::Hash;
use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum Opcode {
  Unknown = 0,

  TransferOp = 11,
  WithdrawOp = 14,
  RelayOp = 21,

  ExternalTransferOp = 10,
  ExternalWithdrawOp = 13,
  ExternalRelayOp = 20,

  ConditionalTransferOp = 12,

  AirdropOp = 30,
}

instruction!(Opcode, TransferOp);
instruction!(Opcode, WithdrawOp);
instruction!(Opcode, RelayOp);
instruction!(Opcode, ExternalTransferOp);
instruction!(Opcode, ExternalWithdrawOp);
instruction!(Opcode, ExternalRelayOp);
instruction!(Opcode, ConditionalTransferOp);
instruction!(Opcode, AirdropOp);

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TransferOp { // transfer_to_internal
    pub signature: [u8; 64],
    pub amount: [u8; 8], // Pack u64 as [u8; 8]
}

impl TransferOp {
    /// Converts the byte array `amount` to `u64`.
    pub fn to_struct(&self) -> Result<ParsedTransferOp, std::io::Error> {
        Ok(ParsedTransferOp {
            signature: self.signature,
            amount: u64::from_le_bytes(self.amount),
        })
    }

    /// Creates `TransferOp` from the parsed struct by converting `u64` back to byte array.
    pub fn from_struct(parsed: ParsedTransferOp) -> Self {
        TransferOp {
            signature: parsed.signature,
            amount: parsed.amount.to_le_bytes(),
        }
    }
}

pub struct ParsedTransferOp {
    pub signature: [u8; 64],
    pub amount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WithdrawOp { // withdraw_to_internal
    pub signature: [u8; 64],
}

impl WithdrawOp {
    // Since WithdrawOp only contains byte arrays, no conversion methods are necessary.
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RelayOp { // relay_to_internal
    pub amount: [u8; 8],       // Pack u64 as [u8; 8]
    pub transcript: Hash,      // no packing needed
    pub recent_root: Hash,     // no packing needed
    pub commitment: Pubkey,    // no packing needed
}

impl RelayOp {
    /// Converts the byte array `amount` to `u64`.
    pub fn to_struct(&self) -> Result<ParsedRelayOp, std::io::Error> {
        Ok(ParsedRelayOp {
            amount: u64::from_le_bytes(self.amount),
            transcript: self.transcript,
            recent_root: self.recent_root,
            commitment: self.commitment,
        })
    }

    /// Creates `RelayOp` from the parsed struct by converting `u64` back to byte array.
    pub fn from_struct(parsed: ParsedRelayOp) -> Self {
        RelayOp {
            amount: parsed.amount.to_le_bytes(),
            transcript: parsed.transcript,
            recent_root: parsed.recent_root,
            commitment: parsed.commitment,
        }
    }
}

pub struct ParsedRelayOp {
    pub amount: u64,
    pub transcript: Hash,
    pub recent_root: Hash,
    pub commitment: Pubkey,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExternalTransferOp { // transfer_to_external
    pub signature: [u8; 64],
    pub amount: [u8; 8], // Pack u64 as [u8; 8]
}

impl ExternalTransferOp {
    /// Converts the byte array `amount` to `u64`.
    pub fn to_struct(&self) -> Result<ParsedExternalTransferOp, std::io::Error> {
        Ok(ParsedExternalTransferOp {
            signature: self.signature,
            amount: u64::from_le_bytes(self.amount),
        })
    }

    /// Creates `ExternalTransferOp` from the parsed struct by converting `u64` back to byte array.
    pub fn from_struct(parsed: ParsedExternalTransferOp) -> Self {
        ExternalTransferOp {
            signature: parsed.signature,
            amount: parsed.amount.to_le_bytes(),
        }
    }
}

pub struct ParsedExternalTransferOp {
    pub signature: [u8; 64],
    pub amount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExternalWithdrawOp { // withdraw_to_external
    pub signature: [u8; 64],
}

impl ExternalWithdrawOp {
    // Since ExternalWithdrawOp only contains byte arrays, no conversion methods are necessary.
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExternalRelayOp { // relay_to_external
    pub amount: [u8; 8],       // Pack u64 as [u8; 8]
    pub transcript: Hash,      // Assuming Hash is [u8; 32], no change needed
    pub recent_root: Hash,     // Assuming Hash is [u8; 32], no change needed
    pub commitment: Pubkey,    // Assuming Pubkey is [u8; 32], no change needed
}

impl ExternalRelayOp {
    /// Converts the byte array `amount` to `u64`.
    pub fn to_struct(&self) -> Result<ParsedExternalRelayOp, std::io::Error> {
        Ok(ParsedExternalRelayOp {
            amount: u64::from_le_bytes(self.amount),
            transcript: self.transcript,
            recent_root: self.recent_root,
            commitment: self.commitment,
        })
    }

    /// Creates `ExternalRelayOp` from the parsed struct by converting `u64` back to byte array.
    pub fn from_struct(parsed: ParsedExternalRelayOp) -> Self {
        ExternalRelayOp {
            amount: parsed.amount.to_le_bytes(),
            transcript: parsed.transcript,
            recent_root: parsed.recent_root,
            commitment: parsed.commitment,
        }
    }
}

pub struct ParsedExternalRelayOp {
    pub amount: u64,
    pub transcript: Hash,
    pub recent_root: Hash,
    pub commitment: Pubkey,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ConditionalTransferOp { // transfer_to_relay
    pub signature: [u8; 64],
    pub amount: [u8; 8], // Pack u64 as [u8; 8]
}

impl ConditionalTransferOp {
    /// Converts the byte array `amount` to `u64`.
    pub fn to_struct(&self) -> Result<ParsedConditionalTransferOp, std::io::Error> {
        Ok(ParsedConditionalTransferOp {
            signature: self.signature,
            amount: u64::from_le_bytes(self.amount),
        })
    }

    /// Creates `ConditionalTransferOp` from the parsed struct by converting `u64` back to byte array.
    pub fn from_struct(parsed: ParsedConditionalTransferOp) -> Self {
        ConditionalTransferOp {
            signature: parsed.signature,
            amount: parsed.amount.to_le_bytes(),
        }
    }
}

pub struct ParsedConditionalTransferOp {
    pub signature: [u8; 64],
    pub amount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct AirdropOp {
    pub signature: [u8; 64],
    pub amount: [u8; 8], // Pack u64 as [u8; 8]
    pub count: u8,       // Up to 255 airdrops in a single tx (but CU limit will be hit first)
}

impl AirdropOp {
    /// Converts the byte array `amount` to `u64`.
    pub fn to_struct(&self) -> Result<ParsedAirdropOp, std::io::Error> {
        Ok(ParsedAirdropOp {
            signature: self.signature,
            amount: u64::from_le_bytes(self.amount),
            count: self.count,
        })
    }

    /// Creates `AirdropOp` from the parsed struct by converting `u64` back to byte array.
    pub fn from_struct(parsed: ParsedAirdropOp) -> Self {
        AirdropOp {
            signature: parsed.signature,
            amount: parsed.amount.to_le_bytes(),
            count: parsed.count,
        }
    }
}

pub struct ParsedAirdropOp {
    pub signature: [u8; 64],
    pub amount: u64,
    pub count: u8,
}
