use solana_program::{
    program_pack::Pack,
    system_instruction, 
    rent::Rent, 
};
use steel::*;

use crate::helpers::check_condition;

pub fn create_token_account<'info>(
    mint: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    seeds: &[&[u8]],
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent_sysvar: &AccountInfo<'info>,
) -> ProgramResult {
    // Create the PDA.
    allocate_account(
        target,
        &spl_token::id(),
        spl_token::state::Account::LEN,
        seeds,
        system_program,
        payer,
    )?;

    // Initialize the PDA.
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            target.key,
            mint.key,
            target.key,
        ).unwrap(),
        &[
            target.clone(),
            mint.clone(),
            target.clone(),
            rent_sysvar.clone(),
        ],
        &[seeds],
    )
}

pub fn create_account_with_size<'a, 'info, T: Discriminator + Pod>(
    target_account: &'a AccountInfo<'info>,
    size: usize,
    owner: &Pubkey,
    seeds: &[&[u8]],
    system_program: &'a AccountInfo<'info>,
    payer: &'a AccountInfo<'info>,
) -> ProgramResult {
    check_condition(
        size >= 8 + std::mem::size_of::<T>(),
        "provided size is too small",
    )?;

    // Allocate space.
    allocate_account(
        target_account,
        owner,
        size,
        seeds,
        system_program,
        payer,
    )?;

    // Set discriminator.
    let mut data = target_account.data.borrow_mut();
    data[0] = T::discriminator();

    Ok(())
}

pub fn resize_account<'info>(
    target_account: &AccountInfo<'info>,
    payer: &AccountInfo<'info>,
    new_size: usize,
    system_program: &AccountInfo<'info>,
) -> ProgramResult {
    let rent = Rent::get()?;
    let rent_exempt_balance = rent
        .minimum_balance(new_size)
        .saturating_sub(target_account.lamports());

    if rent_exempt_balance.gt(&0) {
        solana_program::program::invoke(
            &system_instruction::transfer(
                payer.key, 
                target_account.key,
                rent_exempt_balance,
            ),
            &[
                payer.clone(),
                target_account.clone(),
                system_program.clone(),
            ],
        )?;
    }

    target_account.realloc(new_size, false)?;

    Ok(())
}