use crate::{CebelcaGatewayURL, CebelcaToken};
use clap::{Args, Parser, Subcommand, ValueEnum};

const CEBELCA_GATEWAY_URL: &str = "https://cebelca-gateway.pinkstack.com/api/graphql";

#[derive(Parser, Debug)]
#[command(author, version, about, name = "cb", bin_name = "cb", long_about = None)]
#[command(propagate_version = true)]
pub struct CLI {
    /// API token (or set CEBELCA_TOKEN).
    #[arg(long, env = "CEBELCA_TOKEN", global = true, hide_env_values = true)]
    pub token: Option<CebelcaToken>,

    /// GraphQL gateway endpoint (or set CEBELCA_GATEWAY_URL).
    #[arg(long, env = "CEBELCA_GATEWAY_URL", global = true, default_value = CEBELCA_GATEWAY_URL)]
    pub gateway_url: CebelcaGatewayURL,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage partners (customers and suppliers).
    Partners {
        #[command(subcommand)]
        command: PartnersCommand,
    },
    /// Manage services (pricelist entries).
    Services {
        #[command(subcommand)]
        command: ServicesCommand,
    },
    /// List, create, and finalize invoices.
    Invoices {
        #[command(subcommand)]
        command: InvoicesCommand,
    },
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter results by a free-text search query.
    #[arg(short, long)]
    pub search: Option<String>,

    /// Page number (1-based).
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    //    #[arg(long, default_value_t = 25)]
    //    pub per_page: u32,
}

#[derive(Args, Debug)]
pub struct AddPartnerArgs {
    /// Partner name (required).
    pub name: String,

    /// Street address.
    #[arg(long)]
    pub street: Option<String>,
    /// Postal code.
    #[arg(long)]
    pub postal: Option<String>,
    /// City.
    #[arg(long)]
    pub city: Option<String>,
    /// VAT id.
    #[arg(long)]
    pub vatid: Option<String>,
    /// Country.
    #[arg(long)]
    pub country: Option<String>,
    /// Language code.
    #[arg(long)]
    pub lang: Option<String>,
}

/// Update an existing partner.
///
/// Only the flags you pass are modified; everything else is preserved from the
/// partner's current record. (The gateway's updatePartner is a full replace, so
/// the CLI reads the current values first and overlays these on top.)
#[derive(Args, Debug)]
pub struct UpdatePartnerArgs {
    /// Partner id to update.
    pub id: i64,

    /// New name.
    #[arg(long)]
    pub name: Option<String>,
    /// New street address.
    #[arg(long)]
    pub street: Option<String>,
    /// New postal code.
    #[arg(long)]
    pub postal: Option<String>,
    /// New city.
    #[arg(long)]
    pub city: Option<String>,
    /// New VAT id.
    #[arg(long)]
    pub vatid: Option<String>,
    /// New country.
    #[arg(long)]
    pub country: Option<String>,
    /// New language code.
    #[arg(long)]
    pub lang: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum PartnersCommand {
    /// List partners (optionally filtered with --search).
    #[clap(alias = "ls")]
    List(ListArgs),
    /// Show a single partner by id.
    Show {
        /// Partner id.
        id: i64,
    },
    /// Create a new partner.
    Add(AddPartnerArgs),
    /// Update an existing partner.
    Update(UpdatePartnerArgs),
}

#[derive(Args, Debug)]
pub struct AddServiceArgs {
    /// Service title (required).
    pub title: String,

    /// Price per unit (required).
    #[arg(long)]
    pub price: f64,
    /// Measure unit, e.g. "kos", "ura" (required).
    #[arg(long)]
    pub mu: String,
    /// VAT rate as a percentage, e.g. 22 (required).
    #[arg(long)]
    pub vat: f64,
    /// Group / category.
    #[arg(long)]
    pub group: Option<String>,
    /// Accounting code (konto).
    #[arg(long)]
    pub konto: Option<String>,
}

/// Update an existing service.
///
/// Only the flags you pass are modified; everything else is preserved from the
/// service's current record. (The gateway's updateService is a full replace, so
/// the CLI reads the current values first and overlays these on top.)
#[derive(Args, Debug)]
pub struct UpdateServiceArgs {
    /// Service id to update.
    pub id: i64,

