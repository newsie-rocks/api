//! Article service

use async_openai::types::{
    ChatCompletionRequestMessageArgs, CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs,
    Role,
};
use futures::future::join_all;
use uuid::Uuid;

use crate::{config::OpenAiClient, db::postgres::PostgresClient, error::Error, mdl::Article};

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
    pub async fn process_articles(&self, urls: &[&str]) -> Result<Vec<Article>, Error> {
        // search articles by ID to retrieve already processed articles
        let mut found_articles = self.db.search_articles_by_urls(urls).await?;
        let found_urls = found_articles
            .iter()
            .map(|art| art.url.as_str())
            .collect::<Vec<_>>();

        // insert missing articles
        let not_found_urls = urls
            .iter()
            .filter(|url| !found_urls.contains(url))
            .collect::<Vec<_>>();

        // process missing articles in parallel
        let mut tasks = vec![];
        for url in not_found_urls {
            tasks.push(self.process_article(url))
        }
        let new_articles = join_all(tasks)
            .await
            .into_iter()
            .collect::<Result<Vec<Article>, Error>>()?;
        let mut insert_articles = self.db.insert_articles(new_articles).await?;

        let mut articles = vec![];
        articles.append(&mut found_articles);
        articles.append(&mut insert_articles);
        Ok(articles)
    }

    /// Processes an article
    async fn process_article(&self, url: &str) -> Result<Article, Error> {
        let summary = self.summarize(url).await?;
        let keywords = self.extract_keywords(url).await?;
        let embeddings = self.get_embeddings(&summary).await?.into();

        Ok(Article {
            id: Uuid::new_v4(),
            url: url.to_string(),
            summary,
            keywords,
            embeddings,
        })
    }

    // Summarizes an article
    async fn summarize(&self, url: &str) -> Result<String, Error> {
        const OPENAI_MODEL: &str = "gpt-3.5-turbo";

        // Every request struct has companion builder struct with same name + Args suffix
        let request = CreateChatCompletionRequestArgs::default()
            .model(OPENAI_MODEL)
            .max_tokens(4_096_u16)
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content("You are an assistant which reads and summarizes articles.")
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(format!("Summarize this: {url}"))
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
            .max_tokens(4_096_u16)
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content("Extract the keywords from the provided link. Return the keywords as a list of comma separated values.")
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
        Ok(text.split(',').map(|s| s.to_string()).collect())
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
