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
}

instruction!(Opcode, TransferOp);
instruction!(Opcode, WithdrawOp);
instruction!(Opcode, RelayOp);
instruction!(Opcode, ExternalTransferOp);
instruction!(Opcode, ExternalWithdrawOp);
instruction!(Opcode, ExternalRelayOp);
instruction!(Opcode, ConditionalTransferOp);

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TransferOp { // transfer_to_internal
  pub signature: [u8; 64],
  pub amount: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct WithdrawOp { // withdraw_to_internal
  pub signature: [u8; 64],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RelayOp { // relay_to_internal
  pub amount: u64,
  pub transcript: Hash,
  pub recent_root: Hash,
  pub commitment: Pubkey,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExternalTransferOp { // transfer_to_external
  pub signature: [u8; 64],
  pub amount: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExternalWithdrawOp { // withdraw_to_external
  pub signature: [u8; 64],
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ExternalRelayOp { // relay_to_external
  pub amount: u64,
  pub transcript: Hash,
  pub recent_root: Hash,
  pub commitment: Pubkey,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ConditionalTransferOp { // transfer_to_relay
  pub signature: [u8; 64],
  pub amount: u64,
}