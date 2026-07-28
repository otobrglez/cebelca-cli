//! `ceb invoices` — list, show, create, finalize, duplicate, archive, delete.

use super::{confirm, fmt_fiscalized, fmt_partner, fmt_tags, or_dash, status_label};
use crate::cli::{AddInvoiceArgs, InvoiceRef, InvoiceRefArgs, InvoicesCommand, ListInvoicesArgs};
use crate::gateway_client::GatewayClient;
use crate::graphql::*;
use anyhow::Context;

/// Run whichever `invoices` subcommand was given; `None` means the group was
/// named bare and defaults to `list` (see [`super::partners::dispatch`]).
pub fn dispatch(
    gw: &GatewayClient,
    command: Option<InvoicesCommand>,
    list: ListInvoicesArgs,
) -> anyhow::Result<()> {
    match command.unwrap_or(InvoicesCommand::List(list)) {
        InvoicesCommand::List(args) => self::list(gw, args.filter),
        InvoicesCommand::Show { invoice } => show(gw, invoice_ref(&invoice)?),
        InvoicesCommand::Add(args) => add(gw, args),
        InvoicesCommand::Finalize { invoice, title } => finalize(gw, invoice_ref(&invoice)?, title),
        InvoicesCommand::Duplicate {
            invoice,
            title,
            tags,
        } => duplicate(gw, invoice_ref(&invoice)?, title, tags),
        InvoicesCommand::Delete { invoice, force } => delete(gw, invoice_ref(&invoice)?, force),
        InvoicesCommand::Archive { invoice, restore } => {
            archive(gw, invoice_ref(&invoice)?, restore)
        }
    }
}

fn list(gw: &GatewayClient, filter: Option<InvoiceFilter>) -> anyhow::Result<()> {
    let data = gw.query::<ListInvoices>(list_invoices::Variables { filter })?;

    for i in data.invoices.unwrap_or_default() {
        let status = status_label(i.status, i.date_paid.as_deref());
        let title = if i.title.is_empty() {
            "(draft)"
        } else {
            &i.title
        };
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}{}{}",
            i.id,
            title,
            fmt_partner(i.partner.as_ref().map(|p| p.name.as_str())),
            i.date_sent,
            i.date_to_pay,
            status,
            fmt_fiscalized(i.fiscalized),
            fmt_tags(&i.tags)
        );
    }

    Ok(())
}

/// One invoice flattened into plain owned values, ready to print.
///
/// `invoice(id:)` and `invoiceByTitle(title:)` select the same fields via one
/// shared fragment, but graphql_client renders that fragment separately into each
/// operation's module — so the two structs are identical in shape and distinct in
/// name. Collapsing both into this struct means the detail view is written once
/// instead of once per query.
struct InvoiceView {
    id: i64,
    title: String,
    status: String,
    doc_type: String,
    fiscalized: bool,
    partner: Option<(String, i64)>,
    partner_id: i64,
    date_sent: String,
    date_to_pay: String,
    date_served: String,
    payment: String,
    tags: Vec<String>,
    lines: Vec<LineView>,
}

struct LineView {
    id: i64,
    title: String,
    qty: f64,
    mu: String,
    price: f64,
    vat: f64,
    discount: f64,
}

/// Generate `From<Module::InvoiceDetail> for InvoiceView` for each operation that
/// selects the `InvoiceDetail` fragment.
///
/// Still a macro, but a narrow one: it is invoked once, here, purely to name the
/// two generated fragment types — everything downstream is a plain `.into()` on a
/// single type. The conversion body itself is written once.
macro_rules! impl_invoice_view_from {
    ($($module:ident),+ $(,)?) => {$(
        impl From<$module::InvoiceDetail> for InvoiceView {
            fn from(i: $module::InvoiceDetail) -> Self {
                InvoiceView {
                    status: status_label(i.status, i.date_paid.as_deref()),
                    doc_type: format!("{:?}", i.doc_type),
                    partner: i.partner.map(|p| (p.name, p.id)),
                    lines: i
                        .lines
                        .unwrap_or_default()
                        .into_iter()
                        .map(|l| LineView {
                            id: l.id,
                            title: l.title,
                            qty: l.qty,
                            mu: l.mu,
                            price: l.price,
                            vat: l.vat,
                            discount: l.discount,
                        })
                        .collect(),
                    id: i.id,
                    title: i.title,
                    fiscalized: i.fiscalized,
                    partner_id: i.partner_id,
                    date_sent: i.date_sent,
                    date_to_pay: i.date_to_pay,
                    date_served: i.date_served,
                    payment: i.payment,
                    tags: i.tags,
                }
            }
        }
    )+};
}

impl_invoice_view_from!(show_invoice, show_invoice_by_title);

