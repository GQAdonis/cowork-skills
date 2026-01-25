//! llms.txt generation from parsed source code.
//!
//! Generates documentation in the llms.txt format:
//! https://llmstxt.org/

use crate::parser::{InterfaceItem, ItemKind, Language, ModuleInfo, ParseResult};

use super::templates::Templates;

/// Complete llms.txt document.
#[derive(Debug, Clone)]
pub struct LlmsTxt {
    pub title: String,
    pub description: String,
    pub version: Option<String>,
    pub source_url: String,
    pub language: Language,
    pub modules: Vec<LlmsModule>,
    pub examples: Vec<LlmsExample>,
}

impl LlmsTxt {
    /// Create a new llms.txt document.
    pub fn new(title: impl Into<String>, source_url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: String::new(),
            version: None,
            source_url: source_url.into(),
            language: Language::Unknown,
            modules: vec![],
            examples: vec![],
        }
    }

    /// Convert to markdown string.
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&Templates::llms_header(
            &self.title,
            &self.description,
            &self.source_url,
            self.version.as_deref().unwrap_or("unknown"),
            self.language.as_str(),
        ));

        // Overview section if we have a description
        if !self.description.is_empty() {
            output.push_str("## Overview\n\n");
            output.push_str(&self.description);
            output.push_str("\n\n---\n\n");
        }

        // Modules
        for module in &self.modules {
            output.push_str(&module.to_markdown(self.language));
        }

        // Examples
        if !self.examples.is_empty() {
            output.push_str("## Examples\n\n");
            for example in &self.examples {
                output.push_str(&Templates::llms_example(
                    &example.title,
                    &example.code,
                    self.language.as_str(),
                ));
            }
        }

        output
    }
}

/// A module section in llms.txt.
#[derive(Debug, Clone)]
pub struct LlmsModule {
    pub name: String,
    pub path: Vec<String>,
    pub doc_comment: Option<String>,
    pub functions: Vec<LlmsEntry>,
    pub types: Vec<LlmsEntry>,
    pub traits: Vec<LlmsEntry>,
    pub constants: Vec<LlmsEntry>,
    pub submodules: Vec<LlmsModule>,
}

impl LlmsModule {
    /// Create a new module.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: vec![],
            doc_comment: None,
            functions: vec![],
            types: vec![],
            traits: vec![],
            constants: vec![],
            submodules: vec![],
        }
    }

    /// Convert to markdown.
    pub fn to_markdown(&self, language: Language) -> String {
        let mut output = String::new();

        let module_name = if self.path.is_empty() {
            self.name.clone()
        } else {
            format!("{}::{}", self.path.join("::"), self.name)
        };

        output.push_str(&Templates::llms_module(
            &module_name,
            self.doc_comment.as_deref().unwrap_or(""),
        ));

        // Types (structs, enums)
        if !self.types.is_empty() {
            output.push_str("### Types\n\n");
            for entry in &self.types {
                output.push_str(&entry.to_markdown(language));
            }
        }

        // Traits
        if !self.traits.is_empty() {
            output.push_str("### Traits\n\n");
            for entry in &self.traits {
                output.push_str(&entry.to_markdown(language));
            }
        }

        // Functions
        if !self.functions.is_empty() {
            output.push_str("### Functions\n\n");
            for entry in &self.functions {
                output.push_str(&entry.to_markdown(language));
            }
        }

        // Constants
        if !self.constants.is_empty() {
            output.push_str("### Constants\n\n");
            for entry in &self.constants {
                output.push_str(&entry.to_markdown(language));
            }
        }

        // Submodules
        for submodule in &self.submodules {
            output.push_str(&submodule.to_markdown(language));
        }

        output.push_str("---\n\n");
        output
    }

    /// Check if module has any content.
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
            && self.types.is_empty()
            && self.traits.is_empty()
            && self.constants.is_empty()
            && self.submodules.iter().all(LlmsModule::is_empty)
    }

    /// Get all keywords from this module for skill triggers.
    pub fn get_keywords(&self) -> Vec<String> {
        let mut keywords = vec![self.name.clone()];

        for entry in &self.functions {
            keywords.push(entry.name.clone());
        }
        for entry in &self.types {
            keywords.push(entry.name.clone());
        }
        for entry in &self.traits {
            keywords.push(entry.name.clone());
        }

        for submodule in &self.submodules {
            keywords.extend(submodule.get_keywords());
        }

        keywords
    }
}

