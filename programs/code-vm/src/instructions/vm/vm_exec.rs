use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{TokenAccount, Token, ID as TOKEN_PROGRAM_ID};

use crate::advance_poh_without_event;
use crate::{
    utils,
    program,
    instruction,
    cvm::{
        processor,
        CodeVm,
        CodeVmAccount, 
        RelayAccount,
    },
    types::Hash,
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    opcode: u8,              // The opcode to execute
    mem_indicies: Vec<u16>,  // List of account indexes (where the account is stored in the VM memory buffer)
    mem_banks: Vec<u8>,      // List of memory banks per account index (which memory bank the account is stored in)
    data: Vec<u8>,           // Opaque data for the instruction
)]
pub struct CodeVmExec<'info> {
    #[account(mut)]
    pub vm_authority: Signer<'info>,

    #[account(
        mut, // the POH value is updated
        constraint = vm.authority == vm_authority.key(),

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump = vm.bump
    )]
    pub vm: Box<Account<'info, CodeVmAccount>>,


    // Memory accounts are used to store the accounts that the VM operates on.
    // If an opcode does not access any accounts, then no memory account is
    // required.
    //
    // This address is for memory bank A, the expected type is MemoryAccount.
    // We're not using anchor to deserialize and validate here as it needs
    // internal casting to a MemoryAccountWithData.
    // 
    // Memory bank A is optional.
    #[account(mut)]
    pub mem_a: Option<AccountInfo<'info>>,

    // Memory bank B is optional.
    #[account(mut)]
    pub mem_b: Option<AccountInfo<'info>>,

    // Memory bank C is optional.
    #[account(mut)]
    pub mem_c: Option<AccountInfo<'info>>,

    // Memory bank D is optional.
    #[account(mut)]
    pub mem_d: Option<AccountInfo<'info>>,

    /// The VM Omnibus account is used for external transfers. This is the
    /// account that tokens are transferred from. This account should only be
    /// provided for for external transfers (withdraws, or payments to
    /// non-virtual accounts).
    #[account(mut,
        token::mint = vm.mint,
        token::authority = vm_omnibus,

        // The seeds are used as CPI signer internally, so it must match this
        // section and can't be anything else.
        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_omnibus",
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub vm_omnibus: Option<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_relay_account",
            relay.name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = relay.bump
    )]
    pub relay: Option<Account<'info, RelayAccount>>,

    #[account(
        mut,
        token::mint = vm.mint,
        token::authority = relay_vault,

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_relay_vault",
            relay.as_ref().unwrap().to_account_info().key.as_ref(),
        ],
        bump = relay.as_ref().unwrap().treasury.vault_bump,
    )]
    pub relay_vault: Option<Account<'info, TokenAccount>>,

    /// The external address is used for external transfers. This is the
    /// account that tokens are transferred to.
    #[account(mut,
        token::mint = vm.mint,
    )]
    pub external_address: Option<Account<'info, TokenAccount>>,

    // If the opcode is a transfer to a real account, the token program is
    // required.
    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Option<Program<'info, Token>>,
}

pub fn cvm_exec(
    ctx: Context<CodeVmExec>,
    opcode: u8,              // The opcode to execute
    mem_indicies: Vec<u16>,  // List of account indexes (where the account is stored in the VM memory buffer)
    mem_banks: Vec<u8>,      // List of memory banks per account index (which memory bank the account is stored in)
    data: Vec<u8>,           // Opaque data for the instruction
) -> Result<()> {

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    // Advance the vm state to include this instruction
    advance_poh_without_event!(ctx, vm,
        get_message_hash(
            &vm, 
            &ctx, 
            opcode, 
            mem_indicies.clone(),
            mem_banks.clone(),
            data.clone()
        )
    );

    processor::exec_opcode(ctx, opcode, mem_indicies, mem_banks, data)
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<CodeVmExec>,
    opcode: u8,
    mem_indicies: Vec<u16>,
    mem_banks: Vec<u8>,
    data: Vec<u8>,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::VmExec {
        opcode,
        mem_indicies,
        mem_banks,
        data,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::VmExec::DISCRIMINATOR.to_vec(),
                data,
            ].concat(),
        }
    ];
    
    let message = utils::message_with_sorted_keys(
        &ix,
        Some(&vm.get_authority()),
        &blockhash,
    );

    let message = message.serialize();
    utils::hash(&message)

}