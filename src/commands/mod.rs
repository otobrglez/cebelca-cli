//! One module per command group, each owning its own dispatch.
//!
//! The alternative — a single `match` in `main` over every subcommand of every
//! group — grows with each feature and puts `partners`, `services` and `invoices`
//! arms next to each other for no reason. Here each group exposes one
//! [`dispatch`](partners::dispatch) that maps its own subcommands to its own
//! handlers, so `main` matches three arms and adding a subcommand touches one
//! file.
//!
//! This module holds only what more than one group needs: paging translation, the
//! yes/no prompt, and the shared text-formatting helpers.

pub mod invoices;
pub mod partners;
pub mod services;

use crate::graphql::InvoiceStatus;

/// Translate the CLI's optional 1-based `--page` / `--per-page` into the
/// gateway's paging arguments, matching the gateway's conventions: `page` is
/// 0-based with `-1` meaning "all pages / unpaged", and `perPage` is the page
/// size with `0` meaning "server default".
///
/// Paging only kicks in when the user asks for a size (`--per-page`), since the
/// gateway ignores `page` without one. Without `--per-page` we send the unpaged
/// sentinels so the full list comes back. `--page` defaults to 1 when a size is
/// given but no page is; `saturating_sub` keeps page 0 and 1 both mapping to the
/// first page so out-of-range input can't underflow.
pub fn gql_paging(page: Option<u32>, per_page: Option<u32>) -> (i64, i64) {
    match per_page {
        Some(size) => (page.unwrap_or(1).saturating_sub(1) as i64, size as i64),
        None => (-1, 0),
    }
}

/// Render the `status` column: the invoice's lifecycle state
/// (Draft/Issued/Paid/Cancelled) as derived server-side, with the payment date
/// appended once settled.
///
/// Every operation shares one `InvoiceStatus` (see `extern_enums` in
/// [`crate::graphql`]), so the match is exhaustive with no catch-all: a status
/// added to the schema becomes a compile error here rather than a stringified
/// passthrough.
pub fn status_label(status: InvoiceStatus, date_paid: Option<&str>) -> String {
    use InvoiceStatus as S;
    match (status, date_paid) {
        (S::Paid, Some(d)) => format!("paid {d}"),
        (S::Paid, None) => "paid".to_string(),
        (S::Draft, _) => "draft".to_string(),
        (S::Cancelled, _) => "cancelled".to_string(),
        (S::Issued, _) => "issued".to_string(),
    }
}

/// Show an empty upstream string as `-` in the detail view, so a missing value
/// reads as missing rather than as a blank line.
pub fn or_dash(s: &str) -> &str {
    if s.is_empty() { "-" } else { s }
}

/// Render the client name column. The gateway resolves `partner` to null when the
/// referenced partner is gone (deleted/disabled upstream), so show `-` rather than
/// an empty column — a tab-separated row must keep its field count for `cut`/`awk`.
pub fn fmt_partner(name: Option<&str>) -> &str {
    match name {
        Some(n) if !n.is_empty() => n,
        _ => "-",
    }
}

/// Mark FURS-registered invoices with a trailing ` *` in list output. Deliberately
/// terse and only present when true: most invoices aren't fiscalized, and an extra
/// column would push the tags suffix around for no gain.
pub fn fmt_fiscalized(fiscalized: bool) -> &'static str {
    if fiscalized { " *" } else { "" }
}

/// Render an invoice's tags as a trailing ` [a, b]` suffix for the list/summary
/// lines, or an empty string when there are none — so untagged invoices print
/// exactly as before.
pub fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

/// Ask the user a yes/no question on the terminal, defaulting to "yes" (shown as
/// `[Y/n]`). Anything starting with `n`/`N` is a no; empty input or anything else
/// counts as yes.
pub fn confirm(question: &str) -> anyhow::Result<bool> {
    use std::io::Write;

    print!("{question} [Y/n] ");
    std::io::stdout().flush()?;

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;

    Ok(!answer.trim().eq_ignore_ascii_case("n") && !answer.trim().eq_ignore_ascii_case("no"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_partner_never_yields_an_empty_column() {
        assert_eq!(fmt_partner(Some("Ziverge Inc")), "Ziverge Inc");
        // a missing partner (deleted/disabled upstream) and an unnamed one both
        // become `-`, so every row keeps the same field count for cut/awk
        assert_eq!(fmt_partner(None), "-");
        assert_eq!(fmt_partner(Some("")), "-");
    }

    #[test]
    fn status_label_appends_the_payment_date_only_when_paid() {
        assert_eq!(
            status_label(InvoiceStatus::Paid, Some("2026-07-01")),
            "paid 2026-07-01"
        );
        assert_eq!(status_label(InvoiceStatus::Paid, None), "paid");
        // a date on an unpaid invoice is not the paid date, so it must not show up
        assert_eq!(
            status_label(InvoiceStatus::Issued, Some("2026-07-01")),
            "issued"
        );
        assert_eq!(status_label(InvoiceStatus::Draft, None), "draft");
        assert_eq!(status_label(InvoiceStatus::Cancelled, None), "cancelled");
    }

    #[test]
    fn paging_only_engages_when_a_size_is_given() {
        // no --per-page: the unpaged sentinels, so the full list comes back
        assert_eq!(gql_paging(None, None), (-1, 0));
        assert_eq!(gql_paging(Some(3), None), (-1, 0));
        // 1-based in, 0-based out
        assert_eq!(gql_paging(Some(1), Some(10)), (0, 10));
        assert_eq!(gql_paging(Some(2), Some(10)), (1, 10));
        assert_eq!(gql_paging(None, Some(10)), (0, 10));
        // page 0 can't underflow into a negative (which would mean "unpaged")
        assert_eq!(gql_paging(Some(0), Some(10)), (0, 10));
    }

    #[test]
    fn tag_and_fiscalized_suffixes_vanish_when_absent() {
        assert_eq!(fmt_tags(&[]), "");
        assert_eq!(fmt_tags(&["urgent".to_string()]), " [urgent]");
        assert_eq!(
            fmt_tags(&["urgent".to_string(), "q3".to_string()]),
            " [urgent, q3]"
        );
        assert_eq!(fmt_fiscalized(false), "");
        assert_eq!(fmt_fiscalized(true), " *");
    }
}
