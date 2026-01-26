//! Multi-language source code parser module.
//!
//! Extracts public interfaces from source files:
//! - Rust: using `syn` crate
//! - TypeScript: using `tree-sitter-typescript`
//! - Python: using `tree-sitter-python`
//! - Swift: using regex-based extraction

mod interface;
mod python;
mod rust;
mod swift;
mod typescript;

pub use interface::{InterfaceItem, ItemKind, Language, ModuleInfo, ParseResult};
pub use python::PythonParser;
pub use rust::RustParser;
pub use swift::SwiftParser;
pub use typescript::TypeScriptParser;

use anyhow::Result;

/// Parser trait for extracting public interfaces from source code.
#[allow(dead_code)]
pub trait Parser {
    /// Parse source code and extract public interfaces.
    fn parse(&self, source: &str, file_path: &str) -> Result<ParseResult>;

    /// Get the language this parser handles.
    fn language(&self) -> Language;

    /// Check if this parser can handle the given file extension.
    fn can_parse(&self, file_path: &str) -> bool;
}

/// Parse a file based on its extension.
pub fn parse_file(source: &str, file_path: &str) -> Result<ParseResult> {
    let ext = file_path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" => RustParser.parse(source, file_path),
        "ts" | "tsx" => TypeScriptParser.parse(source, file_path),
        "py" => PythonParser.parse(source, file_path),
        "swift" => SwiftParser.parse(source, file_path),
        _ => Ok(ParseResult {
            language: Language::Unknown,
            module: None,
            items: vec![],
            errors: vec![format!("Unsupported file extension: {ext}")],
        }),
    }
}

/// Detect the primary language of a repository based on file extensions.
#[allow(dead_code)]
pub fn detect_languages(files: &[String]) -> Vec<Language> {
    let mut rust_count = 0;
    let mut ts_count = 0;
    let mut py_count = 0;
    let mut swift_count = 0;

    for file in files {
        let ext = file.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "rs" => rust_count += 1,
            "ts" | "tsx" => ts_count += 1,
            "py" => py_count += 1,
            "swift" => swift_count += 1,
            _ => {}
        }
    }

    let mut languages = Vec::new();
    if rust_count > 0 {
        languages.push(Language::Rust);
    }
    if ts_count > 0 {
        languages.push(Language::TypeScript);
    }
    if py_count > 0 {
        languages.push(Language::Python);
    }
    if swift_count > 0 {
        languages.push(Language::Swift);
    }

    // Sort by count (most common first)
    languages.sort_by(|a, b| {
        let count_a = match a {
            Language::Rust => rust_count,
            Language::TypeScript => ts_count,
            Language::Python => py_count,
            Language::Swift => swift_count,
            Language::Unknown => 0,
        };
        let count_b = match b {
            Language::Rust => rust_count,
            Language::TypeScript => ts_count,
            Language::Python => py_count,
            Language::Swift => swift_count,
            Language::Unknown => 0,
        };
        count_b.cmp(&count_a)
    });

    languages
}
