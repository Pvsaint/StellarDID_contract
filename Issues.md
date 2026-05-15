Issue #1 — Initialise Soroban contract workspace
Labels: setup, contract
Description:
Set up the base Cargo workspace for the did-registry Soroban contract with the correct project structure, dependencies, and build targets.
Acceptance Criteria:

 Cargo.toml workspace configured at repo root
 contracts/did-registry/ created with soroban-sdk as a dependency
 wasm32-unknown-unknown target compiles without errors (stellar contract build)
 Empty contract with a single hello function passes cargo test
 .cargo/config.toml sets default build target to wasm32-unknown-unknown


Issue #2 — Define contract types and storage keys
Labels: setup, contract
Description:
Create the core Rust types and ledger storage key definitions that the registry contract will use across all functions.
Acceptance Criteria:

 src/types.rs defines DidEntry and CredentialEntry structs
 src/errors.rs defines ContractError enum with variants for NotFound, Unauthorized, AlreadyExists, Expired
 src/storage.rs defines LedgerKey enum with Did, Credential, and SubjectCredential variants
 Storage helpers (read_did, write_did, read_credential, write_credential) implemented and unit tested
 All types derive Clone, Debug, and implement Soroban's IntoVal/FromVal traits


Issue #3 — Set up contract test harness
Labels: setup, contract, tests
Description:
Configure the integration test environment so contributors can write and run tests against the contract using Soroban's test utilities.
Acceptance Criteria:

 contracts/did-registry/tests/integration.rs created
 soroban-sdk test environment (Env::default()) initialised in a shared test fixture
 A helper deploy_contract() function available to all tests
 At least one smoke test that deploys the contract and calls a no-op function
 cargo test runs and passes with a clear output


Issue #4 — Configure testnet deployment script
Labels: setup, contract, devops
Description:
Add a shell script (or Makefile targets) that automates building and deploying the contract to Stellar testnet.
Acceptance Criteria:

 scripts/deploy.sh accepts --network flag (testnet | mainnet)
 Script runs stellar contract build before deploying
 Deployed contract ID is written to .contract-ids/<network>.txt for use by backend and frontend
 Script documented in contract README.md under "Deploying to Testnet"
 Works with a funded Stellar CLI account alias