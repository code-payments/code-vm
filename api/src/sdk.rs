#![cfg(not(target_os = "solana"))]

use steel::*;
use crate::prelude::*;

pub fn vm_init(vm_authority: Pubkey, mint: Pubkey, lock_duration: u8) -> Instruction {

    let (vm, vm_bump) = find_vm_pda(&mint, &vm_authority, lock_duration);
    let (omnibus, vm_omnibus_bump) = find_vm_omnibus_pda(&vm);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(omnibus, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: InitVmIx {
            lock_duration,
            vm_bump,
            vm_omnibus_bump,
        }
        .to_bytes(),
    }
}

pub fn vm_memory_init(
    vm_authority: Pubkey,
    vm: Pubkey,
    num_accounts: usize,
    account_size: usize,
    name: &str,
) -> Instruction {
    let name = create_name(name);
    let (vm_memory, vm_memory_bump) = find_vm_memory_pda(&vm, &name);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_memory, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: InitMemoryIx::from_struct(
            ParsedInitMemoryIx {
            num_accounts: num_accounts as u32,
            account_size: account_size as u16,
            name,
            vm_memory_bump,
        })
        .to_bytes(),
    }
}

pub fn vm_memory_resize(
    vm_authority: Pubkey,
    vm: Pubkey,
    vm_memory: Pubkey,
    account_size: u32,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_memory, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: ResizeMemoryIx::from_struct(
            ParsedResizeMemoryIx {
            account_size,
        }).to_bytes(),
    }
}

pub fn vm_storage_init(vm_authority: Pubkey, vm: Pubkey, name: &str) -> Instruction {
    let name = create_name(name);
    let (vm_storage, vm_storage_bump) = find_vm_storage_pda(&vm, &name);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_storage, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: InitStorageIx {
            name,
            vm_storage_bump,
        }
        .to_bytes(),
    }
}

pub fn system_nonce_init(
    vm_authority: Pubkey,
    vm: Pubkey,
    vm_memory: Pubkey,
    virtual_account_owner: Pubkey,
    account_index: u16,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_memory, false),
            AccountMeta::new_readonly(virtual_account_owner, false),
        ],
        data: InitNonceIx::from_struct(
            ParsedInitNonceIx {
            account_index,
        }).to_bytes(),
    }
}

pub fn system_timelock_init(
    vm_authority: Pubkey,
    vm: Pubkey,
    vm_memory: Pubkey,
    virtual_account_owner: Pubkey,
    account_index: u16,
    virtual_timelock_bump: u8,
    virtual_vault_bump: u8,
    unlock_pda_bump: u8,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_memory, false),
            AccountMeta::new_readonly(virtual_account_owner, false),
        ],
        data: InitTimelockIx::from_struct( 
            ParsedInitTimelockIx {
            account_index,
            virtual_timelock_bump,
            virtual_vault_bump,
            unlock_pda_bump,
        }).to_bytes(),
    }
}

pub fn system_account_compress(
    vm_authority: Pubkey,
    vm: Pubkey,
    vm_memory: Pubkey,
    vm_storage: Pubkey,
    account_index: u16,
    signature: Signature,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_memory, false),
            AccountMeta::new(vm_storage, false),
        ],
        data: CompressIx::from_struct(
            ParsedCompressIx {
            account_index,
            signature,
        }).to_bytes(),
    }
}

