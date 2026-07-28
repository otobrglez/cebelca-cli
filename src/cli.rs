use crate::{CebelcaGatewayURL, CebelcaToken};
use clap::{Args, Parser, Subcommand, ValueEnum};

const CEBELCA_GATEWAY_URL: &str = "https://cebelca-gateway.pinkstack.com/api/graphql";

#[derive(Parser, Debug)]
#[command(author, version, about, name = "ceb", bin_name = "ceb", long_about = None)]
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

/// Top-level command groups.
///
/// Each group's subcommand is optional and defaults to `list`, so `ceb partners`
/// is `ceb partners list`. The group's list arguments are `flatten`ed alongside
/// the subcommand, which is what lets the bare form take them directly
/// (`ceb partners -s acme`). clap resolves a leading token to the subcommand when
/// it names one, so `list`/`ls` and the flags never collide.
///
/// Every group defaults to the same verb deliberately: a CLI where only some
/// groups answer bare would be harder to remember than one where none do.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage partners (customers and suppliers). Without a subcommand, lists them.
    Partners {
        #[command(subcommand)]
        command: Option<PartnersCommand>,
        #[command(flatten)]
        list: ListArgs,
    },
    /// Manage services (pricelist entries). Without a subcommand, lists them.
    Services {
        #[command(subcommand)]
        command: Option<ServicesCommand>,
        #[command(flatten)]
        list: SearchArgs,
    },
    /// List, create, and finalize invoices. Without a subcommand, lists them.
    Invoices {
        #[command(subcommand)]
        command: Option<InvoicesCommand>,
        #[command(flatten)]
        list: ListInvoicesArgs,
    },
}

/// Search + pagination for list commands whose gateway query supports both
/// (partners). Pages are 1-based here and translated to the gateway's 0-based
/// index at the call site. `--per-page` sets the page size; omit it (or the whole
/// pair) to fetch the full unpaged list.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter results by a free-text search query.
    #[arg(short, long)]
    pub search: Option<String>,

    /// Page number (1-based). Requires --per-page to take effect.
    #[arg(long)]
    pub page: Option<u32>,

    /// Page size. Omit for the full, unpaged list.
    #[arg(long)]
    pub per_page: Option<u32>,
}

/// Search only, for list commands whose gateway query has no pagination
/// (services, and — because the upstream API ignores it — invoices).
#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Filter results by a free-text search query.
    #[arg(short, long)]
    pub search: Option<String>,
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

/// List invoices for a single partner.
///
/// This is the only way the gateway exposes partner-scoped invoices: the
/// top-level `invoices` query has no partner argument, so we go through
/// `partner(id) { invoices(...) }`.
#[derive(Args, Debug)]
pub struct PartnerInvoicesArgs {
    /// Partner id.
    pub id: i64,

    /// Filter by invoice status.
    #[arg(long, value_enum)]
    pub filter: Option<InvoiceFilter>,
    /// Only invoices sent on/after this date (YYYY-MM-DD).
    #[arg(long)]
    pub from: Option<String>,
    /// Only invoices sent on/before this date (YYYY-MM-DD).
    #[arg(long)]
    pub to: Option<String>,
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
    /// List invoices for a partner.
    Invoices(PartnerInvoicesArgs),
    /// Create a new partner.
    Add(AddPartnerArgs),
    /// Update an existing partner.
    Update(UpdatePartnerArgs),
    /// Delete a partner by id.
    ///
    /// Prompts first, because upstream keeps the partner's invoices and simply
    /// orphans them: their client column then renders as `-`.
    Delete {
        /// Partner id.
        id: i64,
        /// Skip the confirmation prompt.
        #[arg(long)]
        force: bool,
    },
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
    List(SearchArgs),
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
    Draft,
    Paid,
    PastDue,
    Unpaid,
}

/// Which lookup an invoice command should perform: by invoice id, or by document
/// number (the invoice's `title`, e.g. `021/26`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InvoiceRef {
    Id(i64),
    Number(String),
}

