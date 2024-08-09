use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{TokenAccount, Mint, Token};

use crate::advance_poh;
use crate::error::CodeVmError;
use crate::{
    utils,
    program,
    instruction,
    cvm::{ 
        CodeVm, 
        CodeVmAccount, 
        RelayAccount,
    },
    types::{ Hash, CircularBuffer, MerkleTree }, 
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    num_levels: u8, 
    num_history: u8, 
    name: String,
)]
pub struct RelayInit<'info> {
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

    #[account(
        init, 
        constraint = num_levels <= RelayAccount::MAX_MERKLE_LEVELS,
        constraint = num_history <= RelayAccount::MAX_RECENT_HISTORY,

        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_relay_account",
            &name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        payer = vm_authority, 
        space = RelayAccount::max_size_for(num_levels, num_history),
        bump, 
    )]
    pub relay: Account<'info, RelayAccount>,

    #[account(
        init,
        constraint = mint.key() == vm.mint,

        token::mint = mint,
        token::authority = relay_vault,

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_relay_vault",
            relay.to_account_info().key.as_ref(),
        ],
        payer = vm_authority,
        bump,
    )]
    pub relay_vault: Box<Account<'info, TokenAccount>>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn relay_init(
    ctx: Context<RelayInit>, 
    num_levels: u8,
    num_history: u8,
    name: String
) -> Result<()> {
    require!(num_levels > 0, CodeVmError::InvalidRelayLevels);
    require!(num_history > 0, CodeVmError::InvalidRelayHistorySize);
    require!(name.len() <= RelayAccount::MAX_NAME_LEN, CodeVmError::InvalidRelayName);

    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());
    
    let seeds = &[
        b"merkletree",
        ctx.accounts.relay.to_account_info().key.as_ref()
    ];
    let merkle_tree = MerkleTree::new(seeds, num_levels);
    let mut recent_roots = CircularBuffer::<{Hash::LEN}>::new(num_history);

    recent_roots.push(merkle_tree.get_root().as_ref());

    ctx.accounts.relay.vm = ctx.accounts.vm.key();
    ctx.accounts.relay.bump = ctx.bumps.relay;
    ctx.accounts.relay.num_levels = num_levels;
    ctx.accounts.relay.num_history = num_history;
    ctx.accounts.relay.name = name.clone();
    ctx.accounts.relay.treasury.vault = ctx.accounts.relay_vault.key();
    ctx.accounts.relay.treasury.vault_bump = ctx.bumps.relay_vault;
    ctx.accounts.relay.history = merkle_tree;
    ctx.accounts.relay.recent_roots = recent_roots;

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(
            &vm,
            &ctx,
            num_levels,
            num_history,
            name
        ),
        None
    ); 

    Ok(())
}

fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<RelayInit>,
    num_levels: u8, 
    num_history: u8, 
    name: String,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::RelayInit {
        num_levels,
        num_history,
        name,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::RelayInit::DISCRIMINATOR.to_vec(),
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