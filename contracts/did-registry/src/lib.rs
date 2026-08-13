#![no_std]

pub mod errors;
pub mod storage;
pub mod types;

use soroban_sdk::{
    contract, contractimpl, symbol_short, vec, Address, Bytes, BytesN, Env, Symbol, Vec,
};

use crate::{
    errors::ContractError,
    storage::{
        read_credential, read_subject_credential, write_credential, write_subject_credential,
    },
    types::CredentialEntry,
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

    /// Issue a verifiable credential against a subject's DID.
    ///
    /// `issuer` must sign the transaction (`require_auth` is called on them).
    ///
    /// The credential ID is derived deterministically as:
    ///   `sha256(issuer_xdr || subject_xdr || credential_type_xdr || timestamp_le_bytes)`
    ///
    /// Returns the `credential_id` (a `BytesN<32>`) on success.
    ///
    /// # Errors
    /// - [`ContractError::AlreadyExists`] — a non-revoked credential of the same
    ///   type already exists for this subject. Re-issuance is only permitted once
    ///   the previous credential has been revoked.
    pub fn issue_credential(
        env: Env,
        issuer: Address,
        subject: Address,
        credential_type: Symbol,
        credential_hash: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<BytesN<32>, ContractError> {
        // The issuer must authorise this transaction.
        issuer.require_auth();

        // Guard: reject if a non-revoked credential of this type already exists
        // for the subject.
        if let Some(existing_id) = read_subject_credential(&env, &subject, &credential_type) {
            if let Some(existing) = read_credential(&env, &existing_id) {
                if !existing.revoked {
                    return Err(ContractError::AlreadyExists);
                }
            }
        }

        let issued_at = env.ledger().timestamp();

        // Derive a deterministic credential ID from the key inputs.
        let credential_id =
            Self::derive_credential_id(&env, &issuer, &subject, &credential_type, issued_at);

        let entry = CredentialEntry {
            issuer,
            subject: subject.clone(),
            credential_type: credential_type.clone(),
            credential_hash,
            issued_at,
            expires_at,
            revoked: false,
        };

        // Persist the credential record and update the subject index.
        write_credential(&env, &credential_id, &entry);
        write_subject_credential(&env, &subject, &credential_type, &credential_id);

        Ok(credential_id)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Derives a deterministic credential ID:
    ///   `sha256(issuer_xdr || subject_xdr || ctype_xdr || timestamp_le_bytes)`
    fn derive_credential_id(
        env: &Env,
        issuer: &Address,
        subject: &Address,
        credential_type: &Symbol,
        timestamp: u64,
    ) -> BytesN<32> {
        use soroban_sdk::xdr::ToXdr;

        let issuer_bytes = issuer.to_xdr(env);
        let subject_bytes = subject.to_xdr(env);
        let ctype_bytes = credential_type.to_xdr(env);
        let ts_bytes = Bytes::from_slice(env, &timestamp.to_le_bytes());

        let mut preimage = Bytes::new(env);
        preimage.append(&issuer_bytes);
        preimage.append(&subject_bytes);
        preimage.append(&ctype_bytes);
        preimage.append(&ts_bytes);

        env.crypto().sha256(&preimage).into()
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
    // issue_credential integration tests
    // -----------------------------------------------------------------------

    fn make_cred_hash(env: &Env, seed: u8) -> BytesN<32> {
        env.crypto()
            .sha256(&soroban_sdk::Bytes::from_slice(env, &[seed; 64]))
            .into()
    }

    /// Happy path: credential is stored correctly and the subject index is updated.
    #[test]
    fn test_issue_credential_happy_path() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let subject = Address::generate(&env);
        let cred_type = symbol_short!("KYC");
        let cred_hash = make_cred_hash(&env, 10);

        let cred_id = client
            .try_issue_credential(&issuer, &subject, &cred_type, &cred_hash, &None)
            .expect("call should not panic")
            .expect("contract should return Ok");

        // The returned credential_id must be a non-zero 32-byte value.
        assert_ne!(cred_id, BytesN::from_array(&env, &[0u8; 32]));

        // The subject index must point to the new credential.
        env.as_contract(&contract_id, || {
            let indexed_id = read_subject_credential(&env, &subject, &cred_type)
                .expect("subject index should be set");
            assert_eq!(indexed_id, cred_id);

            // The credential entry must be stored with correct fields.
            let entry = read_credential(&env, &cred_id).expect("credential entry should be stored");
            assert_eq!(entry.issuer, issuer);
            assert_eq!(entry.subject, subject);
            assert_eq!(entry.credential_type, cred_type);
            assert_eq!(entry.credential_hash, cred_hash);
            assert!(!entry.revoked);
            assert_eq!(entry.expires_at, None);
        });
    }

    /// A credential with an expiry is stored with the correct `expires_at` and `issued_at`.
    #[test]
    fn test_issue_credential_with_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let subject = Address::generate(&env);
        let cred_type = symbol_short!("ACC");
        let cred_hash = make_cred_hash(&env, 20);
        let expiry: u64 = 9_999_999;

        let cred_id = client
            .try_issue_credential(&issuer, &subject, &cred_type, &cred_hash, &Some(expiry))
            .expect("call should not panic")
            .expect("contract should return Ok");

        env.as_contract(&contract_id, || {
            let entry = read_credential(&env, &cred_id).unwrap();
            assert_eq!(entry.expires_at, Some(expiry));
            assert_eq!(entry.issued_at, 1_000_000);
        });
    }

    /// Issuing a second credential of the same type to the same subject while
    /// the first is still active must return `ContractError::AlreadyExists`.
    #[test]
    fn test_issue_credential_duplicate_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let subject = Address::generate(&env);
        let cred_type = symbol_short!("KYC");
        let cred_hash = make_cred_hash(&env, 30);

        // First issuance — must succeed.
        client
            .try_issue_credential(&issuer, &subject, &cred_type, &cred_hash, &None)
            .expect("call should not panic")
            .expect("first issuance should succeed");

        // Second issuance of the same type while the first is active — must fail.
        let result = client.try_issue_credential(&issuer, &subject, &cred_type, &cred_hash, &None);

        // try_* returns Result<Result<T, ConversionError>, Result<ContractError, InvokeError>>.
        // A contract-returned error surfaces as Err(Ok(ContractError)).
        match result {
            Err(Ok(err)) => assert_eq!(err, ContractError::AlreadyExists),
            other => panic!("expected Err(Ok(AlreadyExists)), got {:?}", other),
        }
    }

    /// After a credential is revoked, re-issuance of the same type must succeed
    /// and the subject index must point to the new credential.
    #[test]
    fn test_issue_credential_allowed_after_revocation() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let contract_id = env.register(DidRegistry, ());
        let client = DidRegistryClient::new(&env, &contract_id);

        let issuer = Address::generate(&env);
        let subject = Address::generate(&env);
        let cred_type = symbol_short!("KYC");
        let cred_hash = make_cred_hash(&env, 40);

        // Issue initial credential at t=1_000_000.
        let first_id = client
            .try_issue_credential(&issuer, &subject, &cred_type, &cred_hash, &None)
            .expect("call should not panic")
            .expect("first issuance should succeed");

        // Manually revoke it so we can test re-issuance.
        env.as_contract(&contract_id, || {
            let mut entry = read_credential(&env, &first_id).unwrap();
            entry.revoked = true;
            write_credential(&env, &first_id, &entry);
        });

        // Advance the ledger timestamp so the derived credential_id differs.
        env.ledger().set_timestamp(2_000_000);

        // Re-issuance must now succeed.
        let second_id = client
            .try_issue_credential(&issuer, &subject, &cred_type, &cred_hash, &None)
            .expect("call should not panic")
            .expect("re-issuance after revocation should succeed");

        assert_ne!(first_id, second_id, "new credential_id must differ");

        env.as_contract(&contract_id, || {
            // Subject index must now point to the new credential.
            let indexed = read_subject_credential(&env, &subject, &cred_type).unwrap();
            assert_eq!(indexed, second_id);
        });
    }
}
