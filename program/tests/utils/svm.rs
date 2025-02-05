#![cfg(test)]
use std::path::PathBuf;
use code_vm_api::prelude::CodeInstruction;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use litesvm::{types::{FailedTransactionMetadata, TransactionMetadata, TransactionResult}, LiteSVM};
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo, spl_token::{state::Account}, get_spl_account};
use pretty_hex::*;

pub fn program_bytes() -> Vec<u8> {
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("../target/deploy/code_vm_program.so");
    std::fs::read(so_path).unwrap()
}

pub fn setup_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    svm.add_program(code_vm_api::ID, &program_bytes());
    svm
}

pub fn send_tx(svm: &mut LiteSVM, tx: Transaction) -> TransactionResult {
    let res = svm.send_transaction(tx.clone());

    let meta = match res.as_ref() {
        Ok(v) => v.clone(),
        Err(v) => v.meta.clone()
    };

    print_tx(meta, tx);

    if res.is_err() {
        println!("error:\t{:?}", res.as_ref().err().unwrap().err);
    }

    res
}

pub fn create_payer(svm: &mut LiteSVM) -> Keypair {
    let payer_kp = Keypair::new();
    let payer_pk = payer_kp.pubkey();
    svm.airdrop(&payer_pk, 64_000_000_000).unwrap();
    payer_kp
}

pub fn create_keypair() -> Keypair {
    Keypair::new()
}

pub fn create_mint(svm: &mut LiteSVM, payer_kp: &Keypair, owner_pk: &Pubkey) -> Pubkey {
    CreateMint::new(svm, payer_kp)
        .authority(owner_pk)
        .send()
        .unwrap()
}

pub fn create_ata(svm: &mut LiteSVM, payer_kp: &Keypair, mint_pk: &Pubkey, owner_pk: &Pubkey) -> Pubkey {
    CreateAssociatedTokenAccount::new(svm, payer_kp, mint_pk)
        .owner(owner_pk)
        .send()
        .unwrap()
}

pub fn get_ata_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    let info:Account = get_spl_account(svm, &ata).unwrap();
    info.amount
}

pub fn mint_to(svm: &mut LiteSVM,
        payer: &Keypair,
        mint: &Pubkey,
        mint_owner: &Keypair,
        destination: &Pubkey,
        amount: u64,
) -> Result<(), FailedTransactionMetadata> {
    MintTo::new(svm, payer, mint, destination, amount)
        .owner(mint_owner)
        .send()
}

pub fn print_tx(meta: TransactionMetadata, tx: Transaction) {
    let msg = tx.message().serialize();

    println!("\n");
    println!("--------------------------------------------------------------------------------");
    println!("sig:\t{:?}", meta.signature);
    println!("len:\t{:?}", msg.len());
    println!("\nbody:\n{}", base64::encode(&msg));

    for i in 0..tx.message.instructions.len() {
        let ix = &tx.message.instructions[i];
        let ix_type = CodeInstruction::try_from(ix.data[0] as u8).unwrap();

        println!("\nix:\t{:?} ({})", ix_type, ix.data[0]);
        println!("accounts:");

        for key in &ix.accounts {
            println!("\t{}: {:?}", key, tx.message.account_keys[*key as usize]);
        }

        println!("\ndata:\n\t{:?}", ix.data);
        println!("\n\n{}\n", pretty_hex(&ix.data))
    }

    println!("");
    println!("cu:\t{:?}", meta.compute_units_consumed);
    println!("logs:");
    for log in &meta.logs {
        println!("\t{:?}", log);
    }
    println!("");
}