/// Lift the id-or-number parse into `anyhow`, so every invoice command's dispatch
/// arm is a one-liner.
fn invoice_ref(args: &InvoiceRefArgs) -> anyhow::Result<InvoiceRef> {
    args.parse().map_err(|e| anyhow::anyhow!(e))
}

/// Reduce an [`InvoiceRef`] to a plain invoice id, for the commands that mutate by
/// id — the gateway exposes no mutate-by-title, so a document number costs one
/// extra lookup first.
///
/// Returns the id alongside a label naming the invoice the way the user did, so
/// prompts and confirmations echo back what they typed (`invoice 021/26`) rather
/// than an id they never mentioned.
fn invoice_id_of(gw: &GatewayClient, invoice: InvoiceRef) -> anyhow::Result<(i64, String)> {
    match invoice {
        InvoiceRef::Id(id) => Ok((id, format!("invoice {id}"))),
        InvoiceRef::Number(number) => {
            let lookup = |filter| {
                gw.query::<ResolveInvoice>(resolve_invoice::Variables {
                    title: number.clone(),
                    filter,
                })
                .map(|d| d.invoice_by_title)
            };

            // The default tab excludes archived invoices, so fall back to it — an
            // archived invoice must stay addressable by number, not least because
            // `archive --restore` is the command that un-archives it.
            let found = match lookup(None) {
                Ok(Some(found)) => found,
                _ => lookup(Some(InvoiceFilter::Archived))
                    .with_context(|| format!("could not resolve invoice number '{number}'"))?
                    .ok_or_else(|| anyhow::anyhow!("no invoice with number '{number}'"))?,
            };
            Ok((found.id, format!("invoice {} ({})", found.title, found.id)))
        }
    }
}

/// Resolve one invoice by id or document number, then show it.
///
/// The number lookup is server-side (`invoiceByTitle`), which matches the whole
/// number against the decoded title — so `021/26` finds the invoice whose stored
/// title is `021&#47;26`, and a partial number matches nothing.
fn show(gw: &GatewayClient, invoice: InvoiceRef) -> anyhow::Result<()> {
    let view: InvoiceView = match invoice {
        InvoiceRef::Id(id) => {
            let found = gw
                .query::<ShowInvoice>(show_invoice::Variables { id })
                .with_context(|| format!("could not look up invoice id {id}"))?
                .invoice
                .ok_or_else(|| anyhow::anyhow!("no invoice with id {id}"))?;
            found.into()
        }
        InvoiceRef::Number(number) => {
            let lookup = |filter| {
                gw.query::<ShowInvoiceByTitle>(show_invoice_by_title::Variables {
                    title: number.clone(),
                    filter,
                })
                .map(|d| d.invoice_by_title)
            };

            // Retry on the archived tab when the default one has nothing: upstream's
            // status tabs are disjoint, so `All` alone can't see an archived invoice.
            let found = match lookup(None) {
                Ok(Some(found)) => found,
                // The gateway sanitizes every resolver error to "Effect failure", so
                // the CLI cannot tell "no such number" from "gateway down" — name
                // what we were doing and let the raw error be the cause.
                _ => lookup(Some(InvoiceFilter::Archived))
                    .with_context(|| format!("could not resolve invoice number '{number}'"))?
                    .ok_or_else(|| anyhow::anyhow!("no invoice with number '{number}'"))?,
            };
            found.into()
        }
    };

    print_invoice(&view);
    Ok(())
}

/// Show one invoice in full: the head as aligned `field: value` lines, then its
/// lines. Unlike the list commands (which stay tab-separated for cut/awk), this
/// is a detail view meant for reading — the one place to inspect a draft's lines
/// before finalizing it.
///
/// Empty upstream strings are printed as `-` rather than blanks, so a missing
/// value is visibly missing. `payment` is the payment terms/method, not the paid
/// status — `status` carries that.
fn print_invoice(i: &InvoiceView) {
    let number = if i.title.is_empty() {
        "(draft)"
    } else {
        &i.title
    };

    println!("id:         {}", i.id);
    println!("number:     {number}");
    println!("status:     {}", i.status);
    println!("type:       {}", i.doc_type);
    println!("fiscalized: {}", if i.fiscalized { "yes" } else { "no" });
    match &i.partner {
        Some((name, id)) => println!("partner:    {name} ({id})"),
        None => println!("partner:    {} (not found)", i.partner_id),
    }
    println!("sent:       {}", or_dash(&i.date_sent));
    println!("due:        {}", or_dash(&i.date_to_pay));
    println!("served:     {}", or_dash(&i.date_served));
    println!("payment:    {}", or_dash(&i.payment));
    if !i.tags.is_empty() {
        println!("tags:       {}", i.tags.join(", "));
    }

    if i.lines.is_empty() {
        println!("lines:      (none)");
        return;
    }

    println!("lines:");
    let mut total = 0.0;
    for l in &i.lines {
        // Mirror the upstream line maths: discount is a percentage off the line's
        // gross, and VAT applies to the discounted amount.
        let net = l.qty * l.price * (1.0 - l.discount / 100.0);
        total += net * (1.0 + l.vat / 100.0);
        let discount = if l.discount == 0.0 {
            String::new()
        } else {
            format!("\t-{}%", l.discount)
        };
        println!(
            "  {}\t{}\t{} {}\t{}\t{}%{}\t{:.2}",
            l.id, l.title, l.qty, l.mu, l.price, l.vat, discount, net
        );
    }
    println!("total:      {total:.2} (incl. VAT)");
}

