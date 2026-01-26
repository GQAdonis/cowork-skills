//! `cowork generate` command - Generate skills from GitHub repositories or local directories.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use colored::Colorize;
use walkdir::WalkDir;

use crate::generator::{LlmsTxtGenerator, SkillGenerator};
use crate::github::{GitHubClient, Repository};
use crate::parser::{Language, ParseResult, parse_file};

/// Options for the generate command.
pub struct GenerateOptions {
    pub repo: Option<String>,
    pub local_path: Option<PathBuf>,
    pub from_llms: Option<PathBuf>,
    pub languages: Vec<String>,
    pub output: Option<PathBuf>,
    pub llms_only: bool,
    pub split_modules: bool,
    pub agents: Vec<String>,
    pub git_ref: Option<String>,
}

/// Execute the generate command.
pub fn execute(options: GenerateOptions) -> Result<()> {
    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        if let Some(ref llms_path) = options.from_llms {
            // Generate skills from existing llms.txt
            generate_from_llms(llms_path, &options).await
        } else if let Some(ref local_path) = options.local_path {
            // Generate from local directory
            generate_from_local(local_path, &options).await
        } else if let Some(ref repo_str) = options.repo {
            // Generate from GitHub repository
            generate_from_github(repo_str, &options).await
        } else {
            anyhow::bail!("Either a repository, --path, or --from-llms must be specified")
        }
    })
}

