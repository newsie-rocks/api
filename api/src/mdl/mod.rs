//! Models

use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::postgres::util::Vector;

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
pub struct UserUpdate {
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

/// Feed update
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FeedUpdate {
    /// ID
    ///
    /// If set, feed already exists
    pub id: Option<Uuid>,
    /// Url
    pub url: String,
    /// Name
    pub name: Option<String>,
}

/// An article
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Article {
    /// ID
    pub id: Uuid,
    /// Url
    pub url: String,
    /// Summary
    pub summary: String,
    /// Keywords
    pub keywords: Vec<String>,
    /// Embeddings (1536 values)
    pub embeddings: Vector,
}
