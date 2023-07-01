//! Users

use uuid::Uuid;

use crate::svc::{
    auth::{NewUser, User, UserFields},
    Context,
};

use super::DbError;

/// Creates the `users` table
pub async fn create_table(ctx: &Context) -> Result<(), DbError> {
    let client = ctx.postgres_pool.get().await?;

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
pub async fn create(ctx: &Context, new_user: NewUser) -> Result<User, DbError> {
    let client = ctx.postgres_pool.get().await?;

    let id = Uuid::new_v4();

    let stmt = "INSERT into users (id, name, email, password) VALUES($1, $2, $3, $4) RETURNING id";
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
        None => Err(DbError::Internal {
            message: "record not created".to_string(),
        }),
    }
}

/// Reads a user with its id
pub async fn read(ctx: &Context, id: Uuid) -> Result<Option<User>, DbError> {
    let client = ctx.postgres_pool.get().await?;

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
pub async fn read_with_email(ctx: &Context, email: &str) -> Result<Option<User>, DbError> {
    let client = ctx.postgres_pool.get().await?;

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
pub async fn update(ctx: &Context, id: Uuid, fields: UserFields) -> Result<(), DbError> {
    let client = ctx.postgres_pool.get().await?;

    let mut stmt_cols = String::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
    params.push(&id);

    let mut i = 1;
    if let Some(name) = fields.name.as_ref() {
        i += 1;
        params.push(name);
        stmt_cols += format!("name = ${i}").as_ref();
    }
    if let Some(email) = fields.email.as_ref() {
        i += 1;
        params.push(email);
        stmt_cols += format!("email = ${i}").as_ref();
    }
    if let Some(password) = fields.password.as_ref() {
        i += 1;
        params.push(password);
        stmt_cols += format!("password = ${i}").as_ref();
    }

    // ... add other fields here

    if i == 1 {
        // Nothing to update
        return Ok(());
    }

    let stmt = format!("UPDATE users SET {} WHERE id=$1", stmt_cols);
    let _res = client.execute(&stmt, &params).await?;

    Ok(())
}

/// Delete a user
pub async fn delete(ctx: &Context, id: Uuid) -> Result<(), DbError> {
    let client = ctx.postgres_pool.get().await?;

    let stmt = "DELETE FROM users WHERE id=$1";
    let _res = client.execute(stmt, &[&id]).await?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use crate::{
        config::AppConfig,
        svc::{
            auth::{NewUser, UserFields},
            Context,
        },
    };

    /// Initializes a dummy [Context] for tests
    async fn init_ctx() -> Context {
        let cfg = AppConfig::load().await;
        let postgres_pool = cfg.postgres.pool();
        let qdrant_client = Arc::new(cfg.qdrant.client().unwrap());

        Context {
            auth_secret: "dummy".to_string(),
            postgres_pool,
            qdrant_client,
            user: None,
        }
    }

    #[tokio::test]
    async fn create_table() {
        let ctx = init_ctx().await;

        super::create_table(&ctx).await.unwrap();
    }

    #[tokio::test]
    async fn create_user() {
        let ctx = init_ctx().await;

        let new_user = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let user = super::create(&ctx, new_user).await.unwrap();

        assert_eq!(user.name, "test_user".to_string())
    }

    #[tokio::test]
    async fn read_with_id() {
        let ctx = init_ctx().await;

        let new_user = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let new_user = super::create(&ctx, new_user).await.unwrap();

        let user = super::read(&ctx, new_user.id).await.unwrap();
        assert_eq!(user.unwrap().id, new_user.id);
    }

    #[tokio::test]
    async fn read_with_email() {
        let ctx = init_ctx().await;

        let new_user = super::create(
            &ctx,
            NewUser {
                name: "test_user".to_string(),
                email: "test@nicklabs.io".to_string(),
                password: "dummy".to_string(),
            },
        )
        .await
        .unwrap();

        let user = super::read_with_email(&ctx, new_user.email.as_str())
            .await
            .unwrap();
        assert_eq!(user.unwrap().email, new_user.email);
    }

    #[tokio::test]
    async fn update() {
        let ctx = init_ctx().await;

        let new_user = super::create(
            &ctx,
            NewUser {
                name: "test_user_update".to_string(),
                email: "test@nicklabs.io".to_string(),
                password: "dummy".to_string(),
            },
        )
        .await
        .unwrap();

        super::update(
            &ctx,
            new_user.id,
            UserFields {
                id: None,
                name: Some("test_user_update_new_name".to_string()),
                email: None,
                password: None,
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn delete() {
        let ctx = init_ctx().await;

        let new_user_input = NewUser {
            name: "test_user".to_string(),
            email: "test@nicklabs.io".to_string(),
            password: "dummy".to_string(),
        };
        let new_user = super::create(&ctx, new_user_input).await.unwrap();

        super::delete(&ctx, new_user.id).await.unwrap();
    }
}
