import * as anchor from "@coral-xyz/anchor";
import * as web3 from "@solana/web3.js";
import { TestVm } from "./vm";

async function createLut(env: TestVm, keys: web3.PublicKey[]) {
    const { connection, signer } = env;

    const slot = await connection.getSlot();
    const [lookupTableInst, lookupTableAddress] =
    web3.AddressLookupTableProgram.createLookupTable({
        authority: signer.publicKey,
        payer: signer.publicKey,
        recentSlot: slot
    });

    const extendInstruction = web3.AddressLookupTableProgram.extendLookupTable({
        payer: signer.publicKey,
        authority: signer.publicKey,
        lookupTable: lookupTableAddress,
        addresses: keys,
    });

    const tx = new anchor.web3.Transaction();
    tx.recentBlockhash = (await connection.getRecentBlockhash()).blockhash;
    tx.add(
        lookupTableInst,
        extendInstruction
    );
    tx.feePayer = signer.publicKey;

    const sig = await connection.sendTransaction(tx, [signer], { skipPreflight: true });
    console.log("tx", sig);
    
    await connection.confirmTransaction(sig, "confirmed");

    const lut = (await connection.getAddressLookupTable(lookupTableAddress)).value;
    console.log("lut", lut);

    return lut;
}

async function sendLutTransaction(env: TestVm, ix: web3.TransactionInstruction[], lut: web3.AddressLookupTableAccount) {
    const { connection, signer } = env;

    const blockhash = (await connection.getRecentBlockhash()).blockhash;
    const messageV0 = new web3.TransactionMessage({
        payerKey: signer.publicKey,
        recentBlockhash: blockhash,
        instructions: ix,
    }).compileToV0Message([lut]);

    const x = messageV0.serialize();
    console.log("len", x.length);

    const transactionV0 = new web3.VersionedTransaction(messageV0);
    transactionV0.sign([signer]);

    const buf = transactionV0.serialize();
    console.log("len", buf.length);
    console.log("buf", Buffer.from(buf).toString('hex'));
    console.log("base64", Buffer.from(buf).toString('base64'));

    const sig = await connection.sendTransaction(transactionV0, {skipPreflight:true});
    await connection.confirmTransaction(sig, "confirmed");

    return sig;
}

export {
    createLut,
    sendLutTransaction,
}