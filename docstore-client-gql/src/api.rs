use async_graphql::extensions::Tracing;
use async_graphql::{Context, EmptySubscription, ErrorExtensions, InputObject, Object, Schema};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use tracing::instrument;
use url::Url;
use uuid::Uuid;

use petstore_domain::model;
use petstore_domain::model::error::Error as ModelError;
use petstore_domain::model::owner::Owner;
use petstore_domain::ports::primary::storage::OwnerStorage;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Model Error: {} - {}", msg, source))]
    Model { msg: String, source: ModelError },

    #[snafu(display("Request Error: {} - {}", msg, source))]
    Reqwest { msg: String, source: reqwest::Error },
}

impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        self.extend_with(|err, e| match err {
            Error::Model { msg, .. } => e.set("reason", msg.to_string()),
            Error::Reqwest { msg, .. } => e.set("reason", msg.to_string()),
        })
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../graphql/schema.gql",
    query_path = "../graphql/list_owners.gql"
)]
struct ListOwners;

#[allow(clippy::upper_case_acronyms)]
type UUID = Uuid;

// This function sends a request to a GraphQL API to obtain a list of Owners.
pub async fn list_owners(
    url: Url,
    request: ListOwnersRequest,
) -> Result<Vec<OwnerResponse>, Error> {
    let ListOwnersRequest { offset, limit } = request;
    let request = list_owners::ListOwnersRequest {
        offset: offset.into(),
        limit: limit.into(),
    };
    let variables = list_owners::Variables { request };
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Accept-Encoding",
        reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "Accept",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context(Reqwest {
            msg: "Cannot request list owners",
        })?;

    let response = post_graphql::<ListOwners, _>(&client, url, variables)
        .await
        .context(Reqwest { msg: "Foo" })?;
    let response_data: list_owners::ResponseData = response.data.expect("response data");
    let owners: Vec<OwnerResponse> = response_data
        .list_owners
        .owners
        .into_iter()
        .map(|o| OwnerResponse {
            id: o.id,
            name: o.name,
        })
        .collect();
    Ok(owners)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../graphql/schema.gql",
    query_path = "../graphql/add_owner.gql"
)]
struct AddOwner;

impl From<add_owner::AddOwnerAddOwner> for OwnerResponse {
    fn from(add_owner: add_owner::AddOwnerAddOwner) -> OwnerResponse {
        OwnerResponse {
            id: add_owner.id,
            name: add_owner.name,
        }
    }
}

// This function sends a request to a GraphQL API to add a new owner.
pub async fn add_owner(url: Url, request: AddOwnerRequest) -> Result<OwnerResponse, Error> {
    let AddOwnerRequest { name } = request;
    let request = add_owner::AddOwnerRequest { name };
    let variables = add_owner::Variables { request };
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Accept-Encoding",
        reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "Accept",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context(Reqwest {
            msg: "Cannot request list owners",
        })?;

    let response = post_graphql::<AddOwner, _>(&client, url, variables)
        .await
        .context(Reqwest { msg: "Foo" })?;
    let response_data: add_owner::ResponseData = response.data.expect("response data");
    let owner = OwnerResponse::from(response_data.add_owner);
    Ok(owner)
}
