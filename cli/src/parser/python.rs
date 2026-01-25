//! Python source code parser using tree-sitter.

use anyhow::Result;
use tree_sitter::{Node, Parser as TsParser};

use super::interface::{
    FieldInfo, InterfaceItem, ItemKind, Language, ModuleInfo, ParseResult, Visibility,
};
use super::Parser;

/// Python source code parser.
pub struct PythonParser;

impl Parser for PythonParser {
    fn parse(&self, source: &str, file_path: &str) -> Result<ParseResult> {
        let mut result = ParseResult::new(Language::Python);

        let mut parser = TsParser::new();
        let language = tree_sitter_python::LANGUAGE;
        parser
            .set_language(&language.into())
            .map_err(|e| anyhow::anyhow!("Failed to set Python language: {e}"))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python"))?;

        let root = tree.root_node();
        let source_bytes = source.as_bytes();

        let mut module = ModuleInfo::new(
            file_path
                .rsplit('/')
                .next()
                .unwrap_or(file_path)
                .trim_end_matches(".py"),
        );

        // Extract module docstring
        module.doc_comment = extract_module_docstring(&root, source_bytes);

        // Walk the AST
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            if let Some(item) = parse_node(&child, source_bytes, file_path) {
                module.items.push(item);
                result.items.push(module.items.last().unwrap().clone());
            }
        }

        result.module = Some(module);
        Ok(result)
    }

    fn language(&self) -> Language {
        Language::Python
    }

    fn can_parse(&self, file_path: &str) -> bool {
        file_path.ends_with(".py")
    }
}

fn parse_node(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    match node.kind() {
        "function_definition" => parse_function(node, source, file_path),
        "class_definition" => parse_class(node, source, file_path),
        "decorated_definition" => parse_decorated(node, source, file_path),
        "expression_statement" => {
            // Check for module-level assignments (constants)
            if let Some(child) = node.child(0) {
                if child.kind() == "assignment" {
                    return parse_assignment(&child, source, file_path);
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_function(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    // Skip private functions (starting with _)
    if name.starts_with('_') && !name.starts_with("__") {
        return None;
    }

    // Skip dunder methods except __init__
    if name.starts_with("__") && name.ends_with("__") && name != "__init__" {
        return None;
    }

    let visibility = if name.starts_with('_') {
        Visibility::Protected
    } else {
        Visibility::Public
    };

    let params = node
        .child_by_field_name("parameters")
        .map(|n| n.utf8_text(source).unwrap_or("()").to_string())
        .unwrap_or_else(|| "()".to_string());

    let return_type = node
        .child_by_field_name("return_type")
        .map(|n| {
            let text = n.utf8_text(source).unwrap_or("");
            format!(" -> {text}")
        })
        .unwrap_or_default();

    let is_async = node
        .children(&mut node.walk())
        .any(|c| c.kind() == "async");

    let async_prefix = if is_async { "async " } else { "" };
    let signature = format!("{async_prefix}def {name}{params}{return_type}");

    let doc = extract_docstring(node, source);
    let line = node.start_position().row + 1;

    Some(
        InterfaceItem::new(ItemKind::Function, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(line))
            .with_doc_option(doc),
    )
}

fn parse_class(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    // Skip private classes (starting with _)
    if name.starts_with('_') {
        return None;
    }

    let superclasses = node
        .child_by_field_name("superclasses")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let signature = format!(
        "class {}{}",
        name,
        superclasses.map(|s| format!("({s})")).unwrap_or_default()
    );

    let doc = extract_docstring(node, source);
    let line = node.start_position().row + 1;

    let mut item = InterfaceItem::new(ItemKind::Class, name)
        .with_visibility(Visibility::Public)
        .with_signature(signature)
        .with_source(file_path, Some(line))
        .with_doc_option(doc);

    // Parse class body for methods and attributes
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            match child.kind() {
                "function_definition" => {
                    if let Some(method) = parse_method(&child, source, file_path) {
                        item.methods.push(method);
                    }
                }
                "expression_statement" => {
                    // Class-level assignments (class attributes)
                    if let Some(assign) = child.child(0) {
                        if assign.kind() == "assignment" {
                            if let Some(attr) = parse_class_attribute(&assign, source) {
                                item.fields.push(attr);
                            }
                        }
                    }
                }
                "decorated_definition" => {
                    if let Some(method) = parse_decorated_method(&child, source, file_path) {
                        item.methods.push(method);
                    }
                }
                _ => {}
            }
        }
    }

    Some(item)
}

fn parse_method(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    // Skip private methods (single underscore) but allow dunder methods
    if name.starts_with('_') && !name.starts_with("__") {
        return None;
    }

    // Skip most dunder methods, keep __init__
    if name.starts_with("__") && name.ends_with("__") && name != "__init__" {
        return None;
    }

    let visibility = if name.starts_with('_') {
        Visibility::Protected
    } else {
        Visibility::Public
    };

    let params = node
        .child_by_field_name("parameters")
        .map(|n| {
            // Remove 'self' from parameters for display
            let text = n.utf8_text(source).unwrap_or("()");
            let text = text.trim_start_matches('(').trim_end_matches(')');
            let params: Vec<&str> = text
                .split(',')
                .map(str::trim)
                .filter(|p| !p.starts_with("self"))
                .collect();
            format!("({})", params.join(", "))
        })
        .unwrap_or_else(|| "()".to_string());

    let return_type = node
        .child_by_field_name("return_type")
        .map(|n| {
            let text = n.utf8_text(source).unwrap_or("");
            format!(" -> {text}")
        })
        .unwrap_or_default();

    let is_async = node
        .children(&mut node.walk())
        .any(|c| c.kind() == "async");

    let async_prefix = if is_async { "async " } else { "" };
    let signature = format!("{async_prefix}def {name}{params}{return_type}");

    let doc = extract_docstring(node, source);
    let line = node.start_position().row + 1;

    Some(
        InterfaceItem::new(ItemKind::Function, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(line))
            .with_doc_option(doc),
    )
}

fn parse_decorated(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    // Find the definition inside the decorated_definition
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "function_definition" => return parse_function(&child, source, file_path),
            "class_definition" => return parse_class(&child, source, file_path),
            _ => {}
        }
    }
    None
}

fn parse_decorated_method(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    // Find the method definition inside
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "function_definition" {
            return parse_method(&child, source, file_path);
        }
    }
    None
}

