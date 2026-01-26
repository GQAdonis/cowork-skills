//! SKILL.md generation from llms.txt.

use std::collections::HashSet;
use std::path::Path;

use super::llms::{LlmsModule, LlmsTxt};
use super::templates::Templates;

/// A generated skill.
#[derive(Debug, Clone)]
pub struct GeneratedSkill {
    pub name: String,
    pub content: String,
    pub references: Vec<(String, String)>, // (filename, content)
}

impl GeneratedSkill {
    /// Create a new skill.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: String::new(),
            references: vec![],
        }
    }

    /// Write skill to directory.
    pub fn write_to_dir(&self, base_dir: &Path) -> anyhow::Result<()> {
        let skill_dir = base_dir.join(&self.name);
        std::fs::create_dir_all(&skill_dir)?;

        // Write SKILL.md
        std::fs::write(skill_dir.join("SKILL.md"), &self.content)?;

        // Write references
        if !self.references.is_empty() {
            let refs_dir = skill_dir.join("references");
            std::fs::create_dir_all(&refs_dir)?;

            for (filename, content) in &self.references {
                std::fs::write(refs_dir.join(filename), content)?;
            }
        }

        Ok(())
    }
}

/// Generator for SKILL.md files.
pub struct SkillGenerator {
    split_modules: bool,
    max_items_per_skill: usize,
}

impl Default for SkillGenerator {
    fn default() -> Self {
        Self {
            split_modules: true,
            max_items_per_skill: 50,
        }
    }
}

impl SkillGenerator {
    /// Create a new generator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to split modules into separate skills.
    pub fn with_split_modules(mut self, split: bool) -> Self {
        self.split_modules = split;
        self
    }

    /// Generate skills from llms.txt.
    pub fn generate(&self, llms: &LlmsTxt) -> Vec<GeneratedSkill> {
        if self.split_modules && llms.modules.len() > 1 {
            // Generate a skill for each top-level module
            llms.modules
                .iter()
                .filter(|m| !m.is_empty())
                .map(|module| self.generate_module_skill(module, llms))
                .collect()
        } else {
            // Generate a single skill for everything
            vec![self.generate_single_skill(llms)]
        }
    }

    /// Generate a single skill from the entire llms.txt.
    fn generate_single_skill(&self, llms: &LlmsTxt) -> GeneratedSkill {
        let name = slugify(&llms.title);
        let mut skill = GeneratedSkill::new(&name);

        // Collect all keywords for triggers
        let keywords = self.collect_keywords(llms);

        // Build concise description (without triggers)
        let description = if llms.description.is_empty() {
            format!("{} API reference and usage guide", llms.title)
        } else {
            // Take first sentence only, max 120 chars
            let first_sentence = llms.description
                .split(". ")
                .next()
                .unwrap_or(&llms.description)
                .trim()
                .trim_end_matches('.');
            smart_truncate(first_sentence, 120)
        };

        // Build overview
        let overview = self.build_overview(llms);

        // Build API reference
        let api_entries = self.build_api_entries(llms);
        let api_section = if api_entries.is_empty() {
            String::new()
        } else {
            format!(
                "## API Reference\n\n{}\n",
                Templates::skill_api_table(&api_entries)
            )
        };

        // Build patterns/examples section
        let patterns = self.build_patterns(llms);

        // Assemble content
        let mut content = Templates::skill_header(
            &name,
            &description,
            &llms.title,
            &overview,
            &keywords,
            llms.language.as_str(),
        );
        content.push_str(&api_section);

        if !patterns.is_empty() {
            content.push_str("## Common Patterns\n\n");
            content.push_str(&patterns);
        }

        // Add related skills section
        content.push_str("\n## Related\n\n");
        content.push_str(&format!("- Source: {}\n", llms.source_url));
        if let Some(version) = &llms.version {
            content.push_str(&format!("- Version: {version}\n"));
        }

        skill.content = content;

        // Generate reference files for detailed API docs
        if llms.modules.len() <= 3 {
            for module in &llms.modules {
                let ref_content = module.to_markdown(llms.language);
                if !ref_content.is_empty() {
                    skill
                        .references
                        .push((format!("{}.md", slugify(&module.name)), ref_content));
                }
            }
        }

        skill
    }

    /// Generate a skill for a specific module.
    fn generate_module_skill(&self, module: &LlmsModule, llms: &LlmsTxt) -> GeneratedSkill {
        let name = format!("{}-{}", slugify(&llms.title), slugify(&module.name));
        let mut skill = GeneratedSkill::new(&name);

        // Collect keywords from this module
        let keywords = module.get_keywords();

        let title = format!("{} - {}", llms.title, module.name);

        // Build concise description
        let description = module
            .doc_comment
            .as_ref()
            .map(|d| {
                let first = d.split(". ").next().unwrap_or(d).trim().trim_end_matches('.');
                smart_truncate(first, 120)
            })
            .unwrap_or_else(|| format!("{} module reference", module.name));

        // Build overview from module doc
        let overview = module
            .doc_comment
            .clone()
            .unwrap_or_else(|| format!("Module `{}` from {}", module.name, llms.title));

        // Build API entries from this module
        let api_entries = self.build_module_api_entries(module);
        let api_section = if api_entries.is_empty() {
            String::new()
        } else {
            format!(
                "## API Reference\n\n{}\n",
                Templates::skill_api_table(&api_entries)
            )
        };

        // Assemble content
        let mut content = Templates::skill_header(
            &name,
            &description,
            &title,
            &overview,
            &keywords,
            llms.language.as_str(),
        );
        content.push_str(&api_section);

        // Add detailed API documentation
        content.push_str(&module.to_markdown(llms.language));

        // Add related section
        content.push_str("\n## Related\n\n");
        content.push_str(&format!("- Source: {}\n", llms.source_url));
        content.push_str(&format!("- Parent: {}\n", llms.title));

        skill.content = content;
        skill
    }

