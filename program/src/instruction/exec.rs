use code_vm_api::prelude::*;
use steel::*;

use crate::opcode::*;

/*
    This instruction is used to execute VM opcodes using virtual account memory
    banks. Opcodes are a handful of purpose built instructions that allow us to
    read, write, and transfer tokens between virtual accounts and external
    accounts.

    Accounts expected by this instruction:

    | # | R/W | Type         | Req | PDA | Name             | Description                                  |
    |---|-----|------------- |-----|-----|------------------|----------------------------------------------|
    | 0 | mut | Signer       | Yes |     | vm_authority     | The authority of the VM.                     |
    | 1 | mut | Vm           | Yes | PDA | vm               | The VM instance state account.               |
    | 2 | mut | Memory       |     | PDA | mem_a            | Memory bank A                                |
    | 3 | mut | Memory       |     | PDA | mem_b            | Memory bank B                                |
    | 4 | mut | Memory       |     | PDA | mem_c            | Memory bank C                                |
    | 5 | mut | Memory       |     | PDA | mem_d            | Memory bank D                                |
    | 6 | mut | TokenAccount |     | PDA | vm_omnibus       | A derived token account owned by the VM.     |
    | 7 | mut | Relay        |     | PDA | relay            | A relay account to use for private transfers.|
    | 8 | mut | TokenAccount |     | PDA | relay_vault      | A derived token account owned by the relay.  |
    | 9 | mut | TokenAccount |     |     | external_address | Required when making external transfers.     |
    | 10|     | Program      |     |     | token_program    | Required when making token transfers.        |


    Derived account seeds:

    1. vm:            [ "code_vm", <mint>, <vm_authority>, <lock_duration> ]
    2. mem_a:         [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    3. mem_b:         [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    4. mem_c:         [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    5. mem_d:         [ "code_vm", "vm_memory_account", <self.name>, <vm> ]
    6. omnibus:       [ "code_vm", "vm_omnibus", <vm> ]
    7. relay:         [ "code_vm", "vm_relay_account", <self.name>, <vm> ]
    8. relay_vault:   [ "code_vm", "vm_relay_vault", <relay> ]

    Instruction data:

    0. opcode: u8          - The opcode to execute.
    1. mem_indicies: [u16] - The account_indicies of the virtual accounts to use.
    2. mem_banks: [u8]     - The memory bank to use for each account.
    3. data: [u8]          - The opaque data to pass into the VM opcode instruction.
*/
pub fn process_exec(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    let args = ExecIx::try_from_slice(data)?;
    let ctx = ExecContext::try_from(accounts)?;

    check_signer(ctx.vm_authority_info)?;
    check_mut(ctx.vm_info)?;

    let vm = load_vm_checked(ctx.vm_info, ctx.vm_authority_info)?;

    ctx.check_memory_banks()?;

    let ix = Opcode::try_from(args.opcode).unwrap();

    match ix {

        Opcode::TransferOp             => process_transfer(&ctx, &args, vm),
        Opcode::WithdrawOp             => process_withdraw(&ctx, &args, vm),
        Opcode::RelayOp                => process_relay(&ctx, &args, vm),

        Opcode::ExternalTransferOp     => process_external_transfer(&ctx, &args, vm),
        Opcode::ExternalWithdrawOp     => process_external_withdraw(&ctx, &args, vm),
        Opcode::ExternalRelayOp        => process_external_relay(&ctx, &args),

        Opcode::ConditionalTransferOp  => process_conditional_transfer(&ctx, &args, vm),

        Opcode::AirdropOp              => process_airdrop(&ctx, &args, vm),

        _ => Err(ProgramError::InvalidInstructionData),
    }?;

    vm.advance_poh(CodeInstruction::ExecIx, accounts, data);

    Ok(())
}

pub struct ExecContext<'a, 'b> {
    pub vm_authority_info: &'a AccountInfo<'b>,
    pub vm_info: &'a AccountInfo<'b>,
    pub mem_a_info: Option<&'a AccountInfo<'b>>,
    pub mem_b_info: Option<&'a AccountInfo<'b>>,
    pub mem_c_info: Option<&'a AccountInfo<'b>>,
    pub mem_d_info: Option<&'a AccountInfo<'b>>,
    pub omnibus_info: Option<&'a AccountInfo<'b>>,
    pub relay_info: Option<&'a AccountInfo<'b>>,
    pub relay_vault_info: Option<&'a AccountInfo<'b>>,
    pub external_address_info: Option<&'a AccountInfo<'b>>,
    pub token_program_info: Option<&'a AccountInfo<'b>>,
}

impl<'a, 'b> ExecContext<'a, 'b> {
    pub fn try_from(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let (
            vm_authority_info,
            vm_info,
            mem_a_info,
            mem_b_info,
            mem_c_info,
            mem_d_info,
            omnibus_info,
            relay_info,
            relay_vault_info,
            external_address_info,
            token_program_info,
        ) = match accounts {
            [ a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10 ] => (
                a0,
                a1,
                get_optional(a2),
                get_optional(a3),
                get_optional(a4),
                get_optional(a5),
                get_optional(a6),
                get_optional(a7),
                get_optional(a8),
                get_optional(a9),
                get_optional(a10),
            ),
            _ => return Err(ProgramError::NotEnoughAccountKeys),
        };

        Ok(Self {
            vm_authority_info,
            vm_info,
            mem_a_info,
            mem_b_info,
            mem_c_info,
            mem_d_info,
            omnibus_info,
            relay_info,
            relay_vault_info,
            external_address_info,
            token_program_info,
        })
    }

    pub fn check_memory_banks(&self) -> Result<(), ProgramError> {
        // Stack array instead of a heap Vec; unused slots hold vm_info as a
        // placeholder and are never read (the slice is capped at n).
        let mut provided: [&AccountInfo; 4] = [self.vm_info; 4];
        let mut n = 0;

        for bank in self.get_banks() {
            if let Some(mem_info) = bank {
                check_mut(mem_info)?;
                check_memory(mem_info, self.vm_info)?;
                provided[n] = mem_info;
                n += 1;
            }
        }

        check_unique(
            &provided[..n],
            "provided memory banks must be unique",
        )?;

        Ok(())
    }

    pub fn get_banks(&self) -> [Option<&AccountInfo<'b>>; 4] {
        [
            self.mem_a_info,
            self.mem_b_info,
            self.mem_c_info,
            self.mem_d_info,
        ]
    }
}
