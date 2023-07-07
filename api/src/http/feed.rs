//! Feeds endpoints

use salvo::{oapi::extract::JsonBody, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::{
    error::Error,
    mdl::{Feed, FeedUpdate, User},
    svc::feed::FeedService,
};

/// Get feeds response body
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetFeedsRespBody {
    /// Feeds
    pub feeds: Vec<Feed>,
}

/// Get all the user feeds
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn get_feeds(depot: &mut Depot) -> Result<Json<GetFeedsRespBody>, Error> {
    trace!("received request");
    let feed_svc = depot.obtain::<FeedService>().unwrap();
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    let feeds = feed_svc.get_feeds(user.id).await?;
    Ok(Json(GetFeedsRespBody { feeds }))
}

/// Sync all the user feeds
#[endpoint]
#[tracing::instrument(skip_all)]
pub async fn put_feeds(
    depot: &mut Depot,
    body: JsonBody<Vec<FeedUpdate>>,
) -> Result<Json<GetFeedsRespBody>, Error> {
    trace!("received request");
    let feed_svc = depot.obtain::<FeedService>().unwrap();
    let user = depot.obtain::<User>().ok_or(Error::Unauthenticated(
        "not authenticated".to_string(),
        None,
    ))?;

    let feeds = feed_svc.sync_feeds(user.id, body.into_inner()).await?;
    Ok(Json(GetFeedsRespBody { feeds }))
}