    /// Collect all keywords for skill triggers.
    fn collect_keywords(&self, llms: &LlmsTxt) -> Vec<String> {
        let mut keywords: HashSet<String> = HashSet::new();

        // Add title words
        for word in llms.title.split_whitespace() {
            let word = word.to_lowercase();
            if word.len() > 2 {
                keywords.insert(word);
            }
        }

        // Add module names and item names
        for module in &llms.modules {
            keywords.extend(module.get_keywords());
        }

        let mut result: Vec<String> = keywords.into_iter().collect();
        result.sort();
        result
    }

    /// Build overview section.
    fn build_overview(&self, llms: &LlmsTxt) -> String {
        let mut overview = llms.description.clone();

        // Only show modules section if there are meaningful modules (not just "root")
        let meaningful_modules: Vec<_> = llms
            .modules
            .iter()
            .filter(|m| m.name != "root" || m.doc_comment.is_some())
            .collect();

        if !meaningful_modules.is_empty() {
            overview.push_str("\n\n## Modules\n\n");
            for module in meaningful_modules {
                let doc = module
                    .doc_comment
                    .as_deref()
                    .and_then(|d| {
                        let first_line = d.lines().next().unwrap_or("");
                        if first_line.is_empty() { None } else { Some(first_line) }
                    })
                    .unwrap_or_else(|| {
                        // Generate description based on module contents
                        self.summarize_module(module)
                    });
                overview.push_str(&format!("- **{}**: {}\n", module.name, doc));
            }
        }

        overview
    }

    /// Generate a summary for a module based on its contents.
    fn summarize_module(&self, module: &LlmsModule) -> &'static str {
        let type_count = module.types.len();
        let func_count = module.functions.len();
        let trait_count = module.traits.len();

        if type_count > 0 && func_count > 0 && trait_count > 0 {
            "Types, functions, and protocols"
        } else if type_count > 0 && func_count > 0 {
            "Core types and functions"
        } else if type_count > 0 && trait_count > 0 {
            "Types and protocols"
        } else if type_count > 5 {
            "Type definitions"
        } else if func_count > 5 {
            "Utility functions"
        } else if trait_count > 0 {
            "Protocol definitions"
        } else {
            "Core module"
        }
    }

    /// Build API entries for table.
    fn build_api_entries(&self, llms: &LlmsTxt) -> Vec<(String, String, String)> {
        let mut entries = Vec::new();

        for module in &llms.modules {
            entries.extend(self.build_module_api_entries(module));
        }

        // Limit to prevent huge tables
        entries.truncate(self.max_items_per_skill);
        entries
    }

    /// Build API entries from a module.
    fn build_module_api_entries(&self, module: &LlmsModule) -> Vec<(String, String, String)> {
        let mut entries = Vec::new();

        for entry in &module.types {
            entries.push((
                entry.name.clone(),
                entry.signature.clone(),
                entry.doc_comment.clone().unwrap_or_default(),
            ));
        }

        for entry in &module.traits {
            entries.push((
                entry.name.clone(),
                entry.signature.clone(),
                entry.doc_comment.clone().unwrap_or_default(),
            ));
        }

        for entry in &module.functions {
            entries.push((
                entry.name.clone(),
                entry.signature.clone(),
                entry.doc_comment.clone().unwrap_or_default(),
            ));
        }

        for submodule in &module.submodules {
            entries.extend(self.build_module_api_entries(submodule));
        }

        entries
    }

    /// Build patterns section from examples.
    fn build_patterns(&self, llms: &LlmsTxt) -> String {
        let mut patterns = String::new();

        for example in &llms.examples {
            patterns.push_str(&format!("### {}\n\n", example.title));
            patterns.push_str(&format!(
                "```{}\n{}\n```\n\n",
                llms.language.as_str(),
                example.code
            ));
        }

        patterns
    }
}

/// Smart truncation that avoids cutting in the middle of markdown links.
fn smart_truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    let truncated = &s[..max_len];

    // Check if we're in the middle of a markdown link [text](url)
    let last_open_bracket = truncated.rfind('[');
    let last_close_bracket = truncated.rfind(']');
    let last_open_paren = truncated.rfind('(');

    let break_point = match (last_open_bracket, last_close_bracket, last_open_paren) {
        // In the middle of [text] part
        (Some(open), close, _) if close.map_or(true, |c| c < open) => open,
        // In the middle of (url) part after ]
        (_, Some(close), Some(paren)) if paren > close => close + 1,
        _ => max_len,
    };

    // Find word boundary
    let final_break = s[..break_point].rfind(' ').unwrap_or(break_point);

    format!("{}...", s[..final_break].trim())
}

/// Convert a string to a URL-safe slug.
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("tokio-rs/tokio"), "tokio-rs-tokio");
        assert_eq!(slugify("My Cool  Library"), "my-cool-library");
    }
}
