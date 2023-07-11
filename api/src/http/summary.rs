//! Articles endpoints

use salvo::{oapi::extract::JsonBody, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{error::Error, http::ApiServices, mdl::Summary};

/// Get articles response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SummariesRespBody {
    /// Summaries
    pub summaries: Vec<Summary>,
}

/// Creates (or retrieve) a summary for a list of articles
///
/// The body contains a list of articles
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn post_summaries(
    depot: &mut Depot,
    body: JsonBody<Vec<String>>,
) -> Result<Json<SummariesRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();

    let urls = body.into_inner();
    let urls = urls.iter().map(|url| url.as_str()).collect::<Vec<_>>();
    let summaries = services.art.process_summaries(&urls).await?;
    Ok(Json(SummariesRespBody { summaries }))
}
