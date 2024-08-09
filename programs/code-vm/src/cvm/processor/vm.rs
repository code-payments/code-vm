use anchor_lang::prelude::*;
use std::cell::RefMut;

use crate::{
    cvm::{
        CodeVmAccount, 
        IndexedMemory, 
        MemoryAccount, 
        MemoryAccountWithData, 
        TimelockState, 
        UnlockStateAccount, 
        VirtualAccount,
        VirtualTimelockAccount
    }, 
    error::CodeVmError,
    types::{Hash, Signature, MerkleTree},
    utils,
};

pub struct CodeVm<'mem> {
    address: Pubkey,
    state: CodeVmAccount,

    mem_a: Option<RefMut<'mem, dyn IndexedMemory>>,
    mem_b: Option<RefMut<'mem, dyn IndexedMemory>>,
    mem_c: Option<RefMut<'mem, dyn IndexedMemory>>,
    mem_d: Option<RefMut<'mem, dyn IndexedMemory>>,
}

impl<'mem> CodeVm<'mem> {

    pub fn new(
        ctx: Box<Account<CodeVmAccount>>,
    ) -> Self {
        let state = ctx.to_owned().into_inner();
        let address = ctx.key();

        Self { 
            address, 
            state, 

            mem_a: None,
            mem_b: None,
            mem_c: None,
            mem_d: None,
        }
    }

    pub fn use_memory(
        &mut self,
        memory: RefMut<'mem, dyn IndexedMemory>,
    ) {
        self.mem_a = Some(memory);
    }

    pub fn try_use_memory_bank(
        &mut self,
        bank: MemoryBank,
        info: &'mem AccountInfo,
    ) -> Result<()> {
        let data = info.try_borrow_mut_data()?;
        let account = MemoryAccount::unpack(data.as_ref())?;

        assert_eq!(account.vm, self.get_address());
        assert_eq!(info.key(), account.get_address());

        self.set_memory(
            bank,
            MemoryAccountWithData::into_indexed_memory(data)
        );

        Ok(())
    }

    fn set_memory(
        &mut self,
        bank: MemoryBank,
        memory: RefMut<'mem, dyn IndexedMemory>,
    ) {
        match bank {
            MemoryBank::A => self.mem_a = Some(memory),
            MemoryBank::B => self.mem_b = Some(memory),
            MemoryBank::C => self.mem_c = Some(memory),
            MemoryBank::D => self.mem_d = Some(memory),
        }
    }

    pub fn has_memory_bank(&self, bank: MemoryBank) -> bool {
        match bank {
            MemoryBank::A => self.mem_a.is_some(),
            MemoryBank::B => self.mem_b.is_some(),
            MemoryBank::C => self.mem_c.is_some(),
            MemoryBank::D => self.mem_d.is_some(),
        }
    }

