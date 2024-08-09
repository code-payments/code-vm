use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    pubkey::Pubkey, program_error::ProgramError,
};
use std::io::Read;
#[derive(Clone, Debug, PartialEq)]
pub enum SplitterProgramIx {
    InitializePool(InitializePoolIxArgs),
    SaveRecentRoot(SaveRecentRootIxArgs),
    TransferWithCommitment(TransferWithCommitmentIxArgs),
    InitializeProof(InitializeProofIxArgs),
    UploadProof(UploadProofIxArgs),
    VerifyProof(VerifyProofIxArgs),
    CloseProof(CloseProofIxArgs),
    OpenTokenAccount(OpenTokenAccountIxArgs),
    CloseTokenAccount(CloseTokenAccountIxArgs),
}
impl SplitterProgramIx {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        match maybe_discm {
            INITIALIZE_POOL_IX_DISCM => {
                Ok(Self::InitializePool(InitializePoolIxArgs::deserialize(&mut reader)?))
            }
            SAVE_RECENT_ROOT_IX_DISCM => {
                Ok(Self::SaveRecentRoot(SaveRecentRootIxArgs::deserialize(&mut reader)?))
            }
            TRANSFER_WITH_COMMITMENT_IX_DISCM => {
                Ok(
                    Self::TransferWithCommitment(
                        TransferWithCommitmentIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            INITIALIZE_PROOF_IX_DISCM => {
                Ok(
                    Self::InitializeProof(
                        InitializeProofIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            UPLOAD_PROOF_IX_DISCM => {
                Ok(Self::UploadProof(UploadProofIxArgs::deserialize(&mut reader)?))
            }
            VERIFY_PROOF_IX_DISCM => {
                Ok(Self::VerifyProof(VerifyProofIxArgs::deserialize(&mut reader)?))
            }
            CLOSE_PROOF_IX_DISCM => {
                Ok(Self::CloseProof(CloseProofIxArgs::deserialize(&mut reader)?))
            }
            OPEN_TOKEN_ACCOUNT_IX_DISCM => {
                Ok(
                    Self::OpenTokenAccount(
                        OpenTokenAccountIxArgs::deserialize(&mut reader)?,
                    ),
                )
            }
            CLOSE_TOKEN_ACCOUNT_IX_DISCM => {
                Ok(
                    Self::CloseTokenAccount(
                        CloseTokenAccountIxArgs::deserialize(&mut reader)?,
                    ),
                )
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
            Self::InitializePool(args) => {
                writer.write_all(&INITIALIZE_POOL_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::SaveRecentRoot(args) => {
                writer.write_all(&SAVE_RECENT_ROOT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::TransferWithCommitment(args) => {
                writer.write_all(&TRANSFER_WITH_COMMITMENT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::InitializeProof(args) => {
                writer.write_all(&INITIALIZE_PROOF_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::UploadProof(args) => {
                writer.write_all(&UPLOAD_PROOF_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::VerifyProof(args) => {
                writer.write_all(&VERIFY_PROOF_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CloseProof(args) => {
                writer.write_all(&CLOSE_PROOF_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::OpenTokenAccount(args) => {
                writer.write_all(&OPEN_TOKEN_ACCOUNT_IX_DISCM)?;
                args.serialize(&mut writer)
            }
            Self::CloseTokenAccount(args) => {
                writer.write_all(&CLOSE_TOKEN_ACCOUNT_IX_DISCM)?;
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
pub const INITIALIZE_POOL_IX_ACCOUNTS_LEN: usize = 8;
#[derive(Copy, Clone, Debug)]
pub struct InitializePoolAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub mint: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitializePoolKeys {
    pub pool: Pubkey,
    pub vault: Pubkey,
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<InitializePoolAccounts<'_, '_>> for InitializePoolKeys {
    fn from(accounts: InitializePoolAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            vault: *accounts.vault.key,
            mint: *accounts.mint.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<InitializePoolKeys> for [AccountMeta; INITIALIZE_POOL_IX_ACCOUNTS_LEN] {
    fn from(keys: InitializePoolKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; INITIALIZE_POOL_IX_ACCOUNTS_LEN]> for InitializePoolKeys {
    fn from(pubkeys: [Pubkey; INITIALIZE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            vault: pubkeys[1],
            mint: pubkeys[2],
            authority: pubkeys[3],
            payer: pubkeys[4],
            token_program: pubkeys[5],
            system_program: pubkeys[6],
            rent: pubkeys[7],
        }
    }
}
impl<'info> From<InitializePoolAccounts<'_, 'info>>
for [AccountInfo<'info>; INITIALIZE_POOL_IX_ACCOUNTS_LEN] {
    fn from(accounts: InitializePoolAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.vault.clone(),
            accounts.mint.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INITIALIZE_POOL_IX_ACCOUNTS_LEN]>
for InitializePoolAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; INITIALIZE_POOL_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            vault: &arr[1],
            mint: &arr[2],
            authority: &arr[3],
            payer: &arr[4],
            token_program: &arr[5],
            system_program: &arr[6],
            rent: &arr[7],
        }
    }
}
pub const INITIALIZE_POOL_IX_DISCM: [u8; 8] = [95, 180, 10, 172, 84, 174, 232, 40];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializePoolIxArgs {
    pub name: String,
    pub levels: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InitializePoolIxData(pub InitializePoolIxArgs);
impl From<InitializePoolIxArgs> for InitializePoolIxData {
    fn from(args: InitializePoolIxArgs) -> Self {
        Self(args)
    }
}
impl InitializePoolIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != INITIALIZE_POOL_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        INITIALIZE_POOL_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(InitializePoolIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&INITIALIZE_POOL_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn initialize_pool_ix_with_program_id(
    program_id: Pubkey,
    keys: InitializePoolKeys,
    args: InitializePoolIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; INITIALIZE_POOL_IX_ACCOUNTS_LEN] = keys.into();
    let data: InitializePoolIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn initialize_pool_ix(
    keys: InitializePoolKeys,
    args: InitializePoolIxArgs,
) -> std::io::Result<Instruction> {
    initialize_pool_ix_with_program_id(crate::ID, keys, args)
}
pub fn initialize_pool_invoke_with_program_id(
    program_id: Pubkey,
    accounts: InitializePoolAccounts<'_, '_>,
    args: InitializePoolIxArgs,
) -> ProgramResult {
    let keys: InitializePoolKeys = accounts.into();
    let ix = initialize_pool_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn initialize_pool_invoke(
    accounts: InitializePoolAccounts<'_, '_>,
    args: InitializePoolIxArgs,
) -> ProgramResult {
    initialize_pool_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn initialize_pool_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: InitializePoolAccounts<'_, '_>,
    args: InitializePoolIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: InitializePoolKeys = accounts.into();
    let ix = initialize_pool_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn initialize_pool_invoke_signed(
    accounts: InitializePoolAccounts<'_, '_>,
    args: InitializePoolIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    initialize_pool_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn initialize_pool_verify_account_keys(
    accounts: InitializePoolAccounts<'_, '_>,
    keys: InitializePoolKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.vault.key, keys.vault),
        (*accounts.mint.key, keys.mint),
        (*accounts.authority.key, keys.authority),
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
pub fn initialize_pool_verify_writable_privileges<'me, 'info>(
    accounts: InitializePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.pool, accounts.vault, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn initialize_pool_verify_signer_privileges<'me, 'info>(
    accounts: InitializePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn initialize_pool_verify_account_privileges<'me, 'info>(
    accounts: InitializePoolAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    initialize_pool_verify_writable_privileges(accounts)?;
    initialize_pool_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN: usize = 3;
#[derive(Copy, Clone, Debug)]
pub struct SaveRecentRootAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SaveRecentRootKeys {
    pub pool: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
}
impl From<SaveRecentRootAccounts<'_, '_>> for SaveRecentRootKeys {
    fn from(accounts: SaveRecentRootAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
        }
    }
}
impl From<SaveRecentRootKeys> for [AccountMeta; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN] {
    fn from(keys: SaveRecentRootKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN]> for SaveRecentRootKeys {
    fn from(pubkeys: [Pubkey; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            authority: pubkeys[1],
            payer: pubkeys[2],
        }
    }
}
impl<'info> From<SaveRecentRootAccounts<'_, 'info>>
for [AccountInfo<'info>; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN] {
    fn from(accounts: SaveRecentRootAccounts<'_, 'info>) -> Self {
        [accounts.pool.clone(), accounts.authority.clone(), accounts.payer.clone()]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN]>
for SaveRecentRootAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            authority: &arr[1],
            payer: &arr[2],
        }
    }
}
pub const SAVE_RECENT_ROOT_IX_DISCM: [u8; 8] = [163, 45, 123, 32, 127, 86, 42, 189];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaveRecentRootIxArgs {
    pub pool_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SaveRecentRootIxData(pub SaveRecentRootIxArgs);
impl From<SaveRecentRootIxArgs> for SaveRecentRootIxData {
    fn from(args: SaveRecentRootIxArgs) -> Self {
        Self(args)
    }
}
impl SaveRecentRootIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != SAVE_RECENT_ROOT_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        SAVE_RECENT_ROOT_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(SaveRecentRootIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&SAVE_RECENT_ROOT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn save_recent_root_ix_with_program_id(
    program_id: Pubkey,
    keys: SaveRecentRootKeys,
    args: SaveRecentRootIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; SAVE_RECENT_ROOT_IX_ACCOUNTS_LEN] = keys.into();
    let data: SaveRecentRootIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn save_recent_root_ix(
    keys: SaveRecentRootKeys,
    args: SaveRecentRootIxArgs,
) -> std::io::Result<Instruction> {
    save_recent_root_ix_with_program_id(crate::ID, keys, args)
}
pub fn save_recent_root_invoke_with_program_id(
    program_id: Pubkey,
    accounts: SaveRecentRootAccounts<'_, '_>,
    args: SaveRecentRootIxArgs,
) -> ProgramResult {
    let keys: SaveRecentRootKeys = accounts.into();
    let ix = save_recent_root_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn save_recent_root_invoke(
    accounts: SaveRecentRootAccounts<'_, '_>,
    args: SaveRecentRootIxArgs,
) -> ProgramResult {
    save_recent_root_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn save_recent_root_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: SaveRecentRootAccounts<'_, '_>,
    args: SaveRecentRootIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: SaveRecentRootKeys = accounts.into();
    let ix = save_recent_root_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn save_recent_root_invoke_signed(
    accounts: SaveRecentRootAccounts<'_, '_>,
    args: SaveRecentRootIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    save_recent_root_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn save_recent_root_verify_account_keys(
    accounts: SaveRecentRootAccounts<'_, '_>,
    keys: SaveRecentRootKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.authority.key, keys.authority),
        (*accounts.payer.key, keys.payer),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn save_recent_root_verify_writable_privileges<'me, 'info>(
    accounts: SaveRecentRootAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.pool, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn save_recent_root_verify_signer_privileges<'me, 'info>(
    accounts: SaveRecentRootAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn save_recent_root_verify_account_privileges<'me, 'info>(
    accounts: SaveRecentRootAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    save_recent_root_verify_writable_privileges(accounts)?;
    save_recent_root_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN: usize = 9;
#[derive(Copy, Clone, Debug)]
pub struct TransferWithCommitmentAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub vault: &'me AccountInfo<'info>,
    pub destination: &'me AccountInfo<'info>,
    pub commitment: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransferWithCommitmentKeys {
    pub pool: Pubkey,
    pub vault: Pubkey,
    pub destination: Pubkey,
    pub commitment: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<TransferWithCommitmentAccounts<'_, '_>> for TransferWithCommitmentKeys {
    fn from(accounts: TransferWithCommitmentAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            vault: *accounts.vault.key,
            destination: *accounts.destination.key,
            commitment: *accounts.commitment.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<TransferWithCommitmentKeys>
for [AccountMeta; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN] {
    fn from(keys: TransferWithCommitmentKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.destination,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.commitment,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN]>
for TransferWithCommitmentKeys {
    fn from(pubkeys: [Pubkey; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            vault: pubkeys[1],
            destination: pubkeys[2],
            commitment: pubkeys[3],
            authority: pubkeys[4],
            payer: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
            rent: pubkeys[8],
        }
    }
}
impl<'info> From<TransferWithCommitmentAccounts<'_, 'info>>
for [AccountInfo<'info>; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN] {
    fn from(accounts: TransferWithCommitmentAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.vault.clone(),
            accounts.destination.clone(),
            accounts.commitment.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<
    'me,
    'info,
> From<&'me [AccountInfo<'info>; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN]>
for TransferWithCommitmentAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            pool: &arr[0],
            vault: &arr[1],
            destination: &arr[2],
            commitment: &arr[3],
            authority: &arr[4],
            payer: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
            rent: &arr[8],
        }
    }
}
pub const TRANSFER_WITH_COMMITMENT_IX_DISCM: [u8; 8] = [
    177,
    247,
    125,
    208,
    153,
    166,
    205,
    120,
];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransferWithCommitmentIxArgs {
    pub pool_bump: u8,
    pub amount: u64,
    pub transcript: [u8; 32],
    pub recent_root: [u8; 32],
}
#[derive(Clone, Debug, PartialEq)]
pub struct TransferWithCommitmentIxData(pub TransferWithCommitmentIxArgs);
impl From<TransferWithCommitmentIxArgs> for TransferWithCommitmentIxData {
    fn from(args: TransferWithCommitmentIxArgs) -> Self {
        Self(args)
    }
}
impl TransferWithCommitmentIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != TRANSFER_WITH_COMMITMENT_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        TRANSFER_WITH_COMMITMENT_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(TransferWithCommitmentIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&TRANSFER_WITH_COMMITMENT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn transfer_with_commitment_ix_with_program_id(
    program_id: Pubkey,
    keys: TransferWithCommitmentKeys,
    args: TransferWithCommitmentIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; TRANSFER_WITH_COMMITMENT_IX_ACCOUNTS_LEN] = keys.into();
    let data: TransferWithCommitmentIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn transfer_with_commitment_ix(
    keys: TransferWithCommitmentKeys,
    args: TransferWithCommitmentIxArgs,
) -> std::io::Result<Instruction> {
    transfer_with_commitment_ix_with_program_id(crate::ID, keys, args)
}
pub fn transfer_with_commitment_invoke_with_program_id(
    program_id: Pubkey,
    accounts: TransferWithCommitmentAccounts<'_, '_>,
    args: TransferWithCommitmentIxArgs,
) -> ProgramResult {
    let keys: TransferWithCommitmentKeys = accounts.into();
    let ix = transfer_with_commitment_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn transfer_with_commitment_invoke(
    accounts: TransferWithCommitmentAccounts<'_, '_>,
    args: TransferWithCommitmentIxArgs,
) -> ProgramResult {
    transfer_with_commitment_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn transfer_with_commitment_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: TransferWithCommitmentAccounts<'_, '_>,
    args: TransferWithCommitmentIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: TransferWithCommitmentKeys = accounts.into();
    let ix = transfer_with_commitment_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn transfer_with_commitment_invoke_signed(
    accounts: TransferWithCommitmentAccounts<'_, '_>,
    args: TransferWithCommitmentIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    transfer_with_commitment_invoke_signed_with_program_id(
        crate::ID,
        accounts,
        args,
        seeds,
    )
}
pub fn transfer_with_commitment_verify_account_keys(
    accounts: TransferWithCommitmentAccounts<'_, '_>,
    keys: TransferWithCommitmentKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.vault.key, keys.vault),
        (*accounts.destination.key, keys.destination),
        (*accounts.commitment.key, keys.commitment),
        (*accounts.authority.key, keys.authority),
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
pub fn transfer_with_commitment_verify_writable_privileges<'me, 'info>(
    accounts: TransferWithCommitmentAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool,
        accounts.vault,
        accounts.destination,
        accounts.payer,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn transfer_with_commitment_verify_signer_privileges<'me, 'info>(
    accounts: TransferWithCommitmentAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn transfer_with_commitment_verify_account_privileges<'me, 'info>(
    accounts: TransferWithCommitmentAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    transfer_with_commitment_verify_writable_privileges(accounts)?;
    transfer_with_commitment_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const INITIALIZE_PROOF_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct InitializeProofAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub proof: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InitializeProofKeys {
    pub pool: Pubkey,
    pub proof: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<InitializeProofAccounts<'_, '_>> for InitializeProofKeys {
    fn from(accounts: InitializeProofAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            proof: *accounts.proof.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<InitializeProofKeys> for [AccountMeta; INITIALIZE_PROOF_IX_ACCOUNTS_LEN] {
    fn from(keys: InitializeProofKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.proof,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
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
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; INITIALIZE_PROOF_IX_ACCOUNTS_LEN]> for InitializeProofKeys {
    fn from(pubkeys: [Pubkey; INITIALIZE_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            proof: pubkeys[1],
            authority: pubkeys[2],
            payer: pubkeys[3],
            system_program: pubkeys[4],
            rent: pubkeys[5],
        }
    }
}
impl<'info> From<InitializeProofAccounts<'_, 'info>>
for [AccountInfo<'info>; INITIALIZE_PROOF_IX_ACCOUNTS_LEN] {
    fn from(accounts: InitializeProofAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.proof.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; INITIALIZE_PROOF_IX_ACCOUNTS_LEN]>
for InitializeProofAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; INITIALIZE_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            proof: &arr[1],
            authority: &arr[2],
            payer: &arr[3],
            system_program: &arr[4],
            rent: &arr[5],
        }
    }
}
pub const INITIALIZE_PROOF_IX_DISCM: [u8; 8] = [165, 188, 188, 88, 25, 248, 86, 93];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeProofIxArgs {
    pub pool_bump: u8,
    pub merkle_root: [u8; 32],
    pub commitment: Pubkey,
}
#[derive(Clone, Debug, PartialEq)]
pub struct InitializeProofIxData(pub InitializeProofIxArgs);
impl From<InitializeProofIxArgs> for InitializeProofIxData {
    fn from(args: InitializeProofIxArgs) -> Self {
        Self(args)
    }
}
impl InitializeProofIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != INITIALIZE_PROOF_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        INITIALIZE_PROOF_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(InitializeProofIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&INITIALIZE_PROOF_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn initialize_proof_ix_with_program_id(
    program_id: Pubkey,
    keys: InitializeProofKeys,
    args: InitializeProofIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; INITIALIZE_PROOF_IX_ACCOUNTS_LEN] = keys.into();
    let data: InitializeProofIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn initialize_proof_ix(
    keys: InitializeProofKeys,
    args: InitializeProofIxArgs,
) -> std::io::Result<Instruction> {
    initialize_proof_ix_with_program_id(crate::ID, keys, args)
}
pub fn initialize_proof_invoke_with_program_id(
    program_id: Pubkey,
    accounts: InitializeProofAccounts<'_, '_>,
    args: InitializeProofIxArgs,
) -> ProgramResult {
    let keys: InitializeProofKeys = accounts.into();
    let ix = initialize_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn initialize_proof_invoke(
    accounts: InitializeProofAccounts<'_, '_>,
    args: InitializeProofIxArgs,
) -> ProgramResult {
    initialize_proof_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn initialize_proof_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: InitializeProofAccounts<'_, '_>,
    args: InitializeProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: InitializeProofKeys = accounts.into();
    let ix = initialize_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn initialize_proof_invoke_signed(
    accounts: InitializeProofAccounts<'_, '_>,
    args: InitializeProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    initialize_proof_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn initialize_proof_verify_account_keys(
    accounts: InitializeProofAccounts<'_, '_>,
    keys: InitializeProofKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.proof.key, keys.proof),
        (*accounts.authority.key, keys.authority),
        (*accounts.payer.key, keys.payer),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.rent.key, keys.rent),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn initialize_proof_verify_writable_privileges<'me, 'info>(
    accounts: InitializeProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.proof, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn initialize_proof_verify_signer_privileges<'me, 'info>(
    accounts: InitializeProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn initialize_proof_verify_account_privileges<'me, 'info>(
    accounts: InitializeProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    initialize_proof_verify_writable_privileges(accounts)?;
    initialize_proof_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const UPLOAD_PROOF_IX_ACCOUNTS_LEN: usize = 4;
#[derive(Copy, Clone, Debug)]
pub struct UploadProofAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub proof: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UploadProofKeys {
    pub pool: Pubkey,
    pub proof: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
}
impl From<UploadProofAccounts<'_, '_>> for UploadProofKeys {
    fn from(accounts: UploadProofAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            proof: *accounts.proof.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
        }
    }
}
impl From<UploadProofKeys> for [AccountMeta; UPLOAD_PROOF_IX_ACCOUNTS_LEN] {
    fn from(keys: UploadProofKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.proof,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; UPLOAD_PROOF_IX_ACCOUNTS_LEN]> for UploadProofKeys {
    fn from(pubkeys: [Pubkey; UPLOAD_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            proof: pubkeys[1],
            authority: pubkeys[2],
            payer: pubkeys[3],
        }
    }
}
impl<'info> From<UploadProofAccounts<'_, 'info>>
for [AccountInfo<'info>; UPLOAD_PROOF_IX_ACCOUNTS_LEN] {
    fn from(accounts: UploadProofAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.proof.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; UPLOAD_PROOF_IX_ACCOUNTS_LEN]>
for UploadProofAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; UPLOAD_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            proof: &arr[1],
            authority: &arr[2],
            payer: &arr[3],
        }
    }
}
pub const UPLOAD_PROOF_IX_DISCM: [u8; 8] = [57, 235, 171, 213, 237, 91, 79, 2];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UploadProofIxArgs {
    pub pool_bump: u8,
    pub proof_bump: u8,
    pub current_size: u8,
    pub data_size: u8,
    pub data: Vec<[u8; 32]>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct UploadProofIxData(pub UploadProofIxArgs);
impl From<UploadProofIxArgs> for UploadProofIxData {
    fn from(args: UploadProofIxArgs) -> Self {
        Self(args)
    }
}
impl UploadProofIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != UPLOAD_PROOF_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        UPLOAD_PROOF_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(UploadProofIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&UPLOAD_PROOF_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn upload_proof_ix_with_program_id(
    program_id: Pubkey,
    keys: UploadProofKeys,
    args: UploadProofIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; UPLOAD_PROOF_IX_ACCOUNTS_LEN] = keys.into();
    let data: UploadProofIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn upload_proof_ix(
    keys: UploadProofKeys,
    args: UploadProofIxArgs,
) -> std::io::Result<Instruction> {
    upload_proof_ix_with_program_id(crate::ID, keys, args)
}
pub fn upload_proof_invoke_with_program_id(
    program_id: Pubkey,
    accounts: UploadProofAccounts<'_, '_>,
    args: UploadProofIxArgs,
) -> ProgramResult {
    let keys: UploadProofKeys = accounts.into();
    let ix = upload_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn upload_proof_invoke(
    accounts: UploadProofAccounts<'_, '_>,
    args: UploadProofIxArgs,
) -> ProgramResult {
    upload_proof_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn upload_proof_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: UploadProofAccounts<'_, '_>,
    args: UploadProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: UploadProofKeys = accounts.into();
    let ix = upload_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn upload_proof_invoke_signed(
    accounts: UploadProofAccounts<'_, '_>,
    args: UploadProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    upload_proof_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn upload_proof_verify_account_keys(
    accounts: UploadProofAccounts<'_, '_>,
    keys: UploadProofKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.proof.key, keys.proof),
        (*accounts.authority.key, keys.authority),
        (*accounts.payer.key, keys.payer),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn upload_proof_verify_writable_privileges<'me, 'info>(
    accounts: UploadProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.proof, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn upload_proof_verify_signer_privileges<'me, 'info>(
    accounts: UploadProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn upload_proof_verify_account_privileges<'me, 'info>(
    accounts: UploadProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    upload_proof_verify_writable_privileges(accounts)?;
    upload_proof_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const VERIFY_PROOF_IX_ACCOUNTS_LEN: usize = 4;
#[derive(Copy, Clone, Debug)]
pub struct VerifyProofAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub proof: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VerifyProofKeys {
    pub pool: Pubkey,
    pub proof: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
}
impl From<VerifyProofAccounts<'_, '_>> for VerifyProofKeys {
    fn from(accounts: VerifyProofAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            proof: *accounts.proof.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
        }
    }
}
impl From<VerifyProofKeys> for [AccountMeta; VERIFY_PROOF_IX_ACCOUNTS_LEN] {
    fn from(keys: VerifyProofKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.proof,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; VERIFY_PROOF_IX_ACCOUNTS_LEN]> for VerifyProofKeys {
    fn from(pubkeys: [Pubkey; VERIFY_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            proof: pubkeys[1],
            authority: pubkeys[2],
            payer: pubkeys[3],
        }
    }
}
impl<'info> From<VerifyProofAccounts<'_, 'info>>
for [AccountInfo<'info>; VERIFY_PROOF_IX_ACCOUNTS_LEN] {
    fn from(accounts: VerifyProofAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.proof.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; VERIFY_PROOF_IX_ACCOUNTS_LEN]>
for VerifyProofAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; VERIFY_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            proof: &arr[1],
            authority: &arr[2],
            payer: &arr[3],
        }
    }
}
pub const VERIFY_PROOF_IX_DISCM: [u8; 8] = [217, 211, 191, 110, 144, 13, 186, 98];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerifyProofIxArgs {
    pub pool_bump: u8,
    pub proof_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct VerifyProofIxData(pub VerifyProofIxArgs);
impl From<VerifyProofIxArgs> for VerifyProofIxData {
    fn from(args: VerifyProofIxArgs) -> Self {
        Self(args)
    }
}
impl VerifyProofIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != VERIFY_PROOF_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        VERIFY_PROOF_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(VerifyProofIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&VERIFY_PROOF_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn verify_proof_ix_with_program_id(
    program_id: Pubkey,
    keys: VerifyProofKeys,
    args: VerifyProofIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; VERIFY_PROOF_IX_ACCOUNTS_LEN] = keys.into();
    let data: VerifyProofIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn verify_proof_ix(
    keys: VerifyProofKeys,
    args: VerifyProofIxArgs,
) -> std::io::Result<Instruction> {
    verify_proof_ix_with_program_id(crate::ID, keys, args)
}
pub fn verify_proof_invoke_with_program_id(
    program_id: Pubkey,
    accounts: VerifyProofAccounts<'_, '_>,
    args: VerifyProofIxArgs,
) -> ProgramResult {
    let keys: VerifyProofKeys = accounts.into();
    let ix = verify_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn verify_proof_invoke(
    accounts: VerifyProofAccounts<'_, '_>,
    args: VerifyProofIxArgs,
) -> ProgramResult {
    verify_proof_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn verify_proof_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: VerifyProofAccounts<'_, '_>,
    args: VerifyProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: VerifyProofKeys = accounts.into();
    let ix = verify_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn verify_proof_invoke_signed(
    accounts: VerifyProofAccounts<'_, '_>,
    args: VerifyProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    verify_proof_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn verify_proof_verify_account_keys(
    accounts: VerifyProofAccounts<'_, '_>,
    keys: VerifyProofKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.proof.key, keys.proof),
        (*accounts.authority.key, keys.authority),
        (*accounts.payer.key, keys.payer),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn verify_proof_verify_writable_privileges<'me, 'info>(
    accounts: VerifyProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.proof, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn verify_proof_verify_signer_privileges<'me, 'info>(
    accounts: VerifyProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn verify_proof_verify_account_privileges<'me, 'info>(
    accounts: VerifyProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    verify_proof_verify_writable_privileges(accounts)?;
    verify_proof_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CLOSE_PROOF_IX_ACCOUNTS_LEN: usize = 6;
#[derive(Copy, Clone, Debug)]
pub struct CloseProofAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub proof: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CloseProofKeys {
    pub pool: Pubkey,
    pub proof: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<CloseProofAccounts<'_, '_>> for CloseProofKeys {
    fn from(accounts: CloseProofAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            proof: *accounts.proof.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<CloseProofKeys> for [AccountMeta; CLOSE_PROOF_IX_ACCOUNTS_LEN] {
    fn from(keys: CloseProofKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.proof,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
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
            AccountMeta {
                pubkey: keys.rent,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}
impl From<[Pubkey; CLOSE_PROOF_IX_ACCOUNTS_LEN]> for CloseProofKeys {
    fn from(pubkeys: [Pubkey; CLOSE_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            proof: pubkeys[1],
            authority: pubkeys[2],
            payer: pubkeys[3],
            system_program: pubkeys[4],
            rent: pubkeys[5],
        }
    }
}
impl<'info> From<CloseProofAccounts<'_, 'info>>
for [AccountInfo<'info>; CLOSE_PROOF_IX_ACCOUNTS_LEN] {
    fn from(accounts: CloseProofAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.proof.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CLOSE_PROOF_IX_ACCOUNTS_LEN]>
for CloseProofAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; CLOSE_PROOF_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            proof: &arr[1],
            authority: &arr[2],
            payer: &arr[3],
            system_program: &arr[4],
            rent: &arr[5],
        }
    }
}
pub const CLOSE_PROOF_IX_DISCM: [u8; 8] = [64, 76, 168, 8, 126, 109, 164, 179];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CloseProofIxArgs {
    pub pool_bump: u8,
    pub proof_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CloseProofIxData(pub CloseProofIxArgs);
impl From<CloseProofIxArgs> for CloseProofIxData {
    fn from(args: CloseProofIxArgs) -> Self {
        Self(args)
    }
}
impl CloseProofIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CLOSE_PROOF_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        CLOSE_PROOF_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(CloseProofIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CLOSE_PROOF_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn close_proof_ix_with_program_id(
    program_id: Pubkey,
    keys: CloseProofKeys,
    args: CloseProofIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CLOSE_PROOF_IX_ACCOUNTS_LEN] = keys.into();
    let data: CloseProofIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn close_proof_ix(
    keys: CloseProofKeys,
    args: CloseProofIxArgs,
) -> std::io::Result<Instruction> {
    close_proof_ix_with_program_id(crate::ID, keys, args)
}
pub fn close_proof_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CloseProofAccounts<'_, '_>,
    args: CloseProofIxArgs,
) -> ProgramResult {
    let keys: CloseProofKeys = accounts.into();
    let ix = close_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn close_proof_invoke(
    accounts: CloseProofAccounts<'_, '_>,
    args: CloseProofIxArgs,
) -> ProgramResult {
    close_proof_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn close_proof_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CloseProofAccounts<'_, '_>,
    args: CloseProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CloseProofKeys = accounts.into();
    let ix = close_proof_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn close_proof_invoke_signed(
    accounts: CloseProofAccounts<'_, '_>,
    args: CloseProofIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    close_proof_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn close_proof_verify_account_keys(
    accounts: CloseProofAccounts<'_, '_>,
    keys: CloseProofKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.proof.key, keys.proof),
        (*accounts.authority.key, keys.authority),
        (*accounts.payer.key, keys.payer),
        (*accounts.system_program.key, keys.system_program),
        (*accounts.rent.key, keys.rent),
    ] {
        if actual != expected {
            return Err((actual, expected));
        }
    }
    Ok(())
}
pub fn close_proof_verify_writable_privileges<'me, 'info>(
    accounts: CloseProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.proof, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn close_proof_verify_signer_privileges<'me, 'info>(
    accounts: CloseProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn close_proof_verify_account_privileges<'me, 'info>(
    accounts: CloseProofAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    close_proof_verify_writable_privileges(accounts)?;
    close_proof_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN: usize = 9;
#[derive(Copy, Clone, Debug)]
pub struct OpenTokenAccountAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub proof: &'me AccountInfo<'info>,
    pub commitment_vault: &'me AccountInfo<'info>,
    pub mint: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
    pub rent: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OpenTokenAccountKeys {
    pub pool: Pubkey,
    pub proof: Pubkey,
    pub commitment_vault: Pubkey,
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl From<OpenTokenAccountAccounts<'_, '_>> for OpenTokenAccountKeys {
    fn from(accounts: OpenTokenAccountAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            proof: *accounts.proof.key,
            commitment_vault: *accounts.commitment_vault.key,
            mint: *accounts.mint.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
            rent: *accounts.rent.key,
        }
    }
}
impl From<OpenTokenAccountKeys> for [AccountMeta; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(keys: OpenTokenAccountKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.proof,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.commitment_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]> for OpenTokenAccountKeys {
    fn from(pubkeys: [Pubkey; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            proof: pubkeys[1],
            commitment_vault: pubkeys[2],
            mint: pubkeys[3],
            authority: pubkeys[4],
            payer: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
            rent: pubkeys[8],
        }
    }
}
impl<'info> From<OpenTokenAccountAccounts<'_, 'info>>
for [AccountInfo<'info>; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(accounts: OpenTokenAccountAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.proof.clone(),
            accounts.commitment_vault.clone(),
            accounts.mint.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
            accounts.rent.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]>
for OpenTokenAccountAccounts<'me, 'info> {
    fn from(arr: &'me [AccountInfo<'info>; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: &arr[0],
            proof: &arr[1],
            commitment_vault: &arr[2],
            mint: &arr[3],
            authority: &arr[4],
            payer: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
            rent: &arr[8],
        }
    }
}
pub const OPEN_TOKEN_ACCOUNT_IX_DISCM: [u8; 8] = [77, 240, 240, 35, 150, 89, 234, 157];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpenTokenAccountIxArgs {
    pub pool_bump: u8,
    pub proof_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct OpenTokenAccountIxData(pub OpenTokenAccountIxArgs);
impl From<OpenTokenAccountIxArgs> for OpenTokenAccountIxData {
    fn from(args: OpenTokenAccountIxArgs) -> Self {
        Self(args)
    }
}
impl OpenTokenAccountIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != OPEN_TOKEN_ACCOUNT_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        OPEN_TOKEN_ACCOUNT_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(OpenTokenAccountIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&OPEN_TOKEN_ACCOUNT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn open_token_account_ix_with_program_id(
    program_id: Pubkey,
    keys: OpenTokenAccountKeys,
    args: OpenTokenAccountIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; OPEN_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN] = keys.into();
    let data: OpenTokenAccountIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn open_token_account_ix(
    keys: OpenTokenAccountKeys,
    args: OpenTokenAccountIxArgs,
) -> std::io::Result<Instruction> {
    open_token_account_ix_with_program_id(crate::ID, keys, args)
}
pub fn open_token_account_invoke_with_program_id(
    program_id: Pubkey,
    accounts: OpenTokenAccountAccounts<'_, '_>,
    args: OpenTokenAccountIxArgs,
) -> ProgramResult {
    let keys: OpenTokenAccountKeys = accounts.into();
    let ix = open_token_account_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn open_token_account_invoke(
    accounts: OpenTokenAccountAccounts<'_, '_>,
    args: OpenTokenAccountIxArgs,
) -> ProgramResult {
    open_token_account_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn open_token_account_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: OpenTokenAccountAccounts<'_, '_>,
    args: OpenTokenAccountIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: OpenTokenAccountKeys = accounts.into();
    let ix = open_token_account_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn open_token_account_invoke_signed(
    accounts: OpenTokenAccountAccounts<'_, '_>,
    args: OpenTokenAccountIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    open_token_account_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn open_token_account_verify_account_keys(
    accounts: OpenTokenAccountAccounts<'_, '_>,
    keys: OpenTokenAccountKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.proof.key, keys.proof),
        (*accounts.commitment_vault.key, keys.commitment_vault),
        (*accounts.mint.key, keys.mint),
        (*accounts.authority.key, keys.authority),
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
pub fn open_token_account_verify_writable_privileges<'me, 'info>(
    accounts: OpenTokenAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [accounts.commitment_vault, accounts.payer] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn open_token_account_verify_signer_privileges<'me, 'info>(
    accounts: OpenTokenAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn open_token_account_verify_account_privileges<'me, 'info>(
    accounts: OpenTokenAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    open_token_account_verify_writable_privileges(accounts)?;
    open_token_account_verify_signer_privileges(accounts)?;
    Ok(())
}
pub const CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN: usize = 8;
#[derive(Copy, Clone, Debug)]
pub struct CloseTokenAccountAccounts<'me, 'info> {
    pub pool: &'me AccountInfo<'info>,
    pub proof: &'me AccountInfo<'info>,
    pub commitment_vault: &'me AccountInfo<'info>,
    pub pool_vault: &'me AccountInfo<'info>,
    pub authority: &'me AccountInfo<'info>,
    pub payer: &'me AccountInfo<'info>,
    pub token_program: &'me AccountInfo<'info>,
    pub system_program: &'me AccountInfo<'info>,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CloseTokenAccountKeys {
    pub pool: Pubkey,
    pub proof: Pubkey,
    pub commitment_vault: Pubkey,
    pub pool_vault: Pubkey,
    pub authority: Pubkey,
    pub payer: Pubkey,
    pub token_program: Pubkey,
    pub system_program: Pubkey,
}
impl From<CloseTokenAccountAccounts<'_, '_>> for CloseTokenAccountKeys {
    fn from(accounts: CloseTokenAccountAccounts) -> Self {
        Self {
            pool: *accounts.pool.key,
            proof: *accounts.proof.key,
            commitment_vault: *accounts.commitment_vault.key,
            pool_vault: *accounts.pool_vault.key,
            authority: *accounts.authority.key,
            payer: *accounts.payer.key,
            token_program: *accounts.token_program.key,
            system_program: *accounts.system_program.key,
        }
    }
}
impl From<CloseTokenAccountKeys> for [AccountMeta; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(keys: CloseTokenAccountKeys) -> Self {
        [
            AccountMeta {
                pubkey: keys.pool,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.proof,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: keys.commitment_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.pool_vault,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: keys.authority,
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
impl From<[Pubkey; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]> for CloseTokenAccountKeys {
    fn from(pubkeys: [Pubkey; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]) -> Self {
        Self {
            pool: pubkeys[0],
            proof: pubkeys[1],
            commitment_vault: pubkeys[2],
            pool_vault: pubkeys[3],
            authority: pubkeys[4],
            payer: pubkeys[5],
            token_program: pubkeys[6],
            system_program: pubkeys[7],
        }
    }
}
impl<'info> From<CloseTokenAccountAccounts<'_, 'info>>
for [AccountInfo<'info>; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN] {
    fn from(accounts: CloseTokenAccountAccounts<'_, 'info>) -> Self {
        [
            accounts.pool.clone(),
            accounts.proof.clone(),
            accounts.commitment_vault.clone(),
            accounts.pool_vault.clone(),
            accounts.authority.clone(),
            accounts.payer.clone(),
            accounts.token_program.clone(),
            accounts.system_program.clone(),
        ]
    }
}
impl<'me, 'info> From<&'me [AccountInfo<'info>; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN]>
for CloseTokenAccountAccounts<'me, 'info> {
    fn from(
        arr: &'me [AccountInfo<'info>; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN],
    ) -> Self {
        Self {
            pool: &arr[0],
            proof: &arr[1],
            commitment_vault: &arr[2],
            pool_vault: &arr[3],
            authority: &arr[4],
            payer: &arr[5],
            token_program: &arr[6],
            system_program: &arr[7],
        }
    }
}
pub const CLOSE_TOKEN_ACCOUNT_IX_DISCM: [u8; 8] = [132, 172, 24, 60, 100, 156, 135, 97];
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CloseTokenAccountIxArgs {
    pub pool_bump: u8,
    pub proof_bump: u8,
    pub vault_bump: u8,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CloseTokenAccountIxData(pub CloseTokenAccountIxArgs);
impl From<CloseTokenAccountIxArgs> for CloseTokenAccountIxData {
    fn from(args: CloseTokenAccountIxArgs) -> Self {
        Self(args)
    }
}
impl CloseTokenAccountIxData {
    pub fn deserialize(buf: &[u8]) -> std::io::Result<Self> {
        let mut reader = buf;
        let mut maybe_discm = [0u8; 8];
        reader.read_exact(&mut maybe_discm)?;
        if maybe_discm != CLOSE_TOKEN_ACCOUNT_IX_DISCM {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "discm does not match. Expected: {:?}. Received: {:?}",
                        CLOSE_TOKEN_ACCOUNT_IX_DISCM, maybe_discm
                    ),
                ),
            );
        }
        Ok(Self(CloseTokenAccountIxArgs::deserialize(&mut reader)?))
    }
    pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> std::io::Result<()> {
        writer.write_all(&CLOSE_TOKEN_ACCOUNT_IX_DISCM)?;
        self.0.serialize(&mut writer)
    }
    pub fn try_to_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut data = Vec::new();
        self.serialize(&mut data)?;
        Ok(data)
    }
}
pub fn close_token_account_ix_with_program_id(
    program_id: Pubkey,
    keys: CloseTokenAccountKeys,
    args: CloseTokenAccountIxArgs,
) -> std::io::Result<Instruction> {
    let metas: [AccountMeta; CLOSE_TOKEN_ACCOUNT_IX_ACCOUNTS_LEN] = keys.into();
    let data: CloseTokenAccountIxData = args.into();
    Ok(Instruction {
        program_id,
        accounts: Vec::from(metas),
        data: data.try_to_vec()?,
    })
}
pub fn close_token_account_ix(
    keys: CloseTokenAccountKeys,
    args: CloseTokenAccountIxArgs,
) -> std::io::Result<Instruction> {
    close_token_account_ix_with_program_id(crate::ID, keys, args)
}
pub fn close_token_account_invoke_with_program_id(
    program_id: Pubkey,
    accounts: CloseTokenAccountAccounts<'_, '_>,
    args: CloseTokenAccountIxArgs,
) -> ProgramResult {
    let keys: CloseTokenAccountKeys = accounts.into();
    let ix = close_token_account_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction(&ix, accounts)
}
pub fn close_token_account_invoke(
    accounts: CloseTokenAccountAccounts<'_, '_>,
    args: CloseTokenAccountIxArgs,
) -> ProgramResult {
    close_token_account_invoke_with_program_id(crate::ID, accounts, args)
}
pub fn close_token_account_invoke_signed_with_program_id(
    program_id: Pubkey,
    accounts: CloseTokenAccountAccounts<'_, '_>,
    args: CloseTokenAccountIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    let keys: CloseTokenAccountKeys = accounts.into();
    let ix = close_token_account_ix_with_program_id(program_id, keys, args)?;
    invoke_instruction_signed(&ix, accounts, seeds)
}
pub fn close_token_account_invoke_signed(
    accounts: CloseTokenAccountAccounts<'_, '_>,
    args: CloseTokenAccountIxArgs,
    seeds: &[&[&[u8]]],
) -> ProgramResult {
    close_token_account_invoke_signed_with_program_id(crate::ID, accounts, args, seeds)
}
pub fn close_token_account_verify_account_keys(
    accounts: CloseTokenAccountAccounts<'_, '_>,
    keys: CloseTokenAccountKeys,
) -> Result<(), (Pubkey, Pubkey)> {
    for (actual, expected) in [
        (*accounts.pool.key, keys.pool),
        (*accounts.proof.key, keys.proof),
        (*accounts.commitment_vault.key, keys.commitment_vault),
        (*accounts.pool_vault.key, keys.pool_vault),
        (*accounts.authority.key, keys.authority),
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
pub fn close_token_account_verify_writable_privileges<'me, 'info>(
    accounts: CloseTokenAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_writable in [
        accounts.pool,
        accounts.commitment_vault,
        accounts.pool_vault,
        accounts.payer,
    ] {
        if !should_be_writable.is_writable {
            return Err((should_be_writable, ProgramError::InvalidAccountData));
        }
    }
    Ok(())
}
pub fn close_token_account_verify_signer_privileges<'me, 'info>(
    accounts: CloseTokenAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    for should_be_signer in [accounts.authority, accounts.payer] {
        if !should_be_signer.is_signer {
            return Err((should_be_signer, ProgramError::MissingRequiredSignature));
        }
    }
    Ok(())
}
pub fn close_token_account_verify_account_privileges<'me, 'info>(
    accounts: CloseTokenAccountAccounts<'me, 'info>,
) -> Result<(), (&'me AccountInfo<'info>, ProgramError)> {
    close_token_account_verify_writable_privileges(accounts)?;
    close_token_account_verify_signer_privileges(accounts)?;
    Ok(())
}
