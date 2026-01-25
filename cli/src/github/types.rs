//! GitHub API type definitions.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

/// Repository identifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
    pub full_name: String,
}

impl Repository {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        let owner = owner.into();
        let name = name.into();
        let full_name = format!("{owner}/{name}");
        Self {
            owner,
            name,
            full_name,
        }
    }

    /// Parse from URL like `https://github.com/owner/repo` or `owner/repo`.
    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let url = url
            .trim()
            .trim_end_matches('/')
            .trim_end_matches(".git");

        // Handle full GitHub URLs
        let path = if url.contains("github.com") {
            url.split("github.com/")
                .nth(1)
                .ok_or_else(|| anyhow::anyhow!("Invalid GitHub URL: {url}"))?
        } else {
            url
        };

        // Handle paths with /tree/branch/... or /blob/branch/...
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid repository format: {url}"));
        }

        Ok(Self::new(parts[0], parts[1]))
    }

    /// Parse from `owner/name` format.
    pub fn from_full_name(full_name: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid repository format. Expected 'owner/name', got: {full_name}"
            ));
        }
        Ok(Self::new(parts[0], parts[1]))
    }
}

/// GitHub file content response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubFile {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub file_type: String,
    pub content: Option<String>,
    pub encoding: Option<String>,
    pub download_url: Option<String>,
    pub html_url: Option<String>,
}

impl GitHubFile {
    /// Decode base64 content to string.
    pub fn decode_content(&self) -> Option<String> {
        if let (Some(content), Some(encoding)) = (&self.content, &self.encoding) {
            if encoding == "base64" {
                let cleaned = content.replace('\n', "");
                if let Ok(decoded) = STANDARD.decode(&cleaned) {
                    return String::from_utf8(decoded).ok();
                }
            }
        }
        None
    }
}

/// GitHub directory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubDirEntry {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: u64,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub download_url: Option<String>,
    pub html_url: Option<String>,
}

impl GitHubDirEntry {
    pub fn is_file(&self) -> bool {
        self.entry_type == "file"
    }

    pub fn is_dir(&self) -> bool {
        self.entry_type == "dir"
    }
}

/// GitHub content response - can be either a file or directory listing.
#[derive(Debug, Clone)]
pub enum GitHubContent {
    File(GitHubFile),
    Directory(Vec<GitHubDirEntry>),
}
