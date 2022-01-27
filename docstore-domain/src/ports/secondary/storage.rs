use async_trait::async_trait;

use crate::model::error::Error;
use crate::model::document::{AddDocumentRequest, ListDocumentsRequest, Document};

#[mockall::automock]
#[async_trait]
pub trait DocumentStorage {
    async fn list_documents(&self, request: &ListDocumentsRequest) -> Result<Vec<Document>, Error>;
    async fn add_document(&self, document: &AddDocumentRequest) -> Result<Document, Error>;
}
