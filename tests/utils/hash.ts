import { sha256 } from '@noble/hashes/sha256';

function hash(val: Uint8Array | Buffer) {
    return Buffer.from(sha256(val));
}

function hashLeftRight(left: Buffer, right: Buffer) : any {
    const combined = [left, right];

    // Sorting here to make the merkle proofs deterministic
    combined.sort(Buffer.compare);
    return hash(Buffer.concat(combined));
}

function hashPairs(pairs: any[], lookupTable: {[key: string]: Buffer[]} = {}) {
    // Helper used by the merkle proof and prettyPrint methods. It is not used
    // by the core logic of the merkle tree itself.
    
    // It takes an array of values and hashes all the pairs into an array of hashes.
    // Optionally, it provides a lookup table to reverse the hashes.

    // The backend server will need something like this, but the clients do
    // not.

    const res = [];
    for (let i = 0; i < pairs.length; i+=2) {
        const left = pairs[i+0];
        const right = pairs[i+1];

        const hashed = hashLeftRight(left, right);

        lookupTable[hashed.toString('hex')] = [left, right];

        res.push(hashed);
    }

    return res;
}

export {
    hash,
    hashLeftRight,
    hashPairs,
}