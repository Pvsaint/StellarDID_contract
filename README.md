# StellarDID — Contract

[![CI](https://github.com/Pvsaint/StellarDID_contract/actions/workflows/ci.yml/badge.svg)](https://github.com/Pvsaint/StellarDID_contract/actions/workflows/ci.yml)

> The on-chain identity registry powering StellarDID, built with Soroban on Stellar.

This package contains the `did-registry` Soroban smart contract — the source of truth for DID registrations, verifiable credential issuance, revocation, and composable on-chain verification.

---

## Overview

The registry contract handles four core responsibilities:

- **DID Registration** — binds a Stellar address to a `did:stellar:G...` identifier and anchors a DID Document hash on-chain
- **Credential Issuance** — lets trusted issuers publish verifiable credential claims against a subject's DID
- **Revocation** — lets issuers invalidate credentials at any time, with immediate effect on `verify()`
- **Composable Verification** — exposes a `verify(subject, credential_type)` function that any other Soroban contract can call as an identity gate

No PII is ever stored on-chain. Only content hashes (SHA-256) are written to the ledger. The full DID Documents and Verifiable Credentials live off-chain on IPFS.

---

## Contract Interface

```rust
/// Register a DID and anchor its document hash on-chain.
/// Can only be called by the account owner.
pub fn register(env: Env, did: String, document_hash: BytesN<32>);

/// Update a DID Document hash.
/// Can only be called by the DID owner.
pub fn update(env: Env, did: String, new_document_hash: BytesN<32>);

/// Issue a verifiable credential against a subject DID.
/// Returns the generated credential_id.
pub fn issue_credential(
    env: Env,
    subject: Address,
    credential_type: Symbol,
    credential_hash: BytesN<32>,
    expires_at: Option<u64>,
) -> BytesN<32>;

/// Revoke a credential by ID.
/// Can only be called by the original issuer.
pub fn revoke_credential(env: Env, credential_id: BytesN<32>);

/// Verify that a subject holds a valid, non-revoked, non-expired credential
/// of the given type. Callable by any Soroban contract.
pub fn verify(env: Env, subject: Address, credential_type: Symbol) -> bool;

/// Resolve a DID to its anchored document hash.
pub fn resolve(env: Env, did: String) -> Option<BytesN<32>>;
```

---

## Storage Layout

```rust
// DID entry
LedgerKey::Did(did: String)
  → DidEntry {
      owner: Address,
      document_hash: BytesN<32>,
      created_at: u64,
    }

// Credential entry
LedgerKey::Credential(credential_id: BytesN<32>)
  → CredentialEntry {
      issuer: Address,
      subject: Address,
      credential_type: Symbol,
      credential_hash: BytesN<32>,
      issued_at: u64,
      expires_at: Option<u64>,
      revoked: bool,
    }

// Subject credential index (lookup by subject + type)
LedgerKey::SubjectCredential(subject: Address, credential_type: Symbol)
  → BytesN<32>  // credential_id
```

---

## Cross-Contract Usage

The `verify()` function is designed to be called by other Soroban contracts:

```rust
use stellardid_registry::Client as DidRegistry;

fn transfer(env: Env, from: Address, to: Address, amount: i128) {
    let registry = DidRegistry::new(&env, &registry_contract_id);

    assert!(
        registry.verify(&to, &Symbol::new(&env, "KYC_VERIFIED")),
        "Recipient is not KYC verified"
    );

    // proceed with transfer...
}
```

---

## Project Structure

```
contracts/did-registry/
├── src/
│   ├── lib.rs          # Contract entry points and public interface
│   ├── storage.rs      # Ledger key definitions and read/write helpers
│   ├── types.rs        # DidEntry, CredentialEntry, ContractError
│   └── errors.rs       # ContractError enum
├── tests/
│   └── integration.rs  # Full lifecycle integration tests
└── Cargo.toml
```

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) stable 1.74+
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)

```bash
cargo install --locked stellar-cli
rustup target add wasm32-unknown-unknown
```

---

## Local Setup

```bash
# Clone the repo
git clone https://github.com/YOUR_USERNAME/stellardid_contract.git
cd stellardid_contract

# Build
stellar contract build

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

---

## Deploying to Testnet

```bash
# Generate a testnet account if you don't have one
stellar keys generate --global alice --network testnet

# Fund it
stellar keys fund alice --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/did_registry.wasm \
  --network testnet \
  --source alice

# Save the returned contract ID — you'll need it for the backend and frontend
```

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](./CONTRIBUTING.md) for the full guide covering prerequisites, fork & clone, branch naming, running tests, formatting, commit message conventions, the PR checklist, and security disclosure process.
