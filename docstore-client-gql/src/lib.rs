use chrono::{DateTime, Utc};
pub use docstore_adapter_1ry_gql::api::{
    AddDocumentRequest, DocumentResponse, GetDocumentRequest, ListDocumentsRequest,
};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
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

#[allow(clippy::upper_case_acronyms)]
type TIMESTAMPZ = DateTime<Utc>;

// This function sends a request to a GraphQL API to obtain a list of Documents.
pub async fn list_documents(
    url: &Url,
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
        .context(ReqwestSnafu {
            msg: "Cannot request list documents",
        })?;

    let response = post_graphql::<ListDocuments, _>(&client, url.to_owned(), variables)
        .await
        .context(ReqwestSnafu { msg: "Foo" })?;
    let response_data: list_documents::ResponseData = response.data.expect("response data");
    let documents: Vec<DocumentResponse> = response_data
        .list_documents
        .documents
        .into_iter()
        .map(|o| DocumentResponse {
            id: o.id,
            title: o.title,
            outline: o.outline,
            content: o.content,
            html: o.html,
            tags: o.tags,
            genre: o.genre,
            created_at: o.created_at,
            updated_at: o.updated_at,
        })
        .collect();
    Ok(documents)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.gql",
    query_path = "graphql/get_document.gql"
)]
struct GetDocument;

// This function sends a request to a GraphQL API to obtain a Document based on its id.
pub async fn get_document(
    url: &Url,
    request: GetDocumentRequest,
) -> Result<DocumentResponse, Error> {
    let GetDocumentRequest { id } = request;
    let request = get_document::GetDocumentRequest { id: id.into() };
    let variables = get_document::Variables { request };
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
        .context(ReqwestSnafu {
            msg: "Cannot request get document",
        })?;

    let response = post_graphql::<GetDocument, _>(&client, url.to_owned(), variables)
        .await
        .context(ReqwestSnafu { msg: "Foo" })?;
    let response_data: get_document::ResponseData = response.data.expect("response data");
    let doc = response_data.get_document.document;
    Ok(DocumentResponse {
        id: doc.id,
        title: doc.title,
        outline: doc.outline,
        content: doc.content,
        html: doc.html,
        tags: doc.tags,
        genre: doc.genre,
        created_at: doc.created_at,
        updated_at: doc.updated_at,
    })
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.gql",
    query_path = "graphql/add_document.gql"
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
            genre: add_document.genre,
            created_at: add_document.created_at,
            updated_at: add_document.updated_at,
        }
    }
}

// This function sends a request to a GraphQL API to add a new document.
pub async fn add_document(
    url: &Url,
    request: AddDocumentRequest,
) -> Result<DocumentResponse, Error> {
    let AddDocumentRequest {
        id,
        title,
        outline,
        content,
        html,
        tags,
        genre,
    } = request;

    tracing::info!("adding document {}", id);

    let request = add_document::AddDocumentRequest {
        id,
        title,
        outline,
        content,
        html,
        tags,
        genre,
    };
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
        .context(ReqwestSnafu {
            msg: "Cannot request list documents",
        })?;

    let response = post_graphql::<AddDocument, _>(&client, url.to_owned(), variables)
        .await
        .context(ReqwestSnafu { msg: "Foo" })?;
    let response_data: add_document::ResponseData = response.data.expect("response data");
    let document = DocumentResponse::from(response_data.add_document);
    Ok(document)
}
