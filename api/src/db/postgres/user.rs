//! Users

use tokio_postgres::Row;
use uuid::Uuid;

use crate::{
    error::Error,
    mdl::{NewUser, User, UserUpdateFields},
};

use super::PostgresDb;

impl<'a> From<&'a Row> for User {
    fn from(value: &'a Row) -> Self {
        User {
            id: value.get::<_, Uuid>("id"),
            name: value.get::<_, String>("name"),
            email: value.get::<_, String>("email"),
            password: value.get::<_, String>("password"),
        }
    }
}

impl PostgresDb {
    /// Creates the `users` table
    pub async fn create_table_users(&self) -> Result<(), Error> {
        let client = self.client().await?;

        let stmt = "
        CREATE TABLE IF NOT EXISTS users (
            id          UUID PRIMARY KEY,
            name        TEXT NOT NULL,
            email       TEXT NOT NULL,
            password    TEXT NOT NULL
        )";
        Ok(client.batch_execute(stmt).await?)
    }

    /// Creates a new user
    ///
    /// A new user is created and its ID is populated
    pub async fn create_user(&self, new_user: NewUser) -> Result<User, Error> {
        let client = self.client().await?;

        let id = Uuid::new_v4();

        let stmt =
            "INSERT into users (id, name, email, password) VALUES($1, $2, $3, $4) RETURNING id";
        let rows = client
            .query(
                stmt,
                &[&id, &new_user.name, &new_user.email, &new_user.password],
            )
            .await?;
        let id = match rows.first() {
            Some(row) => row.get::<_, Uuid>("id"),
            None => return Err(Error::Internal("record not created".to_string(), None)),
        };

        self.read_user(id).await.map(|u| u.unwrap())
    }

    /// Reads a user with its id
    pub async fn read_user(&self, id: Uuid) -> Result<Option<User>, Error> {
        let client = self.client().await?;

        let stmt = "SELECT * FROM users WHERE id = $1";
        let rows = client.query(stmt, &[&id]).await?;
        Ok(rows.first().map(|row| row.into()))
    }

    /// Reads a user with its email
    pub async fn read_user_with_email(&self, email: &str) -> Result<Option<User>, Error> {
        let client = self.client().await?;

        let stmt = "SELECT * FROM users WHERE email = $1";
        let rows = client.query(stmt, &[&email]).await?;
        Ok(rows.first().map(|row| row.into()))
    }

    /// Update a user
    pub async fn update_user(&self, id: Uuid, fields: UserUpdateFields) -> Result<User, Error> {
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
                "UPDATE users SET {} WHERE id=$1",
                cols.iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}=${}", c, i + 2))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let _res = client.execute(&stmt, &params).await?;

            match self.read_user(id).await? {
                Some(u) => Ok(u),
                None => Err(Error::NotFound(format!("no user for id {id}"), None)),
            }
        }
    }

    /// Delete a user
    pub async fn delete_user(&self, id: Uuid) -> Result<(), Error> {
        let client = self.client().await?;

        let stmt = "DELETE FROM users WHERE id=$1";
        let _res = client.execute(stmt, &[&id]).await?;

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
    fn init_db() -> PostgresDb {
        let cfg = AppConfig::load();
        PostgresDb::new(cfg.postgres.new_pool())
    }

    /// Setup a test
    pub async fn setup_test_user() -> (PostgresDb, User) {
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
    pub async fn teardown_test_user(db: PostgresDb, user: User) {
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
                UserUpdateFields {
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
}
