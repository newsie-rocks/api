//! Models

use anyhow::Error;
use rss::validation::Validate;

/// Configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// API URL
    pub api_url: String,
    /// Authentication token
    pub token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000".to_string(),
            token: None,
        }
    }
}

/// A feed
#[derive(Debug, Clone)]
pub struct Feed {
    /// Feed URL
    pub url: String,
    /// Feed type
    pub r#type: FeedType,
    /// Feed name
    pub name: Option<String>,
    /// Folder
    pub folder: Option<String>,
    /// Articles
    pub articles: Vec<Article>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedType {
    Rss,
    Atom,
}

/// An article
#[derive(Debug, Clone)]
pub struct Article {
    /// Article url
    pub url: String,
    /// Title
    pub title: Option<String>,
}

impl From<rss::Channel> for Feed {
    fn from(value: rss::Channel) -> Self {
        Self {
            url: value.link,
            r#type: FeedType::Rss,
            name: None,
            folder: None,
            articles: value.items.into_iter().map(|item| item.into()).collect(),
        }
    }
}

impl From<atom_syndication::Feed> for Feed {
    fn from(value: atom_syndication::Feed) -> Self {
        Self {
            url: value.id().to_string(),
            r#type: FeedType::Atom,
            name: None,
            folder: None,
            articles: value
                .entries
                .into_iter()
                .map(|entry| entry.into())
                .collect(),
        }
    }
}

impl From<rss::Item> for Article {
    fn from(value: rss::Item) -> Self {
        Article {
            url: value.link.unwrap_or_default(),
            title: value.title,
        }
    }
}

impl From<atom_syndication::Entry> for Article {
    fn from(value: atom_syndication::Entry) -> Self {
        Article {
            url: value.id().to_string(),
            title: Some(value.title.as_str().to_string()),
        }
    }
}

impl Feed {
    /// Tries to load a RSS feed from its url
    pub async fn from_url(url: &str) -> Result<Self, Error> {
        let content = reqwest::get(url).await?.bytes().await?;

        // try for RSS
        match rss::Channel::read_from(&content[..]) {
            Ok(channel) => {
                channel.validate()?;
                return Ok(channel.into());
            }
            Err(_err) => {
                // continue
            }
        }

        // try for atom
        match atom_syndication::Feed::read_from(&content[..]) {
            Ok(feed) => {
                return Ok(feed.into());
            }
            Err(err) => {
                // continue
            }
        }

        Err(Error::msg("invalid feed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rss_ok() {
        let feed = Feed::from_url("https://news.ycombinator.com/rss")
            .await
            .unwrap();
        assert_eq!(feed.r#type, FeedType::Rss);
    }

    #[tokio::test]
    #[should_panic]
    async fn test_rss_err() {
        let _feed = Feed::from_url("http://www.google.com").await.unwrap();
    }
}
