use clap::{Args, Parser, Subcommand, ValueEnum};

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

/// Server-side invoice filters. Mirrors the `InvoiceFilter` GraphQL enum.
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum InvoiceFilter {
    All,
    Archived,
    Paid,
    PastDue,
    Unpaid,
}

#[derive(Subcommand, Debug)]
pub enum InvoicesCommand {
    /// List invoices, optionally filtered by status.
    List {
        /// Filter by status (defaults to all if omitted).
        #[arg(long, value_enum)]
        filter: Option<InvoiceFilter>,

        #[command(flatten)]
        list: ListArgs,
    },
    /// Finalize (issue) a draft invoice — invoices-only command.
    Finalize {
        /// Invoice id.
        id: i64,
    },
}
