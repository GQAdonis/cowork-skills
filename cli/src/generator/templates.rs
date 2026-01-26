//! Template strings for generating llms.txt and SKILL.md files.

/// Template constants for generation.
pub struct Templates;

impl Templates {
    /// llms.txt header template.
    pub const LLMS_HEADER: &'static str = r#"# {title}

> {description}

**Source:** {source_url}
**Version:** {version}
**Language:** {language}

---

"#;

    /// llms.txt module section template.
    pub const LLMS_MODULE: &'static str = r#"## {module_name}

{doc_comment}

"#;

    /// llms.txt function entry template.
    pub const LLMS_FUNCTION: &'static str = r#"### `{name}`

```{language}
{signature}
```

{doc_comment}

"#;

    /// llms.txt type entry template.
    pub const LLMS_TYPE: &'static str = r#"### `{name}`

```{language}
{signature}
```

{doc_comment}

{fields}

"#;

    /// llms.txt example template.
    pub const LLMS_EXAMPLE: &'static str = r#"### {title}

```{language}
{code}
```

"#;

    /// SKILL.md header template.
    pub const SKILL_HEADER: &'static str = r#"---
name: {name}
description: "{description}"
triggers: [{triggers}]
---

# {title}

## Overview

{overview}

## Quick Start

```{language}
// TODO: Add basic usage example
```

"#;

    /// SKILL.md API section template.
    #[allow(dead_code)]
    pub const SKILL_API_SECTION: &'static str = r#"## API Reference

{api_entries}

"#;

    /// SKILL.md patterns section template.
    #[allow(dead_code)]
    pub const SKILL_PATTERNS: &'static str = r#"## Common Patterns

{patterns}

"#;

    /// SKILL.md related skills section template.
    #[allow(dead_code)]
    pub const SKILL_RELATED: &'static str = r#"## Related Skills

| When | See |
|------|-----|
{related}

"#;

    /// Generate filled llms.txt header.
    pub fn llms_header(
        title: &str,
        description: &str,
        source_url: &str,
        version: &str,
        language: &str,
    ) -> String {
        Self::LLMS_HEADER
            .replace("{title}", title)
            .replace("{description}", description)
            .replace("{source_url}", source_url)
            .replace("{version}", version)
            .replace("{language}", language)
    }

    /// Generate filled module section.
    pub fn llms_module(module_name: &str, doc_comment: &str) -> String {
        Self::LLMS_MODULE
            .replace("{module_name}", module_name)
            .replace("{doc_comment}", doc_comment)
    }

    /// Generate filled function entry.
    pub fn llms_function(name: &str, signature: &str, doc_comment: &str, language: &str) -> String {
        Self::LLMS_FUNCTION
            .replace("{name}", name)
            .replace("{signature}", signature)
            .replace("{doc_comment}", doc_comment)
            .replace("{language}", language)
    }

    /// Generate filled type entry.
    pub fn llms_type(
        name: &str,
        signature: &str,
        doc_comment: &str,
        fields: &str,
        language: &str,
    ) -> String {
        Self::LLMS_TYPE
            .replace("{name}", name)
            .replace("{signature}", signature)
            .replace("{doc_comment}", doc_comment)
            .replace("{fields}", fields)
            .replace("{language}", language)
    }

    /// Generate filled example section.
    pub fn llms_example(title: &str, code: &str, language: &str) -> String {
        Self::LLMS_EXAMPLE
            .replace("{title}", title)
            .replace("{code}", code)
            .replace("{language}", language)
    }

    /// Generate filled SKILL.md header.
    pub fn skill_header(
        name: &str,
        description: &str,
        title: &str,
        overview: &str,
        triggers: &[String],
        language: &str,
    ) -> String {
        let triggers_str = triggers
            .iter()
            .map(|t| format!("\"{t}\""))
            .collect::<Vec<_>>()
            .join(", ");

        Self::SKILL_HEADER
            .replace("{name}", name)
            .replace("{description}", description)
            .replace("{title}", title)
            .replace("{overview}", overview)
            .replace("{triggers}", &triggers_str)
            .replace("{language}", language)
    }

    /// Generate API table for SKILL.md.
    pub fn skill_api_table(
        entries: &[(String, String, String)], // (name, signature, description)
    ) -> String {
        if entries.is_empty() {
            return String::new();
        }

        let mut output = String::from("| Name | Description |\n|------|-------------|\n");

        for (name, signature, description) in entries {
            // Use description if available, otherwise derive from signature
            let desc = if description.trim().is_empty() {
                // Try to derive description from signature
                derive_description_from_signature(name, signature)
            } else {
                description
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(60)
                    .collect::<String>()
            };
            output.push_str(&format!("| `{name}` | {desc} |\n"));
        }

        output
    }
}

/// Derive a basic description from signature when doc comment is missing.
fn derive_description_from_signature(name: &str, signature: &str) -> String {
    // Check common patterns
    if signature.contains("class ") {
        format!("{name} class")
    } else if signature.contains("struct ") {
        format!("{name} struct")
    } else if signature.contains("enum ") {
        format!("{name} enum")
    } else if signature.contains("protocol ") {
        format!("{name} protocol")
    } else if signature.contains("func ") || signature.contains("fn ") {
        // Try to extract return type for functions
        if signature.contains("->") {
            let return_type = signature
                .split("->")
                .last()
                .map(|s| s.trim().trim_end_matches('{').trim())
                .unwrap_or("");
            if !return_type.is_empty() && return_type != "Void" && return_type != "()" {
                format!("Returns {return_type}")
            } else {
                format!("{name} function")
            }
        } else {
            format!("{name} function")
        }
    } else if signature.contains("init") {
        "Initializer".to_string()
    } else if signature.contains("trait ") {
        format!("{name} trait")
    } else if signature.contains("interface ") {
        format!("{name} interface")
    } else {
        // Default: just use the name
        name.to_string()
    }
}
