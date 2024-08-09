use anchor_lang::prelude::*;

use super::hash::Hash;
use crate::error::CodeVmError;
use crate::utils;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub struct MerkleTree {
    root: Hash,
    levels: u8,
    next_index: u64,

    filled_subtrees: Vec<Hash>,
    zero_values: Vec<Hash>,
}

impl MerkleTree {
        
    fn get_proof_size(levels: u8) -> usize {
        return 32 * levels as usize;
    }

    pub fn max_size_for(levels: u8) -> usize {
        32 +                                       // root
        1  +                                       // levels
        8  +                                       // next_index
        4  + Self::get_proof_size(levels) +  // filled_subtrees
        4  + Self::get_proof_size(levels)    // zero_values
    }

    pub fn new(seeds: &[&[u8]], levels: u8) -> Self {
        let zeros = Self::calc_zeros(levels, seeds);

        Self {
            levels,
            next_index: 0,
            root: zeros.last().unwrap().clone(),
            filled_subtrees: zeros.clone(),
            zero_values: zeros,
        }
    }

    pub fn get_num_levels(&self) -> u8 {
        self.levels
    }

    pub fn get_root(&self) -> Hash {
        self.root.clone()
    }

    pub fn get_empty_leaf(&self) -> Hash {
        self.zero_values.first().unwrap().clone()
    }

    pub fn as_leaf(val: Hash) -> Hash {
        utils::hash(val.as_ref())
    }

    /// Returns a vector of zero_value hashes for a merkle tree given levels and
    /// seeds.
    fn calc_zeros(levels: u8, seeds: &[&[u8]]) -> Vec<Hash> {
        let mut zeros = vec![];
        let mut current = utils::hashv(seeds);

        for _ in 0..levels {
            current = utils::hashv(&[
                current.as_ref(),
                current.as_ref()
            ]);
            zeros.push(current);
        }

        zeros
    }

    /// Returns the hash of left and right sorted and concatinated. We sort so that
    /// proofs are deterministic later.
    pub fn hash_left_right(left: Hash, right: Hash) -> Hash {
        let combined;
        if left.to_bytes() <= right.to_bytes() {
            combined = [left.as_ref(), right.as_ref()]
        } else {
            combined = [right.as_ref(), left.as_ref()]
        }

        utils::hashv(&combined)
    }

    /// Adds a value to the merkle tree at the next_index position.
    pub fn try_insert(&mut self, val: Hash) -> Result<()> {
        if self.next_index >= u64::pow(2, self.levels.into()) {
            return Err(CodeVmError::MerkleTreeFull.into());
        }
        
        let mut current_index : u64 = self.next_index;
        let mut current_hash = Self::as_leaf(val.clone());
        let mut left;
        let mut right;

        for i in 0..self.levels.into() {
            let i:usize = i;

            if current_index % 2 == 0 {
                left = current_hash;
                right = self.zero_values[i].clone();
                self.filled_subtrees[i] = current_hash.clone();
            } else {
                left = self.filled_subtrees[i].clone();
                right = current_hash;
            }

            current_hash = Self::hash_left_right(left, right);
            current_index = current_index / 2;
        }

        self.root = current_hash;

        self.next_index += 1;
        
        Ok(())
    }

    /// Removes a value from the merkle tree given a proof and the value to
    /// remove.
    pub fn try_remove(&mut self, proof: &Vec<Hash>, val: Hash) -> Result<()> {
        self.try_replace_leaf(
            proof,
            Self::as_leaf(val.clone()),
            self.get_empty_leaf(),
        )
    }

    /// Replaces a value in the merkle tree given a proof and the original value
    /// and the new value to insert in its place.
    pub fn try_replace(&mut self, proof: &Vec<Hash>, original_val: Hash, new_val: Hash) -> Result<()> {
        let original_leaf = Self::as_leaf(original_val.clone());
        let new_leaf = Self::as_leaf(new_val.clone());

        self.try_replace_leaf(proof, original_leaf, new_leaf)
    }

    /// Replaces a leaf in the merkle tree given a proof and the original leaf and
    /// the new leaf to insert in its place.
    pub fn try_replace_leaf(&mut self, proof: &Vec<Hash>, original_leaf: Hash, new_leaf: Hash) -> Result<()> {
        let original_path = Self::compute_path(proof, original_leaf.clone());
        let new_path = Self::compute_path(proof, new_leaf.clone());

        if !Self::is_valid_path(&original_path, self.root) {
            return Err(CodeVmError::InvalidMerkleProof.into());
        }

        // Go along the original path and use the new path to replace the
        // filled_subtree hashes where the original path is identical to the
        // filled_subtree hash.
        for i in 0..self.levels.into() {
            if original_path[i].eq(&self.filled_subtrees[i]) {
                self.filled_subtrees[i] = new_path[i].clone();
            }
        }

        // Update the root
        self.root = new_path.last().unwrap().clone();

        Ok(())
    }

    /// Returns true if a `val` can be proved to be a part of a Merkle tree
    pub fn contains(&self, proof: &Vec<Hash>, val: Hash) -> bool {
        let leaf = Self::as_leaf(val);
        self.contains_leaf(proof, leaf)
    }

    /// Returns true if a `leaf` can be proved to be a part of a Merkle tree
    pub fn contains_leaf(&self, proof: &Vec<Hash>, leaf: Hash) -> bool {
        let root = self.get_root();
        Self::is_valid_leaf(proof, root, leaf)
    }

    /// Returns true if a `leaf` can be proved to be a part of a Merkle tree
    /// defined by `root`. For this, a `proof` must be provided, containing
    /// sibling hashes on the branch from the leaf to the root of the tree. Each
    /// pair of leaves and each pair of pre-images are assumed to be sorted.
    pub fn is_valid_leaf(proof: &Vec<Hash>, root: Hash, leaf: Hash) -> bool {
        let computed_path = Self::compute_path(proof, leaf);
        Self::is_valid_path(&computed_path, root)
    }

