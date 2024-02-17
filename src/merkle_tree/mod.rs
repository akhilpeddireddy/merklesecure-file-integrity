use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct MerkleTree {
    root: Vec<u8>,
    leaf_hashes: Vec<Vec<u8>>,
}

impl MerkleTree {
    pub fn new(data: Vec<Vec<u8>>) -> Self {
        let mut leaf_hashes = Vec::new();
        for leaf in &data {
            let mut hasher = Sha256::new();
            hasher.update(leaf);
            leaf_hashes.push(hasher.finalize().to_vec());
        }

        let root = Self::build_tree(leaf_hashes.clone());
        Self { root, leaf_hashes }
    }

    fn build_tree(mut leaves: Vec<Vec<u8>>) -> Vec<u8> {
        if leaves.len() == 1 {
            return leaves[0].clone();
        }

        if leaves.len() % 2 == 1 {
            leaves.push(leaves.last().unwrap().clone());
        }

        let mut parents = Vec::new();
        for i in (0..leaves.len()).step_by(2) {
            let mut hasher = Sha256::new();
            hasher.update(&leaves[i]);
            hasher.update(&leaves[i + 1]);
            parents.push(hasher.finalize().to_vec());
        }

        Self::build_tree(parents)
    }

    pub fn get_root_hash(&self) -> Vec<u8> {
        self.root.clone()
    }

    pub fn get_proof_for(&self, index: usize) -> Vec<(Vec<u8>, bool)> {
        if index >= self.leaf_hashes.len() {
            return Vec::new();
        }

        let mut proof = Vec::new();
        let mut index = index;
        let mut current_level = self.leaf_hashes.clone();

        while current_level.len() > 1 {
            let pair_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            if pair_index < current_level.len() {
                proof.push((current_level[pair_index].clone(), index % 2 == 1));
            } else {
                proof.push((current_level[index].clone(), index % 2 == 1));
            }

            index /= 2;
            current_level = Self::build_parent_level(&mut current_level);
        }

        proof
    }

    fn build_parent_level(leaves: &mut Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        if leaves.len() % 2 == 1 {
            leaves.push(leaves.last().unwrap().clone());
        }

        let mut parents = Vec::new();
        for i in (0..leaves.len()).step_by(2) {
            let mut hasher = Sha256::new();
            hasher.update(&leaves[i]);
            hasher.update(&leaves[i + 1]);
            parents.push(hasher.finalize().to_vec());
        }
        parents
    }

    #[allow(dead_code)]
    pub fn verify_proof(proof: &[(Vec<u8>, bool)], root: &Vec<u8>, leaf: &Vec<u8>) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(leaf);
        let mut current_hash = hasher.finalize().to_vec();

        for (hash, is_left) in proof {
            let mut hasher = Sha256::new();
            if *is_left {
                hasher.update(hash);
                hasher.update(&current_hash);
            } else {
                hasher.update(&current_hash);
                hasher.update(hash);
            }
            current_hash = hasher.finalize().to_vec();
        }

        current_hash.as_slice() == root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::Sha256;

    #[test]
    fn test_merkle_tree_single_node() {
        let data = vec![vec![1, 2, 3, 4]];
        let merkle_tree = MerkleTree::new(data.clone());
        let root_hash = Sha256::digest(&data[0]).to_vec();
        assert_eq!(merkle_tree.get_root_hash(), root_hash);
    }

    #[test]
    fn test_merkle_tree_multiple_nodes() {
        let data = vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8]];
        let merkle_tree = MerkleTree::new(data);

        let leaf1_hash = Sha256::digest(&[1, 2, 3, 4]).to_vec();
        let leaf2_hash = Sha256::digest(&[5, 6, 7, 8]).to_vec();
        let mut hasher = Sha256::new();
        hasher.update(leaf1_hash);
        hasher.update(leaf2_hash);
        let root_hash = hasher.finalize().to_vec();

        assert_eq!(merkle_tree.get_root_hash(), root_hash);
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let data = vec![
            vec![1, 2, 3, 4],
            vec![5, 6, 7, 8],
            vec![9, 10, 11, 12],
            vec![13, 14, 15, 16],
            vec![17, 18, 19, 20],
        ];
        let merkle_tree = MerkleTree::new(data.clone());
        let root_hash = merkle_tree.get_root_hash();

        for (i, leaf_data) in data.iter().enumerate() {
            let proof = merkle_tree.get_proof_for(i);
            let verification_result = MerkleTree::verify_proof(&proof, &root_hash, leaf_data);
            assert!(
                verification_result,
                "Proof verification failed for leaf at index {}",
                i
            );
        }
    }

    #[test]
    fn test_invalid_proof_verification() {
        let data = vec![vec![1], vec![2], vec![3], vec![4]];
        let tree = MerkleTree::new(data.clone());

        // Test with an invalid index
        let invalid_index = 10;
        let proof = tree.get_proof_for(invalid_index);
        assert!(proof.is_empty(), "Proof should be empty for invalid index");

        // Test with a valid index but modified proof
        let index = 2;
        let mut proof = tree.get_proof_for(index);
        proof[0].0[0] ^= 1; // Modify the proof slightly
        let root_hash = tree.get_root_hash();
        let leaf_hash = Sha256::digest(&data[index]);
        assert!(
            !MerkleTree::verify_proof(&proof, &root_hash, &leaf_hash.to_vec()),
            "Proof verification should fail for modified proof"
        );
    }
}
