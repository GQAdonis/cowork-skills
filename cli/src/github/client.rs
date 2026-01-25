//! GitHub API client for fetching repository content.

use octocrab::Octocrab;
use tokio::time::{Duration, sleep};

use super::config::{FetchConfig, GitHubConfig};
use super::error::{GitHubError, Result};
use super::types::{GitHubContent, GitHubDirEntry, GitHubFile, Repository};

/// GitHub API client.
pub struct GitHubClient {
    octocrab: Octocrab,
    rate_limit_delay: Duration,
    #[allow(dead_code)]
    config: GitHubConfig,
}

impl GitHubClient {
    /// Create a new client using `GITHUB_TOKEN` environment variable.
    pub fn new() -> Result<Self> {
        Self::with_config(FetchConfig::default())
    }

    /// Create a new client with custom configuration.
    pub fn with_config(config: FetchConfig) -> Result<Self> {
        let mut builder = Octocrab::builder();

        let token = std::env::var(&config.github.token_env_var).map_err(|_| {
            GitHubError::AuthError(format!(
                "{} environment variable not set",
                config.github.token_env_var
            ))
        })?;

        builder = builder.personal_token(token);

        if !config.github.api_base_url.is_empty()
            && config.github.api_base_url != "https://api.github.com"
        {
            builder = builder
                .base_uri(&config.github.api_base_url)
                .map_err(|e| GitHubError::ConfigError(format!("Invalid base URI: {e}")))?;
        }

        let octocrab = builder.build()?;
        let rate_limit_delay = config.rate_limiting.delay_duration();

        Ok(Self {
            octocrab,
            rate_limit_delay,
            config: config.github,
        })
    }

    /// Test the API connection.
    #[allow(dead_code)]
    pub async fn test_connection(&self) -> Result<()> {
        self.octocrab
            .ratelimit()
            .get()
            .await
            .map_err(|e| GitHubError::ApiError(format!("Connection test failed: {e}")))?;
        Ok(())
    }

    /// Get rate limit status.
    #[allow(dead_code)]
    pub async fn get_rate_limit(&self) -> Result<String> {
        let rate_limit = self
            .octocrab
            .ratelimit()
            .get()
            .await
            .map_err(|e| GitHubError::ApiError(format!("Failed to get rate limit: {e}")))?;

        Ok(format!(
            "Rate limit: {}/{} remaining, resets at {}",
            rate_limit.resources.core.remaining,
            rate_limit.resources.core.limit,
            rate_limit.resources.core.reset
        ))
    }

    /// Fetch file or directory content from a repository.
    pub async fn fetch_content(
        &self,
        repo: &Repository,
        path: &str,
        git_ref: Option<&str>,
    ) -> Result<GitHubContent> {
        sleep(self.rate_limit_delay).await;

        let url = match git_ref {
            Some(r) => format!(
                "/repos/{}/{}/contents/{}?ref={}",
                repo.owner, repo.name, path, r
            ),
            None => format!("/repos/{}/{}/contents/{}", repo.owner, repo.name, path),
        };

        let response: serde_json::Value = self
            .octocrab
            .get(&url, None::<&()>)
            .await
            .map_err(|e| GitHubError::ApiError(format!("Failed to fetch content: {e}")))?;

        // Check if response is an array (directory) or object (file)
        if response.is_array() {
            let entries: Vec<GitHubDirEntry> = serde_json::from_value(response).map_err(|e| {
                GitHubError::ApiError(format!("Failed to parse directory listing: {e}"))
            })?;
            Ok(GitHubContent::Directory(entries))
        } else {
            let file: GitHubFile = serde_json::from_value(response)
                .map_err(|e| GitHubError::ApiError(format!("Failed to parse file content: {e}")))?;
            Ok(GitHubContent::File(file))
        }
    }

    /// Fetch file content as decoded string.
    pub async fn fetch_file_content(
        &self,
        repo: &Repository,
        path: &str,
        git_ref: Option<&str>,
    ) -> Result<String> {
        match self.fetch_content(repo, path, git_ref).await? {
            GitHubContent::File(file) => file.decode_content().ok_or_else(|| {
                GitHubError::ApiError(format!("Failed to decode file content for {path}"))
            }),
            GitHubContent::Directory(_) => Err(GitHubError::ApiError(format!(
                "Expected file but got directory: {path}"
            ))),
        }
    }

    /// List directory contents.
    pub async fn list_directory(
        &self,
        repo: &Repository,
        path: &str,
        git_ref: Option<&str>,
    ) -> Result<Vec<GitHubDirEntry>> {
        match self.fetch_content(repo, path, git_ref).await? {
            GitHubContent::Directory(entries) => Ok(entries),
            GitHubContent::File(_) => Err(GitHubError::ApiError(format!(
                "Expected directory but got file: {path}"
            ))),
        }
    }

    /// Recursively list all files in a directory.
    pub async fn list_all_files(
        &self,
        repo: &Repository,
        path: &str,
        git_ref: Option<&str>,
    ) -> Result<Vec<GitHubDirEntry>> {
        let mut all_files = Vec::new();
        let mut dirs_to_visit = vec![path.to_string()];

        while let Some(current_path) = dirs_to_visit.pop() {
            let entries = self.list_directory(repo, &current_path, git_ref).await?;

            for entry in entries {
                if entry.is_file() {
                    all_files.push(entry);
                } else if entry.is_dir() {
                    dirs_to_visit.push(entry.path);
                }
            }
        }

        Ok(all_files)
    }

    /// Fetch repository metadata (description, topics, etc.).
    pub async fn fetch_repo_info(&self, repo: &Repository) -> Result<RepoInfo> {
        sleep(self.rate_limit_delay).await;

        let url = format!("/repos/{}/{}", repo.owner, repo.name);

        let response: serde_json::Value = self
            .octocrab
            .get(&url, None::<&()>)
            .await
            .map_err(|e| GitHubError::ApiError(format!("Failed to fetch repo info: {e}")))?;

        Ok(RepoInfo {
            name: response["name"].as_str().unwrap_or_default().to_string(),
            full_name: response["full_name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            description: response["description"].as_str().map(String::from),
            default_branch: response["default_branch"]
                .as_str()
                .unwrap_or("main")
                .to_string(),
            language: response["language"].as_str().map(String::from),
            topics: response["topics"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            stars: response["stargazers_count"].as_u64().unwrap_or(0),
            forks: response["forks_count"].as_u64().unwrap_or(0),
            html_url: response["html_url"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
    }

    /// Get the default branch name for a repository.
    #[allow(dead_code)]
    pub async fn get_default_branch(&self, repo: &Repository) -> Result<String> {
        let info = self.fetch_repo_info(repo).await?;
        Ok(info.default_branch)
    }
}

impl GitHubClient {
    /// Get the underlying octocrab instance for advanced operations.
    pub(crate) fn octocrab(&self) -> &Octocrab {
        &self.octocrab
    }
}

/// Repository metadata.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RepoInfo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub language: Option<String>,
    pub topics: Vec<String>,
    pub stars: u64,
    pub forks: u64,
    pub html_url: String,
}
