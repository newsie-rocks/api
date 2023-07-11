//! Users

use tokio_postgres::Row;
use uuid::Uuid;

use crate::{
    error::Error,
    mdl::{NewUser, Subscription, SubscriptionUpdate, User, UserUpdate},
};

use super::PostgresClient;

impl From<Row> for User {
    fn from(value: Row) -> Self {
        User {
            id: value.get::<_, Uuid>("id"),
            name: value.get::<_, String>("name"),
            email: value.get::<_, String>("email"),
            password: value.get::<_, String>("password"),
            subscription: value.get::<_, Subscription>("subscription"),
        }
    }
}

impl PostgresClient {
    /// Creates the `users` table
    pub async fn create_table_users(&self) -> Result<(), Error> {
        let client = self.client().await?;

        Ok(client
            .batch_execute(
                "
                    CREATE TABLE IF NOT EXISTS users (
                        id              UUID PRIMARY KEY,
                        name            TEXT NOT NULL,
                        email           TEXT NOT NULL,
                        password        TEXT NOT NULL,
                        subscription    subscription NOT NULL 
                    )",
            )
            .await?)
    }

    /// Creates a new user
    ///
    /// A new user is created and its ID is populated
    pub async fn create_user(&self, new_user: NewUser) -> Result<User, Error> {
        let client = self.client().await?;

        Ok(client
            .query_one(
                "INSERT into users (id, name, email, password, subscription) VALUES ($1, $2, $3, $4, $5) RETURNING *",
                &[
                    &Uuid::new_v4(),
                    &new_user.name,
                    &new_user.email,
                    &new_user.password,
                    &Subscription::Free
                ],
            )
            .await?
            .into())
    }

    /// Reads a user with its id
    pub async fn read_user(&self, id: Uuid) -> Result<Option<User>, Error> {
        let client = self.client().await?;

        Ok(client
            .query_opt("SELECT * FROM users WHERE id = $1", &[&id])
            .await?
            .map(|row| row.into()))
    }

    /// Reads a user with its email
    pub async fn read_user_with_email(&self, email: &str) -> Result<Option<User>, Error> {
        let client = self.client().await?;

        Ok(client
            .query_opt("SELECT * FROM users WHERE email = $1", &[&email])
            .await?
            .map(|row| row.into()))
    }

    /// Update a user
    pub async fn update_user(&self, id: Uuid, fields: UserUpdate) -> Result<User, Error> {
        let client = self.client().await?;

        let mut cols: Vec<&str> = vec![];
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&id);

        if let Some(name) = fields.name.as_ref() {
            cols.push("name");
            params.push(name);
        }
        if let Some(email) = fields.email.as_ref() {
            cols.push("email");
            params.push(email);
        }
        if let Some(password) = fields.password.as_ref() {
            cols.push("password");
            params.push(password);
        }
        // ... add other fields here

        if cols.is_empty() {
            match self.read_user(id).await? {
                Some(u) => Ok(u),
                None => Err(Error::NotFound(format!("no user for id {id}"), None)),
            }
        } else {
            let stmt = format!(
                "UPDATE users SET {} WHERE id=$1 RETURNING *",
                cols.iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}=${}", c, i + 2))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            Ok(client.query_one(&stmt, &params).await?.into())
        }
    }

    /// Update the user subscription
    pub async fn update_user_subscription(
        &self,
        id: Uuid,
        subscription_update: SubscriptionUpdate,
    ) -> Result<User, Error> {
        let client = self.client().await?;

        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        params.push(&id);
        params.push(&subscription_update.subscription);

        Ok(client
            .query_one(
                "UPDATE users SET id=$1, subscription=$2 WHERE id=$1 RETURNING *",
                &params,
            )
            .await?
            .into())
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> Result<(), Error> {
        let client = self.client().await?;

        let _res = client
            .execute("DELETE FROM users WHERE id=$1", &[&id])
            .await?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::AppConfig;

    use super::*;

    use fake::{
        faker::{internet::en::FreeEmail, name::en::Name},
        Fake,
    };

    /// Initializes the user store
    fn init_db() -> PostgresClient {
        let cfg = AppConfig::load();
        PostgresClient::new(cfg.postgres.new_pool())
    }

    /// Setup a test
    pub async fn setup_test_user() -> (PostgresClient, User) {
        let db = init_db();
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let user = db
            .create_user(NewUser {
                name,
                email,
                password: "dummy".to_string(),
            })
            .await
            .unwrap();
        (db, user)
    }

    /// Teardown a test
    pub async fn teardown_test_user(db: PostgresClient, user: User) {
        db.delete_user(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_table() {
        let db = init_db();
        db.create_table_users().await.unwrap();
    }

    #[tokio::test]
    async fn test_read_with_id() {
        let (db, test_user) = setup_test_user().await;
        let user = db.read_user(test_user.id).await.unwrap();
        assert_eq!(user.unwrap().id, test_user.id);
        teardown_test_user(db, test_user).await;
    }

    #[tokio::test]
    async fn test_read_with_email() {
        let (db, test_user) = setup_test_user().await;
        let user = db.read_user_with_email(&test_user.email).await.unwrap();
        assert_eq!(user.unwrap().email, test_user.email);
        teardown_test_user(db, test_user).await;
    }

    #[tokio::test]
    async fn test_update() {
        let (db, test_user) = setup_test_user().await;
        let new_name = "test_user_update_new_name".to_string();
        let user = db
            .update_user(
                test_user.id,
                UserUpdate {
                    name: Some(new_name.clone()),
                    email: None,
                    password: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(user.name, new_name);
        teardown_test_user(db, test_user).await;
    }

    #[tokio::test]
    async fn test_update_subscription() {
        let (db, test_user) = setup_test_user().await;
        let user = db
            .update_user_subscription(
                test_user.id,
                SubscriptionUpdate {
                    subscription: Subscription::Mid,
                },
            )
            .await
            .unwrap();
        assert_eq!(user.subscription, Subscription::Mid);
        teardown_test_user(db, test_user).await;
    }
}