/// An entry in the llms.txt (function, type, etc.).
#[derive(Debug, Clone)]
pub struct LlmsEntry {
    pub name: String,
    pub kind: ItemKind,
    pub signature: String,
    pub doc_comment: Option<String>,
    pub fields: Vec<(String, String)>, // (name, type)
    pub methods: Vec<LlmsEntry>,
}

impl LlmsEntry {
    /// Create a new entry.
    pub fn new(name: impl Into<String>, kind: ItemKind) -> Self {
        Self {
            name: name.into(),
            kind,
            signature: String::new(),
            doc_comment: None,
            fields: vec![],
            methods: vec![],
        }
    }

    /// Convert to markdown.
    pub fn to_markdown(&self, language: Language) -> String {
        let doc = self.doc_comment.as_deref().unwrap_or("");

        match self.kind {
            ItemKind::Function => {
                Templates::llms_function(&self.name, &self.signature, doc, language.as_str())
            }
            ItemKind::Struct | ItemKind::Class | ItemKind::Enum => {
                let fields = if self.fields.is_empty() {
                    String::new()
                } else {
                    let field_list: Vec<String> = self
                        .fields
                        .iter()
                        .map(|(name, ty)| format!("- `{name}`: `{ty}`"))
                        .collect();
                    format!("**Fields:**\n{}\n", field_list.join("\n"))
                };

                let mut output =
                    Templates::llms_type(&self.name, &self.signature, doc, &fields, language.as_str());

                if !self.methods.is_empty() {
                    output.push_str("**Methods:**\n\n");
                    for method in &self.methods {
                        output.push_str(&format!(
                            "- `{}`: {}\n",
                            method.name,
                            method.doc_comment.as_deref().unwrap_or("")
                        ));
                    }
                    output.push('\n');
                }

                output
            }
            ItemKind::Trait | ItemKind::Interface => {
                let mut output = Templates::llms_type(
                    &self.name,
                    &self.signature,
                    doc,
                    "",
                    language.as_str(),
                );

                if !self.methods.is_empty() {
                    output.push_str("**Required Methods:**\n\n");
                    for method in &self.methods {
                        output.push_str(&format!("```{}\n{}\n```\n\n", language.as_str(), method.signature));
                        if let Some(method_doc) = &method.doc_comment {
                            output.push_str(method_doc);
                            output.push_str("\n\n");
                        }
                    }
                }

                output
            }
            _ => Templates::llms_function(&self.name, &self.signature, doc, language.as_str()),
        }
    }
}

/// An example in the llms.txt.
#[derive(Debug, Clone)]
pub struct LlmsExample {
    pub title: String,
    pub code: String,
}

/// Generator for llms.txt files.
pub struct LlmsTxtGenerator {
    title: String,
    description: String,
    source_url: String,
    version: Option<String>,
}

