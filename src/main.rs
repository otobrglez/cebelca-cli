use cebelca_cli::cli::*;
use cebelca_cli::gateway_client::GatewayClient;
use cebelca_cli::graphql::*;
use clap::Parser;

// TODO: Use better/propper logger.
// use log::{error, info};

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

    let result = match cli.command {
        Commands::Partners { command } => match command {
            PartnersCommand::List(args) => partners_list(&client, args.search, args.page, args.per_page),
            PartnersCommand::Show { id } => partners_show(&client, id),
            PartnersCommand::Invoices(args) => partners_invoices(&client, args),
            PartnersCommand::Add(args) => partners_add(&client, args),
            PartnersCommand::Update(args) => partners_update(&client, args),
        },

        Commands::Services { command } => match command {
            ServicesCommand::List(args) => services_list(&client, args.search),
            ServicesCommand::Show { id } => services_show(&client, id),
            ServicesCommand::Add(args) => services_add(&client, args),
            ServicesCommand::Update(args) => services_update(&client, args),
            ServicesCommand::Delete { id } => services_delete(&client, id),
        },
        Commands::Invoices { command } => match command {
            InvoicesCommand::List { filter } => invoices_list(&client, filter),
            InvoicesCommand::Add(args) => invoices_add(&client, args),
            InvoicesCommand::Finalize { id, title } => invoices_finalize(&client, id, title),
            InvoicesCommand::Duplicate { id, title, tags } => invoices_duplicate(&client, id, title, tags),
            InvoicesCommand::Delete { id, force } => invoices_delete(&client, id, force),
            InvoicesCommand::Archive { id, restore } => invoices_archive(&client, id, restore),
        },
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
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
        // paths read identically. `status` is the invoice's lifecycle state
        // (Draft/Issued/Paid/Cancelled); we append the payment date when settled.
        use partner_invoices::InvoiceStatus as S;
        let status = match (&i.status, i.date_paid.as_deref()) {
            (S::Paid, Some(d)) => format!("paid {d}"),
            (S::Paid, None) => "paid".to_string(),
            (S::Draft, _) => "draft".to_string(),
            (S::Cancelled, _) => "cancelled".to_string(),
            (S::Issued, _) => "issued".to_string(),
            (S::Other(s), _) => s.clone(),
        };
        let title = if i.title.is_empty() { "(draft)" } else { &i.title };
        println!("  {}\t{}\t{}\t{}{}", i.id, title, i.date_sent, status, fmt_tags(&i.tags));
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
        // `status` is the invoice's lifecycle state (Draft/Issued/Paid/Cancelled),
        // derived server-side — no longer inferred from just the payment date, so a
        // draft is no longer mislabelled "unpaid". Append the payment date when settled.
        use list_invoices::InvoiceStatus as S;
        let status = match (&i.status, i.date_paid.as_deref()) {
            (S::Paid, Some(d)) => format!("paid {d}"),
            (S::Paid, None) => "paid".to_string(),
            (S::Draft, _) => "draft".to_string(),
            (S::Cancelled, _) => "cancelled".to_string(),
            (S::Issued, _) => "issued".to_string(),
            (S::Other(s), _) => s.clone(),
        };
        let title = if i.title.is_empty() { "(draft)" } else { &i.title };
        println!("{}\t{}\t{}\t{}{}", i.id, title, i.date_sent, status, fmt_tags(&i.tags));
    }

    Ok(())
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

fn invoices_finalize(gw: &GatewayClient, id: i64, title: Option<String>) -> anyhow::Result<()> {
    let data = gw.query::<FinalizeInvoice>(finalize_invoice::Variables { id, title })?;

    match data.finalize_invoice {
        Some(i) => println!("finalized invoice {} — {}", i.id, i.title),
        None => eprintln!("no invoice returned"),
    }

    Ok(())
}

fn invoices_duplicate(
    gw: &GatewayClient,
    id: i64,
    title: Option<String>,
    tags: Vec<String>,
) -> anyhow::Result<()> {
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

fn invoices_delete(gw: &GatewayClient, id: i64, force: bool) -> anyhow::Result<()> {
    if !force && !confirm(&format!("Delete invoice {id}?"))? {
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

fn invoices_archive(gw: &GatewayClient, id: i64, restore: bool) -> anyhow::Result<()> {
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
