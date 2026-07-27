# (Unofficial) Čebelca BIZ CLI

<img align="right" width="50" src="./cebelca_biz.png">

This is an unofficial Čebelca BIZ CLI interface that makes human or AI usage and integration pleasant and friendly.

You can obtain an API key via ["Nastavitve -> API dostop"](https://www.cebelca.biz/manage/access.html).

## Usage

The binary is `ceb`. Run `ceb --help` or `ceb <command> --help` for full flags.

### Global flags

These apply to every command (and can be set via environment variables):

| Flag | Env var | Description |
|------|---------|-------------|
| `--token <token>` | `CEBELCA_TOKEN` | API token (required) |
| `--gateway-url <url>` | `CEBELCA_GATEWAY_URL` | GraphQL gateway endpoint (defaults to production) |

### `ceb partners` — manage partners (customers and suppliers)

| Command | Description |
|---------|-------------|
| `ceb partners list` (alias `ls`) `[-s <q>] [--page <n>]` | List partners, optionally filtered by search |
| `ceb partners show <id>` | Show a partner |
| `ceb partners invoices <id> [--filter <f>] [--from <d>] [--to <d>] [--page <n>]` | List a partner's invoices |
| `ceb partners add <name> [flags]` | Create a partner |
| `ceb partners update <id> [flags]` | Update a partner (only the flags you pass are changed) |

Flags for `add`/`update`: `--street`, `--postal`, `--city`, `--vatid`, `--country`, `--lang`.

`partners invoices` reuses the same status filter as `invoices list` (`all`, `archived`, `draft`, `paid`, `past-due`, `unpaid`) and takes an optional `--from`/`--to` date window (`YYYY-MM-DD`).

`--page` is 1-based (page 1 is the first page) and is available wherever the gateway supports paging: `partners list`, `partners invoices`, and `invoices list`. `services list` is not paged (the gateway returns the full list).

### `ceb services` — manage services (pricelist entries)

| Command | Description |
|---------|-------------|
| `ceb services list` (alias `ls`) `[-s <q>]` | List services, optionally filtered by search |
| `ceb services show <id>` | Show a service |
| `ceb services add <title> --price <p> --mu <u> --vat <v> [flags]` | Create a service |
| `ceb services update <id> [flags]` | Update a service (only the flags you pass are changed) |
| `ceb services delete <id>` | Delete a service |

Flags for `add`/`update`: `--price`, `--mu`, `--vat`, `--group`, `--konto` (`--price`, `--mu`, `--vat` are required on `add`).

### `ceb invoices` — list, finalize, and duplicate invoices

| Command | Description |
|---------|-------------|
| `ceb invoices list [--filter <f>] [--page <n>]` | List invoices, optionally filtered by status (`all`, `archived`, `draft`, `paid`, `past-due`, `unpaid`) |
| `ceb invoices add --partner-id <id> --date-sent <d> --date-to-pay <d> [--tag ...] [--line ...]` | Create a new draft invoice |
| `ceb invoices finalize <id> [--title <no>]` | Finalize (issue) a draft invoice, optionally overriding the assigned number |
| `ceb invoices duplicate <id> [--title <no>] [--tag ...]` | Duplicate an invoice into a new draft, optionally naming it and carrying tags over |

Each `--line` is repeatable and takes comma-separated `key=value` pairs: `title`, `qty`, `price`, and `vat` are required; `mu` and `discount` are optional. Optionally pass `--date-served <d>`. `--tag` is repeatable and labels the invoice. Example:

```sh
ceb invoices add --partner-id 7 --date-sent 2026-07-30 --date-to-pay 2026-08-10 \
  --tag urgent --tag q3 \
  --line "title=Consulting,qty=10,price=100,vat=22" \
  --line "title=Management,qty=42,price=123,vat=22,mu=kos,discount=0"
```

The listing appends any tags to each row as ` [tag, tag]`.

**Naming invoices.** In Čebelca an invoice's *title* is its document number (e.g. `26-0007`), empty until it's finalized. To control that number, pass `finalize --title` (it must be unique, or the server rejects it). `duplicate` produces a fresh draft — upstream clears both the number and tags on the copy — so use `duplicate --title` to name the new draft up front and `--tag` to carry labels over. Examples:

```sh
ceb invoices finalize 42 --title 26-0100          # issue with a specific number
ceb invoices duplicate 42 --title DRAFT-COPY --tag reissue
```

> **Note on invoice status:** `--filter` selects a Čebelca status *bucket* (the same tabs the web app uses). The listing's status column shows the invoice's lifecycle state — `draft` (not yet finalized, no number), `issued` (finalized, unpaid), `paid <date>` (has a payment date), or `cancelled` (archived/disabled) — derived server-side from the gateway's `Invoice.status` field rather than inferred from the payment date alone. The `draft` filter is a client-side refinement of `all` (the upstream API has no draft tab), so it returns exactly the numberless rows. Note the buckets and the lifecycle state aren't identical: `--filter unpaid` mirrors the upstream "open" tab, which can still list an invoice that already carries a payment date.

## Internals

- CLI uses the [Čebelca BIZ Gateway](https://github.com/otobrglez/cebelca-gateway) instead of the official Čebelca BIZ API directly. This makes the interactions faster and easier to reason about. I.e. ability to "one shot" invoices instead of multi-step process.
- Much of this code is generated from GraphQL schema provided by the Gateway service.

## Development

- This project uses devenv (Nix) for management of versions and dependencies.
- Project is using Rust and is compiled to several binaries for multiple platforms

### Building

The project is built with Cargo. From the repository root:

```sh
# Debug build — produces ./target/debug/ceb
cargo build

# Optimized release build — produces ./target/release/ceb
cargo build --release

# Build and run in one step (args after `--` go to the CLI)
cargo run -- invoices list
```

If you use [devenv](https://devenv.sh), run `devenv shell` first to get the pinned Rust toolchain and dependencies; otherwise a local Rust toolchain (Rust 2024 edition, i.e. Rust 1.85+) is required.

The devenv shell puts `target/release` on `PATH`, so `ceb` runs the release binary:

```sh
cargo build --release   # build it once
ceb partners list       # runs target/release/ceb
```

If you build for the first time in an already-open shell, run `direnv reload` (or `devenv shell`) so the new binary is picked up.


