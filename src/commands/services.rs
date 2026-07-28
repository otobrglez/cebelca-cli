//! `ceb services` — pricelist entries.

use super::{expect_returned, report_deleted};
use crate::cli::{AddServiceArgs, SearchArgs, ServicesCommand, UpdateServiceArgs};
use crate::gateway_client::GatewayClient;
use crate::graphql::*;

/// Run whichever `services` subcommand was given; `None` means the group was
/// named bare and defaults to `list` (see [`super::partners::dispatch`]).
pub fn dispatch(
    gw: &GatewayClient,
    command: Option<ServicesCommand>,
    list: SearchArgs,
) -> anyhow::Result<()> {
    match command.unwrap_or(ServicesCommand::List(list)) {
        ServicesCommand::List(args) => self::list(gw, args.search),
        ServicesCommand::Show { id } => show(gw, id),
        ServicesCommand::Add(args) => add(gw, args),
        ServicesCommand::Update(args) => update(gw, args),
        ServicesCommand::Delete { id } => delete(gw, id),
    }
}

/// The summary row shared by `list`, `add` and `update`. `show` prints a wider
/// row (it adds group and konto), but the leading five columns must match, so
/// both go through here.
fn print_service(id: i64, title: &str, price: f64, mu: &str, vat: f64) {
    println!("{id}\t{title}\t{price}\t{mu}\t{vat}%");
}

/// Fetch one service by id.
///
/// The schema has no singular `service(id)` query (unlike partners), so the only
/// way to read one is to fetch the list and scan it client-side. Both `show` and
/// `update` need this, and `update` needs it because updateService is a full
/// replace.
///
/// TODO: drop this once the gateway grows a `service(id: ServiceID!)` query —
/// then the two callers become one round trip each instead of a full list fetch.
fn service_by_id(
    gw: &GatewayClient,
    id: i64,
) -> anyhow::Result<list_services::ListServicesServices> {
    gw.query::<ListServices>(list_services::Variables { search: None })?
        .services
        .unwrap_or_default()
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("no service with id {id}"))
}

fn list(gw: &GatewayClient, search: Option<String>) -> anyhow::Result<()> {
    let data = gw.query::<ListServices>(list_services::Variables { search })?;

    for s in data.services.unwrap_or_default() {
        print_service(s.id, &s.title, s.price, &s.mu, s.vat);
    }

    Ok(())
}

fn show(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    let s = service_by_id(gw, id)?;

    println!(
        "{}\t{}\t{}\t{}\t{}%\t{}\t{}",
        s.id, s.title, s.price, s.mu, s.vat, s.group, s.konto
    );

    Ok(())
}

fn add(gw: &GatewayClient, args: AddServiceArgs) -> anyhow::Result<()> {
    let input = create_service::ServiceInput {
        title: args.title,
        price: args.price,
        mu: args.mu,
        vat: args.vat,
        group: args.group,
        konto: args.konto,
    };

    let data = gw.query::<CreateService>(create_service::Variables { input })?;
    let s = expect_returned(data.create_service, "service")?;

    print_service(s.id, &s.title, s.price, &s.mu, s.vat);
    Ok(())
}

fn update(gw: &GatewayClient, args: UpdateServiceArgs) -> anyhow::Result<()> {
    // The gateway's updateService is a full replace: omitted fields are
    // overwritten with defaults. So read the current record first and overlay
    // only the flags the user actually passed.
    let current = service_by_id(gw, args.id)?;

    let input = update_service::ServiceInput {
        title: args.title.unwrap_or(current.title),
        price: args.price.unwrap_or(current.price),
        mu: args.mu.unwrap_or(current.mu),
        vat: args.vat.unwrap_or(current.vat),
        group: Some(args.group.unwrap_or(current.group)),
        konto: Some(args.konto.unwrap_or(current.konto)),
    };

    let data = gw.query::<UpdateService>(update_service::Variables { id: args.id, input })?;
    let s = expect_returned(data.update_service, "service")?;

    print_service(s.id, &s.title, s.price, &s.mu, s.vat);
    Ok(())
}

fn delete(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    let data = gw.query::<DeleteService>(delete_service::Variables { id })?;
    report_deleted(data.delete_service, &format!("service {id}"))
}
