# (Unofficial) Čebelca BIZ CLI

This is unofficial Čebelca BIZ CLI interface that makes human or AI usage and integration pleasent and friendly.

## Usage

The binary is `cb`. Run `cb --help` or `cb <command> --help` for full flags.

### Global flags

These apply to every command (and can be set via environment variables):

| Flag | Env var | Description |
|------|---------|-------------|
| `--token <token>` | `CEBELCA_TOKEN` | API token (required) |
| `--gateway-url <url>` | `CEBELCA_GATEWAY_URL` | GraphQL gateway endpoint (defaults to production) |

### `cb partners` — manage partners (customers and suppliers)

| Command | Description |
|---------|-------------|
| `cb partners list` (alias `ls`) `[-s <q>]` | List partners, optionally filtered by search |
| `cb partners show <id>` | Show a partner |
| `cb partners add <name> [flags]` | Create a partner |
| `cb partners update <id> [flags]` | Update a partner (only the flags you pass are changed) |

Flags for `add`/`update`: `--street`, `--postal`, `--city`, `--vatid`, `--country`, `--lang`.

### `cb services` — manage services (pricelist entries)

| Command | Description |
|---------|-------------|
| `cb services list` (alias `ls`) `[-s <q>]` | List services, optionally filtered by search |
| `cb services show <id>` | Show a service |
| `cb services add <title> --price <p> --mu <u> --vat <v> [flags]` | Create a service |
| `cb services update <id> [flags]` | Update a service (only the flags you pass are changed) |
| `cb services delete <id>` | Delete a service |

Flags for `add`/`update`: `--price`, `--mu`, `--vat`, `--group`, `--konto` (`--price`, `--mu`, `--vat` are required on `add`).

### `cb invoices` — list, finalize, and duplicate invoices

| Command | Description |
|---------|-------------|
| `cb invoices list [--filter <f>]` | List invoices, optionally filtered by status (`all`, `archived`, `paid`, `past-due`, `unpaid`) |
| `cb invoices add --partner-id <id> --date-sent <d> --date-to-pay <d> [--line ...]` | Create a new draft invoice |
| `cb invoices finalize <id>` | Finalize (issue) a draft invoice |
| `cb invoices duplicate <id>` | Duplicate an invoice into a new draft |

Each `--line` is repeatable and takes comma-separated `key=value` pairs: `title`, `qty`, `price`, and `vat` are required; `mu` and `discount` are optional. Optionally pass `--date-served <d>`. Example:

```sh
cb invoices add --partner-id 7 --date-sent 2026-07-30 --date-to-pay 2026-08-10 \
  --line "title=Consulting,qty=10,price=100,vat=22" \
  --line "title=Management,qty=42,price=123,vat=22,mu=kos,discount=0"
```

> **Note on invoice status:** `--filter` selects a Čebelca status *bucket* (the same tabs the web app uses), which is not the same as the per-invoice paid flag. The listing's status column shows `paid <date>` when an invoice has a payment date and `unpaid` otherwise — so an invoice can appear under `--filter unpaid` (still open upstream) yet already carry a payment date.

## Internals

- CLI uses the [Čebelca BIZ Gateway](https://github.com/otobrglez/cebelca-gateway) instead of the official Čebelca BIZ API directly. This makes the interactions faster and easier to reason about. I.e. ability to "one shot" invoices instead of multi-step process.
- Much of this code is generated from GraphQL schema provided by the Gateway service.

## Development

- This project uses devenv (Nix) for management of versions and dependencies.
- Project is using Rust and is compiled to several binaries for multiple platforms

### Building

The project is built with Cargo. From the repository root:

```sh
# Debug build — produces ./target/debug/cb
cargo build

# Optimized release build — produces ./target/release/cb
cargo build --release

# Build and run in one step (args after `--` go to the CLI)
cargo run -- invoices list
```

If you use [devenv](https://devenv.sh), run `devenv shell` first to get the pinned Rust toolchain and dependencies; otherwise a local Rust toolchain (Rust 2024 edition, i.e. Rust 1.85+) is required.


