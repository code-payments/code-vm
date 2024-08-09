use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    pubkey::Pubkey, program_error::ProgramError,
};
use std::io::Read;
#[derive(Clone, Debug, PartialEq)]
pub enum TimelockProgramIx {
    Initialize(InitializeIxArgs),
    Activate(ActivateIxArgs),
    Deactivate(DeactivateIxArgs),
    RevokeLockWithTimeout(RevokeLockWithTimeoutIxArgs),
    RevokeLockWithAuthority(RevokeLockWithAuthorityIxArgs),
    CancelLockTimeout(CancelLockTimeoutIxArgs),
    TransferWithAuthority(TransferWithAuthorityIxArgs),
    BurnDustWithAuthority(BurnDustWithAuthorityIxArgs),
    Withdraw(WithdrawIxArgs),
    CloseAccounts(CloseAccountsIxArgs),
}
impl TimelockProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            INITIALIZE_IX_DISCM => {
                Ok(Self::Initialize(InitializeIxArgs::deserialize(&mut reader)?))
            }
            ACTIVATE_IX_DISCM => {
                Ok(Self::Activate(ActivateIxArgs::deserialize(&mut reader)?))
            }
            DEACTIVATE_IX_DISCM => {
                Ok(Self::Deactivate(DeactivateIxArgs::deserialize(&mut reader)?))
            }
            REVOKE_LOCK_WITH_TIMEOUT_IX_DISCM => {
                Ok(
                    Self::RevokeLockWithTimeout(
                        RevokeLockWithTimeoutIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            REVOKE_LOCK_WITH_AUTHORITY_IX_DISCM => {
                Ok(
                    Self::RevokeLockWithAuthority(
                        RevokeLockWithAuthorityIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            CANCEL_LOCK_TIMEOUT_IX_DISCM => {
                Ok(
                    Self::CancelLockTimeout(
                        CancelLockTimeoutIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            TRANSFER_WITH_AUTHORITY_IX_DISCM => {
                Ok(
                    Self::TransferWithAuthority(
                        TransferWithAuthorityIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            BURN_DUST_WITH_AUTHORITY_IX_DISCM => {
                Ok(
                    Self::BurnDustWithAuthority(
                        BurnDustWithAuthorityIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            WITHDRAW_IX_DISCM => {
                Ok(Self::Withdraw(WithdrawIxArgs::deserialize(&mut reader)?))
            }
            CLOSE_ACCOUNTS_IX_DISCM => {
                Ok(Self::CloseAccounts(CloseAccountsIxArgs::deserialize(&mut reader)?))
            }
            _ => {
                Err(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("discm {:?} not found", maybe_discm),
                    ),
                )
            }
        }
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::Initialize(args) => {
                writer.write_all(&INITIALIZE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Activate(args) => {
                writer.write_all(&ACTIVATE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Deactivate(args) => {
                writer.write_all(&DEACTIVATE_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::RevokeLockWithTimeout(args) => {
                writer.write_all(&REVOKE_LOCK_WITH_TIMEOUT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::RevokeLockWithAuthority(args) => {
                writer.write_all(&REVOKE_LOCK_WITH_AUTHORITY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CancelLockTimeout(args) => {
                writer.write_all(&CANCEL_LOCK_TIMEOUT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::TransferWithAuthority(args) => {
                writer.write_all(&TRANSFER_WITH_AUTHORITY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::BurnDustWithAuthority(args) => {
                writer.write_all(&BURN_DUST_WITH_AUTHORITY_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::Withdraw(args) => {
                writer.write_all(&WITHDRAW_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CloseAccounts(args) => {
                writer.write_all(&CLOSE_ACCOUNTS_IX_DISCM)?;
                args.serialize(&mut writer)
            }
        }
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
fn invoke_instruction<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
    ix: &Instruction,
    accounts: A,
) -> ProgramResult {
    let account_info: [AccountInfo<'info>; N] = accounts.into();
    invoke(ix, &account_info)
}
fn invoke_instruction_signed<'info, A: Into<[AccountInfo<'info>; N]>, const N: usize>(
    ix: &Instruction,
    accounts: A,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let account_info: [AccountInfo<'info>; N] = accounts.into();
    invoke_signed(ix, &account_info, seeds)
}
pub const INITIALIZE_IX_ACCOUNTS_LEN: usize = 9;
#[derive(Copy, Clone, Debug)]
pub struct InitializeAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub mint: &'me AccountInfo<'info>,
    pub time_authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitializeKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub mint: Pubkey,
    pub time_authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<InitializeAccounts<'_, '_>> for InitializeKeys {
    fn from(accounts: InitializeAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            vault_owner: *accounts.vault_owner.key,
            mint: *accounts.mint.key,
            time_authority: *accounts.time_authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<InitializeKeys> for [AccountMeta; INITIALIZE_IX_ACCOUNTS_LEN] {
    fn from(keys: InitializeKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.time_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INITIALIZE_IX_ACCOUNTS_LEN]> for InitializeKeys {
    fn from(pubkeys: [Pubkey; INITIALIZE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            vault_owner: pubkeys[2],
            mint: pubkeys[3],
            time_authority: pubkeys[4],
            payer: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
            rent: pubkeys[8],
        }
    }
}
impl<'info> From<InitializeAccounts<'_, 'info>>
for [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN] {
    fn from(accounts: InitializeAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.vault_owner.clone(),
            accounts.mint.clone(),
            accounts.time_authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN]>
for InitializeAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; INITIALIZE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            vault_owner: &arr[2],
            mint: &arr[3],
            time_authority: &arr[4],
            payer: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
            rent: &arr[8],
        }
    }
}
pub const INITIALIZE_IX_DISCM: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeIxArgs {
    pub num_days_locked: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InitializeIxData(pub InitializeIxArgs);
impl From<InitializeIxArgs> for InitializeIxData {
    fn from(args: InitializeIxArgs) -> Self {
        Self(args)
    }
}
impl InitializeIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != INITIALIZE_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        INITIALIZE_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(InitializeIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&INITIALIZE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn initialize_ix_with_program_id(
    program_id: Pubkey,
    keys: InitializeKeys,
    args: InitializeIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; INITIALIZE_IX_ACCOUNTS_LEN] = keys.into();
    let data: InitializeIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn initialize_ix(
    keys: InitializeKeys,
    args: InitializeIxArgs,
) -> std::io::Result<Instruction> {
    initialize_ix_with_program_id(crate::ID, keys, args)
}
pub fn initialize_invoke_with_program_id(
    program_id: Pubkey,
    accounts: InitializeAccounts<'_, '_>,
    args: InitializeIxArgs,
) -> ProgramResult {
    let keys: InitializeKeys = accounts.into();
    let ix = initialize_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn initialize_invoke(
    accounts: InitializeAccounts<'_, '_>,
    args: InitializeIxArgs,
) -> ProgramResult {
    initialize_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn initialize_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: InitializeAccounts<'_, '_>,
    args: InitializeIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: InitializeKeys = accounts.into();
    let ix = initialize_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn initialize_invoke_signed(
    accounts: InitializeAccounts<'_, '_>,
    args: InitializeIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    initialize_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn initialize_verify_account_keys(
    accounts: InitializeAccounts<'_, '_>,
    keys: InitializeKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.mint.key, keys.mint),
        (*accounts.time_authority.key, keys.time_authority),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.rent.key, keys.rent),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn initialize_verify_writable_privileges<'me, 'info>(
    accounts: InitializeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.vault, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn initialize_verify_signer_privileges<'me, 'info>(
    accounts: InitializeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.time_authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn initialize_verify_account_privileges<'me, 'info>(
    accounts: InitializeAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    initialize_verify_writable_privileges(accounts)?;
    initialize_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const ACTIVATE_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct ActivateAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ActivateKeys {
    pub timelock: Pubkey,
    pub vault_owner: Pubkey,
    pub payer: Pubkey,
}
impl From<ActivateAccounts<'_, '_>> for ActivateKeys {
    fn from(accounts: ActivateAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault_owner: *accounts.vault_owner.key,
            payer: *accounts.payer.key,
        }
    }
}
impl From<ActivateKeys> for [AccountMeta; ACTIVATE_IX_ACCOUNTS_LEN] {
    fn from(keys: ActivateKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
        ]
    }
}
impl From<[Pubkey; ACTIVATE_IX_ACCOUNTS_LEN]> for ActivateKeys {
    fn from(pubkeys: [Pubkey; ACTIVATE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault_owner: pubkeys[1],
            payer: pubkeys[2],
        }
    }
}
impl<'info> From<ActivateAccounts<'_, 'info>>
for [AccountInfo<'info>; ACTIVATE_IX_ACCOUNTS_LEN] {
    fn from(accounts: ActivateAccounts<'_, 'info>) -> Self {
        [accounts.timelock.clone(), accounts.vault_owner.clone(), accounts.payer.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; ACTIVATE_IX_ACCOUNTS_LEN]>
for ActivateAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; ACTIVATE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: &arr[0],
            vault_owner: &arr[1],
            payer: &arr[2],
        }
    }
}
pub const ACTIVATE_IX_DISCM: [u8; 8] = [194, 203, 35, 100, 151, 55, 170, 82];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivateIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActivateIxData(pub ActivateIxArgs);
impl From<ActivateIxArgs> for ActivateIxData {
    fn from(args: ActivateIxArgs) -> Self {
        Self(args)
    }
}
impl ActivateIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != ACTIVATE_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        ACTIVATE_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(ActivateIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&ACTIVATE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn activate_ix_with_program_id(
    program_id: Pubkey,
    keys: ActivateKeys,
    args: ActivateIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; ACTIVATE_IX_ACCOUNTS_LEN] = keys.into();
    let data: ActivateIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn activate_ix(
    keys: ActivateKeys,
    args: ActivateIxArgs,
) -> std::io::Result<Instruction> {
    activate_ix_with_program_id(crate::ID, keys, args)
}
pub fn activate_invoke_with_program_id(
    program_id: Pubkey,
    accounts: ActivateAccounts<'_, '_>,
    args: ActivateIxArgs,
) -> ProgramResult {
    let keys: ActivateKeys = accounts.into();
    let ix = activate_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn activate_invoke(
    accounts: ActivateAccounts<'_, '_>,
    args: ActivateIxArgs,
) -> ProgramResult {
    activate_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn activate_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: ActivateAccounts<'_, '_>,
    args: ActivateIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: ActivateKeys = accounts.into();
    let ix = activate_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn activate_invoke_signed(
    accounts: ActivateAccounts<'_, '_>,
    args: ActivateIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    activate_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn activate_verify_account_keys(
    accounts: ActivateAccounts<'_, '_>,
    keys: ActivateKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.payer.key, keys.payer),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn activate_verify_writable_privileges<'me, 'info>(
    accounts: ActivateAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn activate_verify_signer_privileges<'me, 'info>(
    accounts: ActivateAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.vault_owner, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn activate_verify_account_privileges<'me, 'info>(
    accounts: ActivateAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    activate_verify_writable_privileges(accounts)?;
    activate_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const DEACTIVATE_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct DeactivateAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DeactivateKeys {
    pub timelock: Pubkey,
    pub vault_owner: Pubkey,
    pub payer: Pubkey,
}
impl From<DeactivateAccounts<'_, '_>> for DeactivateKeys {
    fn from(accounts: DeactivateAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault_owner: *accounts.vault_owner.key,
            payer: *accounts.payer.key,
        }
    }
}
impl From<DeactivateKeys> for [AccountMeta; DEACTIVATE_IX_ACCOUNTS_LEN] {
    fn from(keys: DeactivateKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
        ]
    }
}
impl From<[Pubkey; DEACTIVATE_IX_ACCOUNTS_LEN]> for DeactivateKeys {
    fn from(pubkeys: [Pubkey; DEACTIVATE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault_owner: pubkeys[1],
            payer: pubkeys[2],
        }
    }
}
impl<'info> From<DeactivateAccounts<'_, 'info>>
for [AccountInfo<'info>; DEACTIVATE_IX_ACCOUNTS_LEN] {
    fn from(accounts: DeactivateAccounts<'_, 'info>) -> Self {
        [accounts.timelock.clone(), accounts.vault_owner.clone(), accounts.payer.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; DEACTIVATE_IX_ACCOUNTS_LEN]>
for DeactivateAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; DEACTIVATE_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: &arr[0],
            vault_owner: &arr[1],
            payer: &arr[2],
        }
    }
}
pub const DEACTIVATE_IX_DISCM: [u8; 8] = [44, 112, 33, 172, 113, 28, 142, 13];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeactivateIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DeactivateIxData(pub DeactivateIxArgs);
impl From<DeactivateIxArgs> for DeactivateIxData {
    fn from(args: DeactivateIxArgs) -> Self {
        Self(args)
    }
}
impl DeactivateIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != DEACTIVATE_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        DEACTIVATE_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(DeactivateIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&DEACTIVATE_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn deactivate_ix_with_program_id(
    program_id: Pubkey,
    keys: DeactivateKeys,
    args: DeactivateIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; DEACTIVATE_IX_ACCOUNTS_LEN] = keys.into();
    let data: DeactivateIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn deactivate_ix(
    keys: DeactivateKeys,
    args: DeactivateIxArgs,
) -> std::io::Result<Instruction> {
    deactivate_ix_with_program_id(crate::ID, keys, args)
}
pub fn deactivate_invoke_with_program_id(
    program_id: Pubkey,
    accounts: DeactivateAccounts<'_, '_>,
    args: DeactivateIxArgs,
) -> ProgramResult {
    let keys: DeactivateKeys = accounts.into();
    let ix = deactivate_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn deactivate_invoke(
    accounts: DeactivateAccounts<'_, '_>,
    args: DeactivateIxArgs,
) -> ProgramResult {
    deactivate_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn deactivate_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: DeactivateAccounts<'_, '_>,
    args: DeactivateIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: DeactivateKeys = accounts.into();
    let ix = deactivate_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn deactivate_invoke_signed(
    accounts: DeactivateAccounts<'_, '_>,
    args: DeactivateIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    deactivate_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn deactivate_verify_account_keys(
    accounts: DeactivateAccounts<'_, '_>,
    keys: DeactivateKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.payer.key, keys.payer),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn deactivate_verify_writable_privileges<'me, 'info>(
    accounts: DeactivateAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn deactivate_verify_signer_privileges<'me, 'info>(
    accounts: DeactivateAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.vault_owner, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn deactivate_verify_account_privileges<'me, 'info>(
    accounts: DeactivateAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    deactivate_verify_writable_privileges(accounts)?;
    deactivate_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct RevokeLockWithTimeoutAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RevokeLockWithTimeoutKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<RevokeLockWithTimeoutAccounts<'_, '_>> for RevokeLockWithTimeoutKeys {
    fn from(accounts: RevokeLockWithTimeoutAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            vault_owner: *accounts.vault_owner.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<RevokeLockWithTimeoutKeys>
for [AccountMeta; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN] {
    fn from(keys: RevokeLockWithTimeoutKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN]>
for RevokeLockWithTimeoutKeys {
    fn from(pubkeys: [Pubkey; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            vault_owner: pubkeys[2],
            payer: pubkeys[3],
            token_program: pubkeys[4],
            system_program: pubkeys[5],
        }
    }
}
impl<'info> From<RevokeLockWithTimeoutAccounts<'_, 'info>>
for [AccountInfo<'info>; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN] {
    fn from(accounts: RevokeLockWithTimeoutAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.vault_owner.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<
    'me,
    'info,
> From<&'me [AccountInfo<'info>; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN]>
for RevokeLockWithTimeoutAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            vault_owner: &arr[2],
            payer: &arr[3],
            token_program: &arr[4],
            system_program: &arr[5],
        }
    }
}
pub const REVOKE_LOCK_WITH_TIMEOUT_IX_DISCM: [u8; 8] = [
    19,
    131,
    47,
    77,
    60,
    188,
    120,
    124,
];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RevokeLockWithTimeoutIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RevokeLockWithTimeoutIxData(pub RevokeLockWithTimeoutIxArgs);
impl From<RevokeLockWithTimeoutIxArgs> for RevokeLockWithTimeoutIxData {
    fn from(args: RevokeLockWithTimeoutIxArgs) -> Self {
        Self(args)
    }
}
impl RevokeLockWithTimeoutIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != REVOKE_LOCK_WITH_TIMEOUT_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        REVOKE_LOCK_WITH_TIMEOUT_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(RevokeLockWithTimeoutIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&REVOKE_LOCK_WITH_TIMEOUT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn revoke_lock_with_timeout_ix_with_program_id(
    program_id: Pubkey,
    keys: RevokeLockWithTimeoutKeys,
    args: RevokeLockWithTimeoutIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; REVOKE_LOCK_WITH_TIMEOUT_IX_ACCOUNTS_LEN] = keys.into();
    let data: RevokeLockWithTimeoutIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn revoke_lock_with_timeout_ix(
    keys: RevokeLockWithTimeoutKeys,
    args: RevokeLockWithTimeoutIxArgs,
) -> std::io::Result<Instruction> {
    revoke_lock_with_timeout_ix_with_program_id(crate::ID, keys, args)
}
pub fn revoke_lock_with_timeout_invoke_with_program_id(
    program_id: Pubkey,
    accounts: RevokeLockWithTimeoutAccounts<'_, '_>,
    args: RevokeLockWithTimeoutIxArgs,
) -> ProgramResult {
    let keys: RevokeLockWithTimeoutKeys = accounts.into();
    let ix = revoke_lock_with_timeout_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn revoke_lock_with_timeout_invoke(
    accounts: RevokeLockWithTimeoutAccounts<'_, '_>,
    args: RevokeLockWithTimeoutIxArgs,
) -> ProgramResult {
    revoke_lock_with_timeout_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn revoke_lock_with_timeout_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: RevokeLockWithTimeoutAccounts<'_, '_>,
    args: RevokeLockWithTimeoutIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: RevokeLockWithTimeoutKeys = accounts.into();
    let ix = revoke_lock_with_timeout_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn revoke_lock_with_timeout_invoke_signed(
    accounts: RevokeLockWithTimeoutAccounts<'_, '_>,
    args: RevokeLockWithTimeoutIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    revoke_lock_with_timeout_invoke_signed_with_program_id(
        crate::ID,
        accounts,
        args,
        seeds,
    )
}
pub fn revoke_lock_with_timeout_verify_account_keys(
    accounts: RevokeLockWithTimeoutAccounts<'_, '_>,
    keys: RevokeLockWithTimeoutKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn revoke_lock_with_timeout_verify_writable_privileges<'me, 'info>(
    accounts: RevokeLockWithTimeoutAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn revoke_lock_with_timeout_verify_signer_privileges<'me, 'info>(
    accounts: RevokeLockWithTimeoutAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.vault_owner, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn revoke_lock_with_timeout_verify_account_privileges<'me, 'info>(
    accounts: RevokeLockWithTimeoutAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    revoke_lock_with_timeout_verify_writable_privileges(accounts)?;
    revoke_lock_with_timeout_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct RevokeLockWithAuthorityAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub time_authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RevokeLockWithAuthorityKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub time_authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<RevokeLockWithAuthorityAccounts<'_, '_>> for RevokeLockWithAuthorityKeys {
    fn from(accounts: RevokeLockWithAuthorityAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            time_authority: *accounts.time_authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<RevokeLockWithAuthorityKeys>
for [AccountMeta; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(keys: RevokeLockWithAuthorityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.time_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN]>
for RevokeLockWithAuthorityKeys {
    fn from(pubkeys: [Pubkey; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            time_authority: pubkeys[2],
            payer: pubkeys[3],
            token_program: pubkeys[4],
            system_program: pubkeys[5],
        }
    }
}
impl<'info> From<RevokeLockWithAuthorityAccounts<'_, 'info>>
for [AccountInfo<'info>; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(accounts: RevokeLockWithAuthorityAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.time_authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<
    'me,
    'info,
> From<&'me [AccountInfo<'info>; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN]>
for RevokeLockWithAuthorityAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            time_authority: &arr[2],
            payer: &arr[3],
            token_program: &arr[4],
            system_program: &arr[5],
        }
    }
}
pub const REVOKE_LOCK_WITH_AUTHORITY_IX_DISCM: [u8; 8] = [
    229,
    181,
    58,
    242,
    171,
    8,
    201,
    144,
];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RevokeLockWithAuthorityIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RevokeLockWithAuthorityIxData(pub RevokeLockWithAuthorityIxArgs);
impl From<RevokeLockWithAuthorityIxArgs> for RevokeLockWithAuthorityIxData {
    fn from(args: RevokeLockWithAuthorityIxArgs) -> Self {
        Self(args)
    }
}
impl RevokeLockWithAuthorityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != REVOKE_LOCK_WITH_AUTHORITY_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        REVOKE_LOCK_WITH_AUTHORITY_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(RevokeLockWithAuthorityIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&REVOKE_LOCK_WITH_AUTHORITY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn revoke_lock_with_authority_ix_with_program_id(
    program_id: Pubkey,
    keys: RevokeLockWithAuthorityKeys,
    args: RevokeLockWithAuthorityIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; REVOKE_LOCK_WITH_AUTHORITY_IX_ACCOUNTS_LEN] = keys.into();
    let data: RevokeLockWithAuthorityIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn revoke_lock_with_authority_ix(
    keys: RevokeLockWithAuthorityKeys,
    args: RevokeLockWithAuthorityIxArgs,
) -> std::io::Result<Instruction> {
    revoke_lock_with_authority_ix_with_program_id(crate::ID, keys, args)
}
pub fn revoke_lock_with_authority_invoke_with_program_id(
    program_id: Pubkey,
    accounts: RevokeLockWithAuthorityAccounts<'_, '_>,
    args: RevokeLockWithAuthorityIxArgs,
) -> ProgramResult {
    let keys: RevokeLockWithAuthorityKeys = accounts.into();
    let ix = revoke_lock_with_authority_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn revoke_lock_with_authority_invoke(
    accounts: RevokeLockWithAuthorityAccounts<'_, '_>,
    args: RevokeLockWithAuthorityIxArgs,
) -> ProgramResult {
    revoke_lock_with_authority_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn revoke_lock_with_authority_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: RevokeLockWithAuthorityAccounts<'_, '_>,
    args: RevokeLockWithAuthorityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: RevokeLockWithAuthorityKeys = accounts.into();
    let ix = revoke_lock_with_authority_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn revoke_lock_with_authority_invoke_signed(
    accounts: RevokeLockWithAuthorityAccounts<'_, '_>,
    args: RevokeLockWithAuthorityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    revoke_lock_with_authority_invoke_signed_with_program_id(
        crate::ID,
        accounts,
        args,
        seeds,
    )
}
pub fn revoke_lock_with_authority_verify_account_keys(
    accounts: RevokeLockWithAuthorityAccounts<'_, '_>,
    keys: RevokeLockWithAuthorityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.time_authority.key, keys.time_authority),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn revoke_lock_with_authority_verify_writable_privileges<'me, 'info>(
    accounts: RevokeLockWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn revoke_lock_with_authority_verify_signer_privileges<'me, 'info>(
    accounts: RevokeLockWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.time_authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn revoke_lock_with_authority_verify_account_privileges<'me, 'info>(
    accounts: RevokeLockWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    revoke_lock_with_authority_verify_writable_privileges(accounts)?;
    revoke_lock_with_authority_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN: usize = 4;
#[derive(Copy, Clone, Debug)]
pub struct CancelLockTimeoutAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub time_authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CancelLockTimeoutKeys {
    pub timelock: Pubkey,
    pub time_authority: Pubkey,
    pub payer: Pubkey,
    pub system_program: Pubkey,
}
impl From<CancelLockTimeoutAccounts<'_, '_>> for CancelLockTimeoutKeys {
    fn from(accounts: CancelLockTimeoutAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            time_authority: *accounts.time_authority.key,
            payer: *accounts.payer.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CancelLockTimeoutKeys> for [AccountMeta; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN] {
    fn from(keys: CancelLockTimeoutKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.time_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN]> for CancelLockTimeoutKeys {
    fn from(pubkeys: [Pubkey; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            time_authority: pubkeys[1],
            payer: pubkeys[2],
            system_program: pubkeys[3],
        }
    }
}
impl<'info> From<CancelLockTimeoutAccounts<'_, 'info>>
for [AccountInfo<'info>; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN] {
    fn from(accounts: CancelLockTimeoutAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.time_authority.clone(),
            accounts.payer.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN]>
for CancelLockTimeoutAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            timelock: &arr[0],
            time_authority: &arr[1],
            payer: &arr[2],
            system_program: &arr[3],
        }
    }
}
pub const CANCEL_LOCK_TIMEOUT_IX_DISCM: [u8; 8] = [
    207,
    206,
    169,
    241,
    34,
    199,
    123,
    249,
];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CancelLockTimeoutIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CancelLockTimeoutIxData(pub CancelLockTimeoutIxArgs);
impl From<CancelLockTimeoutIxArgs> for CancelLockTimeoutIxData {
    fn from(args: CancelLockTimeoutIxArgs) -> Self {
        Self(args)
    }
}
impl CancelLockTimeoutIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CANCEL_LOCK_TIMEOUT_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        CANCEL_LOCK_TIMEOUT_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(CancelLockTimeoutIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CANCEL_LOCK_TIMEOUT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn cancel_lock_timeout_ix_with_program_id(
    program_id: Pubkey,
    keys: CancelLockTimeoutKeys,
    args: CancelLockTimeoutIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CANCEL_LOCK_TIMEOUT_IX_ACCOUNTS_LEN] = keys.into();
    let data: CancelLockTimeoutIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn cancel_lock_timeout_ix(
    keys: CancelLockTimeoutKeys,
    args: CancelLockTimeoutIxArgs,
) -> std::io::Result<Instruction> {
    cancel_lock_timeout_ix_with_program_id(crate::ID, keys, args)
}
pub fn cancel_lock_timeout_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CancelLockTimeoutAccounts<'_, '_>,
    args: CancelLockTimeoutIxArgs,
) -> ProgramResult {
    let keys: CancelLockTimeoutKeys = accounts.into();
    let ix = cancel_lock_timeout_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn cancel_lock_timeout_invoke(
    accounts: CancelLockTimeoutAccounts<'_, '_>,
    args: CancelLockTimeoutIxArgs,
) -> ProgramResult {
    cancel_lock_timeout_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn cancel_lock_timeout_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CancelLockTimeoutAccounts<'_, '_>,
    args: CancelLockTimeoutIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CancelLockTimeoutKeys = accounts.into();
    let ix = cancel_lock_timeout_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn cancel_lock_timeout_invoke_signed(
    accounts: CancelLockTimeoutAccounts<'_, '_>,
    args: CancelLockTimeoutIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    cancel_lock_timeout_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn cancel_lock_timeout_verify_account_keys(
    accounts: CancelLockTimeoutAccounts<'_, '_>,
    keys: CancelLockTimeoutKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.time_authority.key, keys.time_authority),
        (*accounts.payer.key, keys.payer),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn cancel_lock_timeout_verify_writable_privileges<'me, 'info>(
    accounts: CancelLockTimeoutAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn cancel_lock_timeout_verify_signer_privileges<'me, 'info>(
    accounts: CancelLockTimeoutAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.time_authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn cancel_lock_timeout_verify_account_privileges<'me, 'info>(
    accounts: CancelLockTimeoutAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    cancel_lock_timeout_verify_writable_privileges(accounts)?;
    cancel_lock_timeout_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN: usize = 8;
#[derive(Copy, Clone, Debug)]
pub struct TransferWithAuthorityAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub time_authority: &'me AccountInfo<'info>,
    pub destination: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransferWithAuthorityKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub time_authority: Pubkey,
    pub destination: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<TransferWithAuthorityAccounts<'_, '_>> for TransferWithAuthorityKeys {
    fn from(accounts: TransferWithAuthorityAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            vault_owner: *accounts.vault_owner.key,
            time_authority: *accounts.time_authority.key,
            destination: *accounts.destination.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<TransferWithAuthorityKeys>
for [AccountMeta; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(keys: TransferWithAuthorityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.time_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN]>
for TransferWithAuthorityKeys {
    fn from(pubkeys: [Pubkey; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            vault_owner: pubkeys[2],
            time_authority: pubkeys[3],
            destination: pubkeys[4],
            payer: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
        }
    }
}
impl<'info> From<TransferWithAuthorityAccounts<'_, 'info>>
for [AccountInfo<'info>; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(accounts: TransferWithAuthorityAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.vault_owner.clone(),
            accounts.time_authority.clone(),
            accounts.destination.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN]>
for TransferWithAuthorityAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            vault_owner: &arr[2],
            time_authority: &arr[3],
            destination: &arr[4],
            payer: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
        }
    }
}
pub const TRANSFER_WITH_AUTHORITY_IX_DISCM: [u8; 8] = [
    68,
    128,
    222,
    192,
    129,
    69,
    71,
    165,
];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransferWithAuthorityIxArgs {
    pub timelock_bump: u8,
    pub amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TransferWithAuthorityIxData(pub TransferWithAuthorityIxArgs);
impl From<TransferWithAuthorityIxArgs> for TransferWithAuthorityIxData {
    fn from(args: TransferWithAuthorityIxArgs) -> Self {
        Self(args)
    }
}
impl TransferWithAuthorityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != TRANSFER_WITH_AUTHORITY_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        TRANSFER_WITH_AUTHORITY_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(TransferWithAuthorityIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&TRANSFER_WITH_AUTHORITY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn transfer_with_authority_ix_with_program_id(
    program_id: Pubkey,
    keys: TransferWithAuthorityKeys,
    args: TransferWithAuthorityIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; TRANSFER_WITH_AUTHORITY_IX_ACCOUNTS_LEN] = keys.into();
    let data: TransferWithAuthorityIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn transfer_with_authority_ix(
    keys: TransferWithAuthorityKeys,
    args: TransferWithAuthorityIxArgs,
) -> std::io::Result<Instruction> {
    transfer_with_authority_ix_with_program_id(crate::ID, keys, args)
}
pub fn transfer_with_authority_invoke_with_program_id(
    program_id: Pubkey,
    accounts: TransferWithAuthorityAccounts<'_, '_>,
    args: TransferWithAuthorityIxArgs,
) -> ProgramResult {
    let keys: TransferWithAuthorityKeys = accounts.into();
    let ix = transfer_with_authority_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn transfer_with_authority_invoke(
    accounts: TransferWithAuthorityAccounts<'_, '_>,
    args: TransferWithAuthorityIxArgs,
) -> ProgramResult {
    transfer_with_authority_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn transfer_with_authority_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: TransferWithAuthorityAccounts<'_, '_>,
    args: TransferWithAuthorityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: TransferWithAuthorityKeys = accounts.into();
    let ix = transfer_with_authority_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn transfer_with_authority_invoke_signed(
    accounts: TransferWithAuthorityAccounts<'_, '_>,
    args: TransferWithAuthorityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    transfer_with_authority_invoke_signed_with_program_id(
        crate::ID,
        accounts,
        args,
        seeds,
    )
}
pub fn transfer_with_authority_verify_account_keys(
    accounts: TransferWithAuthorityAccounts<'_, '_>,
    keys: TransferWithAuthorityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.time_authority.key, keys.time_authority),
        (*accounts.destination.key, keys.destination),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn transfer_with_authority_verify_writable_privileges<'me, 'info>(
    accounts: TransferWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.vault, accounts.destination, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn transfer_with_authority_verify_signer_privileges<'me, 'info>(
    accounts: TransferWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [
        accounts.vault_owner,
        accounts.time_authority,
        accounts.payer,
    ] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn transfer_with_authority_verify_account_privileges<'me, 'info>(
    accounts: TransferWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    transfer_with_authority_verify_writable_privileges(accounts)?;
    transfer_with_authority_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN: usize = 8;
#[derive(Copy, Clone, Debug)]
pub struct BurnDustWithAuthorityAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub time_authority: &'me AccountInfo<'info>,
    pub mint: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BurnDustWithAuthorityKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub time_authority: Pubkey,
    pub mint: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<BurnDustWithAuthorityAccounts<'_, '_>> for BurnDustWithAuthorityKeys {
    fn from(accounts: BurnDustWithAuthorityAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            vault_owner: *accounts.vault_owner.key,
            time_authority: *accounts.time_authority.key,
            mint: *accounts.mint.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<BurnDustWithAuthorityKeys>
for [AccountMeta; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(keys: BurnDustWithAuthorityKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.time_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.mint,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN]>
for BurnDustWithAuthorityKeys {
    fn from(pubkeys: [Pubkey; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            vault_owner: pubkeys[2],
            time_authority: pubkeys[3],
            mint: pubkeys[4],
            payer: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
        }
    }
}
impl<'info> From<BurnDustWithAuthorityAccounts<'_, 'info>>
for [AccountInfo<'info>; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN] {
    fn from(accounts: BurnDustWithAuthorityAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.vault_owner.clone(),
            accounts.time_authority.clone(),
            accounts.mint.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<
    'me,
    'info,
> From<&'me [AccountInfo<'info>; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN]>
for BurnDustWithAuthorityAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            vault_owner: &arr[2],
            time_authority: &arr[3],
            mint: &arr[4],
            payer: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
        }
    }
}
pub const BURN_DUST_WITH_AUTHORITY_IX_DISCM: [u8; 8] = [
    39,
    42,
    255,
    218,
    14,
    124,
    78,
    45,
];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BurnDustWithAuthorityIxArgs {
    pub timelock_bump: u8,
    pub max_amount: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct BurnDustWithAuthorityIxData(pub BurnDustWithAuthorityIxArgs);
impl From<BurnDustWithAuthorityIxArgs> for BurnDustWithAuthorityIxData {
    fn from(args: BurnDustWithAuthorityIxArgs) -> Self {
        Self(args)
    }
}
impl BurnDustWithAuthorityIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != BURN_DUST_WITH_AUTHORITY_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        BURN_DUST_WITH_AUTHORITY_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(BurnDustWithAuthorityIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&BURN_DUST_WITH_AUTHORITY_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn burn_dust_with_authority_ix_with_program_id(
    program_id: Pubkey,
    keys: BurnDustWithAuthorityKeys,
    args: BurnDustWithAuthorityIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; BURN_DUST_WITH_AUTHORITY_IX_ACCOUNTS_LEN] = keys.into();
    let data: BurnDustWithAuthorityIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn burn_dust_with_authority_ix(
    keys: BurnDustWithAuthorityKeys,
    args: BurnDustWithAuthorityIxArgs,
) -> std::io::Result<Instruction> {
    burn_dust_with_authority_ix_with_program_id(crate::ID, keys, args)
}
pub fn burn_dust_with_authority_invoke_with_program_id(
    program_id: Pubkey,
    accounts: BurnDustWithAuthorityAccounts<'_, '_>,
    args: BurnDustWithAuthorityIxArgs,
) -> ProgramResult {
    let keys: BurnDustWithAuthorityKeys = accounts.into();
    let ix = burn_dust_with_authority_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn burn_dust_with_authority_invoke(
    accounts: BurnDustWithAuthorityAccounts<'_, '_>,
    args: BurnDustWithAuthorityIxArgs,
) -> ProgramResult {
    burn_dust_with_authority_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn burn_dust_with_authority_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: BurnDustWithAuthorityAccounts<'_, '_>,
    args: BurnDustWithAuthorityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: BurnDustWithAuthorityKeys = accounts.into();
    let ix = burn_dust_with_authority_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn burn_dust_with_authority_invoke_signed(
    accounts: BurnDustWithAuthorityAccounts<'_, '_>,
    args: BurnDustWithAuthorityIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    burn_dust_with_authority_invoke_signed_with_program_id(
        crate::ID,
        accounts,
        args,
        seeds,
    )
}
pub fn burn_dust_with_authority_verify_account_keys(
    accounts: BurnDustWithAuthorityAccounts<'_, '_>,
    keys: BurnDustWithAuthorityKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.time_authority.key, keys.time_authority),
        (*accounts.mint.key, keys.mint),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn burn_dust_with_authority_verify_writable_privileges<'me, 'info>(
    accounts: BurnDustWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.vault, accounts.mint, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn burn_dust_with_authority_verify_signer_privileges<'me, 'info>(
    accounts: BurnDustWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [
        accounts.vault_owner,
        accounts.time_authority,
        accounts.payer,
    ] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn burn_dust_with_authority_verify_account_privileges<'me, 'info>(
    accounts: BurnDustWithAuthorityAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    burn_dust_with_authority_verify_writable_privileges(accounts)?;
    burn_dust_with_authority_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const WITHDRAW_IX_ACCOUNTS_LEN: usize = 7;
#[derive(Copy, Clone, Debug)]
pub struct WithdrawAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub vault_owner: &'me AccountInfo<'info>,
    pub destination: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WithdrawKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub vault_owner: Pubkey,
    pub destination: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<WithdrawAccounts<'_, '_>> for WithdrawKeys {
    fn from(accounts: WithdrawAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            vault_owner: *accounts.vault_owner.key,
            destination: *accounts.destination.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<WithdrawKeys> for [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] {
    fn from(keys: WithdrawKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault_owner,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]> for WithdrawKeys {
    fn from(pubkeys: [Pubkey; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            vault_owner: pubkeys[2],
            destination: pubkeys[3],
            payer: pubkeys[4],
            token_program: pubkeys[5],
            system_program: pubkeys[6],
        }
    }
}
impl<'info> From<WithdrawAccounts<'_, 'info>>
for [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN] {
    fn from(accounts: WithdrawAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.vault_owner.clone(),
            accounts.destination.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]>
for WithdrawAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; WITHDRAW_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            vault_owner: &arr[2],
            destination: &arr[3],
            payer: &arr[4],
            token_program: &arr[5],
            system_program: &arr[6],
        }
    }
}
pub const WITHDRAW_IX_DISCM: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithdrawIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WithdrawIxData(pub WithdrawIxArgs);
impl From<WithdrawIxArgs> for WithdrawIxData {
    fn from(args: WithdrawIxArgs) -> Self {
        Self(args)
    }
}
impl WithdrawIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != WITHDRAW_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        WITHDRAW_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(WithdrawIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&WITHDRAW_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn withdraw_ix_with_program_id(
    program_id: Pubkey,
    keys: WithdrawKeys,
    args: WithdrawIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; WITHDRAW_IX_ACCOUNTS_LEN] = keys.into();
    let data: WithdrawIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn withdraw_ix(
    keys: WithdrawKeys,
    args: WithdrawIxArgs,
) -> std::io::Result<Instruction> {
    withdraw_ix_with_program_id(crate::ID, keys, args)
}
pub fn withdraw_invoke_with_program_id(
    program_id: Pubkey,
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
) -> ProgramResult {
    let keys: WithdrawKeys = accounts.into();
    let ix = withdraw_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn withdraw_invoke(
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
) -> ProgramResult {
    withdraw_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn withdraw_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: WithdrawKeys = accounts.into();
    let ix = withdraw_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn withdraw_invoke_signed(
    accounts: WithdrawAccounts<'_, '_>,
    args: WithdrawIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    withdraw_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn withdraw_verify_account_keys(
    accounts: WithdrawAccounts<'_, '_>,
    keys: WithdrawKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.vault_owner.key, keys.vault_owner),
        (*accounts.destination.key, keys.destination),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn withdraw_verify_writable_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.vault, accounts.destination, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn withdraw_verify_signer_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.vault_owner, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn withdraw_verify_account_privileges<'me, 'info>(
    accounts: WithdrawAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    withdraw_verify_writable_privileges(accounts)?;
    withdraw_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct CloseAccountsAccounts<'me, 'info> {
    pub timelock: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub close_authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CloseAccountsKeys {
    pub timelock: Pubkey,
    pub vault: Pubkey,
    pub close_authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<CloseAccountsAccounts<'_, '_>> for CloseAccountsKeys {
    fn from(accounts: CloseAccountsAccounts) -> Self {
        Self {
            timelock: *accounts.timelock.key,
            vault: *accounts.vault.key,
            close_authority: *accounts.close_authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CloseAccountsKeys> for [AccountMeta; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN] {
    fn from(keys: CloseAccountsKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.timelock,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.close_authority,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN]> for CloseAccountsKeys {
    fn from(pubkeys: [Pubkey; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: pubkeys[0],
            vault: pubkeys[1],
            close_authority: pubkeys[2],
            payer: pubkeys[3],
            token_program: pubkeys[4],
            system_program: pubkeys[5],
        }
    }
}
impl<'info> From<CloseAccountsAccounts<'_, 'info>>
for [AccountInfo<'info>; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN] {
    fn from(accounts: CloseAccountsAccounts<'_, 'info>) -> Self {
        [
            accounts.timelock.clone(),
            accounts.vault.clone(),
            accounts.close_authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN]>
for CloseAccountsAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            timelock: &arr[0],
            vault: &arr[1],
            close_authority: &arr[2],
            payer: &arr[3],
            token_program: &arr[4],
            system_program: &arr[5],
        }
    }
}
pub const CLOSE_ACCOUNTS_IX_DISCM: [u8; 8] = [171, 222, 94, 233, 34, 250, 202, 1];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CloseAccountsIxArgs {
    pub timelock_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CloseAccountsIxData(pub CloseAccountsIxArgs);
impl From<CloseAccountsIxArgs> for CloseAccountsIxData {
    fn from(args: CloseAccountsIxArgs) -> Self {
        Self(args)
    }
}
impl CloseAccountsIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CLOSE_ACCOUNTS_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        CLOSE_ACCOUNTS_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(CloseAccountsIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CLOSE_ACCOUNTS_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn close_accounts_ix_with_program_id(
    program_id: Pubkey,
    keys: CloseAccountsKeys,
    args: CloseAccountsIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CLOSE_ACCOUNTS_IX_ACCOUNTS_LEN] = keys.into();
    let data: CloseAccountsIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn close_accounts_ix(
    keys: CloseAccountsKeys,
    args: CloseAccountsIxArgs,
) -> std::io::Result<Instruction> {
    close_accounts_ix_with_program_id(crate::ID, keys, args)
}
pub fn close_accounts_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CloseAccountsAccounts<'_, '_>,
    args: CloseAccountsIxArgs,
) -> ProgramResult {
    let keys: CloseAccountsKeys = accounts.into();
    let ix = close_accounts_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn close_accounts_invoke(
    accounts: CloseAccountsAccounts<'_, '_>,
    args: CloseAccountsIxArgs,
) -> ProgramResult {
    close_accounts_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn close_accounts_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CloseAccountsAccounts<'_, '_>,
    args: CloseAccountsIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CloseAccountsKeys = accounts.into();
    let ix = close_accounts_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn close_accounts_invoke_signed(
    accounts: CloseAccountsAccounts<'_, '_>,
    args: CloseAccountsIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    close_accounts_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn close_accounts_verify_account_keys(
    accounts: CloseAccountsAccounts<'_, '_>,
    keys: CloseAccountsKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.timelock.key, keys.timelock),
        (*accounts.vault.key, keys.vault),
        (*accounts.close_authority.key, keys.close_authority),
        (*accounts.payer.key, keys.payer),
        (*accounts.token_program.key, keys.token_program),
        (*accounts.system_program.key, keys.system_program),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn close_accounts_verify_writable_privileges<'me, 'info>(
    accounts: CloseAccountsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.timelock, accounts.vault, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn close_accounts_verify_signer_privileges<'me, 'info>(
    accounts: CloseAccountsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.close_authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn close_accounts_verify_account_privileges<'me, 'info>(
    accounts: CloseAccountsAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    close_accounts_verify_writable_privileges(accounts)?;
    close_accounts_verify_signer_privileges(accounts)?;
    Ok(())
}
