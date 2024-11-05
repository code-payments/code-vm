use code_vm_api::prelude::*;
use steel::*;

/*
    This instruction is used to non-custodially withdraw tokens from unlocked
    virtual accounts or deposit addresses. The tokens can be withdrawn from the
    VM's memory, storage, or from a deposit account.

    The requirement for this instruction is that the owner's timelock account is
    in the unlocked state.

    Accounts expected by this instruction:

    | # | R/W | Req | PDA | Type         | Name             | Description                            |
    |---|-----|-----|-----|------------  |------------------|----------------------------------------|
    | 0 | mut | Yes |     | Signer       | depositor        | The owner of the deposited tokens.     |
    | 1 | mut | Yes |     | Signer       | payer            | The transaction fee payer.             |
    | 2 | mut | Yes | Yes | Vm           | vm               | The VM instance state account.         |
    | 3 | mut |     | Yes | TokenAccount | vm_omnibus       | The token account for this VM.         |
    | 4 | mut |     | Yes | Memory       | vm_memory        | If withdrawing from memory (hot).      |
    | 5 |     |     | Yes | Storage      | vm_storage       | If withdrawing from storage (cold).    |
    | 6 |     |     | Yes | Address      | deposit_pda      | If withdrawing from deposit (ata).     |
    | 7 | mut |     | Yes | ATA          | deposit_ata      | If withdrawing from deposit.           |
    | 8 |     | Yes | Yes | UnlockState  | unlock_pda       | Timelock unlock state account.         |
    | 9 |     |     | Yes | Receipt      | withdraw_receipt | If withdrawing from memory or storage. |
    |10 | mut | Yes |     | Address      | external_address | External address to send tokens to.    |
    |11 |     | Yes |     | Token        | token_program    | SPL token program account.             |
    |12 |     |     |     | System       | system_program   | System program account.                |
    |13 |     |     |     | Rent         | rent_sysvar      | Rent sysvar account (for receipt).     |

*/
pub fn process_withdraw(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = WithdrawIx::try_from_slice(data)?;
    let ctx = WithdrawContext::try_from(accounts)?;

    check_signer(ctx.depositor_info)?;
    check_signer(ctx.payer_info)?;
    check_mut(ctx.vm_info)?;
    check_mut(ctx.external_address_info)?;
    check_program(ctx.token_program_info, &spl_token::id())?;

    if let Some(vm_omnibus) = ctx.vm_omnibus {
        check_mut(vm_omnibus)?;
    }

    if let Some(vm_memory_info) = ctx.vm_memory_info {
        check_mut(vm_memory_info)?;
        check_memory(vm_memory_info, ctx.vm_info)?;
    }

    if let Some(vm_storage_info) = ctx.vm_storage_info {
        check_mut(vm_storage_info)?;
        check_storage(vm_storage_info, ctx.vm_info)?;
    }

    if let Some(system_program) = ctx.system_program_info {
        check_program(system_program, &system_program::id())?;
    }

    if let Some(rent_sysvar) = ctx.rent_sysvar_info {
        check_sysvar(rent_sysvar, &solana_program::sysvar::rent::id())?;
    }

    if let Some(withdraw_receipt_info) = ctx.withdraw_receipt_info {
        check_mut(withdraw_receipt_info)?;
        withdraw_receipt_info.is_empty()?;
    }

    ctx.check_unlock_state()?;

    match args {

        WithdrawIxData::FromDeposit { .. } => 
            process_withdraw_from_deposit(&ctx, &args),

        WithdrawIxData::FromMemory { .. } => 
            process_withdraw_from_memory(&ctx, &args),

        WithdrawIxData::FromStorage { .. } => 
            process_withdraw_from_storage(&ctx, &args),

    }?;

    let vm = load_vm(ctx.vm_info)?;

    vm.advance_poh(CodeInstruction::WithdrawIx, accounts, data);

    Ok(())
}

