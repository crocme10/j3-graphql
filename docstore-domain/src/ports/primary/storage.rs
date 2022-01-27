use async_trait::async_trait;

use crate::model::document::{AddDocumentRequest, Document, ListDocumentsRequest};
use crate::model::error::Error;

#[async_trait]
pub trait DocumentStorage {
    async fn list_documents(&self, request: &ListDocumentsRequest) -> Result<Vec<Document>, Error>;
    async fn add_document(&self, request: &AddDocumentRequest) -> Result<Document, Error>;
}

#[async_trait]
impl<T> DocumentStorage for T
where
    T: crate::ports::secondary::storage::DocumentStorage + Send + Sync,
{
    async fn list_documents(&self, request: &ListDocumentsRequest) -> Result<Vec<Document>, Error> {
        self.list_documents(request).await
    }
    async fn add_document(&self, request: &AddDocumentRequest) -> Result<Document, Error> {
        self.add_document(request).await
    }
}
