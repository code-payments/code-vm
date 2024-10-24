pub const CODE_VM: &[u8]              = b"code_vm";
pub const VM_OMNIBUS: &[u8]           = b"vm_omnibus";
pub const VM_MEMORY_ACCOUNT: &[u8]    = b"vm_memory_account";
pub const VM_STORAGE_ACCOUNT: &[u8]   = b"vm_storage_account";
pub const VM_DURABLE_NONCE: &[u8]     = b"vm_durable_nonce";
pub const VM_UNLOCK_ACCOUNT: &[u8]    = b"vm_unlock_pda_account";
pub const VM_WITHDRAW_RECEIPT: &[u8]  = b"vm_withdraw_receipt_account";
pub const VM_DEPOSIT_PDA: &[u8]       = b"vm_deposit_pda";
pub const VM_RELAY_ACCOUNT: &[u8]     = b"vm_relay_account";
pub const VM_RELAY_PROOF: &[u8]       = b"vm_proof_account";
pub const VM_RELAY_VAULT: &[u8]       = b"vm_relay_vault";
pub const VM_RELAY_COMMITMENT: &[u8]  = b"relay_commitment";
pub const VM_TIMELOCK_STATE: &[u8]    = b"timelock_state";
pub const VM_TIMELOCK_VAULT: &[u8]    = b"timelock_vault";
pub const MERKLE_TREE_SEED: &[u8]     = b"merkletree";

pub const MAX_NAME_LEN: usize = 32;

pub const MIXED_MEMORY_SECTORS: usize = 2;
pub const MIXED_MEMORY_PAGES: usize = 255;

pub const COMPACT_STATE_ITEMS: usize = 100;
pub const COMPRESSED_STATE_DEPTH: usize = 24;

pub const RELAY_STATE_DEPTH: usize = 64;
pub const RELAY_HISTORY_ITEMS: usize = 32;