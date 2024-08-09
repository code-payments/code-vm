use anchor_lang::{prelude::*, Discriminator};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::{TokenAccount, Token, Transfer, transfer, ID as TOKEN_PROGRAM_ID};

use crate::advance_poh;
use crate::error::CodeVmError;
use crate::{
    utils,
    program,
    instruction,
    cvm::{ 
        CodeVm,
        CodeVmAccount, 
        TimelockState,
        CompressedStorageAccount,
        UnlockStateAccount,
        WithdrawReceiptAccount,
        VirtualAccount,
        VirtualTimelockAccount,
        ChangeLogData,
    },
    types::{ Hash, Signature },
    CODE_VM_PREFIX
};

#[derive(Accounts)]
#[instruction(
    unchecked_vta: VirtualTimelockAccount,
    proof: Vec<Hash>,
    signature: Signature,
)]
pub struct TimelockWithdrawFromStorage<'info> {
    pub virtual_account_owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut, // the POH value is updated
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
        mut,
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_storage_account",
            &vm_storage.name.as_bytes(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = vm_storage.bump, 
    )]
    pub vm_storage: Account<'info, CompressedStorageAccount>,

    #[account(
        mut,
        token::mint = vm.mint,
        token::authority = vm_omnibus,

        seeds=[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_omnibus",
            vm.mint.as_ref(),
            vm.authority.as_ref(),
            vm.lock_duration.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub vm_omnibus: Account<'info, TokenAccount>,

    /// This account contains the timelock state; for example: Locked, Unlocked, etc.
    /// It is a real account that may or may not exist but is checked any time
    /// the associated virtual account is mutated.
    #[account(
        constraint = unlock_pda.owner == virtual_account_owner.key(),
        constraint = unlock_pda.state == TimelockState::Unlocked,

        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_unlock_pda_account",
            unlock_pda.owner.as_ref(),
            unlock_pda.address.as_ref(),
            vm.to_account_info().key.as_ref(),
        ], 
        bump = unlock_pda.bump, 
    )]
    pub unlock_pda: Account<'info, UnlockStateAccount>,

    /// This account is used to store the withdraw receipt. It is used to prove
    /// that a user has non-custodially withdrawn tokens from a virtual account,
    /// removing for concurrent merkle tree updates on the storage side.
    #[account(
        init, 
        seeds = [
            CODE_VM_PREFIX.as_bytes(),
            b"vm_withdraw_receipt_account",
            unlock_pda.to_account_info().key.as_ref(),

            // The VM can have multiple uncompressed or compressed records for
            // the same address at any given time. However, each one has a
            // unique nonce value. It should be set to the nonce value of the
            // record that the user wants to unlock.

            unchecked_vta.nonce.as_ref(), 
            vm.to_account_info().key.as_ref(),
        ], 
        payer = payer, 
        space = WithdrawReceiptAccount::LEN,
        bump,
    )]
    pub withdraw_receipt: Account<'info, WithdrawReceiptAccount>,

    /// The external address is used for external transfers. This is the
    /// account that tokens are transferred to.
    #[account(mut,
        token::mint = vm.mint,
    )]
    pub external_address: Account<'info, TokenAccount>,

    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn timelock_withdraw_from_storage(
    ctx: Context<TimelockWithdrawFromStorage>,
    unchecked_vta: VirtualTimelockAccount,
    proof: Vec<Hash>,
    signature: Signature,
) -> Result<()> {
    let mut vm = CodeVm::new(ctx.accounts.vm.to_owned());

    let unlock_pda = ctx.accounts.unlock_pda.as_ref();
    let receipt = ctx.accounts.withdraw_receipt.as_ref();

    require!(vm.try_verify_timelock_account(unchecked_vta, unlock_pda, receipt).is_ok(), CodeVmError::InvalidTimelockState);

    assert_eq!(unchecked_vta.owner, ctx.accounts.virtual_account_owner.key());
    assert_eq!(unchecked_vta.owner, ctx.accounts.unlock_pda.owner);

    let vta = unchecked_vta;
    let unchecked_va = VirtualAccount::Timelock(vta);
    let tree = &mut ctx.accounts.vm_storage.memory_state;

    vm.try_decompress(unchecked_va, tree, proof.clone(), signature)?;

    let mint = vm.get_mint();
    let authority = vm.get_authority();
    let lock_duration = vm.get_lock_duration().to_le_bytes();
    let vm_omnibus = ctx.accounts.vm_omnibus.to_account_info();
    let destination = ctx.accounts.external_address.to_account_info();
    let token_program = ctx.accounts.token_program.to_account_info();

    let seeds = [
        CODE_VM_PREFIX.as_bytes(),
        b"vm_omnibus",
        mint.as_ref(),
        authority.as_ref(),
        lock_duration.as_ref(),
        &[vm.get_omnibus_bump()],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        authority: vm_omnibus.clone(),
        from: vm_omnibus.clone(),
        to: destination.clone(),
    };
    let cpi_program = token_program;
    let cpi_ctx = CpiContext::new_with_signer(
        cpi_program, 
        cpi_accounts, 
        signer_seeds
    );

    transfer(cpi_ctx, vta.balance)?;

    ctx.accounts.withdraw_receipt.unlock_pda = ctx.accounts.unlock_pda.key();
    ctx.accounts.withdraw_receipt.bump = ctx.bumps.withdraw_receipt;
    ctx.accounts.withdraw_receipt.amount = vta.balance;
    ctx.accounts.withdraw_receipt.nonce = vta.nonce;

    let timelock_address = vta.get_timelock_address(
        vm.get_mint(), 
        vm.get_authority(), 
        vm.get_lock_duration(),
    );

    let token_address = vta.get_token_address(
        timelock_address,
    );

    // Advance the vm state to include this instruction
    advance_poh!(ctx, vm, 
        get_message_hash(
            &vm,
            &ctx,
            unchecked_vta,
            proof,
            signature,
        ),
        Some(ChangeLogData::Withdraw {
            account: Some(vta),
            signature: Some(signature),
            src: token_address,
            dst: ctx.accounts.external_address.key(),
            amount: vta.balance,
        })
    );

    Ok(())
}


fn get_message_hash(
    vm: &CodeVm,
    ctx: &Context<TimelockWithdrawFromStorage>,
    unchecked_vta: VirtualTimelockAccount,
    proof: Vec<Hash>,
    signature: Signature,
) -> Hash {

    let blockhash = vm.get_current_poh();
    let args = instruction::TimelockWithdrawFromStorage {
        unchecked_vta,
        proof,
        signature,
    };

    let accounts = ctx.accounts.to_account_metas(None);
    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts,
            data: [
                instruction::TimelockWithdrawFromStorage::DISCRIMINATOR.to_vec(),
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