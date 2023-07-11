//! Feeds endpoints

use salvo::{oapi::extract::JsonBody, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    error::Error,
    http::ApiServices,
    mdl::{Feed, FeedUpdate, User},
};

/// Get feeds response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetFeedsRespBody {
    /// Feeds
    pub feeds: Vec<Feed>,
}

/// Get all the user feeds
#[endpoint(security(["bearerAuth" = []]))]
#[tracing::instrument(skip_all)]
pub async fn get_feeds(depot: &mut Depot) -> Result<Json<GetFeedsRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    let feeds = services.feeds.get_feeds(user.id).await?;
    Ok(Json(GetFeedsRespBody { feeds }))
}

/// Sync all the user feeds
#[endpoint(security(["bearerAuth" = []]))]
#[tracing::instrument(skip_all)]
pub async fn put_feeds(
    depot: &mut Depot,
    body: JsonBody<Vec<FeedUpdate>>,
) -> Result<Json<GetFeedsRespBody>, Error> {
    trace!("received request");
    let services = depot.obtain::<ApiServices>().unwrap();
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    let feeds = services
        .feeds
        .sync_feeds(user.id, body.into_inner())
        .await?;
    Ok(Json(GetFeedsRespBody { feeds }))
}
