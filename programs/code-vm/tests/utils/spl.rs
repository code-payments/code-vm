use anchor_spl::token;
use litesvm::types::TransactionResult;
use litesvm::LiteSVM;

use solana_sdk::program_pack::Pack;
use solana_sdk::{ 
    system_instruction,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

pub fn create_mint(svm: &mut LiteSVM, mint: &Keypair, payer: &Keypair) -> TransactionResult {
    let token_program_id = &token::spl_token::ID;

    let mint_len = token::spl_token::state::Mint::LEN;
    let mint_rent  = svm.minimum_balance_for_rent_exemption(mint_len);

    let ix = vec![
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(), 
            mint_rent, 
            mint_len as u64, 
            token_program_id
        ),
        token::spl_token::instruction::initialize_mint2(
            token_program_id, 
            &mint.pubkey(), 
            &payer.pubkey(), 
            None,
            5
        ).unwrap()
    ];

    let tx = Transaction::new_signed_with_payer(
        ix.as_slice(),
        Some(&payer.pubkey()),
        &[&payer, &mint],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
}