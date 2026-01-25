//! `cowork search` command - Search GitHub for skill repositories.

use anyhow::{Context, Result};
use colored::Colorize;

use crate::github::{GitHubClient, Repository, search_repos, search_repos_by_topic};

/// Options for the search command.
pub struct SearchOptions {
    pub query: String,
    pub topic: bool,
    pub limit: usize,
    pub verbose: bool,
}

/// Repository type detected from name/description
#[derive(Debug, Clone, Copy, PartialEq)]
enum RepoType {
    SkillsRepo,    // Contains skills (install directly)
    SourceRepo,    // Source code (generate skills from it)
}

/// Detect if a repository is a skills repository based on name and description
fn detect_repo_type(full_name: &str, description: Option<&str>) -> RepoType {
    let name = full_name.split('/').last().unwrap_or(full_name).to_lowercase();

    // Check name patterns
    let name_is_skills = name.ends_with("-skills")
        || name.ends_with("-skill")
        || name.contains("skills-")
        || name == "skills";

    // Check description for skill-related keywords
    let desc_is_skills = description.map(|d| {
        let d = d.to_lowercase();
        d.contains("skill") && (d.contains("claude") || d.contains("agent") || d.contains("llm"))
    }).unwrap_or(false);

    if name_is_skills || desc_is_skills {
        RepoType::SkillsRepo
    } else {
        RepoType::SourceRepo
    }
}

/// Check if repository has skills/ directory
async fn has_skills_directory(client: &GitHubClient, repo: &Repository) -> bool {
    // Try to list the skills/ directory
    match client.list_directory(repo, "skills", None).await {
        Ok(entries) => !entries.is_empty(),
        Err(_) => false,
    }
}

/// Execute the search command.
pub fn execute(options: SearchOptions) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        println!(
            "{} Connecting to GitHub...",
            "→".blue()
        );

        let client = GitHubClient::new().context(
            "Failed to create GitHub client. Make sure GITHUB_TOKEN is set."
        )?;

        println!(
            "{} Searching for '{}'...\n",
            "→".blue(),
            options.query.cyan()
        );

        let results = if options.topic {
            search_repos_by_topic(&client, &options.query, options.limit).await?
        } else {
            search_repos(&client, &options.query, options.limit).await?
        };

        if results.is_empty() {
            println!("{} No repositories found", "⚠".yellow());
            return Ok(());
        }

        println!(
            "{} Found {} repositories:\n",
            "✓".green(),
            results.len()
        );

        // Categorize repositories
        let mut skills_repos = Vec::new();
        let mut source_repos = Vec::new();

        for repo in &results {
            let mut repo_type = detect_repo_type(&repo.full_name, repo.description.as_deref());

            // For repos that look like skills repos, verify by checking for skills/ directory
            if repo_type == RepoType::SkillsRepo {
                let parsed_repo = Repository::from_full_name(&repo.full_name).ok();
                if let Some(ref pr) = parsed_repo {
                    if !has_skills_directory(&client, pr).await {
                        // Has skill-like name but no skills/ directory - treat as source
                        repo_type = RepoType::SourceRepo;
                    }
                }
            } else {
                // For source repos, also check if they happen to have a skills/ directory
                let parsed_repo = Repository::from_full_name(&repo.full_name).ok();
                if let Some(ref pr) = parsed_repo {
                    if has_skills_directory(&client, pr).await {
                        repo_type = RepoType::SkillsRepo;
                    }
                }
            }

            match repo_type {
                RepoType::SkillsRepo => skills_repos.push(repo),
                RepoType::SourceRepo => source_repos.push(repo),
            }
        }

        // Display results with type indicators
        let mut index = 1;

        for repo in &results {
            let is_skills = skills_repos.iter().any(|r| r.full_name == repo.full_name);
            let type_badge = if is_skills {
                "[skills]".green().bold()
            } else {
                "[source]".blue()
            };

            println!(
                "{}. {} {} {}",
                index.to_string().dimmed(),
                repo.full_name.cyan().bold(),
                format!("⭐ {}", repo.stars).yellow(),
                type_badge
            );

            if let Some(desc) = &repo.description {
                println!("   {}", desc);
            }

            if options.verbose {
                if let Some(lang) = &repo.language {
                    println!("   {}: {}", "Language".dimmed(), lang);
                }

                if !repo.topics.is_empty() {
                    println!(
                        "   {}: {}",
                        "Topics".dimmed(),
                        repo.topics.join(", ")
                    );
                }

                println!("   {}: {}", "URL".dimmed(), repo.html_url);
                println!("   {}: {}", "Updated".dimmed(), repo.updated_at);
            }

            println!();
            index += 1;
        }

        // Show recommendations based on repo types found
        if !skills_repos.is_empty() {
            println!(
                "{} {} - Install skills directly:",
                "ℹ".green(),
                "[skills]".green().bold()
            );
            for repo in skills_repos.iter().take(2) {
                println!(
                    "   cowork install {}",
                    repo.full_name.cyan()
                );
            }
        }

        if !source_repos.is_empty() {
            if !skills_repos.is_empty() {
                println!();
            }
            println!(
                "{} {} - Generate skills from source code:",
                "ℹ".blue(),
                "[source]".blue()
            );
            for repo in source_repos.iter().take(2) {
                println!(
                    "   cowork generate {}",
                    repo.full_name.cyan()
                );
            }
        }

        Ok(())
    })
}

/// Search for agent-skill repositories specifically.
#[allow(dead_code)]
pub fn search_agent_skills(limit: usize) -> Result<()> {
    let options = SearchOptions {
        query: "agent-skill".to_string(),
        topic: true,
        limit,
        verbose: true,
    };

    execute(options)
}