    fn get_memory(&self, bank: MemoryBank) 
        -> &Option<RefMut<'mem, dyn IndexedMemory>> {
        match bank {
            MemoryBank::A => &self.mem_a,
            MemoryBank::B => &self.mem_b,
            MemoryBank::C => &self.mem_c,
            MemoryBank::D => &self.mem_d,
        }
    }

    fn get_memory_mut(&mut self, bank: MemoryBank) 
        -> &mut Option<RefMut<'mem, dyn IndexedMemory>> {
        match bank {
            MemoryBank::A => &mut self.mem_a,
            MemoryBank::B => &mut self.mem_b,
            MemoryBank::C => &mut self.mem_c,
            MemoryBank::D => &mut self.mem_d,
        }
    }

    pub fn advance_poh(&mut self, val: Hash) -> Hash {
        self.state.poh = utils::hashv(&[
            self.state.poh.as_ref(),
            val.as_ref(),
        ]);
        self.state.poh
    }

    pub fn advance_slot(&mut self) -> u64 {
        self.state.slot += 1;
        self.state.slot
    }

    pub fn is_empty(&self, account_index: u16) -> bool {
        !self.is_allocated(account_index)
    }

    pub fn is_allocated(&self, account_index: u16) -> bool {
        self.is_allocated_using(MemoryBank::A, account_index)
    }

    pub fn read_account(&self, account_index: u16) -> Option<VirtualAccount> {
        self.read_account_using(MemoryBank::A, account_index)
    }

    pub fn try_write_account(&mut self, account_index: u16, va: VirtualAccount) -> Result<()> {
        self.try_write_account_using(MemoryBank::A, account_index, va)
    }

    pub fn try_delete_account(&mut self, account_index: u16) -> Result<()> {
        self.try_delete_account_using(MemoryBank::A, account_index)
    }

    pub fn is_empty_using(&self, bank: MemoryBank, account_index: u16) -> bool {
        !self.is_allocated_using(bank, account_index)
    }

    pub fn is_allocated_using(&self, bank: MemoryBank, account_index: u16) -> bool {
        let mem = self.get_memory(bank);
        assert!(mem.is_some());

        let reader = mem.as_ref().unwrap();
        reader.has_item(account_index)
    }

    pub fn read_account_using(&self, bank: MemoryBank, account_index: u16) -> Option<VirtualAccount> {
        let memory = self.get_memory(bank);
        assert!(memory.is_some());

        let reader = memory.as_ref().unwrap();
        let buf = reader.read_item(account_index);

        match buf {
            Some(data) => {
                match VirtualAccount::unpack(&data) {
                    Ok(account) => Some(account),
                    Err(_) => None
                }
            },
            _ => None,
        }
    }

    pub fn try_write_account_using(&mut self, bank: MemoryBank, account_index: u16, va: VirtualAccount) -> Result<()> {
        let memory = self.get_memory_mut(bank);
        assert!(memory.is_some());

        let reader = memory.as_ref().unwrap();
        let is_allocated = reader.has_item(account_index);

        let writer = memory.as_mut().unwrap();
        if !is_allocated {
            writer.try_alloc_item(account_index, va.get_size())?;
        }
        writer.try_write_item(account_index, va.pack().as_ref())
    }

    pub fn try_delete_account_using(&mut self, bank: MemoryBank, account_index: u16) -> Result<()> {
        let memory = self.get_memory_mut(bank);
        assert!(memory.is_some());

        let writer = memory.as_mut().unwrap();
        writer.try_free_item(account_index)
    }

    pub fn try_compress(
        &self,
        va: VirtualAccount,
        tree: &mut MerkleTree,
        signature: Signature,

    ) -> Result<()> {
        let va_hash = va.get_hash().to_bytes();

        // Verify that the vm_authority actually saw this account as it currently
        // exists on-chain.
        utils::sig_verify(
            self.get_authority().as_ref(), 
            signature.as_ref(), 
            va_hash.as_ref(),
        )?;

        let sig_hash = utils::hashv(&[&signature.to_bytes(), &va_hash]);

        tree.try_insert(sig_hash)
    }

    pub fn try_decompress(
        &self,
        unchecked_va: VirtualAccount,
        tree: &mut MerkleTree,
        proof: Vec<Hash>,
        signature: Signature,

    ) -> Result<()> {
        let va_hash = unchecked_va.get_hash().to_bytes();

        // Verify that the vm_authority actually saw this account as it currently
        // exists on-chain. This is a check that the account was not tampered with
        // between the time the vm_authority saw it and now (despite the merkle
        // proof)
        utils::sig_verify(
            self.get_authority().as_ref(), 
            signature.as_ref(), 
            va_hash.as_ref(),
        )?;

        let sig_hash = utils::hashv(&[&signature.to_bytes(), &va_hash]);

        tree.try_remove(&proof, sig_hash)
    }

    pub fn try_verify_timelock_account<'info>(
        &self,
        unchecked_vta: VirtualTimelockAccount,
        unlock_pda: &AccountInfo<'info>,
        receipt: &AccountInfo<'info>,
    ) -> Result<()> {

        let timelock_address = unchecked_vta.get_timelock_address(
            self.get_mint(),
            self.get_authority(),
            self.get_lock_duration(),
        );

        let unlock_address = unchecked_vta.get_unlock_address(
            timelock_address,
            self.get_address(),
        );

        assert_eq!(unlock_pda.key(), unlock_address);

        // Only locked or waiting-for-timeout accounts can be decompressed!
        // Otherwise, an account owner could be prevented from withdrawing their
        // funds using continuous compress/uncompress cycles.

        match self.try_get_timelock_state(&unchecked_vta, unlock_pda) {
            Ok(TimelockState::Locked | TimelockState::WaitingForTimeout) => (),
            _ => return Err(CodeVmError::InvalidTimelockState.into()),
        }

        let receipt_address = unchecked_vta.get_withdraw_receipt_address(
            unlock_address,
            self.get_address());
        
        assert_eq!(receipt.key(), receipt_address);

        // Check that the receipt account is empty (no data; len == 0)
        assert_eq!(receipt.try_borrow_data().unwrap().len(), 0);

        Ok(())
    }

    pub fn try_get_timelock_state<'info>(
        &self,
        vta: &VirtualTimelockAccount,
        unlock_pda: &AccountInfo<'info>,
    ) -> Result<TimelockState> {

        let timelock_address = vta.get_timelock_address(
            self.get_mint(),
            self.get_authority(),
            self.get_lock_duration()
        );
        let unlock_address = vta.get_unlock_address(
            timelock_address,
            self.get_address()
        );

        assert_eq!(unlock_address, unlock_pda.key());

        let buf = &mut &**unlock_pda.try_borrow_mut_data()?;
        if buf.len() > 0 {
            match UnlockStateAccount::try_deserialize(buf) {
                Ok(timelock) => {
                    assert_eq!(timelock.vm, self.get_address());
                    assert_eq!(timelock.address, timelock_address);
                    assert_eq!(timelock.owner, vta.owner);

                    Ok(timelock.state)
                },
                Err(e) => {
                    msg!("Error deserializing unlock state account: {:?}", e);
                    Err(CodeVmError::InvalidTimelockState.into())
                }
            }
        } else {
            Ok(TimelockState::Locked)
        }
    }

    #[inline]
    pub fn get_address(&self) -> Pubkey {
        self.address
    }

    #[inline]
    pub fn get_authority(&self) -> Pubkey {
        self.state.authority
    }

    #[inline]
    pub fn get_mint(&self) -> Pubkey {
        self.state.mint
    }

    #[inline]
    pub fn get_bump(&self) -> u8 {
        self.state.bump
    }

    #[inline]
    pub fn get_omnibus_bump(&self) -> u8 {
        self.state.omnibus.vault_bump
    }

    #[inline]
    pub fn get_lock_duration(&self) -> u8 {
        self.state.lock_duration
    }

    #[inline]
    pub fn get_current_poh(&self) -> Hash {
        self.state.poh
    }

    #[inline]
    pub fn get_current_slot(&self) -> u64 {
        self.state.slot
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryBank { A, B, C, D, }

impl From<u8> for MemoryBank {
    fn from(val: u8) -> Self {
        match val {
            0 => MemoryBank::A,
            1 => MemoryBank::B,
            2 => MemoryBank::C,
            3 => MemoryBank::D,
            _ => panic!("Invalid memory bank"),
        }
    }
}
