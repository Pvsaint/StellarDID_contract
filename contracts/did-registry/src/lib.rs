#![no_std]

pub mod errors;
pub mod storage;
pub mod types;

use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct DidRegistry;

#[contractimpl]
impl DidRegistry {
    /// Returns a greeting vector: ["Hello", to].
    /// Smoke-test entry point — verifies the contract deploys and executes correctly.
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, vec, Address, Env, String};
    use soroban_sdk::testutils::Address as _;

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
        let doc_hash: soroban_sdk::BytesN<32> =
            env.crypto().sha256(&soroban_sdk::Bytes::from_slice(&env, &[1u8; 64])).into();

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
        let cred_id: soroban_sdk::BytesN<32> =
            env.crypto().sha256(&soroban_sdk::Bytes::from_slice(&env, &[2u8; 64])).into();
        let cred_hash: soroban_sdk::BytesN<32> =
            env.crypto().sha256(&soroban_sdk::Bytes::from_slice(&env, &[3u8; 64])).into();

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
        let cred_id: soroban_sdk::BytesN<32> =
            env.crypto().sha256(&soroban_sdk::Bytes::from_slice(&env, &[4u8; 64])).into();

        env.as_contract(&contract_id, || {
            assert!(read_subject_credential(&env, &subject, &cred_type).is_none());

            write_subject_credential(&env, &subject, &cred_type, &cred_id);

            let stored = read_subject_credential(&env, &subject, &cred_type)
                .expect("index entry should exist after write");
            assert_eq!(stored, cred_id);
        });
    }
}
