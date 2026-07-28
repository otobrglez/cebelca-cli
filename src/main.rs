use anyhow::Context;
use cebelca_cli::cli::*;
use cebelca_cli::gateway_client::GatewayClient;
use cebelca_cli::graphql::*;
use clap::Parser;

// TODO: Use better/propper logger.
// use log::{error, info};

/// Render the `status` column: the invoice's lifecycle state
/// (Draft/Issued/Paid/Cancelled) as derived server-side, with the payment date
/// appended once settled.
///
/// This is a macro rather than a function because every generated query module
/// gets its own copy of the schema's `InvoiceStatus` enum — there's no shared
/// type to write a signature against — so it expands against whichever module is
/// named at the call site.
macro_rules! status_label {
    ($module:ident, $status:expr, $date_paid:expr) => {{
        use $module::InvoiceStatus as S;
        match ($status, $date_paid) {
            (S::Paid, Some(d)) => format!("paid {d}"),
            (S::Paid, None) => "paid".to_string(),
            (S::Draft, _) => "draft".to_string(),
            (S::Cancelled, _) => "cancelled".to_string(),
            (S::Issued, _) => "issued".to_string(),
            (S::Other(s), _) => s.clone(),
        }
    }};
}

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
fn gql_paging(page: Option<u32>, per_page: Option<u32>) -> (i64, i64) {
    match per_page {
        Some(size) => (page.unwrap_or(1).saturating_sub(1) as i64, size as i64),
        None => (-1, 0),
    }
}

