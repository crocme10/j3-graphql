use async_graphql::extensions::Tracing;
use async_graphql::{Context, EmptySubscription, ErrorExtensions, InputObject, Object, Schema};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use tracing::instrument;
use url::Url;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use petstore_domain::model;
use petstore_domain::model::error::Error as ModelError;
use petstore_domain::model::document::Document;
use petstore_domain::ports::primary::storage::DocumentStorage;

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
    query_path = "../graphql/list_documents.gql"
)]
struct ListDocuments;

#[allow(clippy::upper_case_acronyms)]
type UUID = Uuid;

// This function sends a request to a GraphQL API to obtain a list of Documents.
pub async fn list_documents(
    url: Url,
    request: ListDocumentsRequest,
) -> Result<Vec<DocumentResponse>, Error> {
    let ListDocumentsRequest { offset, limit } = request;
    let request = list_documents::ListDocumentsRequest {
        offset: offset.into(),
        limit: limit.into(),
    };
    let variables = list_documents::Variables { request };
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
            msg: "Cannot request list documents",
        })?;

    let response = post_graphql::<ListDocuments, _>(&client, url, variables)
        .await
        .context(Reqwest { msg: "Foo" })?;
    let response_data: list_documents::ResponseData = response.data.expect("response data");
    let documents: Vec<DocumentResponse> = response_data
        .list_documents
        .documents
        .into_iter()
        .map(|o| DocumentResponse {
            id: o.id,
            name: o.name,
        })
        .collect();
    Ok(documents)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../graphql/schema.gql",
    query_path = "../graphql/add_document.gql"
)]
struct AddDocument;

impl From<add_document::AddDocumentAddDocument> for DocumentResponse {
    fn from(add_document: add_document::AddDocumentAddDocument) -> DocumentResponse {
        DocumentResponse {
            id: add_document.id,
            title: add_document.title,
            outline: add_document.outline,
            content: add_document.content,
            html: add_document.html,
            tags: add_document.tags,
            created_at: add_document.created_at,
            updated_at: add_document.updated_at
        }
    }
}

// This function sends a request to a GraphQL API to add a new document.
pub async fn add_document(url: Url, request: AddDocumentRequest) -> Result<DocumentResponse, Error> {
    let AddDocumentRequest { name } = request;
    let request = add_document::AddDocumentRequest { name };
    let variables = add_document::Variables { request };
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
            msg: "Cannot request list documents",
        })?;

    let response = post_graphql::<AddDocument, _>(&client, url, variables)
        .await
        .context(Reqwest { msg: "Foo" })?;
    let response_data: add_document::ResponseData = response.data.expect("response data");
    let document = DocumentResponse::from(response_data.add_document);
    Ok(document)
}
