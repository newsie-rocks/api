//! Feeds

use tokio_postgres::{types::ToSql, Row};
use uuid::Uuid;

use crate::{
    error::Error,
    mdl::{Feed, FeedUpdateFields, NewFeed},
};

use super::PostgresDb;

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

impl PostgresDb {
    /// Creates the `feeds` table
    pub async fn create_table_feeds(&self) -> Result<(), Error> {
        let client = self.client().await?;

        let stmt = "
            CREATE TABLE IF NOT EXISTS feeds (
                id          UUID PRIMARY KEY,
                user_id     UUID NOT NULL,
                url         TEXT NOT NULL,
                name        TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )
        ";
        Ok(client.batch_execute(stmt).await?)
    }

    /// Adds a user feed to the db
    pub async fn add_user_feed(&self, user_id: Uuid, new_feed: NewFeed) -> Result<Feed, Error> {
        let client = self.client().await?;
        let id = Uuid::new_v4();

        let stmt = "INSERT into feeds (id, user_id, url, name) VALUES ($1, $2, $3, $4) RETURNING *";
        let rows = client
            .query(stmt, &[&id, &user_id, &new_feed.url, &new_feed.name])
            .await?;

        match rows.into_iter().next() {
            Some(row) => Ok(row.into()),
            None => Err(Error::Internal("record not created".to_string(), None)),
        }
    }

    /// Adds user feeds to the db
    pub async fn add_user_feeds(
        &self,
        user_id: Uuid,
        feeds: Vec<NewFeed>,
    ) -> Result<Vec<Feed>, Error> {
        if feeds.is_empty() {
            return Ok(vec![]);
        }

        let client = self.client().await?;

        let ids: Vec<Uuid> = feeds.iter().map(|_| Uuid::new_v4()).collect();
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = vec![];
        let mut values = vec![];
        for (i, feed) in feeds.iter().enumerate() {
            params.append(&mut vec![&ids[i], &user_id, &feed.url, &feed.name]);
            values.push(format!(
                "(${}, ${}, ${}, ${})",
                i * 4 + 1,
                i * 4 + 2,
                i * 4 + 3,
                i * 4 + 4
            ));
        }
        let stmt = format!(
            "INSERT into feeds (id, user_id, url, name) VALUES {} RETURNING *",
            values.join(","),
        );
        let rows = client.query(&stmt, &params).await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    /// Reads a user feed with its id
    pub async fn read_feed(&self, id: Uuid) -> Result<Option<Feed>, Error> {
        let client = self.client().await?;
        let stmt = "SELECT * FROM feeds WHERE id = $1";
        let rows = client.query(stmt, &[&id]).await?;

        Ok(rows.into_iter().next().map(|row| row.into()))
    }

    /// Reads all user feeds for a user
    pub async fn read_feed_for_user(&self, user_id: Uuid) -> Result<Vec<Feed>, Error> {
        let client = self.client().await?;

        let stmt = "SELECT * FROM feeds WHERE user_id = $1";
        let rows = client.query(stmt, &[&user_id]).await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }

    /// Update a user feed
    pub async fn update_feed(&self, id: Uuid, fields: FeedUpdateFields) -> Result<Feed, Error> {
        let client = self.client().await?;

        let mut cols: Vec<&str> = vec![];
        let mut params: Vec<&(dyn ToSql + Sync)> = vec![];
        params.push(&id);

        if let Some(url) = fields.url.as_ref() {
            cols.push("url");
            params.push(url);
        }
        if let Some(name) = fields.name.as_ref() {
            cols.push("name");
            params.push(name);
        }
        // ... add other fields here

        if cols.is_empty() {
            // Nothing to update
            self.read_feed(id).await.map(|u| u.unwrap())
        } else {
            let stmt = format!(
                "UPDATE feeds SET {} WHERE id=$1 RETURNING *",
                cols.iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}=${}", c, i + 2))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let rows = client.query(&stmt, &params).await?;

            match rows.into_iter().next() {
                Some(row) => Ok(row.into()),
                None => Err(Error::Internal("record not updated".to_string(), None)),
            }
        }
    }

    /// Delete a user feed
    pub async fn delete_feed(&self, id: Uuid) -> Result<(), Error> {
        let client = self.client().await?;
        let stmt = "DELETE FROM feeds WHERE id=$1";
        let _res = client.execute(stmt, &[&id]).await?;

        Ok(())
    }

    /// Delete user feeds
    pub async fn delete_feeds(&self, ids: Vec<Uuid>) -> Result<(), Error> {
        if ids.is_empty() {
            return Ok(());
        }

        let client = self.client().await?;
        let stmt = format!(
            "DELETE FROM feeds WHERE id IN({})",
            ids.iter()
                .enumerate()
                .map(|(i, _)| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let params = ids
            .iter()
            .map(|id| id as &(dyn ToSql + Sync))
            .collect::<Vec<_>>();
        let _res = client.execute(&stmt, &params).await?;

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
    fn init_db() -> PostgresDb {
        let cfg = AppConfig::load();
        PostgresDb::new(cfg.postgres.new_pool())
    }

    /// Setup a test
    pub async fn setup() -> (PostgresDb, User, Feed) {
        let (db, user) = setup_test_user().await;
        let url = "https://ai.googleblog.com/atom.xml".to_string();
        let name = Some("my feed".to_string());

        let feed = db
            .add_user_feed(user.id, NewFeed { url, name })
            .await
            .unwrap();
        (db, user, feed)
    }

    /// Teardown a test
    async fn teardown(db: PostgresDb, user: User, feed: Feed) {
        db.delete_feed(feed.id).await.unwrap();
        teardown_test_user(db, user).await;
    }

    #[tokio::test]
    async fn test_create_table() {
        let db = init_db();
        db.create_table_feeds().await.unwrap();
    }

    #[tokio::test]
    async fn test_read() {
        let (db, user, test_feed) = setup().await;
        let feed = db.read_feed(test_feed.id).await.unwrap();
        assert_eq!(feed.unwrap().id, test_feed.id);
        teardown(db, user, test_feed).await;
    }

    #[tokio::test]
    async fn test_update() {
        let (db, user, test_feed) = setup().await;
        let new_name = "my feed 2".to_string();
        let feed = db
            .update_feed(
                test_feed.id,
                FeedUpdateFields {
                    url: None,
                    name: Some(Some(new_name.clone())),
                },
            )
            .await
            .unwrap();
        assert_eq!(feed.name, Some(new_name));
        teardown(db, user, test_feed).await;
    }

    #[tokio::test]
    async fn test_add_multiple() {
        let (db, user, test_feed) = setup().await;
        let feeds = db
            .add_user_feeds(
                user.id,
                vec![
                    NewFeed {
                        url: "http://www.feed.com".to_string(),
                        name: Some("my feed".to_string()),
                    },
                    NewFeed {
                        url: "http://www.fee2d.com".to_string(),
                        name: Some("my feed 2".to_string()),
                    },
                ],
            )
            .await
            .unwrap();
        db.delete_feeds(feeds.into_iter().map(|feed| feed.id).collect())
            .await
            .unwrap();
        teardown(db, user, test_feed).await;
    }
}
