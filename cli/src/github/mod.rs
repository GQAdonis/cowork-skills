//! GitHub API module for fetching repository content and searching repositories.
//!
//! This module provides:
//! - `GitHubClient` for fetching repository content
//! - `search_repos_by_topic` for searching repositories
//! - Type definitions for GitHub API responses

mod client;
mod config;
mod error;
mod search;
mod types;

pub use client::GitHubClient;
pub use search::{search_repos, search_repos_by_topic};
pub use types::Repository;