fn process_withdraw_from_memory(
    ctx: &WithdrawContext,
    data: &WithdrawIxData,
) -> ProgramResult {
    let account_index = match data {
        WithdrawIxData::FromMemory { account_index } => Ok(*account_index),
        _ => Err(ProgramError::InvalidInstructionData),
    }?;

    check_condition(
        ctx.vm_memory_info.is_some(),
        "vm_memory account is required for memory withdraw",
    )?;

    check_condition(
        ctx.vm_omnibus.is_some(),
        "vm_omnibus account is required for memory withdraw",
    )?;

    let vm_info = ctx.vm_info;
    let vm = load_vm(vm_info)?;

    let vm_omnibus = ctx.vm_omnibus.unwrap();
    let vm_memory_info = ctx.vm_memory_info.unwrap();
    let va = try_read(vm_memory_info, account_index)?;
    let vta = va.into_inner_timelock().unwrap();

    check_condition(
        vta.owner.eq(ctx.depositor_info.key),
        "depositor does not match the owner of the timelock account",
    )?;

    transfer_signed(
        vm_omnibus,
        vm_omnibus,
        ctx.external_address_info,
        ctx.token_program_info,
        vta.balance,
        &[&[
            CODE_VM,
            VM_OMNIBUS,
            vm_info.key.as_ref(),
            &[vm.get_omnibus_bump()],
        ]],
    )?;

    try_delete(vm_memory_info, account_index)?;

    ctx.create_receipt(&vta.instance)?;

    Ok(())
}

fn process_withdraw_from_storage(
    ctx: &WithdrawContext,
    data: &WithdrawIxData,
) -> ProgramResult {
    let (packed_va, proof, signature) = match data {
        WithdrawIxData::FromStorage {
            packed_va,
            proof,
            signature,
        } => Ok((packed_va, proof, signature)),
        _ => Err(ProgramError::InvalidInstructionData),
    }?;

    let vm_info = ctx.vm_info;
    let vm = load_vm(vm_info)?;

    let va = VirtualAccount::unpack(packed_va)?;
    let vta = va.into_inner_timelock().unwrap();

    let va_hash = va.get_hash();
    let sig_hash = hashv(&[signature.as_ref(), va_hash.as_ref()]);

    sig_verify(vm.authority.as_ref(), signature.as_ref(), va_hash.as_ref())?;

    check_condition(
        ctx.vm_omnibus.is_some(),
        "vm_omnibus account is required for storage withdraw",
    )?;

    check_condition(
        ctx.vm_storage_info.is_some(),
        "vm_storage account is required for storage withdraw",
    )?;

    let vm_omnibus = ctx.vm_omnibus.unwrap();
    let vm_storage_info = ctx.vm_storage_info.unwrap();

    try_decompress(vm_storage_info, sig_hash, proof)?;

    transfer_signed(
        vm_omnibus,
        vm_omnibus,
        ctx.external_address_info,
        ctx.token_program_info,
        vta.balance,
        &[&[
            CODE_VM,
            VM_OMNIBUS,
            vm_info.key.as_ref(),
            &[vm.get_omnibus_bump()],
        ]],
    )?;

    ctx.create_receipt(&vta.instance)?;

    Ok(())
}

fn process_withdraw_from_deposit(
    ctx: &WithdrawContext,
    data: &WithdrawIxData,
) -> ProgramResult {
    let bump = match data {
        WithdrawIxData::FromDeposit { bump } => Ok(*bump),
        _ => Err(ProgramError::InvalidInstructionData),
    }?;

    check_condition(
        ctx.deposit_pda_info.is_some(),
        "deposit_pda account is required for deposit withdraw",
    )?;

    check_condition(
        ctx.deposit_ata_info.is_some(),
        "deposit_ata account is required for deposit withdraw",
    )?;

    let deposit_ata_info = ctx.deposit_ata_info.unwrap();
    let deposit_pda_info = ctx.deposit_pda_info.unwrap();
    let token_account = deposit_ata_info.to_token_account()?;

    transfer_signed(
        deposit_pda_info,
        deposit_ata_info,
        ctx.external_address_info,
        ctx.token_program_info,
        token_account.amount,
        &[&[
            CODE_VM,
            VM_DEPOSIT_PDA,
            &ctx.depositor_info.key.to_bytes(),
            &ctx.vm_info.key.to_bytes(),
            &[bump],
        ]],
    )?;

    // No receipt is created for this type of withdraw

    Ok(())
}