/// How every invoice command names its invoice: one positional that is either an
/// id or a document number, plus `--number` to settle the ambiguous case.
///
/// Shared via `#[command(flatten)]` so `show`, `finalize`, `duplicate`, `delete`
/// and `archive` all accept the same forms and can't drift apart — and so adding
/// the next by-id command is one line.
#[derive(Args, Debug)]
pub struct InvoiceRefArgs {
    /// Invoice id, or document number (e.g. 021/26).
    #[arg(value_name = "ID|NUMBER")]
    pub invoice: String,
    /// Treat the argument as a document number, never as an id.
    #[arg(long, short = 'n')]
    pub number: bool,
}

impl InvoiceRefArgs {
    /// Resolve the pair into an [`InvoiceRef`], per [`resolve_invoice_ref`].
    pub fn parse(&self) -> Result<InvoiceRef, String> {
        resolve_invoice_ref(&self.invoice, self.number)
    }
}

/// Decide whether `arg` names an invoice id or a document number.
///
/// An all-digit argument is an id, anything else a number — which covers every
/// real Čebelca numbering series (`021/26`, `26-0007`) without needing a flag.
/// The one case the shape can't settle is a number that happens to be all digits,
/// so `--number` (`force_number`) short-circuits the guess.
///
/// Deliberately no try-as-id-then-fall-back-to-number: that costs a second round
/// trip and turns a plain "no such invoice" into an ambiguous two-part failure.
pub fn resolve_invoice_ref(arg: &str, force_number: bool) -> Result<InvoiceRef, String> {
    let arg = arg.trim();
    if arg.is_empty() {
        return Err("invoice id or number must not be blank".to_string());
    }
    if force_number {
        return Ok(InvoiceRef::Number(arg.to_string()));
    }
    if arg.chars().all(|c| c.is_ascii_digit()) {
        // All digits, so an id — unless it doesn't fit in one, which a real id
        // always does. Point at --number rather than silently retrying it as a
        // document number.
        return arg.parse::<i64>().map(InvoiceRef::Id).map_err(|_| {
            format!("`{arg}` is too large to be an invoice id; pass --number to look it up as a document number")
        });
    }
    Ok(InvoiceRef::Number(arg.to_string()))
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
    /// Tag for the invoice, repeatable (e.g. --tag urgent --tag q3).
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,
    /// Invoice line, repeatable. Format:
    /// `title=Consulting,qty=10,price=100,vat=22[,mu=kos,discount=0]`.
    #[arg(long = "line", value_name = "KEY=VAL,...")]
    pub lines: Vec<InvoiceLine>,
}

/// Status filter for `invoices list`. Its own `Args` struct (rather than inline
/// fields on the variant) so the bare `ceb invoices` form can flatten the same
/// arguments — see [[Commands]].
#[derive(Args, Debug)]
pub struct ListInvoicesArgs {
    /// Filter by invoice status.
    #[arg(long, value_enum)]
    pub filter: Option<InvoiceFilter>,
}

