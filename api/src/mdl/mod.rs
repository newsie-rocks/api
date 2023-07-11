//! Models

use postgres_types::{FromSql, ToSql};
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
    /// Subscription
    pub subscription: Subscription,
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

/// Subscription
#[derive(
    Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default, FromSql, ToSql,
)]
#[postgres(name = "subscription")]
pub enum Subscription {
    /// Free tier
    #[default]
    #[postgres(name = "FREE")]
    Free,
    /// Mid tier
    #[postgres(name = "MID")]
    Mid,
}

impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Subscription::Free => "free subscription".to_string(),
            Subscription::Mid => "mid tier subscription".to_string(),
        };
        write!(f, "{}", value)
    }
}

/// Subscription update
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubscriptionUpdate {
    /// Free tier
    pub subscription: Subscription,
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

/// An article summary
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Summary {
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
