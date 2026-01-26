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
            Some("Swift") => vec![Language::Swift],
            _ => vec![Language::Rust, Language::TypeScript, Language::Python, Language::Swift],
        }
    } else {
        options
            .languages
            .iter()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "rust" | "rs" => Some(Language::Rust),
                "typescript" | "ts" => Some(Language::TypeScript),
                "python" | "py" => Some(Language::Python),
                "swift" => Some(Language::Swift),
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
        vec![Language::Rust, Language::TypeScript, Language::Python, Language::Swift]
    } else {
        options
            .languages
            .iter()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "rust" | "rs" => Some(Language::Rust),
                "typescript" | "ts" => Some(Language::TypeScript),
                "python" | "py" => Some(Language::Python),
                "swift" => Some(Language::Swift),
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

    // Try to extract a meaningful description from the project
    let description = extract_project_description(&local_path, project_name);

    let generator = LlmsTxtGenerator::new(project_name, &format!("file://{}", local_path.display()))
        .with_description(&description);

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

/// Extract project description from README.md or package manifest.
fn extract_project_description(project_path: &Path, project_name: &str) -> String {
    // Try README.md first
    for readme_name in &["README.md", "readme.md", "README", "readme.txt"] {
        let readme_path = project_path.join(readme_name);
        if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                if let Some(desc) = extract_description_from_readme(&content) {
                    return desc;
                }
            }
        }
    }

    // Try Cargo.toml
    let cargo_path = project_path.join("Cargo.toml");
    if cargo_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            if let Some(desc) = extract_description_from_cargo(&content) {
                return desc;
            }
        }
    }

    // Try package.json
    let package_json = project_path.join("package.json");
    if package_json.exists() {
        if let Ok(content) = std::fs::read_to_string(&package_json) {
            if let Some(desc) = extract_description_from_package_json(&content) {
                return desc;
            }
        }
    }

    // Try Package.swift
    let package_swift = project_path.join("Package.swift");
    if package_swift.exists() {
        // Swift packages don't typically have descriptions in Package.swift
        // Use a reasonable default
        return format!("{} Swift package", project_name);
    }

    // Default fallback
    format!("{} API reference and usage guide", project_name)
}

/// Extract description from README content.
fn extract_description_from_readme(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    // Skip the title (first # line) and look for the first meaningful paragraph
    let mut found_title = false;
    let mut description_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines at the start
        if trimmed.is_empty() && !found_title {
            continue;
        }

        // Found the title
        if trimmed.starts_with("# ") && !found_title {
            found_title = true;
            continue;
        }

        // Skip badges and images
        if trimmed.starts_with("[![") || trimmed.starts_with("![") {
            continue;
        }

        // Skip empty lines between title and description
        if trimmed.is_empty() && found_title && description_lines.is_empty() {
            continue;
        }

        // Stop at headers or empty lines after we have content
        if (trimmed.starts_with('#') || trimmed.is_empty()) && !description_lines.is_empty() {
            break;
        }

        // Collect description lines
        if found_title && !trimmed.is_empty() {
            description_lines.push(trimmed);
            // Limit to first sentence or 150 chars
            let current = description_lines.join(" ");
            if current.len() > 150 || current.contains(". ") {
                break;
            }
        }
    }

    if description_lines.is_empty() {
        return None;
    }

    let desc = description_lines.join(" ");
    // Take first sentence, max 150 chars
    let first_sentence = desc.split(". ").next().unwrap_or(&desc);

    // Smart truncation: don't cut in the middle of markdown links
    let result = if first_sentence.len() > 150 {
        // Try to find a good break point before 150 chars
        let truncated = &first_sentence[..150];
        // Check if we're in the middle of a markdown link
        let last_open_bracket = truncated.rfind('[');
        let last_close_bracket = truncated.rfind(']');

        let break_point = if let (Some(open), close) = (last_open_bracket, last_close_bracket) {
            // If we have an unclosed bracket, truncate before it
            if close.map_or(true, |c| c < open) {
                open
            } else {
                150
            }
        } else {
            150
        };

        // Find word boundary
        let final_break = first_sentence[..break_point]
            .rfind(' ')
            .unwrap_or(break_point);

        format!("{}...", &first_sentence[..final_break].trim())
    } else {
        first_sentence.to_string()
    };

    Some(result)
}

/// Extract description from Cargo.toml.
fn extract_description_from_cargo(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("description") {
            // Parse: description = "..."
            if let Some(start) = line.find('"') {
                if let Some(end) = line.rfind('"') {
                    if end > start {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract description from package.json.
fn extract_description_from_package_json(content: &str) -> Option<String> {
    // Simple extraction without full JSON parsing
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("\"description\"") {
            // Parse: "description": "..."
            if let Some(colon_pos) = line.find(':') {
                let value_part = line[colon_pos + 1..].trim();
                if let Some(start) = value_part.find('"') {
                    let rest = &value_part[start + 1..];
                    if let Some(end) = rest.find('"') {
                        return Some(rest[..end].to_string());
                    }
                }
            }
        }
    }
    None
}
