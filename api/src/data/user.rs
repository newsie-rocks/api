//! Users

use uuid::Uuid;

use crate::svc::auth::{NewUser, User, UserFields};

use super::error::StoreError;

/// User store
#[derive(Debug, Clone)]
pub struct UserStore {
    /// Postgres pool
    postgres_pool: deadpool_postgres::Pool,
}

impl UserStore {
    /// Creates a new instance
    pub fn new(postgres_pool: deadpool_postgres::Pool) -> Self {
        Self { postgres_pool }
    }
}

impl UserStore {
    /// Returns a postgres client instance
    async fn postgres_client(&self) -> Result<deadpool_postgres::Object, StoreError> {
        Ok(self.postgres_pool.get().await?)
    }

    /// Creates the `users` table
    pub async fn create_table(&self) -> Result<(), StoreError> {
        let client = self.postgres_client().await?;

        let stmt = "
    CREATE TABLE IF NOT EXISTS users (
        id      UUID PRIMARY KEY,
        name    TEXT NOT NULL,
        email    TEXT NOT NULL,
        password    TEXT NOT NULL
    )
    ";
        Ok(client.batch_execute(stmt).await?)
    }

    /// Creates a new user
    ///
    /// A new user is created and its ID is populated
    pub async fn create(&self, new_user: NewUser) -> Result<User, StoreError> {
        let client = self.postgres_client().await?;

        let id = Uuid::new_v4();

        let stmt =
            "INSERT into users (id, name, email, password) VALUES($1, $2, $3, $4) RETURNING id";
        let rows = client
            .query(
                stmt,
                &[&id, &new_user.name, &new_user.email, &new_user.password],
            )
            .await?;

        match rows.first() {
            Some(row) => {
                let id = row.get::<_, Uuid>("id");

                let user = User {
                    id,
                    name: new_user.name,
                    email: new_user.email,
                    password: new_user.password,
                };
                Ok(user)
            }
            None => Err(StoreError::Internal {
                message: "record not created".to_string(),
            }),
        }
    }

    /// Reads a user with its id
    pub async fn read(&self, id: Uuid) -> Result<Option<User>, StoreError> {
        let client = self.postgres_client().await?;

        let stmt = "SELECT * FROM users WHERE id = $1";
        let rows = client.query(stmt, &[&id]).await?;

        Ok(rows.first().map(|row| {
            let id = row.get::<_, Uuid>("id");
            let name = row.get::<_, String>("name");
            let email = row.get::<_, String>("email");
            let password = row.get::<_, String>("password");

            User {
                id,
                name,
                email,
                password,
            }
        }))
    }

    /// Reads a user with its email
    pub async fn read_with_email(&self, email: &str) -> Result<Option<User>, StoreError> {
        let client = self.postgres_client().await?;

        let stmt = "SELECT * FROM users WHERE email = $1";
        let rows = client.query(stmt, &[&email]).await?;

        Ok(rows.first().map(|row| {
            let id = row.get::<_, Uuid>("id");
            let name = row.get::<_, String>("name");
            let email = row.get::<_, String>("email");
            let password = row.get::<_, String>("password");

            User {
                id,
                name,
                email,
                password,
            }
        }))
    }

    /// Update a user
    pub async fn update(&self, id: Uuid, fields: UserFields) -> Result<(), StoreError> {
        let client = self.postgres_client().await?;

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
            // Nothing to update
            return Ok(());
        }

        let stmt = format!(
            "UPDATE users SET {} WHERE id=$1",
            cols.iter()
                .enumerate()
                .map(|(i, c)| format!("{}=${}", c, i + 2))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _res = client.execute(&stmt, &params).await?;

        Ok(())
    }

    /// Delete a user
    pub async fn delete(&self, id: Uuid) -> Result<(), StoreError> {
        let client = self.postgres_client().await?;

        let stmt = "DELETE FROM users WHERE id=$1";
        let _res = client.execute(stmt, &[&id]).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::future::Future;

    use fake::{
        faker::{internet::en::FreeEmail, name::en::Name},
        Fake,
    };

    use crate::{
        config::AppConfig,
        svc::auth::{NewUser, UserFields},
    };

    // Test runner to setup and cleanup a test
    async fn run_test<F>(f: impl Fn(UserStore, User) -> F)
    where
        F: Future<Output = (UserStore, User)>,
    {
        let cfg = AppConfig::load().await;
        let store = UserStore::new(cfg.postgres.new_pool());

        // create dummy user
        let name: String = Name().fake();
        let email: String = FreeEmail().fake();
        let user = store
            .create(NewUser {
                name,
                email,
                password: "dummy".to_string(),
            })
            .await
            .unwrap();

        // Run the test
        let (store, user) = f(store, user).await;

        // cleanup
        store.delete(user.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_table() {
        run_test(|store, user| async {
            store.create_table().await.unwrap();
            (store, user)
        })
        .await;
    }

    #[tokio::test]
    async fn test_read_with_id() {
        run_test(|store, user| async {
            let read_user = store.read(user.id).await.unwrap();
            assert_eq!(read_user.unwrap().id, user.id);
            (store, user)
        })
        .await;
    }

    #[tokio::test]
    async fn test_read_with_email() {
        run_test(|store, user| async {
            let read_user = store.read_with_email(&user.email).await.unwrap();
            assert_eq!(read_user.unwrap().email, user.email);
            (store, user)
        })
        .await;
    }

    #[tokio::test]
    async fn test_update() {
        run_test(|store, user| async {
            store
                .update(
                    user.id,
                    UserFields {
                        name: Some("test_user_update_new_name".to_string()),
                        email: None,
                        password: None,
                    },
                )
                .await
                .unwrap();
            (store, user)
        })
        .await;
    }
}
