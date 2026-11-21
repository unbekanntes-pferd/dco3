# dco3 - DRACOON API wrapper in Rust

![tests](https://github.com/unbekanntes-pferd/dco3/actions/workflows/ci.yml/badge.svg)
![Crates.io Version](https://img.shields.io/crates/v/dco3)
![Crates.io Total Downloads](https://img.shields.io/crates/d/dco3)

## What is this?
This is an async wrapper around the DRACOON API [dracoon.com](https://dracoon.com).

The documentation can be found on [docs.rs](https://docs.rs/dco3/latest/dco3).

## Cryptography
Cryptographic operations are implemented in a separate crate: [dco3-crypto](https://github.com/unbekanntes-pferd/dco3-crypto).
Above mentioned crate is based on `openssl` - therefore, currently `wasm32-unknown-unknown` is **not** supported.

## About

This crate is an async wrapper built around `reqwest` ([reqwest](https://crates.io/crates/reqwest)).
All API calls are async and thread-safe. 
Since this is in an early development stage, expect breaking changes.

## Deprecations

* Uploads to DRACOON Server instances using NFS storage (no S3) are deprecated and will be removed in a future release.
  A warning is logged once per process when such an upload is used.

## Examples

For now, please refer to [dccmd-rs](https://github.com/unbekanntes-pferd/dccmd-rs) as an example client implementation.

## Minimum supported Rust version

The MSRV is declared as `rust-version` in `Cargo.toml` (currently **1.88**) and tested in CI.

## Development

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

### End-to-end tests

`tests/e2e` runs against a real DRACOON instance. Copy `.env.example` to `.env` and set the base url and the
credentials of a dedicated local test user (no MFA; roles: room manager, user manager, group manager):

```bash
cargo test --test e2e                      # all e2e tests (parallel)
cargo test --test e2e -- transfers         # filter by name
E2E_CRYPTO=1 cargo test --test e2e         # include encrypted tests (replaces the test user's keypair)
```

Encrypted tests replace the test user's keypair. After an encrypted run, the server can refuse to delete that keypair
(`404 Node not found`, a known server behaviour after removing an encrypted room with deleted files); the next encrypted run
then fails until the keypair is reset server-side.

Each run creates a top level room `e2e-run-<id>` (plus `e2e-run-<id>-crypto` for encrypted tests), works only inside it
and with uniquely named users and groups, and removes everything afterwards - also if tests fail.
Without configuration, the e2e tests are reported as ignored. In CI they run manually via the `e2e` workflow.

## Releasing

Releases are published to crates.io by the `release` workflow (trusted publishing, no API token):

1. Bump `version` in `Cargo.toml` (and in `dco3_derive/Cargo.toml` if the derive crate changed), merge to `master`.
2. Tag the release commit and push the tag: `git tag vX.Y.Z && git push origin vX.Y.Z`.
3. The workflow checks that the tag matches the `dco3` version, runs the tests, publishes `dco3_derive` (only if its version is new) and `dco3`, and creates the GitHub release.

## Contributing

Contributions, feedback and issues are welcome.
To contribute, fork the repository and create a pull request.

## Roadmap

* implement missing API calls
    * branding
    * reporting
* add examples