fn main() {
    let cli = CLI::parse();

    let gateway_url: String = cli.gateway_url;
    let token = cli.token.unwrap_or_else(|| {
        eprintln!("error: no API token. Pass --token or set CEBELCA_TOKEN.");
        std::process::exit(1);
    });

    let client = GatewayClient::new(gateway_url, token);

    // A group with no subcommand defaults to `list`, taking the arguments clap
    // flattened onto the group itself — so `ceb partners -s acme` is exactly
    // `ceb partners list -s acme`, sharing one handler rather than a second path
    // that could drift from it.
    let result = match cli.command {
        Commands::Partners { command, list } => match command.unwrap_or(PartnersCommand::List(list)) {
            PartnersCommand::List(args) => partners_list(&client, args.search, args.page, args.per_page),
            PartnersCommand::Show { id } => partners_show(&client, id),
            PartnersCommand::Invoices(args) => partners_invoices(&client, args),
            PartnersCommand::Add(args) => partners_add(&client, args),
            PartnersCommand::Update(args) => partners_update(&client, args),
        },

        Commands::Services { command, list } => match command.unwrap_or(ServicesCommand::List(list)) {
            ServicesCommand::List(args) => services_list(&client, args.search),
            ServicesCommand::Show { id } => services_show(&client, id),
            ServicesCommand::Add(args) => services_add(&client, args),
            ServicesCommand::Update(args) => services_update(&client, args),
            ServicesCommand::Delete { id } => services_delete(&client, id),
        },
        Commands::Invoices { command, list } => match command.unwrap_or(InvoicesCommand::List(list)) {
            InvoicesCommand::List(args) => invoices_list(&client, args.filter),
            InvoicesCommand::Show { invoice } => {
                invoice_ref(&invoice).and_then(|r| invoices_show(&client, r))
            }
            InvoicesCommand::Add(args) => invoices_add(&client, args),
            InvoicesCommand::Finalize { invoice, title } => {
                invoice_ref(&invoice).and_then(|r| invoices_finalize(&client, r, title))
            }
            InvoicesCommand::Duplicate {
                invoice,
                title,
                tags,
            } => invoice_ref(&invoice).and_then(|r| invoices_duplicate(&client, r, title, tags)),
            InvoicesCommand::Delete { invoice, force } => {
                invoice_ref(&invoice).and_then(|r| invoices_delete(&client, r, force))
            }
            InvoicesCommand::Archive { invoice, restore } => {
                invoice_ref(&invoice).and_then(|r| invoices_archive(&client, r, restore))
            }
        },
    };

    if let Err(err) = result {
        // `{err:#}` renders anyhow's whole context chain (`context: cause`), not
        // just the outermost message — so a wrapped failure still shows what
        // actually went wrong underneath.
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn partners_list(
    gw: &GatewayClient,
    search: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> anyhow::Result<()> {
    let (page, per_page) = gql_paging(page, per_page);
    let data = gw.query::<ListPartners>(list_partners::Variables {
        search,
        page: Some(page),
        per_page: Some(per_page),
    })?;

    for p in data.partners.unwrap_or_default() {
        println!("{}\t{}\t{}", p.id, p.name, p.vatid);
    }

    Ok(())
}

fn partners_show(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    let data = gw.query::<ShowPartner>(show_partner::Variables { id })?;

    if let Some(p) = data.partner {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            p.id, p.name, p.vatid, p.country, p.city
        );
    }

    Ok(())
}

fn partners_invoices(gw: &GatewayClient, args: PartnerInvoicesArgs) -> anyhow::Result<()> {
    use partner_invoices::InvoiceFilter as G;
    let filter = args.filter.map(|f| match f {
        InvoiceFilter::All => G::All,
        InvoiceFilter::Archived => G::Archived,
        InvoiceFilter::Draft => G::Draft,
        InvoiceFilter::Paid => G::Paid,
        InvoiceFilter::PastDue => G::PastDue,
        InvoiceFilter::Unpaid => G::Unpaid,
    });

    let partner = gw
        .query::<PartnerInvoices>(partner_invoices::Variables {
            id: args.id,
            filter,
            date_from: args.from,
            date_to: args.to,
        })?
        .partner
        .ok_or_else(|| anyhow::anyhow!("no partner with id {}", args.id))?;

    println!("{}\t{}", partner.id, partner.name);
    for i in partner.invoices.unwrap_or_default() {
        // Same status semantics and column layout as `invoices list`, so the two
        // paths read identically.
        let status = status_label!(partner_invoices, &i.status, i.date_paid.as_deref());
        let title = if i.title.is_empty() { "(draft)" } else { &i.title };
        println!(
            "  {}\t{}\t{}\t{}\t{}{}{}",
            i.id,
            title,
            i.date_sent,
            i.date_to_pay,
            status,
            fmt_fiscalized(i.fiscalized),
            fmt_tags(&i.tags)
        );
    }

    Ok(())
}

fn partners_add(gw: &GatewayClient, args: AddPartnerArgs) -> anyhow::Result<()> {
    let input = create_partner::PartnerInput {
        name: args.name,
        street: args.street,
        postal: args.postal,
        city: args.city,
        vatid: args.vatid,
        country: args.country,
        lang: args.lang,
    };

    let data = gw.query::<CreatePartner>(create_partner::Variables { input })?;

    match data.create_partner {
        Some(p) => println!(
            "{}\t{}\t{}\t{}\t{}",
            p.id, p.name, p.vatid, p.country, p.city
        ),
        None => eprintln!("no partner returned"),
    }

    Ok(())
}

fn partners_update(gw: &GatewayClient, args: UpdatePartnerArgs) -> anyhow::Result<()> {
    // The gateway's updatePartner is a full replace: any field left empty is
    // overwritten with "". So fetch the current record first and overlay only
    // the flags the user actually passed.
    let current = gw
        .query::<ShowPartner>(show_partner::Variables { id: args.id })?
        .partner
        .ok_or_else(|| anyhow::anyhow!("no partner with id {}", args.id))?;

    let input = update_partner::PartnerInput {
        name: args.name.unwrap_or(current.name),
        street: Some(args.street.unwrap_or(current.street)),
        postal: Some(args.postal.unwrap_or(current.postal)),
        city: Some(args.city.unwrap_or(current.city)),
        vatid: Some(args.vatid.unwrap_or(current.vatid)),
        country: Some(args.country.unwrap_or(current.country)),
        lang: Some(args.lang.unwrap_or(current.lang)),
    };

    let data = gw.query::<UpdatePartner>(update_partner::Variables { id: args.id, input })?;

    match data.update_partner {
        Some(p) => println!(
            "{}\t{}\t{}\t{}\t{}",
            p.id, p.name, p.vatid, p.country, p.city
        ),
        None => eprintln!("no partner returned"),
    }

    Ok(())
}

fn services_list(gw: &GatewayClient, search: Option<String>) -> anyhow::Result<()> {
    let data = gw.query::<ListServices>(list_services::Variables { search })?;

    for s in data.services.unwrap_or_default() {
        println!("{}\t{}\t{}\t{}\t{}%", s.id, s.title, s.price, s.mu, s.vat);
    }

    Ok(())
}

fn services_show(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    // There's no singular `service(id)` query in the schema (unlike partners),
    // so fetch the full list and pick the matching id client-side.
    let service = gw
        .query::<ListServices>(list_services::Variables { search: None })?
        .services
        .unwrap_or_default()
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("no service with id {id}"))?;

    println!(
        "{}\t{}\t{}\t{}\t{}%\t{}\t{}",
        service.id,
        service.title,
        service.price,
        service.mu,
        service.vat,
        service.group,
        service.konto
    );

    Ok(())
}

fn services_add(gw: &GatewayClient, args: AddServiceArgs) -> anyhow::Result<()> {
    let input = create_service::ServiceInput {
        title: args.title,
        price: args.price,
        mu: args.mu,
        vat: args.vat,
        group: args.group,
        konto: args.konto,
    };

    let data = gw.query::<CreateService>(create_service::Variables { input })?;

    match data.create_service {
        Some(s) => println!("{}\t{}\t{}\t{}\t{}%", s.id, s.title, s.price, s.mu, s.vat),
        None => eprintln!("no service returned"),
    }

    Ok(())
}

fn services_update(gw: &GatewayClient, args: UpdateServiceArgs) -> anyhow::Result<()> {
    // The gateway's updateService is a full replace: omitted fields are
    // overwritten with defaults. No singular `service(id)` query exists, so
    // fetch the list, find the current record, and overlay only the flags the
    // user actually passed.
    let current = gw
        .query::<ListServices>(list_services::Variables { search: None })?
        .services
        .unwrap_or_default()
        .into_iter()
        .find(|s| s.id == args.id)
        .ok_or_else(|| anyhow::anyhow!("no service with id {}", args.id))?;

    let input = update_service::ServiceInput {
        title: args.title.unwrap_or(current.title),
        price: args.price.unwrap_or(current.price),
        mu: args.mu.unwrap_or(current.mu),
        vat: args.vat.unwrap_or(current.vat),
        group: Some(args.group.unwrap_or(current.group)),
        konto: Some(args.konto.unwrap_or(current.konto)),
    };

    let data = gw.query::<UpdateService>(update_service::Variables { id: args.id, input })?;

    match data.update_service {
        Some(s) => println!("{}\t{}\t{}\t{}\t{}%", s.id, s.title, s.price, s.mu, s.vat),
        None => eprintln!("no service returned"),
    }

    Ok(())
}

fn services_delete(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    let data = gw.query::<DeleteService>(delete_service::Variables { id })?;

    if data.delete_service.unwrap_or(false) {
        println!("deleted service {id}");
    } else {
        eprintln!("service {id} was not deleted");
    }

    Ok(())
}

/// Map the CLI's InvoiceFilter to the GraphQL-generated one. Both are derived
/// from the schema's `InvoiceFilter` enum, so the variants line up 1:1.
fn to_gql_filter(f: InvoiceFilter) -> list_invoices::InvoiceFilter {
    use list_invoices::InvoiceFilter as G;
    match f {
        InvoiceFilter::All => G::All,
        InvoiceFilter::Archived => G::Archived,
        InvoiceFilter::Draft => G::Draft,
        InvoiceFilter::Paid => G::Paid,
        InvoiceFilter::PastDue => G::PastDue,
        InvoiceFilter::Unpaid => G::Unpaid,
    }
}

fn invoices_list(gw: &GatewayClient, filter: Option<InvoiceFilter>) -> anyhow::Result<()> {
    let filter = filter.map(to_gql_filter);
    let data = gw.query::<ListInvoices>(list_invoices::Variables { filter })?;

    for i in data.invoices.unwrap_or_default() {
        let status = status_label!(list_invoices, &i.status, i.date_paid.as_deref());
        let title = if i.title.is_empty() { "(draft)" } else { &i.title };
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
/// `invoice(id:)` and `invoiceByTitle(title:)` select the same fields (one shared
/// fragment), but graphql_client generates a separate module — and so a separate
/// set of types — per operation. Collapsing both into this struct means the detail
/// view is written once instead of once per query.
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

/// Build an [`InvoiceView`] from whichever generated module is named at the call
/// site. A macro for the same reason [`status_label`] is one: the two modules'
/// types are structurally identical but nominally distinct, so there's no single
/// signature to write this against.
macro_rules! invoice_view {
    ($module:ident, $invoice:expr) => {{
        let i = $invoice;
        InvoiceView {
            status: status_label!($module, &i.status, i.date_paid.as_deref()),
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
    }};
}

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
                _ => lookup(Some(resolve_invoice::InvoiceFilter::Archived))
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
fn invoices_show(gw: &GatewayClient, invoice: InvoiceRef) -> anyhow::Result<()> {
    let view = match invoice {
        InvoiceRef::Id(id) => {
            let found = gw
                .query::<ShowInvoice>(show_invoice::Variables { id })
                .with_context(|| format!("could not look up invoice id {id}"))?
                .invoice
                .ok_or_else(|| anyhow::anyhow!("no invoice with id {id}"))?;
            invoice_view!(show_invoice, found)
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
                _ => lookup(Some(show_invoice_by_title::InvoiceFilter::Archived))
                    .with_context(|| format!("could not resolve invoice number '{number}'"))?
                    .ok_or_else(|| anyhow::anyhow!("no invoice with number '{number}'"))?,
            };
            invoice_view!(show_invoice_by_title, found)
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

fn invoices_add(gw: &GatewayClient, args: AddInvoiceArgs) -> anyhow::Result<()> {
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
            println!("created invoice {} — {}{}", i.id, i.title, fmt_tags(&i.tags));
            for l in i.lines.unwrap_or_default() {
                println!("  {}\t{}\t{}\t{}%", l.title, l.qty, l.price, l.vat);
            }
        }
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn invoices_finalize(
    gw: &GatewayClient,
    invoice: InvoiceRef,
    title: Option<String>,
) -> anyhow::Result<()> {
    let (id, _) = invoice_id_of(gw, invoice)?;
    let data = gw.query::<FinalizeInvoice>(finalize_invoice::Variables { id, title })?;

    match data.finalize_invoice {
        Some(i) => println!("finalized invoice {} — {}", i.id, i.title),
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn invoices_duplicate(
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
        Some(i) => println!("duplicated invoice {id} into {} — {}{}", i.id, i.title, fmt_tags(&i.tags)),
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn invoices_delete(gw: &GatewayClient, invoice: InvoiceRef, force: bool) -> anyhow::Result<()> {
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

fn invoices_archive(gw: &GatewayClient, invoice: InvoiceRef, restore: bool) -> anyhow::Result<()> {
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
            let title = if i.title.is_empty() { "(draft)" } else { &i.title };
            println!("{verb} invoice {} — {} ({:?})", i.id, title, i.status);
        }
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

/// Show an empty upstream string as `-` in the detail view, so a missing value
/// reads as missing rather than as a blank line.
fn or_dash(s: &str) -> &str {
    if s.is_empty() { "-" } else { s }
}

/// Render the client name column. The gateway resolves `partner` to null when the
/// referenced partner is gone (deleted/disabled upstream), so show `-` rather than
/// an empty column — a tab-separated row must keep its field count for `cut`/`awk`.
fn fmt_partner(name: Option<&str>) -> &str {
    match name {
        Some(n) if !n.is_empty() => n,
        _ => "-",
    }
}

/// Mark FURS-registered invoices with a trailing ` *` in list output. Deliberately
/// terse and only present when true: most invoices aren't fiscalized, and an extra
/// column would push the tags suffix around for no gain.
fn fmt_fiscalized(fiscalized: bool) -> &'static str {
    if fiscalized { " *" } else { "" }
}

/// Render an invoice's tags as a trailing ` [a, b]` suffix for the list/summary
/// lines, or an empty string when there are none — so untagged invoices print
/// exactly as before.
fn fmt_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", tags.join(", "))
    }
}

/// Ask the user a yes/no question on the terminal, defaulting to "yes" (shown as
/// `[Y/n]`). Anything starting with `n`/`N` is a no; empty input or anything else
/// counts as yes.
fn confirm(question: &str) -> anyhow::Result<bool> {
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
}
