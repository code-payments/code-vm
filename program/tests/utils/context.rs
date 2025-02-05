#![cfg(test)]
use super::*;

use steel::*;
use litesvm::{types::TransactionResult, LiteSVM};
use solana_sdk::{signature::{Keypair, Signer }, transaction::Transaction};
use code_vm_api::prelude::*;

pub struct TestContext {
    pub svm: LiteSVM,
    pub payer: Keypair,
    pub mint_owner: Keypair,
    pub mint_pk: Pubkey,
    pub vm_address: Pubkey,
    pub vm: CodeVmAccount,
}

impl TestContext {
    pub fn new(lock_duration: u8) -> Self {
        let (svm, payer, mint_owner, mint_pk, vm_address) =
            setup_svm_with_payer_and_vm(lock_duration);

        let vm = get_vm_account(&svm, vm_address);

        Self {
            svm,
            payer,
            mint_owner,
            mint_pk,
            vm_address,
            vm,
        }
    }

    pub fn create_memory(&mut self, capacity: usize, account_size: usize, name: &str) -> Pubkey {
        let (mem, _) = create_and_resize_memory(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            capacity,
            account_size,
            name,
        );
        mem
    }

    pub fn create_relay(&mut self, name: &str, mint_tokens: u64) -> RelayContext {
        let relay = RelayContext::new(self, name);
        mint_to(
            &mut self.svm,
            &self.payer,
            &self.mint_pk,
            &self.mint_owner,
            &relay.relay.treasury.vault,
            mint_tokens,
        )
        .unwrap();
        relay
    }

    pub fn create_timelock_account(
        &mut self,
        mem_b: Pubkey,
        index: u16,
    ) -> TimelockAccountContext {
        let (account, key) = create_timelock(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            mem_b,
            index,
        );
        TimelockAccountContext {
            account,
            key,
            index,
        }
    }

    pub fn create_durable_nonce_account(
        &mut self,
        mem_a: Pubkey,
        index: u16,
    ) -> DurableNonceContext {
        let (account, key) = create_durable_nonce(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            mem_a,
            index,
        );
        DurableNonceContext {
            account,
            key,
            index,
        }
    }

    pub fn deposit_tokens_to_timelock(
        &mut self,
        mem_b: Pubkey,
        vta_ctx: &TimelockAccountContext,
        amount: u64,
    ) -> TransactionResult {
        let depositor = vta_ctx.key.pubkey();
        let (deposit_pda, deposit_pda_bump) =
            find_timelock_deposit_pda(&self.vm_address, &depositor);
        let deposit_ata = create_ata(&mut self.svm, &self.payer, &self.mint_pk, &deposit_pda);
        mint_to(
            &mut self.svm,
            &self.payer,
            &self.mint_pk,
            &self.mint_owner,
            &deposit_ata,
            amount,
        )
        .unwrap();

        tx_deposit(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            mem_b,
            depositor,
            deposit_pda,
            deposit_ata,
            self.vm.omnibus.vault,
            vta_ctx.index,
            amount,
            deposit_pda_bump,
        )
    }

    pub fn exec_opcode(
        &mut self,
        mems: [Option<Pubkey>; 4],
        vm_omnibus: Option<Pubkey>,
        relay: Option<Pubkey>,
        relay_vault: Option<Pubkey>,
        external_address: Option<Pubkey>,
        token_program: Option<Pubkey>,
        data: Vec<u8>,
        mem_indices: Vec<u16>,
        mem_banks: Vec<u8>,
    ) -> TransactionResult {
        let opcode = data[0];
        let data = data[1..].to_vec();

        tx_exec_opcode(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            mems[0],
            mems[1],
            mems[2],
            mems[3],
            vm_omnibus,
            relay,
            relay_vault,
            external_address,
            token_program,
            opcode,
            mem_indices,
            mem_banks,
            data,
        )
    }

    pub fn exec_relay_op(
        &mut self,
        relay_ctx: &RelayContext,
        mems: [Option<Pubkey>; 4],
        mem_indices: Vec<u16>,
        mem_banks: Vec<u8>,
        data: Vec<u8>,
    ) -> TransactionResult {
        let opcode = data[0];
        let data = data[1..].to_vec();

        tx_exec_opcode(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            mems[0],
            mems[1],
            mems[2],
            mems[3],
            Some(self.vm.omnibus.vault),
            Some(relay_ctx.relay_address),
            Some(relay_ctx.relay.treasury.vault),
            None,
            Some(spl_token::id()),
            opcode,
            mem_indices,
            mem_banks,
            data,
        )
    }

    pub fn exec_conditional_transfer(
        &mut self,
        external_address: Pubkey,
        mems: [Option<Pubkey>; 4],
        mem_indices: Vec<u16>,
        mem_banks: Vec<u8>,
        data: Vec<u8>,
    ) -> TransactionResult {
        let opcode = data[0];
        let data = data[1..].to_vec();

        tx_exec_opcode(
            &mut self.svm,
            &self.payer,
            self.vm_address,
            mems[0],
            mems[1],
            mems[2],
            mems[3],
            Some(self.vm.omnibus.vault),
            None,
            None,
            Some(external_address),
            Some(spl_token::id()),
            opcode,
            mem_indices,
            mem_banks,
            data,
        )
    }

    pub fn ix_send(
        &mut self,
        ix: &[Instruction],
    ) -> TransactionResult {
        let payer_pk = self.payer.pubkey();
        let blockhash = self.svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(
            ix,
            Some(&payer_pk),
            &[&self.payer],
            blockhash
        );

        send_tx(&mut self.svm, tx)
    }

    pub fn get_exec_ix(
        &mut self,
        mems: [Option<Pubkey>; 4],
        vm_omnibus: Option<Pubkey>,
        relay: Option<Pubkey>,
        relay_vault: Option<Pubkey>,
        external_address: Option<Pubkey>,
        token_program: Option<Pubkey>,
        data: Vec<u8>,
        mem_indices: Vec<u16>,
        mem_banks: Vec<u8>,
    ) -> Instruction {
        let opcode = data[0];
        let data = data[1..].to_vec();

        vm_exec(
            self.payer.pubkey(),
            self.vm_address,
            mems[0],
            mems[1],
            mems[2],
            mems[3],
            vm_omnibus,
            relay,
            relay_vault,
            external_address,
            token_program,
            opcode,
            mem_indices,
            mem_banks,
            data,
        )
    }

    pub fn get_virtual_timelock(&self, mem: Pubkey, index: u16) -> VirtualTimelockAccount {
        get_virtual_timelock(&self.svm, mem, index)
    }

    pub fn has_virtual_account(&self, mem: Pubkey, index: u16) -> bool {
        has_virtual_account(&self.svm, mem, index)
    }

    pub fn get_ata_balance(&self, ata: Pubkey) -> u64 {
        get_ata_balance(&self.svm, &ata)
    }
}

pub struct RelayContext {
    pub relay_address: Pubkey,
    pub relay: RelayAccount,
}

impl RelayContext {
    pub fn new(ctx: &mut TestContext, name: &str) -> Self {
        let (relay_address, _) = create_relay_account(
            &mut ctx.svm,
            &ctx.payer,
            &ctx.mint_pk,
            ctx.vm_address,
            name,
        );
        let relay = get_relay_account(&ctx.svm, relay_address);
        Self {
            relay_address,
            relay,
        }
    }
}

pub struct TimelockAccountContext {
    pub account: VirtualTimelockAccount,
    pub key: Keypair,
    pub index: u16,
}

pub struct DurableNonceContext {
    pub account: VirtualDurableNonce,
    pub key: Keypair,
    pub index: u16,
}