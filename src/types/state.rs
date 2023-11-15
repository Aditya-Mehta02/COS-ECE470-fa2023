use ring::signature::{Ed25519KeyPair, KeyPair};

use crate::types::transaction::SignedTransaction;
use std::{collections::HashMap, vec};

use super::address::Address;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountAddress(String); // Replace with your own account address type if necessary

#[derive(Debug, Clone)]
pub struct AccountInfo {
    nonce: u64,    // Nonce of the account
    balance: u128, // Balance of the account
}

impl AccountInfo {
    // Public method to get the nonce
    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }

    // Public method to get the balance
    pub fn get_balance(&self) -> u128 {
        self.balance
    }
}

#[derive(Debug, Clone)]
pub struct State {
    accounts: HashMap<AccountAddress, AccountInfo>,
}

use std::fmt;

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl State {
    // Constructor to create a new State
    pub fn new() -> Self {
        let mut state = Self {
            accounts: HashMap::new(),
        };

        // Call the desired function here
        state.initialize_default_accounts();

        state
    }

    fn initialize_default_accounts(&mut self) {
        // Load the ICO's private key
        let ico_private_key_bytes = include_bytes!("key_pair.pem"); // Load the ICO's private key file
        let key_pair = Ed25519KeyPair::from_pkcs8(ico_private_key_bytes).unwrap();

        // ICO's public key
        let ico_public_key = key_pair.public_key();

        // Encode the public key in a readable format (e.g., Base64)
        let ico_public_key_string = base64::encode(ico_public_key);

        self.add_account_with_balance(AccountAddress(ico_public_key_string), 200000)
    }

    pub fn get_accounts(&self) -> &HashMap<AccountAddress, AccountInfo> {
        &self.accounts
    }

    // Function to add or update an account in the state
    pub fn update_account(&mut self, address: AccountAddress, nonce: u64, balance: u128) {
        let account_info = AccountInfo { nonce, balance };
        self.accounts.insert(address, account_info);
    }

    // Function to get account information
    pub fn get_account(&self, address: &AccountAddress) -> Option<&AccountInfo> {
        self.accounts.get(address)
    }

    // Function to add a new account with a public key and balance
    pub fn add_account_with_balance(&mut self, address: AccountAddress, balance: u128) {
        let account_info = AccountInfo { nonce: 0, balance };
        self.accounts.insert(address, account_info);
    }

    pub fn apply_transaction(&mut self, tx: &SignedTransaction) -> Result<(), String> {
        // Verify the signature of the transaction
        if !tx.verify_signed_transaction() {
            return Err("Invalid transaction signature".to_string());
        }

        let sender_address = AccountAddress(tx.get_sender().clone());
        let receiver_address = AccountAddress(tx.get_receiver().clone());
        let value = tx.get_value() as u128;
        let sender_nonce = tx.get_nonce();

        // Check for sufficient funds and correct nonce
        if let Some(sender_info) = self.accounts.get(&sender_address) {
            println!(
                "balance: {}, value: {}, nonce: {}, sender_nonce: {}",
                sender_info.balance, value, sender_info.nonce, sender_nonce
            );
            // if sender_info.balance < value || sender_info.nonce != sender_nonce {
            if sender_info.balance < value {
                return Err("Insufficient funds or incorrect nonce".to_string());
            }
        } else {
            return Err("Sender account does not exist".to_string());
        }

        // Update sender's balance and nonce
        let sender_info = self.accounts.get_mut(&sender_address).unwrap();
        sender_info.balance -= value;
        sender_info.nonce += 1;

        // Update receiver's balance
        let receiver_info: &mut AccountInfo =
            self.accounts
                .entry(receiver_address)
                .or_insert_with(|| AccountInfo {
                    nonce: 0,
                    balance: 0,
                });
        receiver_info.balance += value;

        Ok(())
    }

    // Function to check if a transaction is valid given the current state
    pub fn is_transaction_valid(&self, tx: &SignedTransaction) -> bool {
        // Verify the signature of the transaction
        if !tx.verify_signed_transaction() {
            return false;
        }

        let sender_address = AccountAddress(tx.get_sender().clone());
        let value = tx.get_value() as u128;
        let sender_nonce = tx.get_nonce();

        if let Some(sender_info) = self.accounts.get(&sender_address) {
            // Check for sufficient balance and correct nonce
            sender_info.balance >= value && sender_info.nonce == sender_nonce
        } else {
            // Sender account does not exist
            false
        }
    }
}
