#[cfg(test)]
pub mod utils;

use litesvm::LiteSVM;
use anchor_lang::{prelude::*, Discriminator};
use solana_sdk::{
    instruction::Instruction,
    message::Message,
    transaction::Transaction,
};

use code_vm::{
    program,
    instruction, 
    cvm::{
        CodeVmAccountWithChangeLog
    }, 
};

use utils::{create_mint, program_bytes, TestContext};

#[test]
fn init_vm() {
    let ctx = TestContext::new();
    let mut svm = LiteSVM::new();

    let payer = ctx.get_vm_authority();

    svm.airdrop(&payer, 1_000_000_000).unwrap();
    svm.add_program(code_vm::ID, &program_bytes());

    create_mint(&mut svm, ctx.get_mint_keypair(), ctx.get_signer()).unwrap();

    let args = crate::instruction::VmInit {
        lock_duration: ctx.get_lock_duration(),
    };

    let accounts = code_vm::accounts::CodeVmInit {
        vm: ctx.get_vm_address(),
        vm_authority: ctx.get_vm_authority(),
        omnibus: ctx.get_vm_omnibus_address(),
        mint: ctx.get_mint(),
        token_program: anchor_spl::token::ID,
        system_program: anchor_lang::system_program::ID,
        rent: anchor_lang::solana_program::sysvar::rent::ID,
    };

    let data = args.try_to_vec().unwrap();
    let ix = vec![
        Instruction {
            program_id: program::CodeVm::id(),
            accounts: accounts.to_account_metas(None),
            data: [
                instruction::VmInit::DISCRIMINATOR.to_vec(),
                data,
            ].concat(),
        }
    ];

    let blockhash = svm.latest_blockhash();
    let message = Message::new(
        ix.as_slice(),
        Some(&payer),
    );

    let tx = Transaction::new(&[ctx.get_signer()], message, blockhash);
    let _ = svm.send_transaction(tx).unwrap();

    let info = svm.get_account(&ctx.get_vm_address()).unwrap();
    let mut data = info.data.as_slice();
    let vm_account = CodeVmAccountWithChangeLog::deserialize(&mut data).unwrap();

    println!("{:?}", vm_account);
    
    assert_eq!(vm_account.info.authority, ctx.get_vm_authority());
}