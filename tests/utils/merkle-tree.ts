import treeify from "treeify";
import * as anchor from "@coral-xyz/anchor";
import { hash, hashLeftRight, hashPairs } from "./hash";

export interface Options {
    shorten: boolean;
    findZeros: boolean;
}

export interface ProgramMerkleTree {
    root: { value: number[]; };
    levels: number;
    nextIndex: anchor.BN;
    filledSubtrees: { value: number[]; }[];
    zeroValues: { value: number[]; }[];
}

export class MerkleTree {
    levels: number;
    seeds: Buffer;
    root: Buffer;
    filledSubtrees: Buffer[];
    zeroTree: Buffer[];
    emptyValue: Buffer;
    nextIndex: number;

    // We only need to store this on the server side. The on-chain program does
    // not have this info.
    leaves: Buffer[];

    constructor(seeds: Buffer, levels: number) {
        this.seeds = seeds;
        this.levels = levels;
        this.nextIndex = 0;

        this.filledSubtrees = [];
        this.zeroTree = [];
        this.leaves = [];

        this.calcZeros();
        this.root = this.zeroTree[this.zeroTree.length-1];

        for (let i = 0; i < levels; i++) {
            this.filledSubtrees[i] = this.zeroTree[i];
        }
    }

    static from(history: ProgramMerkleTree, seeds: Buffer = Buffer.from([])) {
        const tree = new MerkleTree(seeds, history.levels);
        tree.nextIndex = history.nextIndex.toNumber();
        tree.filledSubtrees = history.filledSubtrees.map(v => Buffer.from(v.value));
        tree.zeroTree = history.zeroValues.map(v => Buffer.from(v.value));
        tree.root = Buffer.from(history.root.value);
        return tree;
    }

    insert(val: Buffer) {
        if (this.nextIndex >= Math.pow(2, this.levels)) {
            throw "Merkle tree is full. No more leaves can be added"
        }

        const leaf = asLeaf(val);
        this.leaves.push(leaf);

        let currentIndex = this.nextIndex;
        let currentLevelHash = leaf;
        let left: any, right: any;

        for (let i=0; i<this.levels; i++) {
            if (currentIndex % 2 == 0) {
                left = currentLevelHash;
                right = this.zeroTree[i];
                this.filledSubtrees[i] = currentLevelHash;
            } else {
                left = this.filledSubtrees[i];
                right = currentLevelHash;
            }

            currentLevelHash = hashLeftRight(left, right);
            currentIndex = Math.floor(currentIndex / 2);
        }

        this.root = currentLevelHash;
        this.nextIndex = this.nextIndex + 1;

        return this.nextIndex;
    }

    remove(proof: Buffer[], val: Buffer) {
        this.replaceLeaf(
            proof,
            asLeaf(val),
            this.emptyValue
        )
    }

    replace(proof: Buffer[], originalVal: Buffer, newVal: Buffer) {
        const originalLeaf = asLeaf(originalVal);
        const newLeaf = asLeaf(newVal);

        this.replaceLeaf(proof, originalLeaf, newLeaf);
    }

    replaceLeaf(proof: Buffer[], originalLeaf: Buffer, newLeaf: Buffer) {
        const originalPath = computePath(proof, originalLeaf);
        const newPath = computePath(proof, newLeaf);

        if (!isValidPath(originalPath, this.root)) {
            throw "Invalid proof";
        }

        for (let i=0; i<this.levels; i++) {
            if (Buffer.compare(originalPath[i], this.filledSubtrees[i]) === 0) {
                this.filledSubtrees[i] = newPath[i];
            }
        }

        this.root = newPath[newPath.length-1];
        this.leaves[this.getIndexForLeaf(originalLeaf)] = newLeaf;
    }

    calcZeros() {
        // This matches the on-chain program. 

        let current = hash(this.seeds);
        for (let i=0; i<this.levels; i++) {
            const combined = [
                current,
                current,
            ]

            current = hash(Buffer.concat(combined));
            this.zeroTree.push(current);
        }

        this.emptyValue = this.zeroTree[0];
    }

    toString() {
        // This method is for debugging purposes.

        return JSON.stringify(JSON.parse(`{
                "nextIndex": ${this.nextIndex},
                "zeroTree": ${JSON.stringify(this.zeroTree.map(v => v.toString('hex')))},
                "filledSubtrees": ${JSON.stringify(this.filledSubtrees.map(v => v.toString('hex')))},
                "root": "${this.root.toString('hex')}"
            }`), null, 4);
    }

    prettyPrint() {
        // This method is for debugging purposes.

        return treeify.asTree(
            this.asTree({
                shorten: true,
                findZeros: true,
        }), false, true);
    }

    getRoot(): Buffer {
        return this.root;
    }

    getIndexForLeaf(leaf: Buffer): number {
        for (let i=0; i<this.leaves.length; i++) {
            if (Buffer.compare(leaf, this.leaves[i]) === 0) {
                return i;
            }
        }
    }

    getProofFor(val: Buffer): Buffer[] {
        const leaf = asLeaf(val);
        const index = this.getIndexForLeaf(leaf);
        return this.getProofForLeafAtIndex(index);
    }

