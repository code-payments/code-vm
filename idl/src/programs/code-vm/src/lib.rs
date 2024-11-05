use anchor_lang::prelude::*;

mod types;
mod state;
mod args;
mod instructions;

use types::*;
use args::*;
use instructions::*;

declare_id!("vmT2hAx4N2U6DyjYxgQHER4VGC8tHJCfHNsSepBKCJZ");

#[program]
pub mod code_vm {
    use super::*;

    pub fn init_vm(_ctx: Context<InitVm>, _data: InitVmArgs) -> Result<()> {
        Ok(())
    }
    pub fn init_memory(_ctx: Context<InitMemory>, _data: InitMemoryArgs) -> Result<()> {
        Ok(())
    }

    pub fn init_storage(_ctx: Context<InitStorage>, _data: InitStorageArgs) -> Result<()> {
        Ok(())
    }

    pub fn init_relay(_ctx: Context<InitRelay>, _data: InitRelayArgs) -> Result<()> {
        Ok(())
    }

    pub fn init_nonce(_ctx: Context<InitNonce>, _data: InitNonceArgs) -> Result<()> {
        Ok(())
    }

    pub fn init_timelock(_ctx: Context<InitTimelock>, _data: InitTimelockArgs) -> Result<()> {
        Ok(())
    }

    pub fn init_unlock(_ctx: Context<InitUnlock>, _data: InitUnlockArgs) -> Result<()> {
        Ok(())
    }

    pub fn exec(_ctx: Context<Exec>, _data: ExecArgs) -> Result<()> {
        Ok(())
    }

    pub fn compress(_ctx: Context<Compress>, _data: CompressArgs) -> Result<()> {
        Ok(())
    }

    pub fn decompress(_ctx: Context<Decompress>, _data: DecompressArgs) -> Result<()> {
        Ok(())
    }

    pub fn resize_memory(_ctx: Context<ResizeMemory>, _data: ResizeMemoryArgs) -> Result<()> {
        Ok(())
    }

    pub fn snapshot(_ctx: Context<Snapshot>, _data: SnapshotArgs) -> Result<()> {
        Ok(())
    }

    pub fn deposit(_ctx: Context<Deposit>, _data: DepositArgs) -> Result<()> {
        Ok(())
    }

    pub fn withdraw(_ctx: Context<Withdraw>, _data: WithdrawArgs) -> Result<()> {
        Ok(())
    }

    pub fn unlock(_ctx: Context<Unlock>, _data: UnlockArgs) -> Result<()> {
        Ok(())
    }
}
