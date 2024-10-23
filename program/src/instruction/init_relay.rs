use code_vm_api::prelude::*;
use solana_program::{system_program, sysvar};
use steel::*;

/*
    This instruction creates a new relay account and treasury. Relay accounts
    are used to facilitate private transfers using the Code privacy protocol.

    Accounts expected by this instruction:

    | # | R/W | Type         | PDA | Name           | Description                              |
    |---|-----|--------------|-----|----------------|------------------------------------------|
    | 0 | mut | Signer       |     | vm_authority   | The authority of the VM.                 |
    | 1 | mut | Vm           | PDA | vm             | The VM instance state account.           |
    | 2 | mut | Relay        | PDA | vm_relay       | The relay account to create.             |
    | 3 | mut | TokenAccount | PDA | vm_relay_vault | The relay token account to create.       |
    | 4 |     | TokenMint    |     | mint           | The mint to use for this relay.          |
    | 5 |     | Program      |     | token_program  | The SPL token program.                   |
    | 6 |     | Program      |     | system_program | The system program.                      |
    | 7 |     | Sysvar       |     | rent_sysvar    | The rent sysvar.                         |


    Derived account seeds:

    1. vm:           [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. relay:        [ "code_vm", "vm_relay_account", <data.name>, <vm> ]
    2. relay_vault:  [ "code_vm", "vm_relay_vault", <relay> ]


    Instruction data:

    0. name: [u8; 32]        - The name of this storage module.
    1. relay_bump: u8        - The bump seed for the this relay account.
    2. relay_vault_bump: u8  - The bump seed for the relay token account.
*/
pub fn process_init_relay(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = InitRelayIx::try_from_bytes(data)?;
    let [
        vm_authority_info,
        vm_info,
        relay_info,
        relay_vault_info,
        mint_info,
        token_program_info,
        system_program_info,
        rent_sysvar_info 
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);        
    };

    check_signer(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(relay_info)?;
    check_mut(relay_vault_info)?;
    check_readonly(mint_info)?;
    check_program(token_program_info, &spl_token::id())?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    let vm = load_vm_checked(vm_info, vm_authority_info)?;

    check_condition(
        mint_info.key == &vm.mint,
        "mint account does not match VM instance",
    )?;

    check_uninitialized_pda(
        relay_info,
        &[
            CODE_VM,
            VM_RELAY_ACCOUNT,
            args.name.as_ref(),
            vm_info.key.as_ref(),
        ],
        args.relay_bump,
        &code_vm_api::id(),
    )?;
    check_uninitialized_pda(
        relay_vault_info,
        &[
            CODE_VM, 
            VM_RELAY_VAULT,
            relay_info.key.as_ref()
        ],
        args.relay_vault_bump, 
        &code_vm_api::id(),
    )?;

    create_account::<RelayAccount>(
        relay_info,
        &code_vm_api::ID,
        &[
            CODE_VM,
            VM_RELAY_ACCOUNT,
            args.name.as_ref(),
            vm_info.key.as_ref(),
            &[args.relay_bump],
        ],
        system_program_info,
        vm_authority_info,
    )?;

    create_token_account(
        mint_info,
        relay_vault_info,
        &[
            CODE_VM,
            VM_RELAY_VAULT,
            relay_info.key.as_ref(),
            &[args.relay_vault_bump],
        ],
        vm_authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    let relay = relay_info.to_account_mut::<RelayAccount>(&code_vm_api::ID)?;

    relay.vm = vm_info.key.clone();
    relay.bump = args.relay_bump;
    relay.name = args.name;
    relay.num_levels = RELAY_STATE_DEPTH as u8;
    relay.num_history = RELAY_HISTORY_ITEMS as u8;

    relay.treasury.vault = relay_vault_info.key.clone();
    relay.treasury.vault_bump = args.relay_vault_bump;

    relay
        .history
        .init(&[MERKLE_TREE_SEED, relay_info.key.as_ref()]);

    relay.recent_roots.push(relay.history.get_root().as_ref());

    vm.advance_poh(CodeInstruction::InitRelayIx, accounts, data);

    Ok(())
}