pub struct WithdrawContext<'a, 'b> {
    pub depositor_info: &'a AccountInfo<'b>,
    pub payer_info: &'a AccountInfo<'b>,
    pub vm_info: &'a AccountInfo<'b>,
    pub vm_omnibus: Option<&'a AccountInfo<'b>>,
    pub vm_memory_info: Option<&'a AccountInfo<'b>>,
    pub vm_storage_info: Option<&'a AccountInfo<'b>>,
    pub deposit_pda_info: Option<&'a AccountInfo<'b>>,
    pub deposit_ata_info: Option<&'a AccountInfo<'b>>,
    pub unlock_pda_info: &'a AccountInfo<'b>,
    pub withdraw_receipt_info: Option<&'a AccountInfo<'b>>,
    pub external_address_info: &'a AccountInfo<'b>,
    pub token_program_info: &'a AccountInfo<'b>,
    pub system_program_info: Option<&'a AccountInfo<'b>>,
    pub rent_sysvar_info: Option<&'a AccountInfo<'b>>,
}

impl<'a, 'b> WithdrawContext<'a, 'b> {
    pub fn try_from(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let (
            depositor_info,
            payer_info,
            vm_info,
            vm_omnibus,
            vm_memory_info,
            vm_storage_info,
            deposit_pda_info,
            deposit_ata_info,
            unlock_pda_info,
            withdraw_receipt_info,
            external_address_info,
            token_program_info,
            system_program_info,
            rent_sysvar_info,
        ) = match accounts {
            [ a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13 ] => (
                a0, a1, a2, 
                get_optional(a3),
                get_optional(a4),
                get_optional(a5),
                get_optional(a6),
                get_optional(a7),
                a8,
                get_optional(a9),
                a10,
                a11,
                get_optional(a12),
                get_optional(a13),
            ),
            _ => return Err(ProgramError::NotEnoughAccountKeys),
        };

        Ok(Self {
            depositor_info,
            payer_info,
            vm_info,
            vm_omnibus,
            vm_memory_info,
            vm_storage_info,
            deposit_pda_info,
            deposit_ata_info,
            unlock_pda_info,
            withdraw_receipt_info,
            external_address_info,
            token_program_info,
            system_program_info,
            rent_sysvar_info,
        })
    }

    pub fn check_unlock_state(&self) -> Result<(Pubkey, u8), ProgramError> {
        // Here we're going to derive the unlock address from the owner pubkey
        // and check that it has the correct unlocked state.

        let owner = self.depositor_info.key;
        let vm = load_vm(self.vm_info)?;

        let (timelock_address, _) = find_virtual_timelock_address(
            &vm.get_mint(),
            &vm.get_authority(),
            &owner,
            vm.get_lock_duration(),
        );

        let (unlock_address, bump) =
            find_unlock_address(&owner, &timelock_address, self.vm_info.key);

        check_condition(
            self.unlock_pda_info.key.eq(&unlock_address),
            "unlock_pda does not match the expected unlock address",
        )?;

        let unlock_state = self
            .unlock_pda_info
            .to_account::<UnlockStateAccount>(&code_vm_api::ID)?;

        check_condition(
            unlock_state.is_unlocked(),
            "unlock_pda is not in the unlocked state",
        )?;

        check_condition(
            unlock_state.owner.eq(&owner),
            "unlock_pda owner does not match the expected owner",
        )?;

        check_condition(
            unlock_state.vm.eq(&self.vm_info.key),
            "unlock_pda vm does not match the expected vm",
        )?;

        Ok((unlock_address, bump))
    }

    pub fn create_receipt(&self, nonce: &Hash) -> ProgramResult {
        // The assumption is that we have already checked that the unlock state
        // is both valid and that the address is correct.

        let vm_info = self.vm_info;
        let unlock_pda_info = self.unlock_pda_info;
        let withdraw_receipt_info = self.withdraw_receipt_info.unwrap();

        let (receipt_address, bump) =
            find_withdraw_receipt_address(&unlock_pda_info.key, nonce, vm_info.key);

        withdraw_receipt_info.is_empty()?.is_writable()?;

        check_condition(
            withdraw_receipt_info.key.eq(&receipt_address),
            "withdraw_receipt does not match the expected receipt address",
        )?;

        create_account::<WithdrawReceiptAccount>(
            withdraw_receipt_info,
            &code_vm_api::ID,
            &[
                CODE_VM,
                VM_WITHDRAW_RECEIPT,
                unlock_pda_info.key.as_ref(),
                nonce.as_ref(),
                vm_info.key.as_ref(),
                &[bump],
            ],
            self.system_program_info.unwrap(),
            self.payer_info,
        )?;

        Ok(())
    }
}
