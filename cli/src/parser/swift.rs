//! Swift source code parser using regex-based extraction.
//!
//! Extracts public interfaces from Swift source files:
//! - Classes, structs, enums, protocols
//! - Functions and methods
//! - Properties

use anyhow::Result;
use regex::Regex;

use super::interface::{InterfaceItem, ItemKind, Language, ParseResult, Visibility};
use super::Parser;

/// Swift parser implementation.
pub struct SwiftParser;

impl Parser for SwiftParser {
    fn parse(&self, source: &str, file_path: &str) -> Result<ParseResult> {
        SwiftParser.parse(source, file_path)
    }

    fn language(&self) -> Language {
        Language::Swift
    }

    fn can_parse(&self, file_path: &str) -> bool {
        file_path.ends_with(".swift")
    }
}

impl SwiftParser {
    pub fn parse(&self, source: &str, file_path: &str) -> Result<ParseResult> {
        let mut result = ParseResult::new(Language::Swift);
        let lines: Vec<&str> = source.lines().collect();

        // Parse classes
        let class_re = Regex::new(r"(?m)^(?:public\s+|open\s+)?(?:final\s+)?class\s+(\w+)(?:<[^>]+>)?(?:\s*:\s*[^{]+)?\s*\{").unwrap();
        for cap in class_re.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let line_num = find_line_number(source, cap.get(0).map(|m| m.start()).unwrap_or(0));
            let doc = get_doc_comment(&lines, line_num);

            let visibility = if full_match.contains("public") || full_match.contains("open") {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let mut item = InterfaceItem::new(ItemKind::Class, name)
                .with_visibility(visibility)
                .with_signature(full_match.trim_end_matches('{').trim().to_string())
                .with_source(file_path, Some(line_num + 1));
            item.doc_comment = doc;
            result.items.push(item);
        }

        // Parse structs
        let struct_re = Regex::new(r"(?m)^(?:public\s+)?struct\s+(\w+)(?:<[^>]+>)?(?:\s*:\s*[^{]+)?\s*\{").unwrap();
        for cap in struct_re.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let line_num = find_line_number(source, cap.get(0).map(|m| m.start()).unwrap_or(0));
            let doc = get_doc_comment(&lines, line_num);

            let visibility = if full_match.contains("public") {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let mut item = InterfaceItem::new(ItemKind::Struct, name)
                .with_visibility(visibility)
                .with_signature(full_match.trim_end_matches('{').trim().to_string())
                .with_source(file_path, Some(line_num + 1));
            item.doc_comment = doc;
            result.items.push(item);
        }

        // Parse enums
        let enum_re = Regex::new(r"(?m)^(?:public\s+)?enum\s+(\w+)(?:<[^>]+>)?(?:\s*:\s*[^{]+)?\s*\{").unwrap();
        for cap in enum_re.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let line_num = find_line_number(source, cap.get(0).map(|m| m.start()).unwrap_or(0));
            let doc = get_doc_comment(&lines, line_num);

            let visibility = if full_match.contains("public") {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let mut item = InterfaceItem::new(ItemKind::Enum, name)
                .with_visibility(visibility)
                .with_signature(full_match.trim_end_matches('{').trim().to_string())
                .with_source(file_path, Some(line_num + 1));
            item.doc_comment = doc;
            result.items.push(item);
        }

        // Parse protocols
        let protocol_re = Regex::new(r"(?m)^(?:public\s+)?protocol\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{").unwrap();
        for cap in protocol_re.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let line_num = find_line_number(source, cap.get(0).map(|m| m.start()).unwrap_or(0));
            let doc = get_doc_comment(&lines, line_num);

            let visibility = if full_match.contains("public") {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let mut item = InterfaceItem::new(ItemKind::Interface, name)
                .with_visibility(visibility)
                .with_signature(full_match.trim_end_matches('{').trim().to_string())
                .with_source(file_path, Some(line_num + 1));
            item.doc_comment = doc;
            result.items.push(item);
        }

        // Parse functions
        let func_re = Regex::new(r"(?m)^[ \t]*(?:public\s+|open\s+|@objc\s+|@discardableResult\s+)*(?:static\s+|class\s+)?func\s+(\w+)\s*(?:<[^>]+>)?\s*\([^)]*\)(?:\s*(?:throws\s+)?(?:async\s+)?(?:rethrows\s+)?(?:->\s*[^\{]+)?)?\s*\{").unwrap();
        for cap in func_re.captures_iter(source) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let line_num = find_line_number(source, cap.get(0).map(|m| m.start()).unwrap_or(0));
            let doc = get_doc_comment(&lines, line_num);

            let visibility = if full_match.contains("public") || full_match.contains("open") {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let mut item = InterfaceItem::new(ItemKind::Function, name)
                .with_visibility(visibility)
                .with_signature(full_match.trim_end_matches('{').trim().to_string())
                .with_source(file_path, Some(line_num + 1));
            item.doc_comment = doc;
            result.items.push(item);
        }

        // Parse initializers
        let init_re = Regex::new(r"(?m)^[ \t]*(?:public\s+|required\s+|convenience\s+)*init\s*[?!]?\s*\([^)]*\)").unwrap();
        for cap in init_re.captures_iter(source) {
            let full_match = cap.get(0).map(|m| m.as_str()).unwrap_or("");
            let line_num = find_line_number(source, cap.get(0).map(|m| m.start()).unwrap_or(0));
            let doc = get_doc_comment(&lines, line_num);

            let visibility = if full_match.contains("public") {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let mut item = InterfaceItem::new(ItemKind::Function, "init")
                .with_visibility(visibility)
                .with_signature(full_match.trim().to_string())
                .with_source(file_path, Some(line_num + 1));
            item.doc_comment = doc;
            result.items.push(item);
        }

        Ok(result)
    }
}

/// Find line number for a byte offset in source.
fn find_line_number(source: &str, offset: usize) -> usize {
    source[..offset].chars().filter(|&c| c == '\n').count()
}

/// Get doc comment from lines preceding the given line number.
fn get_doc_comment(lines: &[&str], line_num: usize) -> Option<String> {
    if line_num == 0 {
        return None;
    }

    let mut doc_lines = Vec::new();
    let mut i = line_num.saturating_sub(1);

    loop {
        if let Some(line) = lines.get(i) {
            let trimmed = line.trim();
            if trimmed.starts_with("///") {
                doc_lines.push(trimmed.trim_start_matches('/').trim());
            } else if trimmed.is_empty() || trimmed.starts_with("//") {
                // Skip empty lines and regular comments
            } else {
                break;
            }
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    if doc_lines.is_empty() {
        None
    } else {
        doc_lines.reverse();
        Some(doc_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_class() {
        let source = r#"
/// A view model for the app
public class AppViewModel: ObservableObject {
    var name: String
}
"#;
        let result = SwiftParser.parse(source, "test.swift").unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].name, "AppViewModel");
        assert_eq!(result.items[0].kind, ItemKind::Class);
    }

    #[test]
    fn test_parse_protocol() {
        let source = r#"
public protocol Drawable {
    func draw()
}
"#;
        let result = SwiftParser.parse(source, "test.swift").unwrap();
        assert!(!result.items.is_empty());
        let protocol = result.items.iter().find(|i| i.kind == ItemKind::Interface);
        assert!(protocol.is_some());
    }
}
