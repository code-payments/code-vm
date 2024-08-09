import * as anchor from "@coral-xyz/anchor";
import { sha256 } from '@noble/hashes/sha256';
import { TestSandbox } from "./env";

interface TestPayment {
    mint: anchor.web3.PublicKey;
    src: anchor.web3.PublicKey;
    dest: anchor.web3.PublicKey;
    amount: number;
    timestamp: number;
    nonce: Buffer;
}

export const createPayment = (opt: {
    env: TestSandbox, 
    src: anchor.web3.PublicKey, 
    dest: anchor.web3.PublicKey, 
    amount: number
}) : TestPayment => {
    const mint = opt.env.mint;
    const timestamp = Date.now();
    const nonce = anchor.web3.Keypair.generate().publicKey.toBuffer();

    return { 
        mint, src:opt.src, dest:opt.dest, amount:opt.amount, timestamp, nonce, 
    }
}

export const getTranscript = (payment: TestPayment) => {
    const seeds = Buffer.concat([
        payment.mint.toBuffer(),
        payment.src.toBuffer(),
        payment.dest.toBuffer(),
        Buffer.from(payment.amount.toString()),
        Buffer.from(payment.timestamp.toString()),
        payment.nonce,
    ]);
    return sha256(seeds);
}