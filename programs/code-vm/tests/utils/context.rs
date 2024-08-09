use std::path::PathBuf;
use anchor_lang::prelude::*;
use solana_sdk::signature::{Keypair, Signer};

use code_vm::{
    program,
    CODE_VM_PREFIX
};

pub struct TestContext {
    lock_duration: u8,
    mint: Keypair,
    owner: Keypair,
}

impl TestContext {
    pub fn new() -> Self {
        let lock_duration = 21u8;
        let mint = Keypair::new();
        let owner = Keypair::new();

        Self {
            owner,
            lock_duration,
            mint,
        }
    }

    pub fn get_mint(&self) -> Pubkey {
        self.mint.pubkey()
    }

    pub fn get_mint_keypair(&self) -> &Keypair {
        &self.mint
    }

    pub fn get_lock_duration(&self) -> u8 {
        self.lock_duration
    }

    pub fn get_signer(&self) -> &Keypair {
        &self.owner
    }

    pub fn get_vm_authority(&self) -> Pubkey {
        self.owner.pubkey()
    }

    fn find_vm_address(&self) -> (Pubkey, u8) {
        let mint = self.mint.pubkey();
        let vm_authority = self.get_vm_authority();
        let lock_duration = self.lock_duration.to_le_bytes();
        let seeds: &[&[u8]] = &[
            CODE_VM_PREFIX.as_bytes(),
            mint.as_ref(),
            vm_authority.as_ref(),
            lock_duration.as_ref(),
        ];

        Pubkey::find_program_address(seeds, &program::CodeVm::id())
    }

    pub fn get_vm_address(&self) -> Pubkey {
        self.find_vm_address().0
    }

    pub fn get_vm_bump(&self) -> u8 {
        self.find_vm_address().1
    }

    fn find_vm_omnibus_address(&self) -> (Pubkey, u8) {
        let mint = self.mint.pubkey();
        let vm_authority = self.get_vm_authority();
        let lock_duration = self.lock_duration.to_le_bytes();

        let seeds: &[&[u8]] = &[
            CODE_VM_PREFIX.as_bytes(),
            b"vm_omnibus",
            mint.as_ref(),
            vm_authority.as_ref(),
            lock_duration.as_ref(),
        ];

        Pubkey::find_program_address(seeds, &program::CodeVm::id())
    }

    pub fn get_vm_omnibus_address(&self) -> Pubkey {
        self.find_vm_omnibus_address().0
    }

    pub fn get_vm_omnibus_bump(&self) -> u8 {
        self.find_vm_omnibus_address().1
    }
}

pub fn program_bytes() -> Vec<u8> {
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("../../target/deploy/code_vm.so");
    std::fs::read(so_path).unwrap()
}