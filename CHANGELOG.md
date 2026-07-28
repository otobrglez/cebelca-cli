# Changelog

`dist` uses the section matching the tagged version as the body of the GitHub Release,
so keep the headings as `## <version>` (no `v` prefix).

## 0.1.1

First released version — the CLI is `ceb`, talking to the
[Čebelca BIZ Gateway](https://github.com/otobrglez/cebelca-gateway) over GraphQL.

### Added

- `ceb partners` — list, show, add, update, delete partners, plus `partners invoices`
  for a partner's invoice history. Deletion resolves and prints the partner's name
  before prompting.
- `ceb services` — list, show, add, update and delete pricelist entries.
- `ceb invoices` — list (with status filters), show, create, duplicate, finalize,
  archive (and restore) and delete invoices. Invoices can be addressed by id or by
  document number, e.g. `ceb invoices show 021/26`.
- Global `--token` / `--gateway-url` flags, also readable from `CEBELCA_TOKEN` and
  `CEBELCA_GATEWAY_URL`.
- Prebuilt binaries for Linux, macOS and Windows, with `curl | sh` and
  `irm | iex` installers.
