use steel::*;
use bytemuck::{Pod, Zeroable};
use std::fmt::Debug;

use super::hash::Hash;
use crate::helpers::check_condition;
use crate::utils;

#[repr(C, align(8))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct MerkleTree<const N: usize> {
    root: Hash,
    filled_subtrees: [Hash; N],
    zero_values: [Hash; N],
    next_index: u64,
}

unsafe impl<const N: usize> Zeroable for MerkleTree<N> {}
unsafe impl<const N: usize> Pod for MerkleTree<N> {}

impl<const N: usize> MerkleTree<N> {
    pub const fn get_depth(&self) -> u8 {
        N as u8
    }

    pub const fn get_size() -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn get_root(&self) -> Hash {
        self.root
    }

    pub fn get_empty_leaf(&self) -> Hash {
        self.zero_values[0]
    }

    pub fn new(seeds: &[&[u8]]) -> Self {
        let zeros = Self::calc_zeros(seeds);
        Self {
            next_index: 0,
            root: zeros[N - 1],
            filled_subtrees: zeros,
            zero_values: zeros,
        }
    }

    pub fn init(&mut self, seeds: &[&[u8]]) {
        let zeros = Self::calc_zeros(seeds);
        self.next_index = 0;
        self.root = zeros[N - 1];
        self.filled_subtrees = zeros;
        self.zero_values = zeros;
    }

    fn calc_zeros(seeds: &[&[u8]]) -> [Hash; N] {
        let mut zeros: [Hash; N] = [Hash::default(); N];
        let mut current = utils::hashv(seeds);

        for i in 0..N {
            zeros[i] = current;
            current = utils::hashv(&[current.as_ref(), current.as_ref()]);
        }

        zeros
    }

    pub fn try_insert(&mut self, val: Hash) -> ProgramResult {
        check_condition(
            self.next_index < (1u64 << N),
            "merkle tree is full",
        )?;

        let mut current_index = self.next_index;
        let mut current_hash = MerkleTree::<N>::as_leaf(val);
        let mut left;
        let mut right;

        for i in 0..N {
            if current_index % 2 == 0 {
                left = current_hash;
                right = self.zero_values[i];
                self.filled_subtrees[i] = current_hash;
            } else {
                left = self.filled_subtrees[i];
                right = current_hash;
            }

            current_hash = Self::hash_left_right(left, right);
            current_index /= 2;
        }

        self.root = current_hash;
        self.next_index += 1;

        Ok(())
    }

    pub fn try_remove(&mut self, proof: &[Hash], val: Hash) -> ProgramResult {
        self.check_length(proof)?;

        self.try_replace_leaf(proof, Self::as_leaf(val), self.get_empty_leaf())
    }

    pub fn try_replace(&mut self, proof: &[Hash], original_val: Hash, new_val: Hash) -> ProgramResult {
        self.check_length(proof)?;

        let original_leaf = Self::as_leaf(original_val);
        let new_leaf = Self::as_leaf(new_val);

        self.try_replace_leaf(proof, original_leaf, new_leaf)
    }

    pub fn try_replace_leaf(&mut self, proof: &[Hash], original_leaf: Hash, new_leaf: Hash) -> ProgramResult {
        self.check_length(proof)?;

        let original_path = MerkleTree::<N>::compute_path(proof, original_leaf);
        let new_path = MerkleTree::<N>::compute_path(proof, new_leaf);

        check_condition(
            MerkleTree::<N>::is_valid_path(&original_path, self.root),
            "invalid proof for original leaf",
        )?;

        for i in 0..N {
            if original_path[i] == self.filled_subtrees[i] {
                self.filled_subtrees[i] = new_path[i];
            }
        }

        self.root = *new_path.last().unwrap();

        Ok(())
    }

    pub fn contains(&self, proof: &[Hash], val: Hash) -> bool {
        if let Err(_) = self.check_length(proof) {
            return false;
        }

        let leaf = Self::as_leaf(val);
        self.contains_leaf(proof, leaf)
    }

    pub fn contains_leaf(&self, proof: &[Hash], leaf: Hash) -> bool {
        if let Err(_) = self.check_length(proof) {
            return false;
        }

        let root = self.get_root();
        Self::is_valid_leaf(proof, root, leaf)
    }

    pub fn as_leaf(val: Hash) -> Hash {
        utils::hash(val.as_ref())
    }

    pub fn hash_left_right(left: Hash, right: Hash) -> Hash {
        let combined;
        if left.to_bytes() <= right.to_bytes() {
            combined = [left.as_ref(), right.as_ref()];
        } else {
            combined = [right.as_ref(), left.as_ref()];
        }

        utils::hashv(&combined)
    }

    pub fn compute_path(proof: &[Hash], leaf: Hash) -> Vec<Hash> {
        let mut computed_path = Vec::with_capacity(proof.len() + 1);
        let mut computed_hash = leaf;

        computed_path.push(computed_hash);

        for proof_element in proof.iter() {
            computed_hash = Self::hash_left_right(computed_hash, *proof_element);
            computed_path.push(computed_hash);
        }

        computed_path
    }

