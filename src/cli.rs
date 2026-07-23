use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, name = "cb", bin_name = "cb", long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    /// API token. Falls back to the CEBELCA_TOKEN environment variable.
    #[arg(long, env = "CEBELCA_TOKEN", global = true)]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Partners {
        #[command(subcommand)]
        command: PartnersCommand,
    },
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
    },
    Invoices {
        #[command(subcommand)]
        command: InvoicesCommand,
    },
}

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(long, default_value_t = 1)]
    pub page: u32,

    #[arg(long, default_value_t = 25)]
    pub per_page: u32,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    pub query: String,

    #[command(flatten)]
    pub list: ListArgs,
}

#[derive(Subcommand, Debug)]
pub enum PartnersCommand {
    List(ListArgs),
    Search(SearchArgs),
    Show { id: i64 },
}

#[derive(Subcommand, Debug)]
pub enum ServicesCommand {
    List(ListArgs),
    Search(SearchArgs),
    Show { id: i64 },
}

#[derive(Subcommand, Debug)]
pub enum InvoicesCommand {
    List(ListArgs),
    Search(SearchArgs),
    Show { id: i64 },
    Finalize { id: i64 },
}
