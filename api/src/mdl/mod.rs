//! Models

use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    /// ID
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Password
    pub password: String,
}

/// New user
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewUser {
    /// Name
    pub name: String,
    /// Email
    pub email: String,
    /// Password
    pub password: String,
}

/// User update fields
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserUpdateFields {
    /// Name
    pub name: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Password
    pub password: Option<String>,
}

/// User feed
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Feed {
    /// ID
    pub id: Uuid,
    /// User id
    pub user_id: Uuid,
    /// Feed url
    pub url: String,
    /// Feed name
    pub name: Option<String>,
}

/// New uer feed
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewFeed {
    /// Feed url
    pub url: String,
    /// Feed name
    pub name: Option<String>,
}

/// Feed fields (for update)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FeedUpdateFields {
    /// Url
    pub url: Option<String>,
    /// Name
    pub name: Option<Option<String>>,
}
