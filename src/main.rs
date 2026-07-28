use cebelca_cli::cli::{CLI, Commands};
use cebelca_cli::commands;
use cebelca_cli::gateway_client::GatewayClient;
use clap::Parser;

// TODO: Use better/propper logger.
// use log::{error, info};

fn main() {
    let cli = CLI::parse();

    let gateway_url: String = cli.gateway_url;
    let token = cli.token.unwrap_or_else(|| {
        eprintln!("error: no API token. Pass --token or set CEBELCA_TOKEN.");
        std::process::exit(1);
    });

    let client = GatewayClient::new(gateway_url, token);

    // Each group owns its own subcommand dispatch (see `commands`), so this stays
    // one arm per group no matter how many subcommands each grows.
    let result = match cli.command {
        Commands::Partners { command, list } => {
            commands::partners::dispatch(&client, command, list)
        }
        Commands::Services { command, list } => {
            commands::services::dispatch(&client, command, list)
        }
        Commands::Invoices { command, list } => {
            commands::invoices::dispatch(&client, command, list)
        }
    };

    if let Err(err) = result {
        // `{err:#}` renders anyhow's whole context chain (`context: cause`), not
        // just the outermost message — so a wrapped failure still shows what
        // actually went wrong underneath.
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
