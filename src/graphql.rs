use graphql_client::GraphQLQuery;

pub type Long = i64;

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
    variables_derives = "Debug, Clone"
)]
pub struct ListInvoices;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/finalize_invoice.graphql",
    response_derives = "Debug, Clone",
    variables_derives = "Debug, Clone"
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
    variables_derives = "Debug, Clone"
)]
pub struct DuplicateInvoice;

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