/// Generate skills from a GitHub repository.
async fn generate_from_github(repo_str: &str, options: &GenerateOptions) -> Result<()> {
    println!(
        "{} Connecting to GitHub...",
        "→".blue()
    );

    let client = GitHubClient::new().context("Failed to create GitHub client")?;
    let repo = Repository::from_url(repo_str).or_else(|_| Repository::from_full_name(repo_str))?;

    println!(
        "{} Fetching repository info for {}...",
        "→".blue(),
        repo.full_name.cyan()
    );

    let repo_info = client.fetch_repo_info(&repo).await?;
    let git_ref = options.git_ref.as_deref();

    println!(
        "{} Repository: {} ({})",
        "✓".green(),
        repo_info.full_name,
        repo_info.language.as_deref().unwrap_or("unknown")
    );

    if let Some(desc) = &repo_info.description {
        println!("  {}", desc.dimmed());
    }

    // Determine languages to parse
    let target_languages = if options.languages.is_empty() {
        // Auto-detect from repository
        match repo_info.language.as_deref() {
            Some("Rust") => vec![Language::Rust],
            Some("TypeScript") | Some("JavaScript") => vec![Language::TypeScript],
            Some("Python") => vec![Language::Python],
            _ => vec![Language::Rust, Language::TypeScript, Language::Python],
        }
    } else {
        options
            .languages
            .iter()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "rust" | "rs" => Some(Language::Rust),
                "typescript" | "ts" => Some(Language::TypeScript),
                "python" | "py" => Some(Language::Python),
                _ => None,
            })
            .collect()
    };

    println!(
        "{} Scanning for {} files...",
        "→".blue(),
        target_languages
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Get all source files
    let all_files = client.list_all_files(&repo, "", git_ref).await?;

    // Filter to source files of target languages
    let source_files: Vec<_> = all_files
        .into_iter()
        .filter(|f| {
            let ext = f.name.rsplit('.').next().unwrap_or("");
            target_languages.iter().any(|lang| {
                lang.file_extensions().contains(&ext)
            })
        })
        .collect();

    println!(
        "{} Found {} source files",
        "✓".green(),
        source_files.len()
    );

    if source_files.is_empty() {
        println!("{} No source files found for the specified languages", "⚠".yellow());
        return Ok(());
    }

    // Parse source files
    println!(
        "{} Parsing source files...",
        "→".blue()
    );

    let mut parse_results: Vec<ParseResult> = Vec::new();
    let mut parsed_count = 0;
    let mut error_count = 0;

    for file in &source_files {
        match client.fetch_file_content(&repo, &file.path, git_ref).await {
            Ok(content) => {
                match parse_file(&content, &file.path) {
                    Ok(result) => {
                        if !result.items.is_empty() {
                            parsed_count += 1;
                            parse_results.push(result);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!(
                            "  {} Failed to parse {}: {}",
                            "⚠".yellow(),
                            file.path,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!(
                    "  {} Failed to fetch {}: {}",
                    "⚠".yellow(),
                    file.path,
                    e
                );
            }
        }

        // Progress indicator
        if (parsed_count + error_count) % 10 == 0 {
            print!("\r  {} files processed...", parsed_count + error_count);
        }
    }

    println!(
        "\n{} Parsed {} files ({} errors)",
        "✓".green(),
        parsed_count,
        error_count
    );

    // Generate llms.txt
    println!(
        "{} Generating llms.txt...",
        "→".blue()
    );

    let generator = LlmsTxtGenerator::new(&repo_info.name, &repo_info.html_url)
        .with_description(repo_info.description.as_deref().unwrap_or(""));

    let llms = generator.generate(&parse_results);

    let llms_content = llms.to_markdown();

    // Determine output directory
    let output_dir = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("./generated"));

    std::fs::create_dir_all(&output_dir)?;

    // Write llms.txt
    let llms_path = output_dir.join("llms.txt");
    std::fs::write(&llms_path, &llms_content)?;

    println!(
        "{} Generated {}",
        "✓".green(),
        llms_path.display()
    );

    if options.llms_only {
        println!(
            "\n{} Done! llms.txt generated at {}",
            "✓".green().bold(),
            llms_path.display()
        );
        return Ok(());
    }

    // Generate skills
    println!(
        "{} Generating skills...",
        "→".blue()
    );

    let skill_generator = SkillGenerator::new().with_split_modules(options.split_modules);
    let skills = skill_generator.generate(&llms);

    let skills_dir = output_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    for skill in &skills {
        skill.write_to_dir(&skills_dir)?;
        println!(
            "  {} Created skill: {}",
            "✓".green(),
            skill.name.cyan()
        );
    }

    println!(
        "\n{} Done! Generated {} skills at {}",
        "✓".green().bold(),
        skills.len(),
        skills_dir.display()
    );

    // Install to agents if specified
    if !options.agents.is_empty() {
        println!(
            "\n{} Installing skills to agents: {}",
            "→".blue(),
            options.agents.join(", ")
        );

        // TODO: Integrate with existing install command
        println!(
            "  {} Use 'cowork install' to install generated skills to agents",
            "ℹ".blue()
        );
    }

    Ok(())
}

/// Generate skills from a local directory.
async fn generate_from_local(local_path: &Path, options: &GenerateOptions) -> Result<()> {
    let local_path = local_path.canonicalize()
        .context("Failed to resolve local path")?;

    if !local_path.exists() {
        anyhow::bail!("Directory does not exist: {}", local_path.display());
    }

    if !local_path.is_dir() {
        anyhow::bail!("Path is not a directory: {}", local_path.display());
    }

    let project_name = local_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    println!(
        "{} Scanning local directory: {}",
        "→".blue(),
        local_path.display()
    );

    // Determine languages to parse
    let target_languages: Vec<Language> = if options.languages.is_empty() {
        // Auto-detect: scan for common file extensions
        vec![Language::Rust, Language::TypeScript, Language::Python]
    } else {
        options
            .languages
            .iter()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "rust" | "rs" => Some(Language::Rust),
                "typescript" | "ts" => Some(Language::TypeScript),
                "python" | "py" => Some(Language::Python),
                _ => None,
            })
            .collect()
    };

    println!(
        "{} Scanning for {} files...",
        "→".blue(),
        target_languages
            .iter()
            .map(|l| l.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Collect all source files
    let mut source_files: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(&local_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden directories and common non-source directories
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "__pycache__"
                && name != "venv"
                && name != ".venv"
                && name != "build"
                && name != "dist"
        })
    {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if target_languages.iter().any(|lang| lang.file_extensions().contains(&ext)) {
                    source_files.push(path.to_path_buf());
                }
            }
        }
    }

    println!(
        "{} Found {} source files",
        "✓".green(),
        source_files.len()
    );

    if source_files.is_empty() {
        println!("{} No source files found for the specified languages", "⚠".yellow());
        return Ok(());
    }

    // Parse source files
    println!(
        "{} Parsing source files...",
        "→".blue()
    );

    let mut parse_results: Vec<ParseResult> = Vec::new();
    let mut parsed_count = 0;
    let mut error_count = 0;

    for file_path in &source_files {
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                let relative_path = file_path
                    .strip_prefix(&local_path)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                match parse_file(&content, &relative_path) {
                    Ok(result) => {
                        if !result.items.is_empty() {
                            parsed_count += 1;
                            parse_results.push(result);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        if error_count <= 5 {
                            eprintln!(
                                "  {} Failed to parse {}: {}",
                                "⚠".yellow(),
                                relative_path,
                                e
                            );
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                if error_count <= 5 {
                    eprintln!(
                        "  {} Failed to read {}: {}",
                        "⚠".yellow(),
                        file_path.display(),
                        e
                    );
                }
            }
        }

        // Progress indicator
        if (parsed_count + error_count) % 50 == 0 {
            print!("\r  {} files processed...", parsed_count + error_count);
        }
    }

    if error_count > 5 {
        eprintln!("  {} ... and {} more errors", "⚠".yellow(), error_count - 5);
    }

    println!(
        "\n{} Parsed {} files ({} errors)",
        "✓".green(),
        parsed_count,
        error_count
    );

    // Generate llms.txt
    println!(
        "{} Generating llms.txt...",
        "→".blue()
    );

    let generator = LlmsTxtGenerator::new(project_name, &format!("file://{}", local_path.display()))
        .with_description(&format!("Generated from local directory: {}", local_path.display()));

    let llms = generator.generate(&parse_results);
    let llms_content = llms.to_markdown();

    // Determine output directory
    let output_dir = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("./generated"));

    std::fs::create_dir_all(&output_dir)?;

    // Write llms.txt
    let llms_path = output_dir.join("llms.txt");
    std::fs::write(&llms_path, &llms_content)?;

    println!(
        "{} Generated {}",
        "✓".green(),
        llms_path.display()
    );

    if options.llms_only {
        println!(
            "\n{} Done! llms.txt generated at {}",
            "✓".green().bold(),
            llms_path.display()
        );
        return Ok(());
    }

    // Generate skills
    println!(
        "{} Generating skills...",
        "→".blue()
    );

    let skill_generator = SkillGenerator::new().with_split_modules(options.split_modules);
    let skills = skill_generator.generate(&llms);

    let skills_dir = output_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    for skill in &skills {
        skill.write_to_dir(&skills_dir)?;
        println!(
            "  {} Created skill: {}",
            "✓".green(),
            skill.name.cyan()
        );
    }

    println!(
        "\n{} Done! Generated {} skills at {}",
        "✓".green().bold(),
        skills.len(),
        skills_dir.display()
    );

    // Install to agents if specified
    if !options.agents.is_empty() {
        println!(
            "\n{} Installing skills to agents: {}",
            "→".blue(),
            options.agents.join(", ")
        );

        println!(
            "  {} Use 'cowork install' to install generated skills to agents",
            "ℹ".blue()
        );
    }

    Ok(())
}

/// Generate skills from an existing llms.txt file.
async fn generate_from_llms(llms_path: &PathBuf, options: &GenerateOptions) -> Result<()> {
    println!(
        "{} Reading llms.txt from {}...",
        "→".blue(),
        llms_path.display()
    );

    let content = std::fs::read_to_string(llms_path)
        .context("Failed to read llms.txt file")?;

    // Parse the llms.txt to extract structure
    // For now, we'll create a minimal LlmsTxt from the content
    // A full implementation would parse the markdown structure

    let title = extract_title(&content).unwrap_or_else(|| "Generated".to_string());

    // Create a simple LlmsTxt with the content
    let mut llms = crate::generator::LlmsTxt::new(&title, "");
    llms.description = content.lines().take(10).collect::<Vec<_>>().join("\n");

    // Determine output directory
    let output_dir = options
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("./generated"));

    std::fs::create_dir_all(&output_dir)?;

    // Generate skills
    let skill_generator = SkillGenerator::new().with_split_modules(options.split_modules);
    let skills = skill_generator.generate(&llms);

    let skills_dir = output_dir.join("skills");
    std::fs::create_dir_all(&skills_dir)?;

    for skill in &skills {
        skill.write_to_dir(&skills_dir)?;
        println!(
            "  {} Created skill: {}",
            "✓".green(),
            skill.name.cyan()
        );
    }

    println!(
        "\n{} Done! Generated {} skills at {}",
        "✓".green().bold(),
        skills.len(),
        skills_dir.display()
    );

    Ok(())
}

/// Extract title from llms.txt content.
fn extract_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("# ") {
            return Some(line[2..].trim().to_string());
        }
    }
    None
}
