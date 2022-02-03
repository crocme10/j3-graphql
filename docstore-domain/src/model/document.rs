use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum Genre {
    Tutorial,
    Howto,
    Background,
    Reference,
    Tbd,
}

pub fn default_genre() -> Genre {
    Genre::Tbd
}

impl Genre {
    pub fn as_str(&self) -> &'static str {
        match self {
            Genre::Tutorial => "tutorial",
            Genre::Howto => "howto",
            Genre::Background => "background",
            Genre::Reference => "reference",
            Genre::Tbd => "to be decided",
        }
    }
}

impl std::str::FromStr for Genre {
    type Err = ();

    fn from_str(input: &str) -> Result<Genre, Self::Err> {
        match input {
            "tutorial" => Ok(Genre::Tutorial),
            "howto" => Ok(Genre::Howto),
            "background" => Ok(Genre::Background),
            "reference" => Ok(Genre::Reference),
            _ => Ok(Genre::Tbd),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    #[serde(rename = "abstract")]
    pub outline: String,
    pub content: String,
    pub html: String,
    pub tags: Vec<String>,
    #[serde(default = "default_genre")]
    pub genre: Genre,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListDocumentsRequest {
    pub offset: u32,
    pub limit: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddDocumentRequest {
    pub id: Uuid,
    pub title: String,
    #[serde(rename = "abstract")]
    pub outline: String,
    pub content: String,
    pub html: String,
    pub tags: Vec<String>,
    #[serde(default = "default_genre")]
    pub genre: Genre,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDocumentRequest {
    pub id: Uuid,
}
