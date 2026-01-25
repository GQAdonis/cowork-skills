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
---

# {title}

{overview}

---

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
    pub fn skill_header(name: &str, description: &str, title: &str, overview: &str) -> String {
        Self::SKILL_HEADER
            .replace("{name}", name)
            .replace("{description}", description)
            .replace("{title}", title)
            .replace("{overview}", overview)
    }

    /// Generate skill triggers string from keywords.
    pub fn skill_triggers(keywords: &[String]) -> String {
        if keywords.is_empty() {
            return String::new();
        }

        // Format triggers for description field
        let triggers = keywords
            .iter()
            .take(20) // Limit to first 20 keywords
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");

        format!("Triggers on: {triggers}")
    }

    /// Generate API table for SKILL.md.
    pub fn skill_api_table(
        entries: &[(String, String, String)], // (name, signature, description)
    ) -> String {
        if entries.is_empty() {
            return String::new();
        }

        let mut output = String::from("| Name | Description |\n|------|-------------|\n");

        for (name, _, description) in entries {
            let desc = description
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>();
            output.push_str(&format!("| `{name}` | {desc} |\n"));
        }

        output
    }
}