pub fn system_account_decompress(
    vm_authority: Pubkey,
    vm: Pubkey,
    vm_memory: Pubkey,
    vm_storage: Pubkey,
    unlock_pda: Option<Pubkey>,
    withdraw_receipt: Option<Pubkey>,
    account_index: u16,
    packed_va: Vec<u8>,
    proof: Vec<Hash>,
    signature: Signature,
) -> Instruction {
    let args = DecompressIxData {
        account_index,
        packed_va,
        proof,
        signature,
    };

    let data = DecompressIx::try_to_bytes(args).unwrap();

    let accounts = vec![
        AccountMeta::new(vm_authority, true),
        AccountMeta::new(vm, false),
        AccountMeta::new(vm_memory, false),
        AccountMeta::new(vm_storage, false),
        optional_meta(unlock_pda, false),
        optional_meta(withdraw_receipt, false),
    ];

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn vm_exec(
    vm_authority: Pubkey,
    vm: Pubkey,
    mem_a: Option<Pubkey>,
    mem_b: Option<Pubkey>,
    mem_c: Option<Pubkey>,
    mem_d: Option<Pubkey>,
    vm_omnibus: Option<Pubkey>,
    relay: Option<Pubkey>,
    relay_vault: Option<Pubkey>,
    external_address: Option<Pubkey>,
    token_program: Option<Pubkey>,
    opcode: u8,
    mem_indicies: Vec<u16>,
    mem_banks: Vec<u8>,
    data: Vec<u8>,
) -> Instruction {
    let args = ExecIxData {
        opcode,
        mem_indicies,
        mem_banks,
        data,
    };
    let data = ExecIx::try_to_bytes(args).unwrap();

    let accounts = vec![
        AccountMeta::new(vm_authority, true),
        AccountMeta::new(vm, false),
        optional_meta(mem_a, false),
        optional_meta(mem_b, false),
        optional_meta(mem_c, false),
        optional_meta(mem_d, false),
        optional_meta(vm_omnibus, false),
        optional_meta(relay, false),
        optional_meta(relay_vault, false),
        optional_meta(external_address, false),
        optional_readonly_meta(token_program, false),
    ];

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn relay_init(vm_authority: Pubkey, vm: Pubkey, mint: Pubkey, name: &str) -> Instruction {
    let name = create_name(name);
    let (relay, relay_bump) = find_vm_relay_pda(&vm, &name);
    let (relay_vault, relay_vault_bump) = find_vm_relay_vault_pda(&relay);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(relay, false),
            AccountMeta::new(relay_vault, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: InitRelayIx {
            name,
            relay_bump,
            relay_vault_bump,
        }
        .to_bytes(),
    }
}

pub fn relay_save_root(vm_authority: Pubkey, vm: Pubkey, relay: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(relay, false),
        ],
        data: SnapshotIx {}.to_bytes(),
    }
}

pub fn timelock_deposit_from_pda(
    vm_authority: Pubkey,
    vm: Pubkey,
    vm_memory: Pubkey,
    depositor: Pubkey,
    deposit_pda: Pubkey,
    deposit_ata: Pubkey,
    omnibus: Pubkey,
    account_index: u16,
    amount: u64,
    bump: u8,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(vm_authority, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(vm_memory, false),
            AccountMeta::new_readonly(depositor, false),
            AccountMeta::new_readonly(deposit_pda, false),
            AccountMeta::new(deposit_ata, false),
            AccountMeta::new(omnibus, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: DepositIx::from_struct(
            ParsedDepositIx{
            account_index,
            amount,
            bump,
        }).to_bytes(),
    }
}

pub fn timelock_unlock_init(
    account_owner: Pubkey,
    payer: Pubkey,
    vm: Pubkey,
    unlock_pda: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(account_owner, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(unlock_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        ],
        data: InitUnlockIx {}.to_bytes(),
    }
}

pub fn timelock_unlock_finalize(
    account_owner: Pubkey,
    payer: Pubkey,
    vm: Pubkey,
    unlock_pda: Pubkey,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(account_owner, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(vm, false),
            AccountMeta::new(unlock_pda, false),
        ],
        data: UnlockIx {}.to_bytes(),
    }
}

