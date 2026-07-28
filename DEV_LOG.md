# DEV LOG

This is development log. It prevents me from going insane. ;)

## Questions for Janko
- Can I fetch invoice by title if title has special characters. i.e. "021/26"?

## 2026-07-28

- [x] Fetching invoices via title not just ID

# Backlog

- [ ] JSON output
- [ ] YAML output
- [ ] Introduce [cargo-dist](https://github.com/axodotdev/cargo-dist) to this project and start versioning releases
- [ ] Update GitHub Actions so that cargo-dist is used and CLI is built for Linux, Mac and Windows platforms
- [ ] Start emitting gateway-client version to the server so that "old client" missmatch can be detected.
- [ ] Getting PDFs or other attachments for invoices
- [ ] Editing invoice made in similar way than git works with help of `$EDITOR`. Meaning it opens file that is in YAML format. Upon submit it gets pushed to Gateway
