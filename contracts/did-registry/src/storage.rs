use soroban_sdk::{contracttype, Address, BytesN, Env, String, Symbol};

use crate::types::{CredentialEntry, DidEntry};

// ---------------------------------------------------------------------------
// Ledger key definitions
// ---------------------------------------------------------------------------

/// All storage keys used by the did-registry contract.
#[contracttype]
#[derive(Clone, Debug)]
pub enum LedgerKey {
    /// Primary DID record keyed by the `did:stellar:G…` string.
    Did(String),
    /// Credential record keyed by its unique credential ID (SHA-256 hash).
    Credential(BytesN<32>),
    /// Index mapping (subject, credential_type) → credential_id.
    /// One active credential per type per subject.
    SubjectCredential(Address, Symbol),
}

// ---------------------------------------------------------------------------
// DID helpers
// ---------------------------------------------------------------------------

/// Read a DID entry from persistent storage. Returns `None` if not found.
pub fn read_did(env: &Env, did: &String) -> Option<DidEntry> {
    env.storage().persistent().get(&LedgerKey::Did(did.clone()))
}

/// Write (create or overwrite) a DID entry in persistent storage.
pub fn write_did(env: &Env, did: &String, entry: &DidEntry) {
    env.storage()
        .persistent()
        .set(&LedgerKey::Did(did.clone()), entry);
}

// ---------------------------------------------------------------------------
// Credential helpers
// ---------------------------------------------------------------------------

/// Read a credential entry by its ID. Returns `None` if not found.
pub fn read_credential(env: &Env, credential_id: &BytesN<32>) -> Option<CredentialEntry> {
    env.storage()
        .persistent()
        .get(&LedgerKey::Credential(credential_id.clone()))
}

/// Write (create or overwrite) a credential entry in persistent storage.
pub fn write_credential(env: &Env, credential_id: &BytesN<32>, entry: &CredentialEntry) {
    env.storage()
        .persistent()
        .set(&LedgerKey::Credential(credential_id.clone()), entry);
}

// ---------------------------------------------------------------------------
// Subject credential index helpers
// ---------------------------------------------------------------------------

/// Look up the active credential ID for a given (subject, credential_type) pair.
pub fn read_subject_credential(
    env: &Env,
    subject: &Address,
    credential_type: &Symbol,
) -> Option<BytesN<32>> {
    env.storage().persistent().get(&LedgerKey::SubjectCredential(
        subject.clone(),
        credential_type.clone(),
    ))
}

/// Record the active credential ID for a given (subject, credential_type) pair.
pub fn write_subject_credential(
    env: &Env,
    subject: &Address,
    credential_type: &Symbol,
    credential_id: &BytesN<32>,
) {
    env.storage().persistent().set(
        &LedgerKey::SubjectCredential(subject.clone(), credential_type.clone()),
        credential_id,
    );
}
