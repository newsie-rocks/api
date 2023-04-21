//! Users

use crate::svc::{
    auth::{NewUser, User, UserFields},
    Context,
};

use super::DbError;

/// Creates the `users` table
pub async fn create_table(ctx: &Context) -> Result<(), DbError> {
    let client = ctx.db_pool.get().await?;

    let stmt = "
    CREATE TABLE IF NOT EXISTS users (
        id      SERIAL PRIMARY KEY,
        name    TEXT NOT NULL
    )
    ";
    Ok(client.batch_execute(stmt).await?)
}

/// Creates a new user
///
/// A new user is created and its ID is populated
pub async fn create(ctx: &Context, new_user: NewUser) -> Result<User, DbError> {
    let client = ctx.db_pool.get().await?;

    let stmt = "INSERT into users (name) VALUES($1) RETURNING id";
    let rows = client.query(stmt, &[&new_user.name]).await?;

    match rows.first() {
        Some(row) => {
            let id = row.get::<_, i32>("id");

            let user = User {
                id,
                name: new_user.name,
            };
            Ok(user)
        }
        None => Err(DbError::Internal {
            message: "record not created".to_string(),
        }),
    }
}

/// Reads a user with its id
pub async fn read(ctx: &Context, id: i32) -> Result<Option<User>, DbError> {
    let client = ctx.db_pool.get().await?;

    let stmt = "SELECT * FROM users WHERE id = $1";
    let rows = client.query(stmt, &[&id]).await?;

    Ok(rows.first().map(|row| {
        let id = row.get::<_, i32>("id");
        let name = row.get::<_, String>("name");

        User { id, name }
    }))
}

/// Update a user
pub async fn update(ctx: &Context, fields: UserFields) -> Result<(), DbError> {
    let client = ctx.db_pool.get().await?;

    let mut stmt_cols = String::new();
    let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
    params.push(&fields.id);

    let mut i = 0;
    if let Some(name) = fields.name.as_ref() {
        i += 1;
        params.push(name);
        stmt_cols += "name = $2";
    }

    // ... add other fields here

    if i == 0 {
        // Nothing to update
        return Ok(());
    }

    let stmt = format!("UPDATE users SET {} WHERE id=$1", stmt_cols);
    let _res = client.execute(&stmt, &params).await?;

    Ok(())
}

/// Delete a user
pub async fn delete(ctx: &Context, user: User) -> Result<(), DbError> {
    let client = ctx.db_pool.get().await?;

    let stmt = "DELETE FROM users WHERE id=$1";
    let _res = client.execute(stmt, &[&user.id]).await?;

    Ok(())
}

#[cfg(test)]
mod tests {

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
        let db_pool = cfg.db.pool();

        Context {
            auth_secret: cfg.auth.secret.clone(),
            db_pool,
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

        let input = NewUser {
            name: "test_user".to_string(),
        };
        let user = super::create(&ctx, input).await.unwrap();

        assert_eq!(user.name, "test_user".to_string())
    }

    #[tokio::test]
    async fn read_with_id() {
        let ctx = init_ctx().await;

        let new_user_input = NewUser {
            name: "test_user".to_string(),
        };
        let new_user = super::create(&ctx, new_user_input).await.unwrap();

        let user = super::read(&ctx, new_user.id).await.unwrap();
        assert_eq!(user.unwrap().id, new_user.id);
    }

    #[tokio::test]
    async fn update() {
        let ctx = init_ctx().await;

        let new_user_input = NewUser {
            name: "test_user_update".to_string(),
        };
        let new_user = super::create(&ctx, new_user_input).await.unwrap();

        let fields = UserFields {
            id: new_user.id,
            name: Some("test_user_update_new_name".to_string()),
        };
        super::update(&ctx, fields).await.unwrap();
    }

    #[tokio::test]
    async fn delete() {
        let ctx = init_ctx().await;

        let new_user_input = NewUser {
            name: "test_user".to_string(),
        };
        let new_user = super::create(&ctx, new_user_input).await.unwrap();

        super::delete(&ctx, new_user).await.unwrap();
    }
}
