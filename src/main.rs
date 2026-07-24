use cebelca_cli::cli::*;
use cebelca_cli::gateway_client::GatewayClient;
use cebelca_cli::graphql::*;
use clap::Parser;

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
            // TODO: Missing pagination
            PartnersCommand::List(args) => partners_list(&client, args.search),
            PartnersCommand::Show { id } => partners_show(&client, id),
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
        Commands::Invoices { .. } => Err(not_implemented("invoices")),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn not_implemented(what: &str) -> anyhow::Error {
    anyhow::anyhow!("`{what}` is not implemented yet")
}

fn partners_list(gw: &GatewayClient, search: Option<String>) -> anyhow::Result<()> {
    let data = gw.query::<ListPartners>(list_partners::Variables { search })?;

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
