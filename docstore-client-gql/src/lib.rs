use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use docstore_adapter_1ry_gql::api::{AddDocumentRequest, ListDocumentsRequest, DocumentResponse};
use snafu::{ResultExt, Snafu};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Request Error: {} - {}", msg, source))]
    Reqwest { msg: String, source: reqwest::Error },
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.gql",
    query_path = "graphql/list_documents.gql"
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
    schema_path = "graphql/schema.gql",
    query_path = "graphql/add_document.gql"
)]
struct AddDocument;

impl From<add_document::AddDocumentAdddocument> for DocumentResponse {
    fn from(add_document: add_document::AddDocumentAddDocument) -> DocumentResponse {
        DocumentResponse {
            id: add_document.id,
            name: add_document.name,
        }
    }
}

// This function sends a request to a GraphQL API to add a new document.
pub async fn add_document(url: Url, request: AddDocumentRequest) -> Result<documentResponse, Error> {
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
