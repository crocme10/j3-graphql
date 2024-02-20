use async_trait::async_trait;

// use crate::model::document::{
//     AddDocumentRequest, Document, GetDocumentRequest, ListDocumentsRequest,
// };
use crate::model::error::Error;

#[async_trait]
pub trait Session {
    async fn started_at(&self) -> DateTime<Utc>;
    async fn add_document(&self, request: &AddDocumentRequest) -> Result<Document, Error>;
    async fn get_document(&self, request: &GetDocumentRequest) -> Result<Document, Error>;
}

pub struct SessionImpl<A: SessionStorage, B: FlashcardStorage> {
    pub session_storage: A,
    pub flashcard_storage: B,
}

#[async_trait]
impl <A, B> Session for SessionImpl<A, B> 
where A: SessionStorage + Send + Sync,
      B: FlashcardStorage + Send + Sync {

          async fn started_at(self: &Self) -> DateTime<Utc> {
          }
      }


// #[async_trait]
// impl<T> DocumentStorage for T
// where
//     T: crate::ports::secondary::storage::DocumentStorage + Send + Sync,
// {
//     async fn list_documents(&self, request: &ListDocumentsRequest) -> Result<Vec<Document>, Error> {
//         self.list_documents(request).await
//     }
//     async fn add_document(&self, request: &AddDocumentRequest) -> Result<Document, Error> {
//         self.add_document(request).await
//     }
//     async fn get_document(&self, request: &GetDocumentRequest) -> Result<Document, Error> {
//         self.get_document(request).await
//     }
// }
