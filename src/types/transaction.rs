use crate::types::hash::{Hashable, H256};
use crate::types::key_pair;
use rand::Rng;
use ring::signature::KeyPair;
use ring::signature::{Ed25519KeyPair, Signature};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender: String,
    receiver: String,
    value: i64,
    nonce: u64,
}

impl Transaction {
    pub fn generate_random_transaction() -> Self {
        let mut rng = rand::thread_rng();
        let sender = format!("Sender{}", rng.gen::<u32>());
        let receiver: String = format!("Receiver{}", rng.gen::<u32>());
        let value = rng.gen::<i64>();
        let nonce = rng.gen::<u64>();

        Transaction {
            sender,
            receiver,
            value,
            nonce,
        }
    }

    pub fn generate_random_transaction_from_ico(nonce: u64, reciever_addr: String) -> Self {
        let mut rng = rand::thread_rng();
        let sender = "DIc8B6v4D6pHAaPfOIwLxzugi49T+ooEU9zKelCZyCg=".to_string(); // The ICO's address
        let receiver = reciever_addr;
        let value = rng.gen_range(1..=5); // Value between 1 and 5
        let nonce = nonce;

        Transaction {
            sender,
            receiver,
            value,
            nonce,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    transaction: Transaction,
    signature: Vec<u8>,
    public_key: Vec<u8>,
}

impl SignedTransaction {
    // Getter for the transaction
    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }
    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }
    pub fn public_key(&self) -> &Vec<u8> {
        &self.public_key
    }

    /// Generates a random signed transaction for testing purposes.
    pub fn get_random_signed_transaction() -> Self {
        // Generate a random transaction.
        let random_transaction = Transaction::generate_random_transaction();

        // Generate a random key pair.
        let key_pair = key_pair::random();

        // Sign the transaction with the generated key pair.
        let signature = sign(&random_transaction, &key_pair);

        // Create the signed transaction.
        SignedTransaction {
            transaction: random_transaction,
            signature,
            public_key: key_pair.public_key().as_ref().to_vec(),
        }
    }

    /// Generates a random signed transaction from the ICO
    pub fn get_random_signed_transaction_from_ico(nonce: u64) -> Self {
        // Generate a random key pair.
        let receiver_keypair: Ed25519KeyPair = key_pair::random();
        let reciever_addr = base64::encode(receiver_keypair.public_key());

        // Generate a random transaction from the ICO
        let random_transaction: Transaction =
            Transaction::generate_random_transaction_from_ico(nonce, reciever_addr);

        // Load the ICO's private key
        let ico_private_key_bytes = include_bytes!("key_pair.pem"); // Load the ICO's private key file
        let key_pair = Ed25519KeyPair::from_pkcs8(ico_private_key_bytes).unwrap();

        // Sign the transaction with the ICO's private key
        let signature = sign(&random_transaction, &key_pair);

        // ICO's public key
        let ico_public_key = key_pair.public_key();

        // Create the signed transaction
        SignedTransaction {
            transaction: random_transaction,
            signature,
            public_key: ico_public_key.as_ref().to_vec(),
        }
    }

    /// Verifies the digital signature of this signed transaction.
    pub fn verify_signed_transaction(&self) -> bool {
        verify(&self.transaction, &self.public_key, &self.signature)
    }

    /// Returns the sender of the transaction.
    pub fn get_sender(&self) -> &String {
        &self.transaction.sender
    }

    /// Returns the receiver of the transaction.
    pub fn get_receiver(&self) -> &String {
        &self.transaction.receiver
    }

    /// Returns the value of the transaction.
    pub fn get_value(&self) -> i64 {
        self.transaction.value
    }

    /// Returns the nonce of the transaction.
    pub fn get_nonce(&self) -> u64 {
        self.transaction.nonce
    }
}

impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        let encoded = bincode::serialize(&self).expect("failed to serialize");
        ring::digest::digest(&ring::digest::SHA256, &encoded).into()
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Vec<u8> {
    let bytes_to_sign: &[u8] = &bincode::serialize(t).unwrap();
    key.sign(&bytes_to_sign).as_ref().to_vec()
}

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    let message = bincode::serialize(t).unwrap(); // Serialize the transaction
    let peer_public_key =
        ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, public_key);
    peer_public_key.verify(message.as_ref(), signature).is_ok()
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    let mut rng = rand::thread_rng();
    let sender = format!("Sender{}", rng.gen::<u32>());
    let receiver = format!("Receiver{}", rng.gen::<u32>());
    let value = rng.gen::<i64>();
    let nonce = rng.gen::<u64>();

    Transaction {
        sender,
        receiver,
        value,
        nonce,
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
