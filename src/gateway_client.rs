use graphql_client::{GraphQLQuery, Response};
use reqwest::blocking::Client;
use reqwest::header::*;
use std::time::Duration;

use crate::{CebelcaGatewayURL, CebelcaToken};

static CEBELCA_USER_AGENT: &str = concat!(
    "cebelca-cli/",
    env!("CARGO_PKG_VERSION"),
    "(+https://github.com/otobrglez/cebelca-cli)"
);

pub struct GatewayClient {
    client: Client,
    url: String,
}

impl GatewayClient {
    pub fn new(url: CebelcaGatewayURL, token: CebelcaToken) -> Self {
        let mut headers = HeaderMap::new();
        let mut auth = HeaderValue::from_str(&format!("Bearer {}", token)).expect("invalid token");
        auth.set_sensitive(true);
        headers.insert(AUTHORIZATION, auth);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .user_agent(CEBELCA_USER_AGENT)
            .default_headers(headers)
            .build()
            .expect("failed to build a client");

        Self { client, url }
    }

    pub fn query<Q>(&self, variables: Q::Variables) -> anyhow::Result<Q::ResponseData>
    where
        Q: GraphQLQuery,
        Q::ResponseData: serde::de::DeserializeOwned,
    {
        let body = Q::build_query(variables);

        let response: Response<Q::ResponseData> =
            self.client.post(&self.url).json(&body).send()?.json()?;

        if let Some(errors) = response.errors {
            anyhow::bail!("GraphQL errors: {errors:?}");
        }
        response
            .data
            .ok_or_else(|| anyhow::anyhow!("no data returned"))
    }
}
