//! Articles endpoints

use salvo::{oapi::extract::JsonBody, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{error::Error, http::ApiServices, mdl::Article};

/// Get articles response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArticlesRespBody {
    /// Articles
    pub articles: Vec<Article>,
}

/// Creates (or retrieve) a summary for a list of articles
///
/// The body contains a list of articles
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn post_articles(
    depot: &mut Depot,
    body: JsonBody<Vec<String>>,
) -> Result<Json<ArticlesRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();

    let urls = body.into_inner();
    let urls = urls.iter().map(|url| url.as_str()).collect::<Vec<_>>();
    let articles = services.art.process_articles(&urls).await?;
    Ok(Json(ArticlesRespBody { articles }))
}
