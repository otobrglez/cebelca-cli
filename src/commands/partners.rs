//! `ceb partners` — customers and suppliers.

use super::{confirm, fmt_fiscalized, fmt_tags, gql_paging, status_label};
use crate::cli::{
    AddPartnerArgs, ListArgs, PartnerInvoicesArgs, PartnersCommand, UpdatePartnerArgs,
};
use crate::gateway_client::GatewayClient;
use crate::graphql::*;

/// Run whichever `partners` subcommand was given.
///
/// `command` is `None` when the group was named bare (`ceb partners`), which
/// defaults to `list` and takes the arguments clap flattened onto the group — so
/// `ceb partners -s acme` is exactly `ceb partners list -s acme`, sharing one
/// handler rather than a second path that could drift from it.
pub fn dispatch(
    gw: &GatewayClient,
    command: Option<PartnersCommand>,
    list: ListArgs,
) -> anyhow::Result<()> {
    match command.unwrap_or(PartnersCommand::List(list)) {
        PartnersCommand::List(args) => self::list(gw, args.search, args.page, args.per_page),
        PartnersCommand::Show { id } => show(gw, id),
        PartnersCommand::Invoices(args) => invoices(gw, args),
        PartnersCommand::Add(args) => add(gw, args),
        PartnersCommand::Update(args) => update(gw, args),
        PartnersCommand::Delete { id, force } => delete(gw, id, force),
    }
}

fn list(
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

fn show(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    let data = gw.query::<ShowPartner>(show_partner::Variables { id })?;

    if let Some(p) = data.partner {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            p.id, p.name, p.vatid, p.country, p.city
        );
    }

    Ok(())
}

fn invoices(gw: &GatewayClient, args: PartnerInvoicesArgs) -> anyhow::Result<()> {
    let partner = gw
        .query::<PartnerInvoices>(partner_invoices::Variables {
            id: args.id,
            filter: args.filter,
            date_from: args.from,
            date_to: args.to,
        })?
        .partner
        .ok_or_else(|| anyhow::anyhow!("no partner with id {}", args.id))?;

    println!("{}\t{}", partner.id, partner.name);
    for i in partner.invoices.unwrap_or_default() {
        // Same status semantics and column layout as `invoices list`, so the two
        // paths read identically.
        let status = status_label(i.status, i.date_paid.as_deref());
        let title = if i.title.is_empty() {
            "(draft)"
        } else {
            &i.title
        };
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

fn add(gw: &GatewayClient, args: AddPartnerArgs) -> anyhow::Result<()> {
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

fn update(gw: &GatewayClient, args: UpdatePartnerArgs) -> anyhow::Result<()> {
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

fn delete(gw: &GatewayClient, id: i64, force: bool) -> anyhow::Result<()> {
    // Look the partner up first so the prompt names who is about to go, and so an
    // unknown id fails before asking. Deleting is not fully reversible from the
    // CLI: upstream keeps any invoices that referenced this partner but orphans
    // them, and `invoices list` then shows `-` in their client column.
    let partner = gw
        .query::<ShowPartner>(show_partner::Variables { id })?
        .partner
        .ok_or_else(|| anyhow::anyhow!("no partner with id {id}"))?;

    let question = format!("Delete partner {} ({})?", partner.name, partner.id);
    if !force && !confirm(&question)? {
        println!("aborted");
        return Ok(());
    }

    let data = gw.query::<DeletePartner>(delete_partner::Variables { id })?;

    if data.delete_partner.unwrap_or(false) {
        println!("deleted partner {id}");
    } else {
        eprintln!("partner {id} was not deleted");
    }

    Ok(())
}
