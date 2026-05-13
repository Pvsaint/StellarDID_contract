# StellarDID

**Self-sovereign identity infrastructure for the Stellar ecosystem.**

A Soroban-powered decentralized identity (DID) registry that lets users own their credentials, issuers publish verifiable claims, and smart contracts gate access on verified identity — without ever centralizing personal data.

---

## Problem

Stellar is one of the most active blockchain networks for real-world financial infrastructure — cross-border payments, stablecoins, tokenized assets, and regulated financial services. But identity remains the missing layer.

Today, every application on Stellar handles identity independently:

- **Anchors re-verify the same users** across different services, creating redundant KYC friction and storing sensitive PII in multiple places
- **Regulated asset contracts have no composable way to enforce compliance** — there is no on-chain primitive that says "this address is a verified accredited investor" that other contracts can trustlessly read
- **Users have no portable credential layer** — completing verification on one platform gives you nothing on the next
- **Sybil attacks are trivial** in Stellar-based governance and DAO structures, since one person can control many accounts with no identity binding
- **No `did:stellar:` method exists** — Stellar is unregistered in the W3C DID registry, leaving developers without a standardized identity primitive to build on

The result is a fragmented ecosystem where every project that touches identity rebuilds the same trust infrastructure from scratch, or outsources it to centralized providers — defeating the point of decentralization.

---

## Solution

StellarDID provides a standards-compliant, composable identity layer built natively on Stellar:

**A Soroban DID Registry Contract** — the on-chain source of truth. Any Stellar address can register a DID, anchor a DID Document hash, and manage its associated credentials. The registry is permissionless and non-custodial: the contract never holds private keys or PII.

**Verifiable Credential Issuance** — trusted issuers (anchors, KYC providers, DAOs) can publish signed credential claims against a subject's DID. Credentials are stored off-chain (IPFS/Arweave) with only the hash and metadata anchored on Stellar, keeping sensitive data off the ledger while preserving tamper-evidence.

**On-Chain Revocation** — issuers can revoke credentials at any time. Any contract or application querying `verify()` gets the current revocation status, not a stale snapshot.

**Composable `verify()` Primitive** — other Soroban contracts can call `verify(address, credential_type)` as a single-line gate. A token transfer contract, a DAO voting contract, or a lending protocol can all enforce identity requirements without building their own identity stack.

**A `did:stellar:` Method Spec** — StellarDID ships with a draft W3C DID method specification for `did:stellar:`, giving the ecosystem a standardized identifier format that maps cleanly to Stellar's account model.

**Credential Management UI** — a minimal web frontend where holders view and share their credentials, issuers publish claims, and verifiers can inspect a DID's current status — no command line required.

### How It Works

```
1. Holder registers DID
   └─ Stellar account G... → did:stellar:G...
   └─ DID Document (keys, service endpoints) → IPFS
   └─ Document hash anchored via registry contract

2. Issuer publishes credential
   └─ Issuer signs VC (e.g. "KYC_VERIFIED", "ACCREDITED_INVESTOR")
   └─ VC stored on IPFS
   └─ Credential hash + metadata registered on-chain
   └─ Subject notified via Stellar memo or SEP-0006 callback

3. Contract verifies identity
   └─ contract.call("transfer", amount, recipient)
   └─ internally: registry.verify(recipient, "KYC_VERIFIED") → true/false
   └─ transfer proceeds or reverts based on result

4. Revocation
   └─ Issuer calls registry.revoke(credential_id)
   └─ verify() immediately returns false for revoked credentials
```

---

## Stack

### Smart Contracts
| Component | Technology |
|---|---|
| DID Registry | Soroban (Rust) on Stellar |
| Credential issuance & revocation | Soroban contract storage + events |
| Verification primitive | Soroban cross-contract calls |
| Testing | `soroban-sdk` test harness, Stellar testnet |

### Off-Chain Storage
| Component | Technology |
|---|---|
| DID Documents | IPFS via `ipfs-http-client` |
| Verifiable Credentials | IPFS (content-addressed, tamper-evident) |
| Pinning | Pinata (free tier for MVP) |

### Frontend
| Component | Technology |
|---|---|
| Framework | Next.js 14 (App Router) + TypeScript |
| Styling | Tailwind CSS |
| Stellar SDK | `@stellar/stellar-sdk` v12 |
| Wallet | Freighter via `@stellar/freighter-api` |
| Deployment | Vercel |

### Standards Compliance
| Standard | Role |
|---|---|
| W3C DID Core 1.0 | DID Document structure, resolution |
| W3C Verifiable Credentials 1.1 | Credential schema and proof format |
| `did:stellar:` method spec | Identifier format (drafted as part of this project) |
| Stellar SEP-0030 | Account recovery model compatibility |

---

## Contract Interface (Draft)

```rust
// Register a DID and anchor its document hash
fn register(env: Env, did: String, document_hash: BytesN<32>);

// Issue a credential against a subject DID
fn issue_credential(
    env: Env,
    subject: Address,
    credential_type: Symbol,
    credential_hash: BytesN<32>,
    expiry: Option<u64>,
);

// Revoke a previously issued credential
fn revoke_credential(env: Env, credential_id: BytesN<32>);

// Composable verification — callable by other contracts
fn verify(env: Env, subject: Address, credential_type: Symbol) -> bool;

// Resolve a DID to its document hash
fn resolve(env: Env, did: String) -> Option<BytesN<32>>;
```

---

## Roadmap

**v0.1 — MVP (Drips submission)**
- [ ] Soroban DID registry contract (register, issue, revoke, verify)
- [ ] `did:stellar:` method draft spec
- [ ] IPFS document storage + hash anchoring
- [ ] Credential management UI (Freighter-connected)
- [ ] One end-to-end demo: token-gated page behind a Stellar DID credential
- [ ] Full test suite on testnet

**v0.2**
- [ ] Guardian-based DID recovery (leveraging Stellar's multi-signer model)
- [ ] Credential schema registry (standardized VC types for the ecosystem)
- [ ] SEP-0006 anchor integration (anchors as credential issuers)
- [ ] SDK: `stellar-did-js` — TypeScript client for issuers and verifiers

**v0.3**
- [ ] W3C DID method submission
- [ ] Mainnet deployment
- [ ] Integration guides for Stellar anchors

---

## Why Stellar

Stellar's account model maps naturally to decentralized identity in ways EVM chains do not:

- **Multi-signer accounts** → guardian recovery without smart contract complexity
- **Data entries** → lightweight claim anchoring at the account level
- **Low fees** → credential operations stay economically viable at scale
- **Existing anchor network** → ready-made issuer infrastructure via SEP-compliant anchors
- **SEP-0030** → recovery standard already aligned with DID key management patterns

---

## License

Apache 2.0
