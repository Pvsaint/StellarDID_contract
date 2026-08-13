# Contributing to StellarDID — Contract

Thank you for your interest in contributing! This guide covers everything you need to go from zero to an open pull request.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Prerequisites](#prerequisites)
3. [Fork & Clone](#fork--clone)
4. [Branch Naming](#branch-naming)
5. [Making Changes](#making-changes)
6. [Running Tests](#running-tests)
7. [Formatting & Linting](#formatting--linting)
8. [Commit Messages](#commit-messages)
9. [Opening a Pull Request](#opening-a-pull-request)
10. [Security Disclosures](#security-disclosures)

---

## Code of Conduct

We are committed to a welcoming, respectful, and harassment-free environment for everyone. By participating you agree to treat all contributors with professionalism and kindness. Unacceptable behaviour should be reported directly to the maintainers — do not open a public issue.

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Rust](https://www.rust-lang.org/tools/install) | stable 1.74+ | `rustup update stable` |
| `wasm32-unknown-unknown` target | — | `rustup target add wasm32-unknown-unknown` |
| [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) | latest | `cargo install --locked stellar-cli` |

Verify your setup:

```bash
rustc --version          # rustc 1.74.0 or newer
stellar --version        # stellar 0.x.x or newer
```

---

## Fork & Clone

1. **Fork** the repo on GitHub (top-right "Fork" button).
2. **Clone** your fork locally:

```bash
git clone https://github.com/<your-username>/StellarDID_contract.git
cd StellarDID_contract
```

3. **Add the upstream remote** so you can stay in sync:

```bash
git remote add upstream https://github.com/Pvsaint/StellarDID_contract.git
```

4. **Sync before starting new work**:

```bash
git fetch upstream
git rebase upstream/main
```

---

## Branch Naming

Create a branch off `main` using one of these prefixes:

| Prefix | When to use |
|--------|------------|
| `feat/` | New feature or function |
| `fix/` | Bug fix |
| `docs/` | Documentation only |
| `test/` | Tests only |
| `refactor/` | Code change with no behaviour change |
| `chore/` | Tooling, CI, dependency updates |

```bash
git checkout -b feat/credential-expiry-enforcement
git checkout -b fix/verify-expired-credentials
git checkout -b docs/contributing-guide
```

Keep the description short and hyphen-separated. One branch per issue.

---

## Making Changes

- Keep each PR focused — **one issue per PR**.
- Store all storage logic in `storage.rs`, not `lib.rs`.
- All public functions must have a doc comment that explains:
  - What the function does
  - Who is authorised to call it
  - What it returns

```rust
/// Registers a new DID and anchors its document hash on-chain.
/// Can only be called by the account owner (`did` must map to `env.invoker()`).
/// Panics with `ContractError::AlreadyRegistered` if the DID already exists.
pub fn register(env: Env, did: String, document_hash: BytesN<32>) { ... }
```

---

## Running Tests

Use the `make` targets — they set the correct native host target automatically:

```bash
# Run the full test suite
make test

# Run tests with printed output (useful for debugging)
cargo test --target $(rustc -vV | sed -n 's/^host: //p') -- --nocapture
```

All tests must pass before you open a PR. If you are fixing a bug, add a regression test that would have caught it.

---

## Formatting & Linting

Run both before every commit:

```bash
# Auto-format all Rust code
make fmt

# Check for Clippy warnings (treated as errors in CI)
make lint
```

CI will fail on any `cargo clippy` warning, so resolve them locally first.

---

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>(<scope>): <short imperative summary>
```

**Types:** `feat`, `fix`, `docs`, `test`, `refactor`, `chore`  
**Scopes:** `registry`, `storage`, `types`, `errors`, `ci`, `deps`

Examples:

```bash
git commit -m "feat(registry): add credential expiry enforcement"
git commit -m "fix(verify): return false for expired credentials"
git commit -m "docs(storage): document SubjectCredential key layout"
git commit -m "test(integration): add revocation lifecycle test"
git commit -m "chore(ci): pin stellar-cli version in workflow"
```

- Use the **imperative mood** ("add", not "added" or "adds")
- Keep the subject line under 72 characters
- Reference the issue number in the body if applicable: `Closes #16`

---

## Opening a Pull Request

Before submitting, run through this checklist:

- [ ] `make test` passes with no failures
- [ ] `make lint` passes with no Clippy warnings
- [ ] `make fmt` has been run and code is formatted
- [ ] New or changed public functions have doc comments
- [ ] A test is included for any new behaviour or bug fix
- [ ] Commit messages follow Conventional Commits format
- [ ] PR is scoped to a single issue

**PR description should include:**

1. **What** — a short summary of the change
2. **Why** — the problem it solves or the issue it closes (`Closes #<number>`)
3. **How tested** — which tests cover it
4. **Known limitations** — anything intentionally left out or deferred

Target the `main` branch. Maintainers may request changes; address feedback with new commits (do not force-push to a PR branch).

---

## Security Disclosures

**Do not open a public GitHub issue for security vulnerabilities.**

Contract bugs on a live network can have real consequences. If you discover a vulnerability in the contract logic, please email the maintainers directly. Include a clear description of the issue, reproduction steps, and potential impact. We will acknowledge receipt promptly and coordinate a fix before any public disclosure.