pub fn timelock_withdraw(
    depositor: Pubkey,
    payer: Pubkey,
    vm: Pubkey,
    vm_omnibus: Option<Pubkey>,
    vm_memory: Option<Pubkey>,
    vm_storage: Option<Pubkey>,
    deposit_pda: Option<Pubkey>,
    deposit_ata: Option<Pubkey>,
    unlock_pda: Pubkey,
    withdraw_receipt: Option<Pubkey>,
    external_address: Pubkey,
    data: WithdrawIxData,
) -> Instruction {

    // This instruction has 3 variants, each with a slightly different set of
    // accounts.

    let accounts = match data {
        WithdrawIxData::FromDeposit { .. } => 
            withdraw_from_deposit(
                depositor, payer, vm, deposit_pda, deposit_ata, unlock_pda, external_address),

        WithdrawIxData::FromMemory { .. } => 
            withdraw_from_memory(
                depositor, payer, vm, vm_omnibus, vm_memory, unlock_pda, withdraw_receipt, external_address),

        WithdrawIxData::FromStorage { .. } => 
            withdraw_from_storage(
                depositor, payer, vm, vm_omnibus, vm_storage, unlock_pda, withdraw_receipt, external_address),
    };

    let data = WithdrawIx::try_to_bytes(data).unwrap();

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

fn withdraw_from_deposit(
    depositor: Pubkey,
    payer: Pubkey,
    vm: Pubkey,
    deposit_pda: Option<Pubkey>,
    deposit_ata: Option<Pubkey>,
    unlock_pda: Pubkey,
    external_address: Pubkey,
) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(depositor, true),
        AccountMeta::new(payer, true),
        AccountMeta::new(vm, false),
        optional_meta(None, false), // vm_omnibus
        optional_meta(None, false), // vm_memory
        optional_meta(None, false), // vm_storage
        optional_meta(deposit_pda, false),
        optional_meta(deposit_ata, false),
        AccountMeta::new(unlock_pda, false),
        optional_meta(None, false), // withdraw_receipt
        AccountMeta::new(external_address, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        optional_meta(None, false), // system_program
        optional_meta(None, false), // rent_sysvar
    ]
}

pub fn withdraw_from_memory(
    depositor: Pubkey,
    payer: Pubkey,
    vm: Pubkey,
    vm_omnibus: Option<Pubkey>,
    vm_memory: Option<Pubkey>,
    unlock_pda: Pubkey,
    withdraw_receipt: Option<Pubkey>,
    external_address: Pubkey,
) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(depositor, true),
        AccountMeta::new(payer, true),
        AccountMeta::new(vm, false),
        optional_meta(vm_omnibus, false),
        optional_meta(vm_memory, false),
        optional_meta(None, false), // vm_storage
        optional_meta(None, false), // deposit_pda
        optional_meta(None, false), // deposit_ata
        AccountMeta::new(unlock_pda, false),
        optional_meta(withdraw_receipt, false),
        AccountMeta::new(external_address, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        optional_readonly_meta(Some(system_program::id()), false),
        optional_readonly_meta(Some(solana_program::sysvar::rent::id()), false),
    ]
}

fn withdraw_from_storage(
    depositor: Pubkey,
    payer: Pubkey,
    vm: Pubkey,
    vm_omnibus: Option<Pubkey>,
    vm_storage: Option<Pubkey>,
    unlock_pda: Pubkey,
    withdraw_receipt: Option<Pubkey>,
    external_address: Pubkey,
) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(depositor, true),
        AccountMeta::new(payer, true),
        AccountMeta::new(vm, false),
        optional_meta(vm_omnibus, false),
        optional_meta(None, false), // vm_memory
        optional_meta(vm_storage, false),
        optional_meta(None, false), // deposit_pda
        optional_meta(None, false), // deposit_ata
        AccountMeta::new(unlock_pda, false),
        optional_meta(withdraw_receipt, false),
        AccountMeta::new(external_address, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        optional_readonly_meta(Some(system_program::id()), false),
        optional_readonly_meta(Some(solana_program::sysvar::rent::id()), false),
    ]
}