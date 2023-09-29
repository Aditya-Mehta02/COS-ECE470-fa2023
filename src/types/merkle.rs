use super::hash::{Hashable, H256};
use ring::digest;

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    root: H256,
    layers: Vec<Vec<H256>>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self
    where
        T: Hashable,
    {
        let mut layers = Vec::new();
        let mut current_layer = Vec::new();

        for datum in data.iter() {
            current_layer.push(datum.hash());
        }

        while current_layer.len() > 1 {
            if current_layer.len() % 2 != 0 {
                let last = current_layer.last().unwrap().clone();
                current_layer.push(last);
            }
            let mut next_layer = Vec::new();
            for i in (0..current_layer.len()).step_by(2) {
                let left = &current_layer[i];
                let right = &current_layer[i + 1];
                let mut concated = Vec::new();
                concated.extend_from_slice(left.as_ref());
                concated.extend_from_slice(right.as_ref());
                let hash = ring::digest::digest(&ring::digest::SHA256, &concated).into();
                next_layer.push(hash);
            }
            layers.push(current_layer.clone());
            current_layer = next_layer;
        }

        let root = if !current_layer.is_empty() {
            current_layer[0].clone()
        } else {
            H256::default()
        };

        Self { root, layers }
    }

    pub fn root(&self) -> H256 {
        self.root
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut proof = Vec::new();
        let mut index = index;

        for layer in &self.layers {
            if index % 2 == 0 {
                proof.push(layer[index + 1].clone());
            } else {
                proof.push(layer[index - 1].clone());
            }
            index /= 2;
        }

        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(
    root: &H256,
    datum: &H256,
    proof: &[H256],
    mut index: usize,
    leaf_size: usize,
) -> bool {
    let mut hash = datum.clone();
    let mut leaf_count = leaf_size;
    // println!("Proof vector: {:?}", proof);
    // println!("hash: {:?}", &datum);
    // println!("root: {:?}", root);

    if leaf_count % 2 != 0 {
        leaf_count += 1;
    }

    index += leaf_count - 1;

    for &sibling in proof.iter() {
        let concatenated = if index % 2 == 0 {
            // println!("right, sibling: {:?}", &sibling);
            [sibling.as_ref(), hash.as_ref()].concat()
        } else {
            // println!("left, sibling: {:?}", &sibling);
            [hash.as_ref(), sibling.as_ref()].concat()
        };

        hash = H256::from(digest::digest(&digest::SHA256, &concatenated));
        // println!("proof iter hash: {:?}", &hash);
        index = (index - 1) / 2;
    }
    &hash == root
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::hash::H256;
    // Custom Test
    // macro_rules! gen_merkle_tree_data_custom {
    //     () => {{
    //         vec![
    //             (hex!("6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b")).into(),
    //             (hex!("d4735e3a265e16eee03f59718b9b5d03019c07d8b6c51f90da3a666eec13ab35")).into(),
    //             (hex!("4e07408562bedb8b60ce05c1decfe3ad16b72230967de01f640b7e4729b49fce")).into(),
    //             // (hex!("4b227777d4dd1fc61c6f884f48641d02b4d121d3fd328cb08b5531fcacdabf8a")).into(),
    //         ]
    //     }}; //9c2e4d8fe97d881430de4e754b4205b9c27ce96715231cffc4337340cb110280
    //         //0c08173828583fc6ecd6ecdbcca7b6939c49c242ad5107e39deb7b0a5996b903
    //         //80903da4e6bbdf96e8ff6fc3966b0cfd355c7e860bdd1caa8e4722d9230e40ac
    //         //5a9eab9148389395eff050ddf00220d722123ca8736c862bf200316389b3f611
    // }

    // #[test]
    // fn merkle_verifying_custom() {
    //     let input_data: Vec<H256> = gen_merkle_tree_data_custom!();
    //     let merkle_tree = MerkleTree::new(&input_data);
    //     let proof = merkle_tree.proof(3);
    //     assert!(verify(
    //         &merkle_tree.root(),
    //         &input_data[3].hash(),
    //         &proof,
    //         3,
    //         input_data.len()
    //     ));
    // }

    // End Custom Test

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        println!("\nroot:{}", root); // using Display trait
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(
            proof,
            vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(
            &merkle_tree.root(),
            &input_data[0].hash(),
            &proof,
            0,
            input_data.len()
        ));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