    getProofForCommitment(commitment: Buffer, commitmentRootIndex: number): Buffer[] {
        const leaf = asLeaf(commitment);
        const index = this.getIndexForLeaf(leaf);
        return this.getProofForLeafAtIndex(index, commitmentRootIndex);
    }

    getProofForLeafAtIndex(
        for_leaf: number,                   // The leaf index to get a merkle proof for.
        until_leaf: number = this.nextIndex // The leaf index to stop at. (default: all leaves)
    ): Buffer[] {
        // Helper used for generating merkle proofs. Not used by the core logic
        // of the tree itself. The backend server will need something like this,
        // but the clients do not.

        // Important: The algorithm used here is inefficient. It is a brute
        // force solution. We will need a better approach in production. We
        // should be able to generate the proofs from periodically stored proofs
        // instead of hashing all the leaves.

        if (for_leaf > until_leaf) {
            throw "'for_leaf' must be less than 'until_leaf'";
        }

        if (this.leaves.length == 0) {
            return [...this.zeroTree];
        }

        // Hash all leaves from the bottom up; use the empty tree value as the
        // rightmost leaf.

        let layers =[];
        let currentLayer = this.leaves.slice(0, until_leaf + 1);
        for (let i = 0; i < this.levels; i++) {
            layers.push(currentLayer);

            if (currentLayer.length % 2 != 0) {
                currentLayer.push(this.zeroTree[i]);
            }
            currentLayer = hashPairs(currentLayer);
        }

        // At this point we have all the layers of the merkle tree in an array
        // of arrays. The next step is to find the siblings of the provided
        // for_leaf all the way up the tree.

        const proof = [];
        let currentIndex = for_leaf;
        let layerIndex = 0;
        let sibling : Buffer;

        for (let i=0; i<this.levels; i++) {
            if (currentIndex % 2 == 0) {
                sibling = layers[layerIndex][currentIndex+1];
            } else {
                sibling = layers[layerIndex][currentIndex-1];
            }
            proof.push(sibling);

            currentIndex = Math.floor(currentIndex / 2);
            layerIndex = layerIndex + 1;
        }

        return proof;
    }


    asTree(opt: Options = {shorten: false, findZeros: false}) {
        // This method is for debugging purposes. It returns a nested object
        // representation of the tree.

        // Important: The algorithm used here is inefficient. It is a brute
        // force solution. We will need a better approach in production. We
        // should be able to generate the proofs from periodically stored proofs
        // instead of hashing all the leaves.

        // On a positive note, it is not clear that we need this method outside
        // debuggigng purposes..

        const self = this;
        const lookupTable : {[key: string]: Buffer[]} = {};

        let layers =[];
        let currentLayer = [...this.leaves];
        for (let i = 0; i < this.levels; i++) {
            layers.push(currentLayer);

            if (currentLayer.length % 2 != 0) {
                currentLayer.push(self.zeroTree[i]);
            }
            currentLayer = hashPairs(currentLayer, lookupTable);
        }

        return { 
            [this.root.toString('hex')]: _getObj(this.root)
        };

        // Private function to get the object representation of a tree.
        function _getObj(val: any) {
            // Recursive helper for generating the structure expected by the
            // treeify.asTree() library.

            const hash = val.toString('hex');
            if (!lookupTable[hash]) { 
                if (opt.shorten) {
                    return hash.substring(0, 4) + '...' + hash.substring(hash.length - 4);
                }

                return hash;
            }

            let left = lookupTable[hash][0];
            let right = lookupTable[hash][1];

            let lkey = left.toString('hex'),
                rkey = right.toString('hex');

            if (opt.shorten) {
                lkey = lkey.substring(0, 4) + '...' + lkey.substring(lkey.length - 4);
                rkey = rkey.substring(0, 4) + '...' + rkey.substring(rkey.length - 4);
            }

            if (opt.findZeros) {
                if (self.zeroTree.includes(left)) {
                    lkey += " <Empty>";
                }
                if (self.zeroTree.includes(right)) {
                    rkey += " <Empty>";
                }
            }

            return {
                [lkey]: _getObj(left),
                [rkey]: _getObj(right),
            };
        }
    }
}

// This function is used to verify a merkle proof. It takes a proof, a
// root hash, and a leaf hash. It returns true if the leaf hash can be
// proven to be a part of the Merkle tree defined by the root hash.
// The proof is an array of sibling hashes, where each pair of leaves
// and each pair of pre-images are assumed to be sorted.
export function verifyLeaf(proof: Buffer[], root: Buffer, leaf: Buffer) : boolean {
    const computed_path = computePath(proof, leaf);
    return isValidPath(computed_path, root);
}

function isValidPath(path: Buffer[], root: Buffer) : boolean {
    if (path.length == 0) {
        return false;
    }

    // Check if the computed hash (root) is equal to the provided root
    return Buffer.compare(path[path.length-1], root) == 0;
}

function asLeaf(val: Buffer) {
    return hash(val);
}

function computePath(proof: Buffer[], leaf: Buffer) : Buffer[] {
    // Computes the path from a leaf to the root of the tree given a proof.
    // The path is returned as a vector of hashes but is not verified.

    let computed_path = [];
    let computed_hash = leaf;

    computed_path.push(computed_hash);
    for (let proof_element of proof) {
        computed_hash = hashLeftRight(computed_hash, proof_element);
        computed_path.push(computed_hash);
    }

    return computed_path;
}