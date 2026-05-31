# Contributing to Veritix Contracts

Welcome, and thank you for your interest in contributing! Veritix Contracts is an open-source Soroban smart contract project and we are actively looking for contributors to help build it out. This project is part of an active open-source funding wave on [Drips Network](https://www.drips.network/) — contributors who ship meaningful features are part of something real.

Whether you are new to Soroban or an experienced Rust developer, there is a place for you here. Read through this guide, pick up an issue, and start building.

---

## What is Veritix Pay?

Veritix Pay is the on-chain payment module for the Veritix ticketing platform. It is built in **Rust** using **Soroban**, Stellar's smart contract platform, and deployed on the **Stellar network**.

It is responsible for:

- **On-chain payments** — token transfers between parties
- **Escrow** — hold funds until a condition is met, then release or refund
- **Recurring payments** — schedule periodic charges between a payer and payee
- **Payment splitting** — distribute a single payment across multiple recipients
- **Dispute resolution** — allow a third-party resolver to adjudicate contested payments

Core token primitives, escrow, and freeze modules are implemented and tested. Some modules (recurring payments, splitter, dispute) exist as draft files in `src/` but are not yet wired into the compiled crate. Contributors pick up open GitHub Issues to finish, extend, or expose these modules.

---

## Prerequisites

| Tool | Notes | Install |
|------|-------|---------|
| Rust (stable) | Required to compile Soroban contracts | https://rustup.rs |
| wasm32 target | Required build target for Soroban | `rustup target add wasm32-unknown-unknown` |
| Stellar CLI (latest) | For building and deploying contracts | `cargo install stellar-cli` |

Verify your setup:

```bash
rustc --version
stellar --version
```

**First-time setup:** Run `make preflight` from the `veritixpay/contract/token/` directory to verify all required tools are installed. This will give you fast, actionable feedback if anything is missing.

---

## Getting the Code

```bash
git clone https://github.com/Lead-Studios/veritix-contract.git
cd veritix-contract/veritixpay/contract/token
```

---

## Project Structure

```
veritixpay/
├── contract/
│   └── token/
│       ├── src/
│       │   ├── lib.rs              # Crate entry point — module declarations
│       │   ├── contract.rs         # Public Soroban interface (VeritixToken)
│       │   ├── admin.rs            # Admin storage and rotation
│       │   ├── allowance.rs        # Spending approvals
│       │   ├── balance.rs          # Ledger balance helpers
│       │   ├── escrow.rs           # Escrow logic (compiled)
│       │   ├── freeze.rs           # Account freeze/unfreeze (compiled)
│       │   ├── metadata.rs         # Token metadata
│       │   ├── storage_types.rs    # Shared DataKey enum and structs
│       │   ├── test.rs             # Compiled unit tests
│       │   ├── escrow_test.rs      # Escrow-specific tests (compiled)
│       │   ├── admin_test.rs       # Admin rotation tests (compiled)
│       │   ├── recurring.rs        # Draft — not yet declared in lib.rs
│       │   ├── splitter.rs         # Draft — not yet declared in lib.rs
│       │   └── dispute.rs          # Draft — not yet declared in lib.rs
│       ├── Cargo.toml
│       └── Makefile
├── Cargo.toml
├── Cargo.lock
├── .gitignore
└── README.md
```

Each compiled module is declared in `lib.rs` with `pub mod module_name;`. Draft files exist on disk but are **not** declared in `lib.rs` and are therefore not compiled or tested until a contributor wires them in.

---

## How to Pick Up an Issue

1. Browse [open issues](https://github.com/Lead-Studios/veritix-contract/issues) on GitHub
2. Find one labeled and comment **"I'd like to work on this"** to get assigned
3. Branch from `main`:
   ```bash
   git checkout -b feat/your-feature
   ```
4. Build your module, write tests, and open a PR against `main`
5. Fill in the PR description explaining what you built and why

If you have an idea not covered by an existing issue, open one first and describe what you want to build before starting work.

---

## How to Add a New Module

1. Create `src/your_module.rs`
2. Add `pub mod your_module;` to `src/lib.rs`
3. Add any new `DataKey` variants to `storage_types.rs` (create the file if it does not exist yet)
4. Write tests in a `#[cfg(test)]` block inside your module, or in a dedicated `test.rs`
5. Run `make test` to confirm everything passes
6. Run `make fmt` to format the code

---

## Building and Testing

All commands run from inside `veritixpay/contract/token/`:

```bash
make preflight  # verify all required tools are installed
make build    # compile the contract to WASM
make test     # run the full test suite
make fmt      # format all Rust code with rustfmt
make clean    # remove build artifacts
```

All tests must pass and `make fmt` must produce no diffs before a PR can be merged.

---

## Writing Tests

Tests use the Soroban test environment. The key patterns are:

- Use `Env::default()` to create an isolated test environment
- Register the contract with `env.register_contract()`
- Mock authorization with `env.mock_all_auths()` so you do not need real signers in tests

Boilerplate example:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_your_feature() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, YourContract);
        let client = YourContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        // call client methods and assert expected state
    }
}
```

Run only your tests:

```bash
cargo test your_feature
```

> **Compiled vs. dormant test files:** Only test modules declared in `lib.rs` under `#[cfg(test)]` are compiled and run by `cargo test`. A test file that exists in `src/` but is not declared in `lib.rs` is silently ignored by the compiler. When adding a new `*_test.rs` file, always add the corresponding `#[cfg(test)] mod your_test;` line to `lib.rs`.

