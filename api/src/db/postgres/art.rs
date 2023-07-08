//! Articles

use tokio_postgres::{types::ToSql, Row};
use uuid::Uuid;

use crate::{error::Error, mdl::Article};

use super::{util::Vector, PostgresClient};

impl From<Row> for Article {
    fn from(value: Row) -> Self {
        Article {
            id: value.get::<_, Uuid>("id"),
            url: value.get::<_, String>("url"),
            summary: value.get::<_, String>("summary"),
            keywords: value.get::<_, Vec<String>>("keywords"),
            embeddings: value.get::<_, Vector>("embeddings"),
        }
    }
}

impl PostgresClient {
    /// Creates the `articles` table
    pub async fn create_table_articles(&self) -> Result<(), Error> {
        let client = self.client().await?;
        Ok(client
            .batch_execute(
                "
                    CREATE TABLE IF NOT EXISTS articles (
                        id          UUID PRIMARY KEY,
                        url         TEXT NOT NULL UNIQUE,    
                        summary     TEXT,
                        keywords    TEXT[],
                        embeddings  VECTOR(1536)
                    )",
            )
            .await?)
    }
}

impl PostgresClient {
    /// Search articles by url
    pub async fn search_articles_by_urls(&self, urls: &[&str]) -> Result<Vec<Article>, Error> {
        let client = self.client().await?;
        Ok(client
            .query(
                &format!(
                    "SELECT * FROM articles WHERE id IN({})",
                    urls.iter()
                        .enumerate()
                        .map(|(i, _url)| format!("${}", i + 1))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                urls.iter()
                    .map(|u| {
                        let v: &(dyn ToSql + Sync) = u;
                        v
                    })
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
            .await?
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<_>>())
    }

    /// Insert articles in the DB
    pub async fn insert_articles(&self, articles: Vec<Article>) -> Result<Vec<Article>, Error> {
        let client = self.client().await?;
        let stmt = format!(
            "INSERT INTO articles (id, url, summary, keywords, embeddings) VALUES {} RETURNING *",
            articles
                .iter()
                .enumerate()
                .map(|(i, _art)| {
                    format!(
                        "(${}, ${}, ${}, ${}, ${})",
                        i * 5 + 1,
                        i * 5 + 2,
                        i * 5 + 3,
                        i * 5 + 4,
                        i * 5 + 5
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
        let params = articles
            .iter()
            .flat_map(|art| {
                let params: Vec<&(dyn ToSql + Sync)> = vec![
                    &art.id,
                    &art.url,
                    &art.summary,
                    &art.keywords,
                    &art.embeddings,
                ];
                params
            })
            .collect::<Vec<_>>();
        Ok(client
            .query(&stmt, &params)
            .await?
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<_>>())
    }

    /// Remove articles in the DB
    pub async fn remove_articles(&self, articles: Vec<Article>) -> Result<(), Error> {
        let client = self.client().await?;
        let _res = client
            .execute(
                format!(
                    "DELETE FROM articles WHERE id IN({})",
                    articles
                        .iter()
                        .enumerate()
                        .map(|(i, _art)| format!("${}", i + 1))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .as_str(),
                &articles
                    .iter()
                    .map(|art| &art.id as &(dyn ToSql + Sync))
                    .collect::<Vec<_>>(),
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::lorem::en::Word;
    use fake::Fake;
    use rand::distributions::Uniform;
    use rand::Rng;

    use super::*;

    use crate::config::AppConfig;
    use crate::mdl::Article;

    /// Initializes the user store
    fn init_client() -> PostgresClient {
        let cfg = AppConfig::load();
        PostgresClient::new(cfg.postgres.new_pool())
    }

    /// Setup a test
    pub async fn setup() -> PostgresClient {
        init_client()
    }

    /// Teardown a test
    async fn teardown(_db: PostgresClient) {}

    #[tokio::test]
    async fn test_create_table() {
        let client = init_client();
        client.create_table_articles().await.unwrap();
    }

    #[tokio::test]
    async fn test_insert_articles() {
        let client = setup().await;

        let mut articles = vec![];
        for _i in 0..5 {
            let name: String = Word().fake();
            let url = format!("https:://www.link.com/{name}");
            let summary = "Lore ipsum".to_string();
            let keywords = vec!["kw1".to_string(), "kw2".to_string()];
            let embeddings = rand::thread_rng()
                .sample_iter(Uniform::from(0.0..1.0))
                .take(2)
                .collect::<Vec<_>>()
                .into();
            articles.push(Article {
                id: Uuid::new_v4(),
                url,
                summary,
                keywords,
                embeddings,
            })
        }
        let articles = client.insert_articles(articles).await.unwrap();
        assert_eq!(articles.len(), 5);

        client.remove_articles(articles).await.unwrap();
        teardown(client).await;
    }
}
