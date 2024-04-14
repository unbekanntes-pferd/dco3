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

## Examples

For now, please refer to [dccmd-rs](https://github.com/unbekanntes-pferd/dccmd-rs) as an example client implementation.

## Contributing

Contributions, feedback and issues are welcome.
To contribute, fork the repository and create a pull request.

## Roadmap

* implement missing API calls
    * branding
* improve upload callback (chunk stream)
* add examples
