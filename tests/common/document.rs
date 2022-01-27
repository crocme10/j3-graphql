use cucumber::{given, then, when};
use snafu::ResultExt;
use url::Url;

use super::error::{
    EnvironmentVariable as EnvError, GraphqlAPI as GraphqlAPIError, Url as UrlError,
};
use super::state::World;
use docstore_adapter_1ry_gql::api::{AddDocumentRequest, ListDocumentsRequest};
use docstore_client_gql::{add_document, list_documents};

#[given(regex = r"there are no documents")]
async fn delete_documents(_state: &mut World) {}

#[when(regex = r"^I add a new document with the title ([a-zA-Z0-9_-]{3,})$")]
async fn when_add_document(_state: &mut World, name: String) {
    let url = std::env::var("TEST_GRAPHQL_URL")
        .context(EnvError {
            details: "Missing TEST_GRAPHQL_URL env definition".to_string(),
        })
        .expect("Missing Env");

    let url = Url::parse(&url)
        .context(UrlError { details: url })
        .expect("url");

    let request = AddDocumentRequest {
        name: name.to_string(),
    };

    let _ = add_document(url, request)
        .await
        .context(GraphqlAPIError)
        .expect("request api");
}

#[then(
    regex = r"^I find the document with the title ([a-zA-Z0-9_-]{3,}) in the list of documents$"
)]
async fn then_find_document(_state: &mut World, name: String) {
    let url = std::env::var("TEST_GRAPHQL_URL")
        .context(EnvError {
            details: "Missing TEST_GRAPHQL_URL env definition".to_string(),
        })
        .expect("Missing Env");

    let url = Url::parse(&url)
        .context(UrlError { details: url })
        .expect("url");

    let request = ListDocumentsRequest {
        offset: 0,
        limit: 10,
    };

    let documents = list_documents(url, request)
        .await
        .context(GraphqlAPIError)
        .expect("request api");

    let res = documents.iter().find(|&document| document.name == name);

    assert!(res.is_some());
}