    /// New title.
    #[arg(long)]
    pub title: Option<String>,
    /// New price per unit.
    #[arg(long)]
    pub price: Option<f64>,
    /// New measure unit.
    #[arg(long)]
    pub mu: Option<String>,
    /// New VAT rate (percentage).
    #[arg(long)]
    pub vat: Option<f64>,
    /// New group / category.
    #[arg(long)]
    pub group: Option<String>,
    /// New accounting code (konto).
    #[arg(long)]
    pub konto: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ServicesCommand {
    /// List services (optionally filtered with --search).
    #[clap(alias = "ls")]
    List(ListArgs),
    /// Show a single service by id.
    Show {
        /// Service id.
        id: i64,
    },
    /// Create a new service.
    Add(AddServiceArgs),
    /// Update an existing service.
    Update(UpdateServiceArgs),
    /// Delete a service by id.
    Delete {
        /// Service id.
        id: i64,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum InvoiceFilter {
    All,
    Archived,
    Paid,
    PastDue,
    Unpaid,
}

/// A single invoice line, given as `key=value` pairs separated by commas, e.g.
/// `title=Consulting,qty=10,price=100,vat=22,mu=kos,discount=0`.
///
/// `title`, `qty`, `price`, and `vat` are required; `mu` and `discount` are
/// optional.
#[derive(Clone, Debug)]
pub struct InvoiceLine {
    pub title: String,
    pub qty: f64,
    pub price: f64,
    pub vat: f64,
    pub mu: Option<String>,
    pub discount: Option<f64>,
}

impl std::str::FromStr for InvoiceLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut title = None;
        let mut qty = None;
        let mut price = None;
        let mut vat = None;
        let mut mu = None;
        let mut discount = None;

        for pair in s.split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let (key, value) = pair
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, got `{pair}`"))?;
            let key = key.trim();
            let value = value.trim();
            let parse_num = |v: &str| v.parse::<f64>().map_err(|_| format!("`{key}` must be a number, got `{v}`"));
            match key {
                "title" => title = Some(value.to_string()),
                "qty" => qty = Some(parse_num(value)?),
                "price" => price = Some(parse_num(value)?),
                "vat" => vat = Some(parse_num(value)?),
                "mu" => mu = Some(value.to_string()),
                "discount" => discount = Some(parse_num(value)?),
                other => return Err(format!("unknown line field `{other}`")),
            }
        }

        Ok(InvoiceLine {
            title: title.ok_or("line is missing `title`")?,
            qty: qty.ok_or("line is missing `qty`")?,
            price: price.ok_or("line is missing `price`")?,
            vat: vat.ok_or("line is missing `vat`")?,
            mu,
            discount,
        })
    }
}

#[derive(Args, Debug)]
pub struct AddInvoiceArgs {
    /// Partner id to invoice (required).
    #[arg(long)]
    pub partner_id: i64,
    /// Date the invoice is sent, e.g. 2026-07-30 (required).
    #[arg(long)]
    pub date_sent: String,
    /// Payment due date, e.g. 2026-08-10 (required).
    #[arg(long)]
    pub date_to_pay: String,
    /// Date the service was rendered / goods delivered.
    #[arg(long)]
    pub date_served: Option<String>,
    /// Invoice line, repeatable. Format:
    /// `title=Consulting,qty=10,price=100,vat=22[,mu=kos,discount=0]`.
    #[arg(long = "line", value_name = "KEY=VAL,...")]
    pub lines: Vec<InvoiceLine>,
}

#[derive(Subcommand, Debug)]
pub enum InvoicesCommand {
    /// List invoices, optionally filtered by status.
    List {
        /// Filter by invoice status.
        #[arg(long, value_enum)]
        filter: Option<InvoiceFilter>,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Create a new draft invoice.
    Add(AddInvoiceArgs),
    /// Finalize (issue) a draft invoice.
    Finalize {
        /// Invoice id.
        id: i64,
    },
    /// Duplicate an existing invoice into a new draft.
    Duplicate {
        /// Invoice id to duplicate.
        id: i64,
    },
}
