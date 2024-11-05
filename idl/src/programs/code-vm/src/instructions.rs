use anchor_lang::prelude::*;
use anchor_spl::token::Token;

// Not using the Mint/TokenAccount structs from anchor-spl because they require
// more fields than we need for generating the IDL.

use crate::state::*;

#[derive(Accounts)]
pub struct InitVm<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub omnibus: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitMemory<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitStorage<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_storage: Account<'info, StorageAccount>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitNonce<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    pub virtual_account_owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitRelay<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub relay: Account<'info, RelayAccount>,
    #[account(mut)]
    pub relay_vault: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

 #[derive(Accounts)]
pub struct InitTimelock<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    pub virtual_account_owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitUnlock<'info> {
    #[account(mut)]
    pub account_owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub unlock_pda: Account<'info, UnlockStateAccount>,
    pub system_program: Program<'info, System>,
    pub rent_sysvar: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Compress<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    #[account(mut)]
    pub vm_storage: Account<'info, StorageAccount>,
}

#[derive(Accounts)]
pub struct Decompress<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    #[account(mut)]
    pub vm_storage: Account<'info, StorageAccount>,
    pub unlock_pda: Account<'info, UnlockStateAccount>,
    pub withdraw_receipt: Account<'info, WithdrawReceiptAccount>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    pub depositor: AccountInfo<'info>,
    pub deposit_pda: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_ata: AccountInfo<'info>,
    #[account(mut)]
    pub omnibus: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Exec<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub mem_a: Option<Account<'info, MemoryAccount>>,
    #[account(mut)]
    pub mem_b: Option<Account<'info, MemoryAccount>>,
    #[account(mut)]
    pub mem_c: Option<Account<'info, MemoryAccount>>,
    #[account(mut)]
    pub mem_d: Option<Account<'info, MemoryAccount>>,
    #[account(mut)]
    pub vm_omnibus: Option<AccountInfo<'info>>,
    #[account(mut)]
    pub relay: Option<Account<'info, RelayAccount>>,
    #[account(mut)]
    pub relay_vault: Option<AccountInfo<'info>>,
    #[account(mut)]
    pub external_address: Option<AccountInfo<'info>>,
    pub token_program: Option<Program<'info, Token>>,
}

#[derive(Accounts)]
pub struct ResizeMemory<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_memory: Account<'info, MemoryAccount>,
    pub system_program: Program<'info, System>,
    pub rent_sysvar: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Snapshot<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub relay: Account<'info, RelayAccount>,
}

#[derive(Accounts)]
pub struct Unlock<'info> {
    #[account(mut)]
    pub account_owner: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub unlock_pda: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub vm: Account<'info, CodeVmAccount>,
    #[account(mut)]
    pub vm_omnibus: Option<AccountInfo<'info>>,
    #[account(mut)]
    pub vm_memory: Option<AccountInfo<'info>>,
    pub vm_storage: Option<AccountInfo<'info>>,
    pub deposit_pda: Option<AccountInfo<'info>>,
    #[account(mut)]
    pub deposit_ata: Option<AccountInfo<'info>>,
    pub unlock_pda: Account<'info, UnlockStateAccount>,
    pub withdraw_receipt: Option<Account<'info, WithdrawReceiptAccount>>,
    #[account(mut)]
    pub external_address: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Option<Program<'info, System>>,
    pub rent_sysvar: Option<Sysvar<'info, Rent>>,
}