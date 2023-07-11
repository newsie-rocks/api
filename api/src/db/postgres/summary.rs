//! Articles

use tokio_postgres::{types::ToSql, Row};
use uuid::Uuid;

use crate::{error::Error, mdl::Summary};

use super::{util::Vector, PostgresClient};

impl From<Row> for Summary {
    fn from(value: Row) -> Self {
        Summary {
            id: value.get::<_, Uuid>("id"),
            url: value.get::<_, String>("url"),
            summary: value.get::<_, String>("summary"),
            keywords: value.get::<_, Vec<String>>("keywords"),
            embeddings: value.get::<_, Vector>("embeddings"),
        }
    }
}

impl PostgresClient {
    /// Creates the `summaries` table
    pub async fn create_table_summaries(&self) -> Result<(), Error> {
        let client = self.client().await?;
        Ok(client
            .batch_execute(
                "
                    CREATE TABLE IF NOT EXISTS summaries (
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
    /// Search summaries by url
    pub async fn search_summaries_by_urls(&self, urls: &[&str]) -> Result<Vec<Summary>, Error> {
        let client = self.client().await?;
        Ok(client
            .query(
                &format!(
                    "SELECT * FROM summaries WHERE url IN({})",
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

    /// Insert summaries in the DB
    pub async fn insert_summaries(&self, articles: Vec<Summary>) -> Result<Vec<Summary>, Error> {
        let client = self.client().await?;
        let stmt = format!(
            "INSERT INTO summaries (id, url, summary, keywords, embeddings) VALUES {} RETURNING *",
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

    /// Remove summaries in the DB
    pub async fn remove_summaries(&self, summaries: Vec<Summary>) -> Result<(), Error> {
        let client = self.client().await?;
        let _res = client
            .execute(
                format!(
                    "DELETE FROM summaries WHERE id IN({})",
                    summaries
                        .iter()
                        .enumerate()
                        .map(|(i, _art)| format!("${}", i + 1))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .as_str(),
                &summaries
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
    use crate::mdl::Summary;

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
        client.create_table_summaries().await.unwrap();
    }

    #[tokio::test]
    async fn test_insert_summaries() {
        let client = setup().await;

        let mut summaries = vec![];
        for _i in 0..5 {
            let name: String = Word().fake();
            let url = format!("https:://www.link.com/{name}");
            let summary = "Lore ipsum".to_string();
            let keywords = vec!["kw1".to_string(), "kw2".to_string()];
            let embeddings = rand::thread_rng()
                .sample_iter(Uniform::from(0.0..1.0))
                .take(1536)
                .collect::<Vec<_>>()
                .into();
            summaries.push(Summary {
                id: Uuid::new_v4(),
                url,
                summary,
                keywords,
                embeddings,
            })
        }
        let summaries = client.insert_summaries(summaries).await.unwrap();
        assert_eq!(summaries.len(), 5);

        client.remove_summaries(summaries).await.unwrap();
        teardown(client).await;
    }
}
