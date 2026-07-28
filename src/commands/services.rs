//! `ceb services` — pricelist entries.

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

fn list(gw: &GatewayClient, search: Option<String>) -> anyhow::Result<()> {
    let data = gw.query::<ListServices>(list_services::Variables { search })?;

    for s in data.services.unwrap_or_default() {
        println!("{}\t{}\t{}\t{}\t{}%", s.id, s.title, s.price, s.mu, s.vat);
    }

    Ok(())
}

fn show(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
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

    match data.create_service {
        Some(s) => println!("{}\t{}\t{}\t{}\t{}%", s.id, s.title, s.price, s.mu, s.vat),
        None => eprintln!("no service returned"),
    }

    Ok(())
}

fn update(gw: &GatewayClient, args: UpdateServiceArgs) -> anyhow::Result<()> {
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

fn delete(gw: &GatewayClient, id: i64) -> anyhow::Result<()> {
    let data = gw.query::<DeleteService>(delete_service::Variables { id })?;

    if data.delete_service.unwrap_or(false) {
        println!("deleted service {id}");
    } else {
        eprintln!("service {id} was not deleted");
    }

    Ok(())
}
