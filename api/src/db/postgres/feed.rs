//! Feeds

use tokio_postgres::{types::ToSql, Row};
use uuid::Uuid;

use crate::{
    error::Error,
    mdl::{Feed, FeedUpdate},
};

use super::PostgresClient;

impl From<Row> for Feed {
    fn from(value: Row) -> Self {
        Feed {
            id: value.get::<_, Uuid>("id"),
            user_id: value.get::<_, Uuid>("user_id"),
            url: value.get::<_, String>("url"),
            name: value.get::<_, Option<String>>("name"),
        }
    }
}

impl PostgresClient {
    /// Creates the `feeds` table
    pub async fn create_table_feeds(&self) -> Result<(), Error> {
        let client = self.client().await?;

        Ok(client
            .batch_execute(
                "
                CREATE TABLE IF NOT EXISTS feeds (
                    id          UUID PRIMARY KEY,
                    user_id     UUID NOT NULL,
                    url         TEXT NOT NULL,
                    name        TEXT,
                    FOREIGN KEY (user_id) REFERENCES users(id)
                )
            ",
            )
            .await?)
    }

    /// Reads all user feeds for a user
    pub async fn read_user_feeds(&self, user_id: Uuid) -> Result<Vec<Feed>, Error> {
        let client = self.client().await?;

        Ok(client
            .query("SELECT * FROM feeds WHERE user_id = $1", &[&user_id])
            .await?
            .into_iter()
            .map(|row| row.into())
            .collect())
    }

    /// Sync all the user feeds
    pub async fn sync_user_feeds(
        &self,
        user_id: Uuid,
        feeds: Vec<FeedUpdate>,
    ) -> Result<Vec<Feed>, Error> {
        let mut client = self.client().await?;
        let trx = client.transaction().await?;

        // // read all user feeds
        // let curr_feeds = trx
        //     .query("SELECT * FROM users WHERE user_id=$1", &[&user_id])
        //     .await?
        //     .into_iter()
        //     .map(|row| row.into())
        //     .collect::<Vec<Feed>>();

        // remove all feeds
        let _res = trx
            .execute("DELETE FROM feeds WHERE user_id=$1", &[&user_id])
            .await?;

        // insert all feeds
        let new_feeds = if !feeds.is_empty() {
            let mut insert_stmt_values: Vec<String> = vec![];
            let mut insert_params: Vec<(Uuid, &Uuid, &String, &Option<String>)> = vec![];
            for (i, f) in feeds.iter().enumerate() {
                let id = match f.id {
                    Some(id) => id,
                    None => Uuid::new_v4(),
                };
                insert_stmt_values.push(format!(
                    "(${}, ${}, ${}, ${})",
                    i * 4 + 1,
                    i * 4 + 2,
                    i * 4 + 3,
                    i * 4 + 4
                ));
                insert_params.push((id, &user_id, &f.url, &f.name));
            }
            trx.query(
                &format!(
                    "INSERT into feeds (id, user_id, url, name) VALUES {} RETURNING *",
                    insert_stmt_values.join(", ")
                ),
                &insert_params
                    .iter()
                    .flat_map(|(id, user_id, url, name)| {
                        let v: Vec<&(dyn ToSql + Sync)> = vec![id, *user_id, *url, *name];
                        v
                    })
                    .collect::<Vec<_>>(),
            )
            .await?
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<Feed>>()
        } else {
            vec![]
        };

        // commit the transaction
        trx.commit().await?;
        Ok(new_feeds)
    }

    /// Delete all user feeds
    pub async fn delete_user_feeds(&self, user_id: Uuid) -> Result<(), Error> {
        let client = self.client().await?;
        let _res = client
            .execute("DELETE FROM feeds WHERE user_id=$1", &[&user_id])
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::AppConfig;
    use crate::db::postgres::user::tests::{setup_test_user, teardown_test_user};
    use crate::mdl::User;

    /// Initializes the user store
    fn init_db() -> PostgresClient {
        let cfg = AppConfig::load();
        PostgresClient::new(cfg.postgres.new_pool())
    }

    /// Setup a test
    pub async fn setup() -> (PostgresClient, User, Vec<Feed>) {
        let (db, user) = setup_test_user().await;

        let feeds = db
            .sync_user_feeds(
                user.id,
                vec![
                    FeedUpdate {
                        id: None,
                        url: "https://ai.googleblog.com/atom.xml".to_string(),
                        name: Some("my feed".to_string()),
                    },
                    FeedUpdate {
                        id: Some(Uuid::new_v4()),
                        url: "https://ai.googleblog.com/atom2.xml".to_string(),
                        name: None,
                    },
                ],
            )
            .await
            .unwrap();
        (db, user, feeds)
    }

    /// Teardown a test
    async fn teardown(db: PostgresClient, user: User) {
        db.delete_user_feeds(user.id).await.unwrap();
        teardown_test_user(db, user).await;
    }

    #[tokio::test]
    async fn test_create_table() {
        let db = init_db();
        db.create_table_feeds().await.unwrap();
    }

    #[tokio::test]
    async fn test_read_feeds() {
        let (db, test_user, _test_feeds) = setup().await;
        let feeds = db.read_user_feeds(test_user.id).await.unwrap();
        assert_eq!(feeds.len(), 2);
        teardown(db, test_user).await;
    }
}
