use cebelca_cli::cli::*;
use cebelca_cli::graphql::*;
use clap::Parser;
use graphql_client::{GraphQLQuery, Response};

const GRAPHQL_URL: &str = "https://cebelca-gateway.pinkstack.com/api/graphql";

fn main() {
    let cli = CLI::parse();

    // Token comes from --token or the CEBELCA_TOKEN env var (see cli.rs).
    let cebelca_token = cli.token.unwrap_or_else(|| {
        eprintln!("error: no API token. Pass --token or set CEBELCA_TOKEN.");
        std::process::exit(1);
    });

    let client = reqwest::blocking::Client::new();

    match cli.command {
        Commands::Partners { command } => match command {
            // `partners list` — no search term.
            PartnersCommand::List(_) => fetch_partners(&client, &cebelca_token, None),
            // `partners search <query>` — pass the query through as the search var.
            PartnersCommand::Search(args) => {
                fetch_partners(&client, &cebelca_token, Some(args.query))
            }
            PartnersCommand::Show { id } => eprintln!("partners show {id}: not implemented yet"),
        },
        Commands::Services { .. } => eprintln!("services: not implemented yet"),
        Commands::Invoices { command } => match command {
            InvoicesCommand::List { filter, .. } => {
                fetch_invoices(&client, &cebelca_token, filter.map(to_gql_filter))
            }
            InvoicesCommand::Finalize { id } => finalize_invoice(&client, &cebelca_token, id),
        },
    }
}

/// Map the CLI's InvoiceFilter to the GraphQL-generated one. They have the same
/// variants (both derived from the schema's `InvoiceFilter` enum).
fn to_gql_filter(f: InvoiceFilter) -> list_invoices::InvoiceFilter {
    use list_invoices::InvoiceFilter as G;
    match f {
        InvoiceFilter::All => G::All,
        InvoiceFilter::Archived => G::Archived,
        InvoiceFilter::Paid => G::Paid,
        InvoiceFilter::PastDue => G::PastDue,
        InvoiceFilter::Unpaid => G::Unpaid,
    }
}

/// Fetch partners, optionally filtered by a free-text `search` term, and print
/// them one per line. `search = None` lists everything.
fn fetch_partners(client: &reqwest::blocking::Client, token: &str, search: Option<String>) {
    let vars = list_partners::Variables { search };

    // Serializes the query + variables into the `{ "query": ..., "variables": ... }`
    // JSON body the GraphQL server expects.
    let body = ListPartners::build_query(vars);

    let http_res = client
        .post(GRAPHQL_URL)
        .bearer_auth(token) // sets `Authorization: Bearer <token>`
        .json(&body) // sets Content-Type: application/json + serializes body
        .send()
        .expect("request failed");

    // Deserialize into the typed GraphQL response for this query.
    let res: Response<list_partners::ResponseData> =
        http_res.json().expect("failed to decode response body");

    if let Some(errors) = res.errors {
        eprintln!("GraphQL errors: {errors:?}");
    }

    match res.data {
        Some(data) => {
            for p in data.partners.unwrap_or_default() {
                println!("{}\t{}\t{}", p.id, p.name, p.city);
            }
        }
        None => eprintln!("no data returned"),
    }
}

/// List invoices, optionally filtered by status. `filter = None` lists all.
fn fetch_invoices(
    client: &reqwest::blocking::Client,
    token: &str,
    filter: Option<list_invoices::InvoiceFilter>,
) {
    let vars = list_invoices::Variables { filter };
    let body = ListInvoices::build_query(vars);

    let http_res = client
        .post(GRAPHQL_URL)
        .bearer_auth(token)
        .json(&body)
        .send()
        .expect("request failed");

    let res: Response<list_invoices::ResponseData> =
        http_res.json().expect("failed to decode response body");

    if let Some(errors) = res.errors {
        eprintln!("GraphQL errors: {errors:?}");
    }

    match res.data {
        Some(data) => {
            for i in data.invoices.unwrap_or_default() {
                let paid = if i.paid { "paid" } else { "unpaid" };
                println!("{}\t{}\t{}\t{}", i.id, i.title, i.date_sent, paid);
            }
        }
        None => eprintln!("no data returned"),
    }
}

/// Finalize (issue) a draft invoice by id.
fn finalize_invoice(client: &reqwest::blocking::Client, token: &str, id: i64) {
    let vars = finalize_invoice::Variables { id };
    let body = FinalizeInvoice::build_query(vars);

    let http_res = client
        .post(GRAPHQL_URL)
        .bearer_auth(token)
        .json(&body)
        .send()
        .expect("request failed");

    let res: Response<finalize_invoice::ResponseData> =
        http_res.json().expect("failed to decode response body");

    if let Some(errors) = res.errors {
        eprintln!("GraphQL errors: {errors:?}");
    }

    match res.data.and_then(|d| d.finalize_invoice) {
        Some(inv) => println!("finalized invoice {} — {}", inv.id, inv.title),
        None => eprintln!("no invoice returned"),
    }
}

/*
async fn run(client: &reqwest::Client, url: &str) -> anyhow::Result<()> {
    let vars = list_partners::Variables {
        filter: Some(list_partners::PartnerFilter::Debtors),
        search: None,
        page: Some(0),
    };
    let body = ListPartners::build_query(vars);
    let res: Response<list_partners::ResponseData> =
        client.post(url).json(&body).send().await?.json().await?;

    if let Some(errs) = res.errors {
        anyhow::bail!("{errs:?}");
    }
    for p in res.data.and_then(|d| d.partners).unwrap_or_default() {
        println!("{}\t{}\t{}", p.id, p.name, p.city);
    }
    Ok(())
} */