fn parse_assignment(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    let left = node.child_by_field_name("left")?;

    // Only handle simple identifier assignments
    if left.kind() != "identifier" {
        return None;
    }

    let name = left.utf8_text(source).ok()?.to_string();

    // Skip private variables (starting with _)
    if name.starts_with('_') {
        return None;
    }

    // Check if it looks like a constant (ALL_CAPS)
    if !name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_numeric()) {
        return None;
    }

    let type_annotation = node
        .child_by_field_name("type")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let signature = format!(
        "{name}{}",
        type_annotation.map(|t| format!(": {t}")).unwrap_or_default()
    );

    let line = node.start_position().row + 1;

    Some(
        InterfaceItem::new(ItemKind::Constant, name)
            .with_visibility(Visibility::Public)
            .with_signature(signature)
            .with_source(file_path, Some(line)),
    )
}

fn parse_class_attribute(node: &Node, source: &[u8]) -> Option<FieldInfo> {
    let left = node.child_by_field_name("left")?;

    // Only handle simple identifier assignments
    if left.kind() != "identifier" {
        return None;
    }

    let name = left.utf8_text(source).ok()?.to_string();

    // Skip private attributes
    if name.starts_with('_') {
        return None;
    }

    let type_annotation = node
        .child_by_field_name("type")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string())
        .unwrap_or_default();

    Some(FieldInfo {
        name,
        type_annotation,
        visibility: Visibility::Public,
        doc_comment: None,
    })
}

// Helper functions

fn extract_docstring(node: &Node, source: &[u8]) -> Option<String> {
    // Look for the body and check if first statement is a string
    let body = node.child_by_field_name("body")?;

    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        if child.kind() == "expression_statement" {
            if let Some(string_node) = child.child(0) {
                if string_node.kind() == "string" {
                    let text = string_node.utf8_text(source).ok()?;
                    return Some(clean_docstring(text));
                }
            }
        }
        // Only check first statement
        break;
    }

    None
}

fn extract_module_docstring(root: &Node, source: &[u8]) -> Option<String> {
    // First statement in module should be a string literal for it to be a docstring
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "expression_statement" {
            if let Some(string_node) = child.child(0) {
                if string_node.kind() == "string" {
                    let text = string_node.utf8_text(source).ok()?;
                    return Some(clean_docstring(text));
                }
            }
        }
        // Only check first statement
        break;
    }

    None
}

fn clean_docstring(text: &str) -> String {
    let text = text.trim();

    // Remove triple quotes
    let text = if text.starts_with("\"\"\"") && text.ends_with("\"\"\"") {
        &text[3..text.len() - 3]
    } else if text.starts_with("'''") && text.ends_with("'''") {
        &text[3..text.len() - 3]
    } else if text.starts_with('"') && text.ends_with('"') {
        &text[1..text.len() - 1]
    } else if text.starts_with('\'') && text.ends_with('\'') {
        &text[1..text.len() - 1]
    } else {
        text
    };

    // Clean up indentation
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= 1 {
        return text.trim().to_string();
    }

    // Find minimum indentation (excluding first and empty lines)
    let min_indent = lines
        .iter()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);

    // Remove common indentation
    let cleaned: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 || line.trim().is_empty() {
                line.trim().to_string()
            } else if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    cleaned.join("\n").trim().to_string()
}

// Extension trait for InterfaceItem builder
trait InterfaceItemExt {
    fn with_doc_option(self, doc: Option<String>) -> Self;
}

impl InterfaceItemExt for InterfaceItem {
    fn with_doc_option(mut self, doc: Option<String>) -> Self {
        self.doc_comment = doc;
        self
    }
}