    /// Returns true if this path leads to the root of the tree and is non-empty.
    fn is_valid_path(path: &Vec<Hash>, root: Hash) -> bool {
        if path.is_empty() {
            return false;
        }

        // Check if the computed hash (root) is equal to the provided root
        path.last()
            .unwrap()
            .eq(&root)
    }

    /// Computes the path from a leaf to the root of the tree given a proof.
    /// The path is returned as a vector of hashes but is not verified.
    fn compute_path(proof: &Vec<Hash>, leaf: Hash) -> Vec<Hash> {
        let mut computed_path = vec![];
        let mut computed_hash = leaf;

        computed_path.push(computed_hash);
        for proof_element in proof.into_iter() {
            computed_hash = Self::hash_left_right(
                computed_hash, 
                proof_element.clone()
            );
            computed_path.push(computed_hash);
        }

        computed_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tree() {
        let levels = 3;
        let seeds : &[&[u8]] = &[b"test"];
        let tree = MerkleTree::new(seeds, levels);

        assert_eq!(tree.get_num_levels(), levels);
        assert_eq!(tree.get_root(), tree.zero_values.last().unwrap().clone());
    }

    #[test]
    fn test_insert_and_remove() {
        let levels = 3; // 4 levels if you count the root
        let seeds : &[&[u8]] = &[b"test"];

        let mut tree = MerkleTree::new(seeds, levels);
        let empty = tree.zero_values.first().unwrap().clone();

        let val1 = utils::hash(b"val_1");
        let val2 = utils::hash(b"val_2");
        let val3 = utils::hash(b"val_3");
        let val4 = utils::hash(b"val_4");

        // Tree structure:
        // 
        //              root
        //            /     \
        //         m           n
        //       /   \       /   \
        //      i     j     k     l
        //     / \   / \   / \   / \
        //    a  b  c  d  e  f  g  h

        let a = MerkleTree::as_leaf(val1);
        let b = MerkleTree::as_leaf(val2);
        let c = MerkleTree::as_leaf(val3);

        let d = empty.clone();
        let e = empty.clone();
        let f = empty.clone();
        let g = empty.clone();
        let h = empty.clone();

        let i = MerkleTree::hash_left_right(a, b);
        let j: Hash = MerkleTree::hash_left_right(c, d);
        let k: Hash = MerkleTree::hash_left_right(e, f);
        let l: Hash = MerkleTree::hash_left_right(g, h);
        let m: Hash = MerkleTree::hash_left_right(i, j);
        let n: Hash = MerkleTree::hash_left_right(k, l);
        let root = MerkleTree::hash_left_right(m, n);

        assert!(tree.try_insert(val1.clone()).is_ok());
        assert!(tree.filled_subtrees[0].eq(&a));

        assert!(tree.try_insert(val2.clone()).is_ok());
        assert!(tree.filled_subtrees[0].eq(&a)); // Not a typo

        assert!(tree.try_insert(val3.clone()).is_ok());
        assert!(tree.filled_subtrees[0].eq(&c)); // Not a typo

        assert_eq!(tree.filled_subtrees[0], c);
        assert_eq!(tree.filled_subtrees[1], i);
        assert_eq!(tree.filled_subtrees[2], m);
        assert_eq!(root, tree.get_root());

        let val1_proof = vec![b.clone(), j.clone(), n.clone()];
        let val2_proof = vec![a.clone(), j.clone(), n.clone()];
        let val3_proof = vec![d.clone(), i.clone(), n.clone()];

        // Check filled leaves
        assert!(tree.contains(&val1_proof, val1));
        assert!(tree.contains(&val2_proof, val2));
        assert!(tree.contains(&val3_proof, val3));

        // Check empty leaves
        assert!(tree.contains_leaf(&vec![c.clone(), i.clone(), n.clone()], empty));
        assert!(tree.contains_leaf(&vec![f.clone(), l.clone(), m.clone()], empty));
        assert!(tree.contains_leaf(&vec![e.clone(), l.clone(), m.clone()], empty));
        assert!(tree.contains_leaf(&vec![h.clone(), k.clone(), m.clone()], empty));
        assert!(tree.contains_leaf(&vec![g.clone(), k.clone(), m.clone()], empty));

        // Remove val2 from the tree
        assert!(tree.try_remove(&val2_proof, val2).is_ok());

        // Update the expected tree structure
        let i = MerkleTree::hash_left_right(a, empty);
        let m: Hash = MerkleTree::hash_left_right(i, j);
        let root = MerkleTree::hash_left_right(m, n);

        assert_eq!(root, tree.get_root());

        let val1_proof = vec![empty.clone(), j.clone(), n.clone()];
        let val3_proof = vec![d.clone(), i.clone(), n.clone()];

        assert!(tree.contains_leaf(&val1_proof, a));
        assert!(tree.contains_leaf(&val2_proof, empty));
        assert!(tree.contains_leaf(&val3_proof, c));

        // Check that val2 is no longer in the tree
        assert!(!tree.contains_leaf(&val2_proof, b));

        // Insert val4 into the tree
        assert!(tree.try_insert(val4.clone()).is_ok());
        assert!(tree.filled_subtrees[0].eq(&c)); // Not a typo

        // Update the expected tree structure
        let d = MerkleTree::as_leaf(val4.clone());
        let j = MerkleTree::hash_left_right(c, d);
        let m = MerkleTree::hash_left_right(i, j);
        let root = MerkleTree::hash_left_right(m, n);

        assert_eq!(root, tree.get_root());

    }
}