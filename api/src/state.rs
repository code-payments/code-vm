
use steel::*;
use crate::cvm::{
    CodeVmAccount, 
    MemoryAccount, 
    RelayAccount, 
    StorageAccount, 
    UnlockStateAccount, 
    WithdrawReceiptAccount
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AccountType {
    Unknown = 0,
    CodeVmAccount,
    MemoryAccount,
    StorageAccount,
    RelayAccount,
    UnlockStateAccount,
    WithdrawReceiptAccount,
}


account!(AccountType, CodeVmAccount);
account!(AccountType, MemoryAccount);
account!(AccountType, StorageAccount);
account!(AccountType, RelayAccount);
account!(AccountType, UnlockStateAccount);
account!(AccountType, WithdrawReceiptAccount);