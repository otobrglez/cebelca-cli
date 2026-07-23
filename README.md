# (Unofficial) Čebelca BIZ CLI

This is unofficial Čebelca BIZ CLI interface that makes human or AI usage and integration pleasent and friendly.

## Internals

- CLI uses the [Čebelca BIZ Gateway](https://github.com/otobrglez/cebelca-gateway) instead of the official Čebelca BIZ API directly. This makes the interactions faster and easier to reason about. I.e. ability to "one shot" invoices instead of multi-step process.
- Much of this code is generated from GraphQL schema provided by the Gateway service.

## Development

- This project uses devenv (Nix) for management of versions and dependencies.
- Project is using Rust and is compiled to several binaries for multiple platforms


