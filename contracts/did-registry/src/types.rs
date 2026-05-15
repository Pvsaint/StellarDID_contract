use soroban_sdk::{contracttype, Address, BytesN, Symbol};

/// On-chain record for a registered DID.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DidEntry {
    /// The Stellar account that owns and controls this DID.
    pub owner: Address,
    /// SHA-256 hash of the DID Document stored off-chain (IPFS).
    pub document_hash: BytesN<32>,
    /// Ledger timestamp at registration time.
    pub created_at: u64,
}

/// On-chain record for an issued verifiable credential.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CredentialEntry {
    /// The address that issued this credential.
    pub issuer: Address,
    /// The DID subject this credential was issued to.
    pub subject: Address,
    /// Credential type identifier, e.g. `KYC_VERIFIED`, `ACCREDITED_INVESTOR`.
    pub credential_type: Symbol,
    /// SHA-256 hash of the full VC document stored off-chain (IPFS).
    pub credential_hash: BytesN<32>,
    /// Ledger timestamp at issuance time.
    pub issued_at: u64,
    /// Optional expiry timestamp. `None` means the credential does not expire.
    pub expires_at: Option<u64>,
    /// Whether the issuer has revoked this credential.
    pub revoked: bool,
}
