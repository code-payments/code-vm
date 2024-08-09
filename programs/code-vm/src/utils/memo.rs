use std::str::FromStr;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;

pub fn build_memo(memo: &[u8], signer_pubkeys: &[&Pubkey]) -> Instruction {
    let memo_v1 = Pubkey::from_str("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo").unwrap();

    Instruction {
        program_id: memo_v1,
        accounts: signer_pubkeys
            .iter()
            .map(|&pubkey| AccountMeta::new_readonly(*pubkey, true))
            .collect(),
        data: memo.to_vec(),
    }
}

pub fn build_kre_memo() -> Instruction {
    let msg = "ZTAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let signer_pubkeys: &[&Pubkey] = &[]; // No signer accounts
    build_memo(msg.as_ref(), signer_pubkeys)
}