//! GitHub Search API for finding repositories.

use super::error::{GitHubError, Result};
use super::GitHubClient;
use serde::{Deserialize, Serialize};

/// Search result from GitHub Search API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub stars: u64,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub updated_at: String,
}

/// Search for repositories by topic using GitHub Search API.
pub async fn search_repos_by_topic(
    client: &GitHubClient,
    topic: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    search_repos(client, &format!("topic:{topic}"), limit).await
}

/// Search for repositories by query.
pub async fn search_repos(
    client: &GitHubClient,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchResult>> {
    let per_page = limit.min(100);

    let url = format!(
        "/search/repositories?q={}&sort=stars&order=desc&per_page={}",
        urlencoding::encode(query),
        per_page
    );

    let response: serde_json::Value = client
        .octocrab()
        .get(&url, None::<&()>)
        .await
        .map_err(|e| GitHubError::ApiError(format!("Search failed: {e}")))?;

    let items = response["items"]
        .as_array()
        .ok_or_else(|| GitHubError::ApiError("Invalid search response".to_string()))?;

    let results: Vec<SearchResult> = items
        .iter()
        .take(limit)
        .map(|item| SearchResult {
            full_name: item["full_name"].as_str().unwrap_or_default().to_string(),
            description: item["description"].as_str().map(String::from),
            html_url: item["html_url"].as_str().unwrap_or_default().to_string(),
            stars: item["stargazers_count"].as_u64().unwrap_or(0),
            language: item["language"].as_str().map(String::from),
            topics: item["topics"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            updated_at: item["updated_at"].as_str().unwrap_or_default().to_string(),
        })
        .collect();

    Ok(results)
}

