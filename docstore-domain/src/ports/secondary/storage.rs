use async_trait::async_trait;

use crate::model::document::{
    AddDocumentRequest, Document, GetDocumentRequest, ListDocumentsRequest,
};
use crate::model::error::Error;

#[mockall::automock]
#[async_trait]
pub trait DocumentStorage {
    async fn list_documents(&self, request: &ListDocumentsRequest) -> Result<Vec<Document>, Error>;
    async fn add_document(&self, document: &AddDocumentRequest) -> Result<Document, Error>;
    async fn get_document(&self, document: &GetDocumentRequest) -> Result<Document, Error>;
}
