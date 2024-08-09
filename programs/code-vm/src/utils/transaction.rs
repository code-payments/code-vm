use anchor_lang::solana_program::{
    message::CompileError,
    message::MessageHeader, 
    pubkey::Pubkey,
    instruction::Instruction,
    instruction::CompiledInstruction,
    message::legacy::Message,
};
use std::{cmp::Ordering, collections::BTreeMap};
use crate::types::Hash;

/// A helper struct to collect sorted pubkeys compiled for a set of
/// instructions. The standard solana approach does not generate deterministic
/// transactions accross platforms.
/// 
/// Derived from: https://github.com/solana-labs/solana/blob/27eff8408b7223bb3c4ab70523f8a8dca3ca6645/sdk/program/src/message/compiled_keys.rs#L12
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct SortedCompiledKeys {
    payer: Option<Pubkey>,
    key_meta_map: BTreeMap<Pubkey, CompiledKeyMeta>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct CompiledKeyMeta {
    is_signer: bool,
    is_writable: bool,
    is_payer: bool,     // non-standard param
    is_program: bool,   // non-standard param
}

impl SortedCompiledKeys {
    /// Compiles the pubkeys referenced by a list of instructions and organizes by
    /// signer/non-signer and writable/readonly.
    pub(crate) fn compile(instructions: &[Instruction], payer: Option<Pubkey>) -> Self {
        let mut key_meta_map = BTreeMap::<Pubkey, CompiledKeyMeta>::new();

        for ix in instructions {
            let meta = key_meta_map.entry(ix.program_id).or_default();
            meta.is_program = true;

            for account_meta in &ix.accounts {
                let meta = key_meta_map.entry(account_meta.pubkey).or_default();
                meta.is_signer |= account_meta.is_signer;
                meta.is_writable |= account_meta.is_writable;
            }
        }

        if let Some(payer) = &payer {
            let meta = key_meta_map.entry(*payer).or_default();
            meta.is_signer = true;
            meta.is_writable = true;
            meta.is_payer = true;
        }

        Self {
            payer,
            key_meta_map,
        }
    }

    pub(crate) fn try_into_message_components(
        self,
    ) -> std::result::Result<(MessageHeader, Vec<Pubkey>), CompileError> {
        let try_into_u8 = |num: usize| -> std::result::Result<u8, CompileError> {
            u8::try_from(num).map_err(|_| CompileError::AccountIndexOverflow)
        };

        let Self {
            payer,
            mut key_meta_map,
        } = self;

        if let Some(payer) = &payer {
            key_meta_map.remove_entry(payer);
        }

        // First, collect all keys and their metadata into a single vector for sorting.
        let mut accounts: Vec<(Pubkey, &CompiledKeyMeta)> = key_meta_map
            .iter()
            .map(|(key, meta)| (*key, meta))
            .collect();

        // Sort accounts by custom criteria to ensure deterministic ordering.
        accounts.sort_unstable_by(|(key_a, meta_a), (key_b, meta_b)| {
            // Payer is always first.
            if meta_a.is_payer != meta_b.is_payer {
                return if meta_a.is_payer { Ordering::Less } else { Ordering::Greater };
            }

            // Programs always come after accounts.
            if meta_a.is_program != meta_b.is_program {
                return if meta_a.is_program { Ordering::Greater } else { Ordering::Less };
            }

            // Signers always come before non-signers.
            if meta_a.is_signer != meta_b.is_signer {
                return if meta_a.is_signer { Ordering::Less } else { Ordering::Greater };
            }

            // Writable accounts always come before read-only accounts.
            if meta_a.is_writable != meta_b.is_writable {
                return if meta_a.is_writable { Ordering::Less } else { Ordering::Greater };
            }

            // If all else is equal, sort by pubkey bytes.
            key_a.to_bytes().cmp(&key_b.to_bytes())
        });

        // If the payer is specified, ensure it is the first account.
        if let Some(payer_key) = payer {
            accounts.insert(0, (payer_key, &CompiledKeyMeta {
                is_signer: true,
                is_writable: true,
                is_payer: true,
                is_program: false,
            }));
        }

        // Count types of accounts for the message header
        let num_required_signatures = accounts.iter().filter(|(_, meta)| meta.is_signer).count();
        let num_readonly_signed_accounts = accounts.iter().filter(|(_, meta)| meta.is_signer && !meta.is_writable).count();
        let num_readonly_unsigned_accounts = accounts.iter().filter(|(_, meta)| !meta.is_signer && !meta.is_writable).count();

        // Now build the final accounts list for the message
        let final_account_keys: Vec<Pubkey> = accounts.into_iter().map(|(key, _)| key).collect();

        let header = MessageHeader {
            num_required_signatures: try_into_u8(num_required_signatures)?,
            num_readonly_signed_accounts: try_into_u8(num_readonly_signed_accounts)?,
            num_readonly_unsigned_accounts: try_into_u8(num_readonly_unsigned_accounts)?,
        };

        Ok((header, final_account_keys))
    }
}

fn position(keys: &[Pubkey], key: &Pubkey) -> u8 {
    keys.iter().position(|k| k == key).unwrap() as u8
}

fn compile_instruction(ix: &Instruction, keys: &[Pubkey]) -> CompiledInstruction {
    let accounts: Vec<_> = ix
        .accounts
        .iter()
        .map(|account_meta| position(keys, &account_meta.pubkey))
        .collect();

    CompiledInstruction {
        program_id_index: position(keys, &ix.program_id),
        data: ix.data.clone(),
        accounts,
    }
}

fn compile_instructions(ixs: &[Instruction], keys: &[Pubkey]) -> Vec<CompiledInstruction> {
    ixs.iter().map(|ix| compile_instruction(ix, keys)).collect()
}

pub fn message_with_sorted_keys(
    instructions: &[Instruction],
    payer: Option<&Pubkey>,
    blockhash: &Hash,
) -> Message {
    let compiled_keys = SortedCompiledKeys::compile(instructions, payer.cloned());
    let (header, account_keys) = compiled_keys
        .try_into_message_components()
        .expect("overflow when compiling message keys");

    let instructions = compile_instructions(instructions, &account_keys);
    Message::new_with_compiled_instructions(
        header.num_required_signatures,
        header.num_readonly_signed_accounts,
        header.num_readonly_unsigned_accounts,
        account_keys,
        blockhash.to_bytes().into(),
        instructions,
    )
}