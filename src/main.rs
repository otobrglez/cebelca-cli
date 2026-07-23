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

    let vars = list_partners::Variables {
        // filter: Some(list_partners::PartnerFilter::Debtors),
        search: None,
        // page: Some(0),
    };

    // Serializes the query + variables into the `{ "query": ..., "variables": ... }`
    // JSON body the GraphQL server expects.
    let body = ListPartners::build_query(vars);

    let client = reqwest::blocking::Client::new();

    let http_res = client
        .post(GRAPHQL_URL)
        .bearer_auth(cebelca_token) // sets `Authorization: Bearer <token>`
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

    println!("--- done ---");
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