---

## Storage Conventions

Soroban uses a key-value store. Follow these conventions:

- Define all storage keys as variants of the `DataKey` enum in `storage_types.rs` — do not invent inline keys
- Use `env.storage().persistent()` for user records (balances, escrows, payments) and bump TTL on access
- Use `env.storage().instance()` for contract-wide config (admin address, token metadata)
- For collections (e.g. escrows), use a count key (`EscrowCount`) paired with an indexed key (`Escrow(u32)`)

---

## Authorization Rules

Every function that mutates state on behalf of a user **must** call `address.require_auth()` (or `require_auth_for_args`) before touching storage.

```rust
sender.require_auth();
// only then: read/write storage
```

Never skip or defer authorization. PRs that mutate state without `require_auth()` will not be merged.

---

## Pull Request Checklist

Before marking your PR ready for review, confirm all of the following:

- [ ] `make test` passes with no failures
- [ ] `make fmt` has been run (no formatting diffs)
- [ ] New logic has at least one test covering the happy path
- [ ] Error and edge cases are tested where practical
- [ ] `cargo clippy --all -- -D warnings` is clean
- [ ] New/updated modules include `//!` module-level docs

## Contract Contributor Conventions

### Build and test commands
- `make preflight`
- `make build`
- `make test`

### Adding a new module
1. Create `src/<module>.rs`.
2. Add `pub mod <module>;` in `src/lib.rs`.
3. Wire public entrypoints from `contract.rs` when exposed.
4. Add/update tests and ensure module is included under `#[cfg(test)]` if split test files are used.

### Storage conventions
- Use instance storage for global config and counters.
- Use persistent storage for account/payment state.
- Extend TTL on important reads/writes with module-appropriate constants.

### Auth conventions
- Use `require_auth` for user-authorized actions.
- Use `check_admin` for admin-gated actions.

### Event conventions
- Use short symbols (`symbol_short!`) for event names.
- Keep topic/data structure deterministic and stable for indexers.

### Panic test conventions
- Use `#[should_panic(expected = \"...\")]` when asserting deterministic panic paths.
- [ ] No new `unwrap()` calls on untrusted or external data
- [ ] `storage_types.rs` updated if new storage keys were added
- [ ] `lib.rs` updated if a new module was added
- [ ] PR description explains what the change does and why

---

## Branching Strategy

- `main` is the stable branch — do not push directly
- Branch naming conventions:
  - `feat/` — new features or modules
  - `fix/` — bug fixes
  - `docs/` — documentation only
  - `chore/` — maintenance, tooling, config
- All PRs go against `main`

---

## Reporting Issues

When opening a bug report or unexpected-behavior issue, include:

- What you expected to happen
- What actually happened
- Steps to reproduce
- Your Rust version (`rustc --version`) and Stellar CLI version (`stellar --version`)

---

## License

By contributing, you agree that your contributions will be licensed under the **MIT License**.