    pub fn is_valid_leaf(proof: &[Hash], root: Hash, leaf: Hash) -> bool {
        let computed_path = Self::compute_path(proof, leaf);
        Self::is_valid_path(&computed_path, root)
    }

    pub fn is_valid_path(path: &[Hash], root: Hash) -> bool {
        if path.is_empty() {
            return false;
        }

        *path.last().unwrap() == root
    }

    #[cfg(not(target_os = "solana"))]
    fn hash_pairs(pairs: Vec<Hash>) -> Vec<Hash> {
        // A helper function that hashes all pairs of hashes into a new array of
        // hashes.
        let mut res = Vec::with_capacity(pairs.len() / 2);

        for i in (0..pairs.len()).step_by(2) {
            let left = pairs[i];
            let right = pairs[i + 1];

            let hashed = Self::hash_left_right(left, right);
            res.push(hashed);
        }

        res
    }

    #[cfg(not(target_os = "solana"))]
    pub fn get_merkle_proof(&self, values: &[Hash], index: usize) -> Vec<Hash> {
        let mut layers = Vec::with_capacity(N);
        let mut current_layer = values.to_vec();
        for i in 0..N {
            if current_layer.len() % 2 != 0 {
                current_layer.push(self.zero_values[i]);
            }

            layers.push(current_layer.clone());
            current_layer = Self::hash_pairs(current_layer);
        }

        // At this point we have all the layers of the merkle tree in an array
        // of arrays. The next step is to find the siblings of the provided
        // for_leaf all the way up the tree.

        let mut proof = Vec::with_capacity(N);
        let mut current_index = index;
        let mut layer_index = 0;
        let mut sibling;

        for _ in 0..N {
            if current_index % 2 == 0 {
                sibling = layers[layer_index][current_index + 1];
            } else {
                sibling = layers[layer_index][current_index - 1];
            }

            proof.push(sibling);

            current_index /= 2;
            layer_index += 1;
        }

        proof
    }

    fn check_length(&self, proof: &[Hash]) -> Result<(), ProgramError> {
        check_condition(
            proof.len() == N,
            "merkle proof length does not match tree depth",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestTree = MerkleTree<3>;

    #[test]
    fn test_create_tree() {
        let seeds : &[&[u8]] = &[b"test"];
        let tree = TestTree::new(seeds);

        assert_eq!(tree.get_depth(), 3);
        assert_eq!(tree.get_root(), tree.zero_values.last().unwrap().clone());
    }

    #[test]
    fn test_insert_and_remove() {
        let seeds : &[&[u8]] = &[b"test"];

        let mut tree = TestTree::new(seeds);
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

        let a = TestTree::as_leaf(val1);
        let b = TestTree::as_leaf(val2);
        let c = TestTree::as_leaf(val3);

        let d = empty.clone();
        let e = empty.clone();
        let f = empty.clone();
        let g = empty.clone();
        let h = empty.clone();

        let i = TestTree::hash_left_right(a, b);
        let j: Hash = TestTree::hash_left_right(c, d);
        let k: Hash = TestTree::hash_left_right(e, f);
        let l: Hash = TestTree::hash_left_right(g, h);
        let m: Hash = TestTree::hash_left_right(i, j);
        let n: Hash = TestTree::hash_left_right(k, l);
        let root = TestTree::hash_left_right(m, n);

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
        let i = TestTree::hash_left_right(a, empty);
        let m: Hash = TestTree::hash_left_right(i, j);
        let root = TestTree::hash_left_right(m, n);

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
        let d = TestTree::as_leaf(val4.clone());
        let j = TestTree::hash_left_right(c, d);
        let m = TestTree::hash_left_right(i, j);
        let root = TestTree::hash_left_right(m, n);

        assert_eq!(root, tree.get_root());

    }

    #[test]
    fn test_proof() {
        let seeds : &[&[u8]] = &[b"test"];

        let mut tree = TestTree::new(seeds);

        let val1 = utils::hash(b"val_1");
        let val2 = utils::hash(b"val_2");
        let val3 = utils::hash(b"val_3");

        let leaves = [
            TestTree::as_leaf(val1), 
            TestTree::as_leaf(val2), 
            TestTree::as_leaf(val3), 
        ];

        assert!(tree.try_insert(val1.clone()).is_ok());
        assert!(tree.try_insert(val2.clone()).is_ok());
        assert!(tree.try_insert(val3.clone()).is_ok());

        let val1_proof = tree.get_merkle_proof(&leaves, 0);
        let val2_proof = tree.get_merkle_proof(&leaves, 1);
        let val3_proof = tree.get_merkle_proof(&leaves, 2);

        assert!(tree.contains(&val1_proof, val1));
        assert!(tree.contains(&val2_proof, val2));
        assert!(tree.contains(&val3_proof, val3));

        // Invalid Proof Length
        let invalid_proof_short = &val1_proof[..2]; // Shorter than depth
        let invalid_proof_long = [&val1_proof[..], &val1_proof[..]].concat(); // Longer than depth

        assert!(!tree.contains(&invalid_proof_short, val1));
        assert!(!tree.contains(&invalid_proof_long, val1));

        // Empty Proof
        let empty_proof: Vec<Hash> = Vec::new();
        assert!(!tree.contains(&empty_proof, val1));
    }
}