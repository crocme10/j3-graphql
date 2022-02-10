#[cfg(feature = "graphql")]
use async_graphql::{
    extensions::Tracing, Context, EmptySubscription, ErrorExtensions, InputObject, Object, Schema,
};
use chrono::{DateTime, Utc};
use docstore_domain::model;
use docstore_domain::model::document::{Document, Genre};
use docstore_domain::model::error::Error as ModelError;
#[cfg(feature = "graphql")]
use docstore_domain::ports::primary::storage::DocumentStorage;
use serde::{Deserialize, Serialize};
#[cfg(feature = "graphql")]
use snafu::ResultExt;
use snafu::Snafu;
use std::str::FromStr;
#[cfg(feature = "graphql")]
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Model Error: {} - {}", msg, source))]
    Model { msg: String, source: ModelError },

    #[snafu(display("Request Error: {} - {}", msg, source))]
    Reqwest { msg: String, source: reqwest::Error },
}

#[cfg(feature = "graphql")]
impl ErrorExtensions for Error {
    fn extend(&self) -> async_graphql::Error {
        self.extend_with(|err, e| match err {
            Error::Model { msg, .. } => e.set("reason", msg.to_string()),
            Error::Reqwest { msg, .. } => e.set("reason", msg.to_string()),
        })
    }
}

// A GraphQL Input Object to encapsulate the request parameters to list documents.
// We cannot directly use the model's ListDocumentsRequest for that because we must
// derive InputObject, and we cannot do that on the model's type without creating
// a dependency between the model and the outer adapters, which would break the
// hexagonal architecture.
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "graphql", derive(InputObject))]
pub struct ListDocumentsRequest {
    pub offset: u32,
    pub limit: u32,
}

impl From<ListDocumentsRequest> for model::document::ListDocumentsRequest {
    fn from(request: ListDocumentsRequest) -> Self {
        let ListDocumentsRequest { offset, limit } = request;
        model::document::ListDocumentsRequest { offset, limit }
    }
}

// A GraphQL Input Object to encapsulate the request parameters to get a document.
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "graphql", derive(InputObject))]
pub struct GetDocumentRequest {
    pub id: Uuid,
}

impl From<GetDocumentRequest> for model::document::GetDocumentRequest {
    fn from(request: GetDocumentRequest) -> Self {
        let GetDocumentRequest { id } = request;
        model::document::GetDocumentRequest { id }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub outline: String,
    pub content: String,
    pub html: String,
    pub tags: Vec<String>,
    pub genre: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "graphql")]
#[Object]
impl DocumentResponse {
    async fn id(&self) -> &Uuid {
        &self.id
    }

    async fn title(&self) -> &String {
        &self.title
    }

    async fn outline(&self) -> &String {
        &self.outline
    }

    async fn content(&self) -> &String {
        &self.content
    }

    async fn html(&self) -> &String {
        &self.html
    }

    async fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    async fn genre(&self) -> &String {
        &self.genre
    }

    async fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    async fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

impl From<Document> for DocumentResponse {
    fn from(document: Document) -> Self {
        let Document {
            id,
            title,
            outline,
            content,
            html,
            tags,
            genre,
            created_at,
            updated_at,
        } = document;

        DocumentResponse {
            id,
            title,
            outline,
            content,
            html,
            tags,
            genre: genre.as_str().to_string(),
            created_at,
            updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListDocumentsResponse {
    pub documents: Vec<DocumentResponse>,
    pub count: usize,
}

#[cfg(feature = "graphql")]
#[Object]
impl ListDocumentsResponse {
    async fn documents(&self) -> &Vec<DocumentResponse> {
        &self.documents
    }

    async fn count(&self) -> &usize {
        &self.count
    }
}

impl From<Vec<Document>> for ListDocumentsResponse {
    fn from(documents: Vec<Document>) -> Self {
        let documents = documents
            .into_iter()
            .map(DocumentResponse::from)
            .collect::<Vec<_>>();
        let count = documents.len();
        ListDocumentsResponse { documents, count }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDocumentResponse {
    pub document: DocumentResponse,
}

#[cfg(feature = "graphql")]
#[Object]
impl GetDocumentResponse {
    async fn document(&self) -> &DocumentResponse {
        &self.document
    }
}

impl From<Document> for GetDocumentResponse {
    fn from(document: Document) -> Self {
        GetDocumentResponse {
            document: DocumentResponse::from(document),
        }
    }
}

pub struct Query;

#[cfg(feature = "graphql")]
#[Object]
impl Query {
    async fn list_documents(
        &self,
        context: &Context<'_>,
        request: ListDocumentsRequest,
    ) -> async_graphql::Result<ListDocumentsResponse> {
        let service = get_service_from_context(context)?;
        let documents = service
            .list_documents(&model::document::ListDocumentsRequest::from(request))
            .await
            .context(Model {
                msg: "Error Listing Documents",
            })
            .map_err(|e| e.extend())?;
        Ok(ListDocumentsResponse::from(documents))
    }

    async fn get_document(
        &self,
        context: &Context<'_>,
        request: GetDocumentRequest,
    ) -> async_graphql::Result<GetDocumentResponse> {
        let service = get_service_from_context(context)?;
        let document = service
            .get_document(&model::document::GetDocumentRequest::from(request))
            .await
            .context(Model {
                msg: "Error Getting Document",
            })
            .map_err(|e| e.extend())?;
        Ok(GetDocumentResponse::from(document))
    }
}

pub struct Mutation;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "graphql", derive(InputObject))]
pub struct AddDocumentRequest {
    pub id: Uuid,
    pub title: String,
    pub outline: String,
    pub content: String,
    pub html: String,
    pub tags: Vec<String>,
    pub genre: String,
}

impl From<AddDocumentRequest> for model::document::AddDocumentRequest {
    fn from(request: AddDocumentRequest) -> Self {
        let AddDocumentRequest {
            id,
            title,
            outline,
            content,
            html,
            tags,
            genre,
        } = request;
        model::document::AddDocumentRequest {
            id,
            title,
            outline,
            content,
            html,
            tags,
            genre: Genre::from_str(&genre).unwrap(),
        }
    }
}

#[cfg(feature = "graphql")]
#[Object]
impl Mutation {
    #[instrument(skip(self, context))]
    async fn add_document(
        &self,
        context: &Context<'_>,
        request: AddDocumentRequest,
    ) -> async_graphql::Result<DocumentResponse> {
        let service = get_service_from_context(context)?;
        let document = service
            .add_document(&model::document::AddDocumentRequest::from(request))
            .await
            .context(Model {
                msg: "Error Adding Document",
            })
            .map_err(|e| e.extend())?;

        Ok(DocumentResponse::from(document))
    }
}

#[cfg(feature = "graphql")]
pub type DocStoreSchema = Schema<Query, Mutation, EmptySubscription>;

#[cfg(feature = "graphql")]
pub fn schema(service: Box<dyn DocumentStorage + Send + Sync>) -> DocStoreSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .extension(Tracing)
        .data(service)
        .finish()
}

#[cfg(feature = "graphql")]
#[allow(clippy::borrowed_box)]
pub fn get_service_from_context<'ctx>(
    context: &'ctx Context,
) -> Result<&'ctx Box<dyn DocumentStorage + Send + Sync>, async_graphql::Error>
where
{
    context.data::<Box<dyn DocumentStorage + Send + Sync>>()
}
