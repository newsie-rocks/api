//! Article service

use async_openai::types::{
    ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs,
    Role,
};
use futures::future::join_all;
use uuid::Uuid;

use crate::{config::OpenAiClient, db::postgres::PostgresClient, error::Error, mdl::Summary};

/// Article service
#[derive(Clone)]
pub struct ArticleService {
    /// Postgres client
    pub db: PostgresClient,
    /// OpenAI client
    pub openai: OpenAiClient,
}

impl ArticleService {
    /// Creates a new service instance
    pub fn new(postgres_client: PostgresClient, openai_client: OpenAiClient) -> Self {
        Self {
            db: postgres_client,
            openai: openai_client,
        }
    }
}

impl ArticleService {
    /// Retrieves a list of articles with their summaries
    ///
    /// # Notes
    ///
    /// To keep a cache of already processed articles, we first check if articles are
    /// already in the database of articles
    pub async fn process_summaries(&self, urls: &[&str]) -> Result<Vec<Summary>, Error> {
        // search articles by ID to retrieve already processed articles
        let mut found_articles = self.db.search_summaries_by_urls(urls).await?;

        // discriminate new vs already processed articles
        let found_urls = found_articles
            .iter()
            .map(|art| art.url.as_str())
            .collect::<Vec<_>>();
        let not_found_urls = urls
            .iter()
            .filter(|url| !found_urls.contains(url))
            .collect::<Vec<_>>();

        // process new articles in parallel
        let mut new_articles = if !not_found_urls.is_empty() {
            let mut tasks = vec![];
            for url in not_found_urls {
                tasks.push(self.process_article(url))
            }
            let new_articles = join_all(tasks)
                .await
                .into_iter()
                .collect::<Result<Vec<Summary>, Error>>()?;
            self.db.insert_summaries(new_articles).await?
        } else {
            vec![]
        };

        let mut articles = vec![];
        articles.append(&mut found_articles);
        articles.append(&mut new_articles);
        Ok(articles)
    }

    /// Processes an article
    async fn process_article(&self, url: &str) -> Result<Summary, Error> {
        let summary = self.summarize(url).await?;
        let keywords = self.extract_keywords(url).await?;
        let embeddings = self.get_embeddings(&summary).await?.into();

        Ok(Summary {
            id: Uuid::new_v4(),
            url: url.to_string(),
            summary,
            keywords,
            embeddings,
        })
    }

    // Summarizes an article
    async fn summarize(&self, url: &str) -> Result<String, Error> {
        // NB: we use the 16k model to allow for longer context.
        const OPENAI_MODEL: &str = "gpt-3.5-turbo";

        // Every request struct has companion builder struct with same name + Args suffix
        let request = CreateChatCompletionRequestArgs::default()
            .model(OPENAI_MODEL)
            .temperature(0.0)
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::Assistant)
                    .content("You are an assistant which reads and summarizes articles.")
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(format!("Summarize this link: {url}"))
                    .build()?,
            ])
            .build()?;

        // Call API
        let response = self
            .openai
            .chat() // Get the API "group" (completions, images, etc.) from the client
            .create(request) // Make the API call in that "group"
            .await?;

        let summary = response
            .choices
            .get(0)
            .ok_or(Error::Internal("missing OpenAI response".to_string(), None))?
            .message
            .content
            .clone()
            .ok_or(Error::Internal("missing OpenAI response".to_string(), None))?;
        Ok(summary)
    }

    // Extract keywords from an article
    async fn extract_keywords(&self, url: &str) -> Result<Vec<String>, Error> {
        const OPENAI_MODEL: &str = "gpt-3.5-turbo";

        // Every request struct has companion builder struct with same name + Args suffix
        let request = CreateChatCompletionRequestArgs::default()
            .model(OPENAI_MODEL)
            .temperature(0.0)
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::Assistant)
                    .content("Extract the keywords from the provided link. Return the keywords as a list of comma separated values, with a maximum number of 5 keywords")
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(url.to_string())
                    .build()?,
            ])
            .build()?;

        // Call API
        let response = self
            .openai
            .chat() // Get the API "group" (completions, images, etc.) from the client
            .create(request) // Make the API call in that "group"
            .await?;

        let text = response
            .choices
            .get(0)
            .ok_or(Error::Internal("missing OpenAI response".to_string(), None))?
            .message
            .content
            .clone()
            .ok_or(Error::Internal("missing OpenAI response".to_string(), None))?;
        Ok(text.split(',').map(|s| s.trim().to_string()).collect())
    }

    /// Gets the embeddings for a text
    async fn get_embeddings(&self, text: &str) -> Result<Vec<f32>, Error> {
        const OPENAI_MODEL: &str = "text-embedding-ada-002";

        let request = CreateEmbeddingRequestArgs::default()
            .model(OPENAI_MODEL)
            .input(text)
            .build()?;

        Ok(self
            .openai
            .embeddings() // Get the API "group" (completions, images, etc.) from the client
            .create(request) // Make the API call in that "group"
            .await?
            .data
            .remove(0)
            .embedding)
    }
}

#[cfg(test)]
mod tests {
    use time::Instant;

    use crate::config::AppConfig;

    use super::*;

    async fn setup() -> ArticleService {
        let cfg = AppConfig::load();
        let postgres_pool = cfg.postgres.new_pool();
        let postgres_client = PostgresClient::new(postgres_pool);
        let openai_client = cfg.openai.new_client();

        ArticleService::new(postgres_client, openai_client)
    }

    #[tokio::test]
    async fn test_process_one_article() {
        let service = setup().await;
        let url = "http://ai.googleblog.com/2023/07/modular-visual-question-answering-via.html";
        let start = Instant::now();
        let article = service.process_article(url).await.unwrap();
        let duration = start.elapsed();
        println!("{} secs", duration.as_seconds_f32());
        println!("\n{}", article.summary);
        println!("\n{:?}", article.keywords);
    }

    #[tokio::test]
    async fn test_process_articles() {
        let service = setup().await;
        let urls = [
            "http://ai.googleblog.com/2023/07/modular-visual-question-answering-via.html",
            "http://jalammar.github.io/illustrated-stable-diffusion/",
            "https://github.com/raghavan/PdfGptIndexer",
        ];
        let start = Instant::now();
        let articles = service.process_summaries(&urls).await.unwrap();
        let duration = start.elapsed();
        println!("{} secs", duration.as_seconds_f32());
        for art in &articles {
            println!("\n{}", art.summary);
            println!("\n{:?}", art.keywords);
        }
    }
}