impl LlmsTxtGenerator {
    /// Create a new generator.
    pub fn new(title: impl Into<String>, source_url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: String::new(),
            source_url: source_url.into(),
            version: None,
        }
    }

    /// Set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set version.
    #[allow(dead_code)]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Generate llms.txt from parse results.
    pub fn generate(&self, results: &[ParseResult]) -> LlmsTxt {
        let mut llms = LlmsTxt::new(&self.title, &self.source_url);
        llms.description = self.description.clone();
        llms.version = self.version.clone();

        // Determine primary language
        let language = results
            .iter()
            .map(|r| r.language)
            .find(|l| *l != Language::Unknown)
            .unwrap_or(Language::Unknown);
        llms.language = language;

        // Build module tree from all parse results
        let mut root_module = LlmsModule::new("root");

        for result in results {
            if let Some(module) = &result.module {
                merge_module(&mut root_module, module);
            } else {
                // Add items directly to root
                for item in &result.items {
                    add_item_to_module(&mut root_module, item);
                }
            }
        }

        // Convert module tree to llms.txt modules
        if !root_module.is_empty() {
            // If only one submodule, use it as root
            if root_module.functions.is_empty()
                && root_module.types.is_empty()
                && root_module.traits.is_empty()
                && root_module.constants.is_empty()
                && root_module.submodules.len() == 1
            {
                llms.modules = root_module.submodules;
            } else {
                llms.modules = vec![root_module];
            }
        }

        // Collect examples from all modules
        llms.examples = collect_examples(results);

        llms
    }
}

/// Merge a parsed module into an llms module.
fn merge_module(target: &mut LlmsModule, source: &ModuleInfo) {
    // Find or create submodule
    let submodule = if target.name == source.name {
        target
    } else {
        let existing = target.submodules.iter_mut().find(|m| m.name == source.name);
        if let Some(m) = existing {
            m
        } else {
            let mut new_module = LlmsModule::new(&source.name);
            new_module.path = source.path.clone();
            new_module.doc_comment = source.doc_comment.clone();
            target.submodules.push(new_module);
            target.submodules.last_mut().unwrap()
        }
    };

    // Add items
    for item in &source.items {
        add_item_to_module(submodule, item);
    }

    // Recursively add submodules
    for child in &source.submodules {
        merge_module(submodule, child);
    }
}

/// Add an interface item to an llms module.
fn add_item_to_module(module: &mut LlmsModule, item: &InterfaceItem) {
    if !item.visibility.is_public() {
        return;
    }

    let entry = item_to_entry(item);

    match item.kind {
        ItemKind::Function => module.functions.push(entry),
        ItemKind::Struct | ItemKind::Class | ItemKind::Enum => module.types.push(entry),
        ItemKind::Trait | ItemKind::Interface => module.traits.push(entry),
        ItemKind::Constant | ItemKind::Static => module.constants.push(entry),
        ItemKind::Module => {
            // Handle module as submodule
            let mut submodule = LlmsModule::new(&item.name);
            submodule.doc_comment = item.doc_comment.clone();
            for method in &item.methods {
                add_item_to_module(&mut submodule, method);
            }
            if !submodule.is_empty() {
                module.submodules.push(submodule);
            }
        }
        ItemKind::TraitImpl => {
            // Add impl methods to functions
            for method in &item.methods {
                module.functions.push(item_to_entry(method));
            }
        }
        _ => {}
    }
}

/// Convert an interface item to an llms entry.
fn item_to_entry(item: &InterfaceItem) -> LlmsEntry {
    let mut entry = LlmsEntry::new(&item.name, item.kind);
    entry.signature = item.signature.clone();
    entry.doc_comment = item.doc_comment.clone();

    // Add fields
    for field in &item.fields {
        if field.visibility.is_public() {
            entry
                .fields
                .push((field.name.clone(), field.type_annotation.clone()));
        }
    }

    // Add methods
    for method in &item.methods {
        entry.methods.push(item_to_entry(method));
    }

    entry
}

/// Collect examples from all parse results.
fn collect_examples(results: &[ParseResult]) -> Vec<LlmsExample> {
    let mut examples = Vec::new();

    for result in results {
        for item in &result.items {
            for example in &item.examples {
                examples.push(LlmsExample {
                    title: example.title.clone().unwrap_or_else(|| {
                        format!("{} - Example", item.name)
                    }),
                    code: example.code.clone(),
                });
            }
        }
    }

    examples
}
