# DEV LOG

This is development log. It prevents me from going insane. ;)

## Questions for Janko
- ~~Can I fetch invoice by title if title has special characters. i.e. "021/26"?~~
  Answered by probing the live API: `search` on `invoice-sent` is an **exact
  whole-title** match against the *decoded* title, so `021/26` works as typed and
  the stored/encoded `021&#47;26` matches nothing. Blank `search` is a wildcard
  (returns everything), not a no-match. Substring matching only applies to
  `invoice-sent-o` (services).

## 2026-07-28

- [x] Fetching invoices via title not just ID
- [x] `partners delete`
- [x] Introduce [dist](https://github.com/axodotdev/cargo-dist) (née cargo-dist) and
  GitHub Actions that build `ceb` for Linux, macOS and Windows on every version tag

# Backlog

- [ ] Gateway `sanitizeError` throws away the error message: every `CebelcaError`
  is wrapped in a plain `RuntimeException`, which Caliban renders as
  `"Effect failure"`. So the CLI can't tell "no such invoice number" from
  "gateway is down" even though the gateway already computed a good message
  (`no invoice with title '021/2'`). Fix by surfacing it as an `ExecutionError`.
- [ ] JSON output
- [ ] YAML output
- [ ] Ship musl Linux builds too, so the binaries run on Alpine and older glibc
  distros (add the two `*-linux-musl` targets to `dist-workspace.toml`, re-run
  `dist init --yes`) — cheap because we're on rustls, not OpenSSL
- [ ] Homebrew tap for `ceb` (needs a `homebrew-tap` repo + a PAT secret)
- [ ] Start emitting gateway-client version to the server so that "old client" missmatch can be detected.
- [ ] Getting PDFs or other attachments for invoices
- [ ] Editing invoice made in similar way than git works with help of `$EDITOR`. Meaning it opens file that is in YAML format. Upon submit it gets pushed to Gateway
