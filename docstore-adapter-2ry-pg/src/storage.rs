use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use super::Error as PostgresError;
use super::PostgresqlStorage;
use docstore_domain::model::document::{AddDocumentRequest, Document, Genre, ListDocumentsRequest};
use docstore_domain::model::error::Error;
use docstore_domain::ports::secondary::storage::DocumentStorage;

#[derive(sqlx::Type)]
#[sqlx(type_name = "genre")] // only for PostgreSQL to match a type definition
#[sqlx(rename_all = "lowercase")]
enum GenreEntity {
    Tutorial,
    Howto,
    Background,
    Reference,
    Tbd,
}

impl From<GenreEntity> for Genre {
    fn from(entity: GenreEntity) -> Genre {
        match entity {
            GenreEntity::Tutorial => Genre::Tutorial,
            GenreEntity::Howto => Genre::Howto,
            GenreEntity::Background => Genre::Background,
            GenreEntity::Reference => Genre::Reference,
            GenreEntity::Tbd => Genre::Tbd,
        }
    }
}

impl From<&Genre> for GenreEntity {
    fn from(entity: &Genre) -> GenreEntity {
        match entity {
            Genre::Tutorial => GenreEntity::Tutorial,
            Genre::Howto => GenreEntity::Howto,
            Genre::Background => GenreEntity::Background,
            Genre::Reference => GenreEntity::Reference,
            Genre::Tbd => GenreEntity::Tbd,
        }
    }
}

// impl std::str::FromStr for  GenreEntity {
//     type Err = ();
// 
//     fn from_str(input: &str) -> Result<GenreEntity, Self::Err> {
//         match input {
//             "tutorial" => Ok(GenreEntity::Tutorial), 
//             "howto" => Ok(GenreEntity::Howto), 
//             "background" => Ok(GenreEntity::Background), 
//             "reference" => Ok(GenreEntity::Reference), 
//             _ => Ok(GenreEntity::Tbd), 
//         }
//     }
// }

// We need a struct to receive the data from sqlx
// This struct needs to interface with sqlx (so it must implement FromRow)
// and it must be turned into what the function signature of the port expect (a Document).
// But Document cannot implement FromRow because Document is in the domain, and cannot
// depend on an adapter.
struct DocumentEntity {
    pub id: Uuid,
    pub title: String,
    pub outline: String,
    pub content: String,
    pub tags: Vec<String>,
    pub genre: GenreEntity,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'c> FromRow<'c, PgRow> for DocumentEntity {
    fn from_row(row: &'c PgRow) -> Result<Self, sqlx::Error> {
        Ok(DocumentEntity {
            id: row.try_get(0)?,
            title: row.try_get(1)?,
            outline: row.try_get(2)?,
            content: row.try_get(3)?,
            tags: row.try_get(4)?,
            genre: row.try_get(5)?,
            created_at: row.try_get(6)?,
            updated_at: row.try_get(7)?,
        })
    }
}

impl From<DocumentEntity> for Document {
    fn from(entity: DocumentEntity) -> Self {
        let DocumentEntity {
            id,
            title,
            outline,
            content,
            tags,
            genre,
            created_at,
            updated_at,
        } = entity;
        Document {
            id,
            title,
            outline,
            content,
            tags,
            genre: Genre::from(genre),
            created_at,
            updated_at,
        }
    }
}

#[async_trait]
impl DocumentStorage for PostgresqlStorage {
    async fn list_documents(&self, request: &ListDocumentsRequest) -> Result<Vec<Document>, Error> {
        let entities: Vec<DocumentEntity> =
            sqlx::query_as(r#"SELECT * FROM api.list_documents($1::INTEGER, $2::INTEGER)"#)
                .bind(&request.limit)
                .bind(&request.offset)
                .fetch_all(&*self.pool)
                .await
                .map_err(PostgresError::from)?;

        let documents = entities.into_iter().map(Document::from).collect::<Vec<_>>();

        Ok(documents)
    }

    async fn add_document(&self, request: &AddDocumentRequest) -> Result<Document, Error> {
        let entity: DocumentEntity =
            sqlx::query_as(r#"SELECT * FROM api.add_document($1::TEXT, $2::TEXT, $3::TEXT, $4::TEXT, $5::TEXT[], $6::main.GENRE)"#)
                .bind(&request.id)
                .bind(&request.title)
                .bind(&request.outline)
                .bind(&request.content)
                .bind(&request.tags)
                .bind(GenreEntity::from(&request.genre))
                .fetch_one(&*self.pool)
                .await
                .map_err(PostgresError::from)?;
        Ok(Document::from(entity))
    }
}
