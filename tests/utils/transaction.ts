import * as anchor from "@coral-xyz/anchor";
import { CompiledInstruction,  Message, PublicKey, Transaction, TransactionInstruction } from "@solana/web3.js";
const bs58 = anchor.utils.bytes.bs58;

interface ExtendedAccountMeta {
    pubkey: PublicKey;
    isSigner: Boolean;
    isWritable: Boolean;
    isPayer: Boolean;
    isProgram: Boolean;
}

export function compileMessage(input: Transaction): Message {
    // Workaround for typescript not allowing us to access private members
    const tx = input as any;

    const accountMetas: ExtendedAccountMeta[] = [];
    let instructions: TransactionInstruction[];
    let feePayer: PublicKey;
    let recentBlockhash;

    // Find the recentBlockhash
    if (tx.nonceInfo) {
        // Handle the durable nonce special case
        recentBlockhash = tx.nonceInfo.nonce;
        if (tx.instructions[0] != tx.nonceInfo.nonceInstruction) {
            instructions = [tx.nonceInfo.nonceInstruction, ...tx.instructions];
        } else {
            instructions = tx.instructions;
        }
    } else {
        recentBlockhash = tx.recentBlockhash;
        instructions = tx.instructions;
    }

    if (!recentBlockhash) {
        throw new Error('Transaction recentBlockhash required');
    }

    if (instructions.length < 1) {
        console.warn('No instructions provided');
    }

    // Find the fee payer
    if (tx.feePayer) {
        feePayer = tx.feePayer;
    } else if (tx.signatures.length > 0 && tx.signatures[0].publicKey) {
        // Use implicit fee payer
        feePayer = tx.signatures[0].publicKey;
    } else {
        throw new Error('Transaction fee payer required');
    }

    // Add the fee payer
    accountMetas.push({
        pubkey: feePayer,
        isPayer: true,
        isSigner: true,
        isWritable: true,
        isProgram: false,
    });

    instructions.forEach(instruction => {
        // Append programId accounts
        const programId = instruction.programId?.toString();
        accountMetas.push({
            pubkey: new PublicKey(programId),
            isSigner: false,
            isWritable: false,
            isProgram: true,
            isPayer: false,
        });

        // Append instruction accounts
        instruction.keys.forEach(accountMeta => {
            accountMetas.push({
                ...accountMeta,
                isProgram: false,
                isPayer: feePayer.equals(accountMeta.pubkey),
            });
        });
    });


    // Cull duplicate account metas
    const uniqueMetas: ExtendedAccountMeta[] = [];
    accountMetas.forEach(accountMeta => {
        const pubkeyString = accountMeta.pubkey.toString();
        const uniqueIndex = uniqueMetas.findIndex(x => { return x.pubkey.toString() === pubkeyString; });

        if (uniqueIndex > -1) {
            uniqueMetas[uniqueIndex].isWritable = uniqueMetas[uniqueIndex].isWritable || accountMeta.isWritable;
            uniqueMetas[uniqueIndex].isSigner = uniqueMetas[uniqueIndex].isSigner || accountMeta.isSigner;
        } else {
            uniqueMetas.push(accountMeta);
        }
    });

    // Sort. Prioritizing first by signer, then by writable
    uniqueMetas.sort((lhs, rhs) => {
        // Payer is always first
        if (lhs.isPayer !== rhs.isPayer) {
            return lhs.isPayer ? -1 : 1;
        }

        // Programs always come after accounts
        if (lhs.isProgram !== rhs.isProgram) {
            return lhs.isProgram ? 1 : -1;
        }

        // Signers always come before non-signers
        if (lhs.isSigner !== rhs.isSigner) {
            return lhs.isSigner ? -1 : 1;
        }

        // Writable accounts always come before read-only accounts
        if (lhs.isWritable !== rhs.isWritable) {
            return lhs.isWritable ? -1 : 1;
        }

        // Otherwise, sort by pubkey, stringwise.
        //return lhs.pubkey.toBase58().localeCompare(rhs.pubkey.toBase58());
        return Buffer.compare(lhs.pubkey.toBuffer(), rhs.pubkey.toBuffer());
    });

    let numRequiredSignatures = 0;
    let numReadonlySignedAccounts = 0;
    let numReadonlyUnsignedAccounts = 0;

    // Split out signing from non-signing keys and count header values
    const signedKeys: string[] = [];
    const unsignedKeys: string[] = [];
    uniqueMetas.forEach(({ pubkey, isSigner, isWritable }) => {
        if (isSigner) {
            signedKeys.push(pubkey.toString());
            numRequiredSignatures += 1;
            if (!isWritable) {
                numReadonlySignedAccounts += 1;
            }
        } else {
            unsignedKeys.push(pubkey.toString());
            if (!isWritable) {
                numReadonlyUnsignedAccounts += 1;
            }
        }
    });

    const accountKeys = signedKeys.concat(unsignedKeys);
    const compiledInstructions: CompiledInstruction[] = instructions.map(
        instruction => {
            const { data, programId } = instruction;
            return {
                programIdIndex: accountKeys.indexOf(programId.toString()),
                accounts: instruction.keys.map(meta =>
                    accountKeys.indexOf(meta.pubkey.toString()),
                ),
                data: bs58.encode(data),
            };
        },
    );

    return new Message({
        header: {
            numRequiredSignatures,
            numReadonlySignedAccounts,
            numReadonlyUnsignedAccounts,
        },
        accountKeys,
        recentBlockhash,
        instructions: compiledInstructions,
    });
}