fn add(gw: &GatewayClient, args: AddInvoiceArgs) -> anyhow::Result<()> {
    let lines: Vec<create_invoice::LineInput> = args
        .lines
        .into_iter()
        .map(|l| create_invoice::LineInput {
            title: l.title,
            qty: l.qty,
            price: l.price,
            vat: l.vat,
            mu: l.mu,
            discount: l.discount,
        })
        .collect();

    let input = create_invoice::InvoiceInput {
        date_sent: args.date_sent,
        date_to_pay: args.date_to_pay,
        partner_id: args.partner_id,
        date_served: args.date_served,
        // clap collects zero occurrences as an empty Vec; send None so we don't
        // overwrite with an explicit empty tag list.
        tags: (!args.tags.is_empty()).then_some(args.tags),
        lines: Some(lines),
    };

    let data = gw.query::<CreateInvoice>(create_invoice::Variables { input })?;

    match data.create_invoice {
        Some(i) => {
            println!(
                "created invoice {} — {}{}",
                i.id,
                i.title,
                fmt_tags(&i.tags)
            );
            for l in i.lines.unwrap_or_default() {
                println!("  {}\t{}\t{}\t{}%", l.title, l.qty, l.price, l.vat);
            }
        }
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn finalize(gw: &GatewayClient, invoice: InvoiceRef, title: Option<String>) -> anyhow::Result<()> {
    let (id, _) = invoice_id_of(gw, invoice)?;
    let data = gw.query::<FinalizeInvoice>(finalize_invoice::Variables { id, title })?;

    match data.finalize_invoice {
        Some(i) => println!("finalized invoice {} — {}", i.id, i.title),
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn duplicate(
    gw: &GatewayClient,
    invoice: InvoiceRef,
    title: Option<String>,
    tags: Vec<String>,
) -> anyhow::Result<()> {
    let (id, _) = invoice_id_of(gw, invoice)?;
    let data = gw.query::<DuplicateInvoice>(duplicate_invoice::Variables {
        id,
        title,
        // Only carry tags when the user asked; an empty Vec would tell the gateway
        // to explicitly set no tags (which is the duplicate's default anyway).
        tags: (!tags.is_empty()).then_some(tags),
    })?;

    match data.duplicate_invoice {
        Some(i) => println!(
            "duplicated invoice {id} into {} — {}{}",
            i.id,
            i.title,
            fmt_tags(&i.tags)
        ),
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn delete(gw: &GatewayClient, invoice: InvoiceRef, force: bool) -> anyhow::Result<()> {
    // Resolve before prompting, so the confirmation names the invoice the server
    // actually matched — deleting by number should never ask about one invoice and
    // delete another, and an unknown number fails here rather than after a "yes".
    let (id, label) = invoice_id_of(gw, invoice)?;

    if !force && !confirm(&format!("Delete {label}?"))? {
        println!("aborted");
        return Ok(());
    }

    let data = gw.query::<DeleteInvoice>(delete_invoice::Variables { id })?;

    if data.delete_invoice.unwrap_or(false) {
        println!("deleted invoice {id}");
    } else {
        eprintln!("invoice {id} was not deleted");
    }

    Ok(())
}

fn archive(gw: &GatewayClient, invoice: InvoiceRef, restore: bool) -> anyhow::Result<()> {
    let (id, _) = invoice_id_of(gw, invoice)?;

    // `restore` flips the archive: --restore sets archived=false (un-archive),
    // otherwise archived=true. The gateway toggles the upstream `disabled` flag
    // and re-reads the invoice, so `status` reflects the result.
    let data = gw.query::<ArchiveInvoice>(archive_invoice::Variables {
        id,
        archived: Some(!restore),
    })?;

    match data.archive_invoice {
        Some(i) => {
            let verb = if restore { "restored" } else { "archived" };
            let title = if i.title.is_empty() {
                "(draft)"
            } else {
                &i.title
            };
            println!("{verb} invoice {} — {} ({:?})", i.id, title, i.status);
        }
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}