#[derive(Subcommand, Debug)]
pub enum InvoicesCommand {
    /// List invoices, optionally filtered by status.
    ///
    /// Not paginated: the upstream cebelca API ignores paging for invoices and
    /// returns the whole (filtered) set, so the gateway exposes no page argument.
    #[clap(alias = "ls")]
    List(ListInvoicesArgs),
    /// Show a single invoice, with its partner and lines.
    ///
    /// Takes either an invoice id or a document number: an all-digit argument is
    /// read as an id, anything else (e.g. 021/26) as a number. Pass --number to
    /// force a number lookup for a number that is all digits.
    ///
    /// Drafts have no number yet, so they are only reachable by id.
    Show {
        #[command(flatten)]
        invoice: InvoiceRefArgs,
    },
    /// Create a new draft invoice.
    Add(AddInvoiceArgs),
    /// Finalize (issue) a draft invoice.
    ///
    /// Takes an id or a document number (see `show`). A draft has no number yet,
    /// so in practice this is called by id — a number would have to belong to an
    /// already-issued invoice.
    Finalize {
        #[command(flatten)]
        invoice: InvoiceRefArgs,
        /// Override the assigned invoice number (e.g. 26-0007). Omit to let the
        /// server assign the next number in the series. Must be unique.
        #[arg(long)]
        title: Option<String>,
    },
    /// Duplicate an existing invoice into a new draft.
    ///
    /// Takes an id or a document number (see `show`). The copy is a fresh draft
    /// (no number). Upstream clears the number and tags on the copy, so pass
    /// --title to name it and --tag to carry labels over.
    Duplicate {
        #[command(flatten)]
        invoice: InvoiceRefArgs,
        /// Set the new draft's number/title. Omit for a numberless draft.
        #[arg(long)]
        title: Option<String>,
        /// Tag(s) for the new draft, repeatable (e.g. --tag urgent --tag q3).
        #[arg(long = "tag", value_name = "TAG")]
        tags: Vec<String>,
    },
    /// Delete an invoice, by id or document number (see `show`).
    Delete {
        #[command(flatten)]
        invoice: InvoiceRefArgs,
        /// Skip the confirmation prompt.
        #[arg(long)]
        force: bool,
    },
    /// Archive an invoice (or restore it with --restore).
    ///
    /// Takes an id or a document number (see `show`). Archiving moves the invoice
    /// into the "archived" tab (status Cancelled) without deleting it; --restore
    /// reverses that. Both preserve the invoice number and all other fields.
    Archive {
        #[command(flatten)]
        invoice: InvoiceRefArgs,
        /// Restore (un-archive) instead of archiving.
        #[arg(long)]
        restore: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_digits_is_an_id() {
        assert_eq!(resolve_invoice_ref("323", false), Ok(InvoiceRef::Id(323)));
        assert_eq!(resolve_invoice_ref(" 323 ", false), Ok(InvoiceRef::Id(323)));
    }

    #[test]
    fn anything_else_is_a_document_number() {
        // the real numbering series in use, slash and all
        assert_eq!(
            resolve_invoice_ref("021/26", false),
            Ok(InvoiceRef::Number("021/26".to_string()))
        );
        assert_eq!(
            resolve_invoice_ref("26-0007", false),
            Ok(InvoiceRef::Number("26-0007".to_string()))
        );
        // a leading `-` is not a negative id: ids are never negative, so this is a number
        assert_eq!(
            resolve_invoice_ref("-5", false),
            Ok(InvoiceRef::Number("-5".to_string()))
        );
    }

    #[test]
    fn force_number_overrides_the_all_digit_guess() {
        assert_eq!(
            resolve_invoice_ref("0210", true),
            Ok(InvoiceRef::Number("0210".to_string()))
        );
    }

    #[test]
    fn blank_is_rejected() {
        // upstream treats an empty search as a wildcard, so never send one
        assert!(resolve_invoice_ref("", false).is_err());
        assert!(resolve_invoice_ref("   ", true).is_err());
    }

    #[test]
    fn an_oversized_id_points_at_the_number_flag() {
        let err = resolve_invoice_ref("99999999999999999999", false).unwrap_err();
        assert!(err.contains("--number"), "unhelpful message: {err}");
    }

    /// Parse an argv (minus the binary name), with a token supplied so the
    /// environment can't influence the result.
    fn parse(args: &[&str]) -> Result<CLI, clap::Error> {
        let mut argv = vec!["ceb", "--token", "t"];
        argv.extend_from_slice(args);
        CLI::try_parse_from(argv)
    }

    #[test]
    fn a_bare_group_leaves_the_subcommand_unset() {
        // `None` is what main turns into the group's `list` default
        assert!(parse_partners(&["partners"]).0.is_none());
        match parse(&["services"]).unwrap().command {
            Commands::Services { command, .. } => assert!(command.is_none()),
            other => panic!("expected Services, got {other:?}"),
        }
        match parse(&["invoices"]).unwrap().command {
            Commands::Invoices { command, .. } => assert!(command.is_none()),
            other => panic!("expected Invoices, got {other:?}"),
        }
    }

    /// Parse a `partners` argv down to its group parts, panicking on anything else.
    fn parse_partners(args: &[&str]) -> (Option<PartnersCommand>, ListArgs) {
        match parse(args).unwrap().command {
            Commands::Partners { command, list } => (command, list),
            other => panic!("expected Partners, got {other:?}"),
        }
    }

    #[test]
    fn a_bare_group_still_takes_the_list_arguments() {
        let (command, list) = parse_partners(&["partners", "-s", "acme", "--per-page", "5"]);
        assert!(command.is_none());
        assert_eq!(list.search.as_deref(), Some("acme"));
        assert_eq!(list.per_page, Some(5));
    }

    #[test]
    fn an_explicit_list_subcommand_still_parses() {
        let (command, _) = parse_partners(&["partners", "list", "-s", "acme"]);
        match command {
            Some(PartnersCommand::List(args)) => assert_eq!(args.search.as_deref(), Some("acme")),
            other => panic!("expected List, got {other:?}"),
        }
        // and so does the alias
        let (alias, _) = parse_partners(&["partners", "ls"]);
        assert!(matches!(alias, Some(PartnersCommand::List(_))));
    }

    #[test]
    fn a_subcommand_name_wins_over_the_flattened_arguments() {
        // the risk of flattening: `show` must stay a subcommand, not become a value
        let (command, _) = parse_partners(&["partners", "show", "7"]);
        assert!(matches!(command, Some(PartnersCommand::Show { id: 7 })));
    }

    #[test]
    fn an_unknown_subcommand_is_an_error_not_a_silent_list() {
        assert!(parse(&["partners", "bogus"]).is_err());
    }

    #[test]
    fn partners_delete_takes_an_id_and_an_opt_out_of_the_prompt() {
        // the prompt is the default; --force is the only way past it
        let deleted = |args: &[&str]| match parse_partners(args).0 {
            Some(PartnersCommand::Delete { id, force }) => (id, force),
            other => panic!("expected Delete, got {other:?}"),
        };
        assert_eq!(deleted(&["partners", "delete", "7"]), (7, false));
        assert_eq!(deleted(&["partners", "delete", "7", "--force"]), (7, true));
    }

    /// Pull the [`InvoiceRefArgs`] out of whichever invoice subcommand carries one.
    fn invoice_ref_of(args: &[&str]) -> InvoiceRef {
        let command = match parse(args).unwrap().command {
            Commands::Invoices { command, .. } => command.expect("expected a subcommand"),
            other => panic!("expected Invoices, got {other:?}"),
        };
        let refargs = match &command {
            InvoicesCommand::Show { invoice }
            | InvoicesCommand::Finalize { invoice, .. }
            | InvoicesCommand::Duplicate { invoice, .. }
            | InvoicesCommand::Delete { invoice, .. }
            | InvoicesCommand::Archive { invoice, .. } => invoice,
            other => panic!("no invoice ref on {other:?}"),
        };
        refargs.parse().unwrap()
    }

    #[test]
    fn every_invoice_command_takes_an_id_or_a_number() {
        for verb in ["show", "finalize", "duplicate", "delete", "archive"] {
            assert_eq!(
                invoice_ref_of(&["invoices", verb, "323"]),
                InvoiceRef::Id(323),
                "{verb} did not read 323 as an id"
            );
            assert_eq!(
                invoice_ref_of(&["invoices", verb, "021/26"]),
                InvoiceRef::Number("021/26".to_string()),
                "{verb} did not read 021/26 as a number"
            );
            assert_eq!(
                invoice_ref_of(&["invoices", verb, "-n", "0210"]),
                InvoiceRef::Number("0210".to_string()),
                "{verb} ignored --number"
            );
        }
    }

    #[test]
    fn invoice_commands_keep_their_own_flags_alongside_the_ref() {
        match parse(&["invoices", "delete", "021/26", "--force"])
            .unwrap()
            .command
        {
            Commands::Invoices {
                command: Some(InvoicesCommand::Delete { invoice, force }),
                ..
            } => {
                assert!(force);
                assert_eq!(invoice.parse(), Ok(InvoiceRef::Number("021/26".into())));
            }
            other => panic!("expected Delete, got {other:?}"),
        }
        match parse(&["invoices", "finalize", "565", "--title", "26-0100"])
            .unwrap()
            .command
        {
            Commands::Invoices {
                command: Some(InvoicesCommand::Finalize { invoice, title }),
                ..
            } => {
                assert_eq!(title.as_deref(), Some("26-0100"));
                assert_eq!(invoice.parse(), Ok(InvoiceRef::Id(565)));
            }
            other => panic!("expected Finalize, got {other:?}"),
        }
    }
}
