# (Unofficial) Čebelca BIZ CLI

This is unofficial Čebelca BIZ CLI interface that makes human or AI usage and integration pleasent and friendly.

## Usage

The binary is `cb`. Pass `--token` (or set `CEBELCA_TOKEN`). Run `cb <command> --help` for full flags.

| Command | Description |
|---------|-------------|
| `cb partners list [-s <q>]` | List partners, optionally filtered by search |
| `cb partners show <id>` | Show a partner |
| `cb partners add <name> [flags]` | Create a partner |
| `cb partners update <id> [flags]` | Update a partner |
| `cb services list [-s <q>]` | List services, optionally filtered by search |
| `cb services show <id>` | Show a service |
| `cb services add <title> --price --mu --vat [flags]` | Create a service |
| `cb services update <id> [flags]` | Update a service |
| `cb services delete <id>` | Delete a service |
| `cb invoices list [--filter <f>]` | List invoices _(not yet implemented)_ |
| `cb invoices finalize <id>` | Finalize a draft invoice _(not yet implemented)_ |

> **Note:** the `invoices` subcommands are defined but not yet wired up — running them exits with `error: `invoices` is not implemented yet`.

## Internals

- CLI uses the [Čebelca BIZ Gateway](https://github.com/otobrglez/cebelca-gateway) instead of the official Čebelca BIZ API directly. This makes the interactions faster and easier to reason about. I.e. ability to "one shot" invoices instead of multi-step process.
- Much of this code is generated from GraphQL schema provided by the Gateway service.

## Development

- This project uses devenv (Nix) for management of versions and dependencies.
- Project is using Rust and is compiled to several binaries for multiple platforms


