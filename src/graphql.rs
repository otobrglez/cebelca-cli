use clap::ValueEnum;
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};

// Entity-id scalars from the schema. graphql_client resolves each custom scalar
// by name against a type alias in scope here; all Čebelca ids are 64-bit ints.
pub type InvoiceID = i64;
pub type LineID = i64;
pub type PartnerID = i64;
pub type PaymentID = i64;
pub type PaymentMethodID = i64;
pub type ServiceID = i64;

// Schema enums, defined once here instead of per operation.
//
// By default graphql_client emits a private copy of every enum into each
// generated operation module, so `list_invoices::InvoiceStatus` and
// `show_invoice::InvoiceStatus` are distinct types that no single function can
// accept. Listing an enum in an operation's `extern_enums(...)` suppresses that
// copy and makes the generated code reference the type below, which is what lets
// the rest of the CLI share one `status_label`, one filter type, and (later) one
// serialization.
//
// The trade-off: codegen no longer checks these against the schema, so the names
// must match `graphql/schema.graphql` exactly and a variant added upstream would
// otherwise fail to deserialize at runtime. `enums_match_the_schema` in the tests
// below pins every variant against the schema file to turn that into a build
// failure instead.

/// An invoice's lifecycle state, as derived server-side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    Cancelled,
    Draft,
    Issued,
    Paid,
}

/// Which status bucket to list — the same tabs the Čebelca web app uses.
///
/// Doubles as the clap type for `--filter`, so a filter goes from argv to the
/// wire with no mapping in between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum InvoiceFilter {
    All,
    Archived,
    Draft,
    Paid,
    PastDue,
    Unpaid,
}

/// The kind of document an invoice is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    Advance,
    CreditNote,
    FinalInvoice,
    Invoice,
    Storno,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/list_partners.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct ListPartners;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/list_invoices.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus", "InvoiceFilter")
)]
pub struct ListInvoices;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/show_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus", "InvoiceFilter", "DocumentType")
)]
pub struct ShowInvoice;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/show_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus", "InvoiceFilter", "DocumentType")
)]
pub struct ShowInvoiceByTitle;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/resolve_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceFilter")
)]
pub struct ResolveInvoice;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/finalize_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus")
)]
pub struct FinalizeInvoice;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct CreateInvoice;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/duplicate_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus")
)]
pub struct DuplicateInvoice;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/archive_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus")
)]
pub struct ArchiveInvoice;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/show_partner.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct ShowPartner;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/partner_invoices.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone",
    extern_enums("InvoiceStatus", "InvoiceFilter")
)]
pub struct PartnerInvoices;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_partner.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct CreatePartner;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_partner.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct UpdatePartner;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_partner.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct DeletePartner;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/list_services.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct ListServices;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_service.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct CreateService;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/update_service.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct UpdateService;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_service.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct DeleteService;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/delete_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
)]
pub struct DeleteInvoice;

/// Guard the hand-written schema enums above against schema drift.
///
/// `extern_enums` opts those types out of codegen, and with it out of the
/// compile-time check that they match the schema. Without this test, a variant
/// added to `graphql/schema.graphql` (which `bin/update_schema.sh` copies in
/// wholesale from the gateway) would only surface as a deserialization failure at
/// runtime, on whichever invoice happened to carry the new value.
#[cfg(test)]
mod tests {
    /// Pull the variant names out of `enum <name> { ... }` in the schema file.
    fn schema_variants(name: &str) -> Vec<String> {
        let schema = include_str!("../graphql/schema.graphql");
        let header = format!("enum {name} {{");
        let body = schema
            .split_once(&header)
            .unwrap_or_else(|| panic!("no `{header}` in schema.graphql"))
            .1
            .split_once('}')
            .expect("unterminated enum block")
            .0;

        body.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()
    }

    /// Compare against `Debug`, which for a fieldless enum prints the variant name
    /// verbatim — so this asserts on the same spelling the wire format uses.
    macro_rules! assert_matches_schema {
        ($enum:ty, [$($variant:expr),+ $(,)?]) => {{
            let ours: Vec<String> = vec![$(format!("{:?}", $variant)),+];
            let mut sorted = ours.clone();
            sorted.sort();
            let mut theirs = schema_variants(stringify!($enum));
            theirs.sort();
            assert_eq!(
                sorted, theirs,
                "{} drifted from graphql/schema.graphql — update the enum in src/graphql.rs",
                stringify!($enum)
            );
        }};
    }

    #[test]
    fn enums_match_the_schema() {
        use super::*;

        // Listing every variant explicitly (rather than iterating) is deliberate:
        // adding one to the enum without adding it here fails to compile the
        // exhaustive match below, so neither side can drift silently.
        assert_matches_schema!(
            InvoiceStatus,
            [
                InvoiceStatus::Cancelled,
                InvoiceStatus::Draft,
                InvoiceStatus::Issued,
                InvoiceStatus::Paid,
            ]
        );
        assert_matches_schema!(
            InvoiceFilter,
            [
                InvoiceFilter::All,
                InvoiceFilter::Archived,
                InvoiceFilter::Draft,
                InvoiceFilter::Paid,
                InvoiceFilter::PastDue,
                InvoiceFilter::Unpaid,
            ]
        );
        assert_matches_schema!(
            DocumentType,
            [
                DocumentType::Advance,
                DocumentType::CreditNote,
                DocumentType::FinalInvoice,
                DocumentType::Invoice,
                DocumentType::Storno,
            ]
        );

        // The exhaustive matches that make the lists above self-checking: a new
        // variant is a compile error here, not a silently untested one.
        fn _status_is_exhaustive(s: InvoiceStatus) {
            match s {
                InvoiceStatus::Cancelled
                | InvoiceStatus::Draft
                | InvoiceStatus::Issued
                | InvoiceStatus::Paid => {}
            }
        }
        fn _filter_is_exhaustive(f: InvoiceFilter) {
            match f {
                InvoiceFilter::All
                | InvoiceFilter::Archived
                | InvoiceFilter::Draft
                | InvoiceFilter::Paid
                | InvoiceFilter::PastDue
                | InvoiceFilter::Unpaid => {}
            }
        }
        fn _doc_type_is_exhaustive(d: DocumentType) {
            match d {
                DocumentType::Advance
                | DocumentType::CreditNote
                | DocumentType::FinalInvoice
                | DocumentType::Invoice
                | DocumentType::Storno => {}
            }
        }
    }

    /// The wire format is the variant name verbatim: no rename, no case change.
    /// This is what `extern_enums` relies on, and what a stray `rename_all` would
    /// quietly break.
    #[test]
    fn variants_serialize_as_their_schema_names() {
        use super::*;

        assert_eq!(
            serde_json::to_string(&InvoiceStatus::Cancelled).unwrap(),
            "\"Cancelled\""
        );
        assert_eq!(
            serde_json::to_string(&InvoiceFilter::PastDue).unwrap(),
            "\"PastDue\""
        );
        assert_eq!(
            serde_json::to_string(&DocumentType::CreditNote).unwrap(),
            "\"CreditNote\""
        );
        assert_eq!(
            serde_json::from_str::<InvoiceStatus>("\"Issued\"").unwrap(),
            InvoiceStatus::Issued
        );
    }
}
