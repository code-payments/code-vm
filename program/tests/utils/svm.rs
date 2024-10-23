use std::path::PathBuf;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use litesvm::{types::{FailedTransactionMetadata, TransactionResult}, LiteSVM};
use litesvm_token::{CreateAssociatedTokenAccount, CreateMint, MintTo};

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
    let res = svm.send_transaction(tx);

    println!("");

    let meta = match res.as_ref() {
        Ok(v) => v.clone(),
        Err(v) => v.meta.clone()
    };

    if res.is_err() {
        println!("error:\t{:?}", res.as_ref().err().unwrap().err);
    }

    println!("tx:\t{:?}", meta.signature);
    println!("cu:\t{:?}", meta.compute_units_consumed);
    println!("logs:");
    for log in &meta.logs {
        println!("\t{:?}", log);
    }

    println!("");

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