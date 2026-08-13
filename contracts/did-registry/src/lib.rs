#![no_std]

pub mod errors;
pub mod storage;
pub mod types;

use soroban_sdk::{
    contract, contractimpl, symbol_short, vec, Address, BytesN, Env, String, Symbol, Vec,
};

use crate::{
    errors::ContractError,
    storage::{read_did, write_did},
    types::DidEntry,
};

#[contract]
pub struct DidRegistry;

#[contractimpl]
impl DidRegistry {
    /// Returns a greeting vector: ["Hello", to].
    /// Smoke-test entry point — verifies the contract deploys and executes correctly.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }

    /// Register a DID and anchor its document hash on-chain.
    ///
    /// `owner` must sign the transaction (`require_auth` is called on them).
    /// Only the account owner may register their own DID.
    ///
    /// `created_at` is set to `env.ledger().timestamp()`.
    ///
    /// # Errors
    /// - [`ContractError::AlreadyExists`] — a DID with this identifier is
    ///   already registered on-chain.
    pub fn register(
        env: Env,
        owner: Address,
        did: String,
        document_hash: BytesN<32>,
    ) -> Result<(), ContractError> {
        // Only the owner may register their own DID.
        owner.require_auth();

        // Reject if the DID is already registered.
        if read_did(&env, &did).is_some() {
            return Err(ContractError::AlreadyExists);
        }

        let entry = DidEntry {
            owner,
            document_hash,
            created_at: env.ledger().timestamp(),
        };

        write_did(&env, &did, &entry);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{symbol_short, vec, Address, Env, String};

    use crate::{
        storage::{
            read_credential, read_did, read_subject_credential, write_credential, write_did,
            write_subject_credential,
        },
        types::{CredentialEntry, DidEntry},
    };

    // -----------------------------------------------------------------------
    // hello smoke test
    // -----------------------------------------------------------------------

    #[test]
    fn test_hello() {
        let env = Env::default();
        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let result = client.hello(&symbol_short!("World"));
        assert_eq!(
            result,
            vec![&env, symbol_short!("Hello"), symbol_short!("World")]
        );
    }

    // -----------------------------------------------------------------------
    // Storage round-trip tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_did_storage_round_trip() {
        let env = Env::default();
        let contract_id = env.register(DidRegistry, ());
        let did = String::from_str(&env, "did:stellar:GABC1234");
        let owner = Address::generate(&env);
        let doc_hash: soroban_sdk::BytesN<32> = env
            .crypto()
            .sha256(&soroban_sdk::Bytes::from_slice(&env, &[1u8; 64]))
            .into();

        env.as_contract(&contract_id, || {
            assert!(read_did(&env, &did).is_none());

            let entry = DidEntry {
                owner: owner.clone(),
                document_hash: doc_hash.clone(),
                created_at: 1_000_000,
            };
            write_did(&env, &did, &entry);

            let stored = read_did(&env, &did).expect("DID should exist after write");
            assert_eq!(stored.owner, owner);
            assert_eq!(stored.document_hash, doc_hash);
            assert_eq!(stored.created_at, 1_000_000);
        });
    }

    #[test]
    fn test_credential_storage_round_trip() {
        let env = Env::default();
        let contract_id = env.register(DidRegistry, ());
        let issuer = Address::generate(&env);
        let subject = Address::generate(&env);
        let cred_type = symbol_short!("KYC");
        let cred_id: soroban_sdk::BytesN<32> = env
            .crypto()
            .sha256(&soroban_sdk::Bytes::from_slice(&env, &[2u8; 64]))
            .into();
        let cred_hash: soroban_sdk::BytesN<32> = env
            .crypto()
            .sha256(&soroban_sdk::Bytes::from_slice(&env, &[3u8; 64]))
            .into();

        env.as_contract(&contract_id, || {
            assert!(read_credential(&env, &cred_id).is_none());

            let entry = CredentialEntry {
                issuer: issuer.clone(),
                subject: subject.clone(),
                credential_type: cred_type.clone(),
                credential_hash: cred_hash.clone(),
                issued_at: 2_000_000,
                expires_at: Some(9_999_999),
                revoked: false,
            };
            write_credential(&env, &cred_id, &entry);

            let stored =
                read_credential(&env, &cred_id).expect("credential should exist after write");
            assert_eq!(stored.issuer, issuer);
            assert_eq!(stored.subject, subject);
            assert_eq!(stored.credential_type, cred_type);
            assert_eq!(stored.expires_at, Some(9_999_999));
            assert!(!stored.revoked);
        });
    }

    #[test]
    fn test_subject_credential_index_round_trip() {
        let env = Env::default();
        let contract_id = env.register(DidRegistry, ());
        let subject = Address::generate(&env);
        let cred_type = symbol_short!("KYC");
        let cred_id: soroban_sdk::BytesN<32> = env
            .crypto()
            .sha256(&soroban_sdk::Bytes::from_slice(&env, &[4u8; 64]))
            .into();

        env.as_contract(&contract_id, || {
            assert!(read_subject_credential(&env, &subject, &cred_type).is_none());

            write_subject_credential(&env, &subject, &cred_type, &cred_id);

            let stored = read_subject_credential(&env, &subject, &cred_type)
                .expect("index entry should exist after write");
            assert_eq!(stored, cred_id);
        });
    }

    // -----------------------------------------------------------------------
    // register() integration tests
    // -----------------------------------------------------------------------

    fn make_doc_hash(env: &Env, seed: u8) -> BytesN<32> {
        env.crypto()
            .sha256(&soroban_sdk::Bytes::from_slice(env, &[seed; 64]))
            .into()
    }

    /// Happy path: DID is stored with the correct owner, document hash, and
    /// created_at timestamp.
    #[test]
    fn test_register_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(5_000_000);

        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let did = String::from_str(&env, "did:stellar:GABC5678");
        let doc_hash = make_doc_hash(&env, 1);

        client
            .try_register(&owner, &did, &doc_hash)
            .expect("call should not panic")
            .expect("register should succeed");

        // Verify the stored entry directly.
        env.as_contract(&contract_id, || {
            let entry = read_did(&env, &did).expect("DID entry should exist after register");
            assert_eq!(entry.owner, owner);
            assert_eq!(entry.document_hash, doc_hash);
            assert_eq!(entry.created_at, 5_000_000);
        });
    }

    /// Registering the same DID a second time must return
    /// `ContractError::AlreadyExists`.
    #[test]
    fn test_register_duplicate_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let did = String::from_str(&env, "did:stellar:GDUP9999");
        let doc_hash = make_doc_hash(&env, 2);

        // First registration — must succeed.
        client
            .try_register(&owner, &did, &doc_hash)
            .expect("call should not panic")
            .expect("first registration should succeed");

        // Second registration of the same DID — must fail.
        let result = client.try_register(&owner, &did, &doc_hash);

        // try_* returns Result<Result<T, ConversionError>, Result<ContractError, InvokeError>>.
        // A contract-returned error surfaces as Err(Ok(ContractError)).
        match result {
            Err(Ok(err)) => assert_eq!(err, crate::errors::ContractError::AlreadyExists),
            other => panic!("expected Err(Ok(AlreadyExists)), got {:?}", other),
        }
    }
}
