//! TypeScript source code parser using tree-sitter.

use anyhow::Result;
use tree_sitter::{Node, Parser as TsParser};

use super::interface::{
    FieldInfo, InterfaceItem, ItemKind, Language, ModuleInfo, ParseResult, Visibility,
};
use super::Parser;

/// TypeScript source code parser.
pub struct TypeScriptParser;

impl Parser for TypeScriptParser {
    fn parse(&self, source: &str, file_path: &str) -> Result<ParseResult> {
        let mut result = ParseResult::new(Language::TypeScript);

        let mut parser = TsParser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
        parser
            .set_language(&language.into())
            .map_err(|e| anyhow::anyhow!("Failed to set TypeScript language: {e}"))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript"))?;

        let root = tree.root_node();
        let source_bytes = source.as_bytes();

        let mut module = ModuleInfo::new(
            file_path
                .rsplit('/')
                .next()
                .unwrap_or(file_path)
                .trim_end_matches(".ts")
                .trim_end_matches(".tsx"),
        );

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
        Language::TypeScript
    }

    fn can_parse(&self, file_path: &str) -> bool {
        file_path.ends_with(".ts") || file_path.ends_with(".tsx")
    }
}

fn parse_node(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    match node.kind() {
        "export_statement" => parse_export(node, source, file_path),
        "function_declaration" => parse_function(node, source, file_path, false),
        "class_declaration" => parse_class(node, source, file_path, false),
        "interface_declaration" => parse_interface(node, source, file_path, false),
        "type_alias_declaration" => parse_type_alias(node, source, file_path, false),
        "lexical_declaration" => parse_variable(node, source, file_path, false),
        _ => None,
    }
}

fn parse_export(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    // Find the declaration inside the export
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "function_declaration" => return parse_function(&child, source, file_path, true),
            "class_declaration" => return parse_class(&child, source, file_path, true),
            "interface_declaration" => return parse_interface(&child, source, file_path, true),
            "type_alias_declaration" => return parse_type_alias(&child, source, file_path, true),
            "lexical_declaration" => return parse_variable(&child, source, file_path, true),
            _ => {}
        }
    }
    None
}

fn parse_function(
    node: &Node,
    source: &[u8],
    file_path: &str,
    is_exported: bool,
) -> Option<InterfaceItem> {
    if !is_exported {
        return None;
    }

    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    let params = node
        .child_by_field_name("parameters")
        .map(|n| n.utf8_text(source).unwrap_or("()").to_string())
        .unwrap_or_else(|| "()".to_string());

    let return_type = node
        .child_by_field_name("return_type")
        .map(|n| {
            let text = n.utf8_text(source).unwrap_or("");
            if text.starts_with(':') {
                text.to_string()
            } else {
                format!(": {text}")
            }
        })
        .unwrap_or_default();

    let is_async = node
        .children(&mut node.walk())
        .any(|c| c.kind() == "async");

    let async_prefix = if is_async { "async " } else { "" };
    let signature = format!("export {async_prefix}function {name}{params}{return_type}");

    let doc = extract_preceding_comment(node, source);
    let line = node.start_position().row + 1;

    Some(
        InterfaceItem::new(ItemKind::Function, name)
            .with_visibility(Visibility::Public)
            .with_signature(signature)
            .with_source(file_path, Some(line))
            .with_doc_option(doc),
    )
}

fn parse_class(
    node: &Node,
    source: &[u8],
    file_path: &str,
    is_exported: bool,
) -> Option<InterfaceItem> {
    if !is_exported {
        return None;
    }

    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let extends = find_child_by_kind(node, "class_heritage")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let signature = format!(
        "export class {}{}{}",
        name,
        type_params.as_deref().unwrap_or(""),
        extends.map(|e| format!(" {e}")).unwrap_or_default()
    );

    let doc = extract_preceding_comment(node, source);
    let line = node.start_position().row + 1;

    let mut item = InterfaceItem::new(ItemKind::Class, name)
        .with_visibility(Visibility::Public)
        .with_signature(signature)
        .with_source(file_path, Some(line))
        .with_doc_option(doc)
        .with_generics_option(type_params);

    // Parse class body for methods and fields
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            match child.kind() {
                "method_definition" | "public_field_definition" => {
                    if let Some(member) = parse_class_member(&child, source, file_path) {
                        if matches!(member.kind, ItemKind::Function) {
                            item.methods.push(member);
                        } else {
                            item.fields.push(FieldInfo {
                                name: member.name,
                                type_annotation: member.signature,
                                visibility: member.visibility,
                                doc_comment: member.doc_comment,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Some(item)
}

fn parse_class_member(node: &Node, source: &[u8], file_path: &str) -> Option<InterfaceItem> {
    match node.kind() {
        "method_definition" => {
            let name = node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())?
                .to_string();

            // Skip private methods (starting with #)
            if name.starts_with('#') {
                return None;
            }

            let params = node
                .child_by_field_name("parameters")
                .map(|n| n.utf8_text(source).unwrap_or("()").to_string())
                .unwrap_or_else(|| "()".to_string());

            let return_type = node
                .child_by_field_name("return_type")
                .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                .unwrap_or_default();

            let is_async = node
                .children(&mut node.walk())
                .any(|c| c.kind() == "async");

            let async_prefix = if is_async { "async " } else { "" };
            let signature = format!("{async_prefix}{name}{params}{return_type}");

            let doc = extract_preceding_comment(node, source);
            let line = node.start_position().row + 1;

            Some(
                InterfaceItem::new(ItemKind::Function, name)
                    .with_visibility(Visibility::Public)
                    .with_signature(signature)
                    .with_source(file_path, Some(line))
                    .with_doc_option(doc),
            )
        }
        "public_field_definition" => {
            let name = node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(source).ok())?
                .to_string();

            // Skip private fields
            if name.starts_with('#') || name.starts_with('_') {
                return None;
            }

            let type_annotation = node
                .child_by_field_name("type")
                .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                .unwrap_or_default();

            let doc = extract_preceding_comment(node, source);
            let line = node.start_position().row + 1;

            Some(
                InterfaceItem::new(ItemKind::Variable, name)
                    .with_visibility(Visibility::Public)
                    .with_signature(type_annotation)
                    .with_source(file_path, Some(line))
                    .with_doc_option(doc),
            )
        }
        _ => None,
    }
}

fn parse_interface(
    node: &Node,
    source: &[u8],
    file_path: &str,
    is_exported: bool,
) -> Option<InterfaceItem> {
    if !is_exported {
        return None;
    }

    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let extends = find_child_by_kind(node, "extends_type_clause")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let signature = format!(
        "export interface {}{}{}",
        name,
        type_params.as_deref().unwrap_or(""),
        extends.map(|e| format!(" {e}")).unwrap_or_default()
    );

    let doc = extract_preceding_comment(node, source);
    let line = node.start_position().row + 1;

    let mut item = InterfaceItem::new(ItemKind::Interface, name)
        .with_visibility(Visibility::Public)
        .with_signature(signature)
        .with_source(file_path, Some(line))
        .with_doc_option(doc)
        .with_generics_option(type_params);

    // Parse interface body for properties and methods
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            if child.kind() == "property_signature" || child.kind() == "method_signature" {
                if let Some(prop_name) = child.child_by_field_name("name") {
                    let prop_name_str = prop_name.utf8_text(source).unwrap_or("").to_string();
                    let type_ann = child
                        .child_by_field_name("type")
                        .map(|n| n.utf8_text(source).unwrap_or("").to_string())
                        .unwrap_or_default();

                    item.fields.push(FieldInfo {
                        name: prop_name_str,
                        type_annotation: type_ann,
                        visibility: Visibility::Public,
                        doc_comment: None,
                    });
                }
            }
        }
    }

    Some(item)
}

fn parse_type_alias(
    node: &Node,
    source: &[u8],
    file_path: &str,
    is_exported: bool,
) -> Option<InterfaceItem> {
    if !is_exported {
        return None;
    }

    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    let type_params = node
        .child_by_field_name("type_parameters")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let value = node
        .child_by_field_name("value")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string())
        .unwrap_or_default();

    let signature = format!(
        "export type {}{} = {}",
        name,
        type_params.as_deref().unwrap_or(""),
        value
    );

    let doc = extract_preceding_comment(node, source);
    let line = node.start_position().row + 1;

    Some(
        InterfaceItem::new(ItemKind::TypeDef, name)
            .with_visibility(Visibility::Public)
            .with_signature(signature)
            .with_source(file_path, Some(line))
            .with_doc_option(doc)
            .with_generics_option(type_params),
    )
}

fn parse_variable(
    node: &Node,
    source: &[u8],
    file_path: &str,
    is_exported: bool,
) -> Option<InterfaceItem> {
    if !is_exported {
        return None;
    }

    // Get the variable declarator
    let declarator = find_child_by_kind(node, "variable_declarator")?;

    let name = declarator
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source).ok())?
        .to_string();

    let type_annotation = declarator
        .child_by_field_name("type")
        .map(|n| n.utf8_text(source).unwrap_or("").to_string());

    let kind = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "const" || c.kind() == "let" || c.kind() == "var")
        .map(|c| c.kind())
        .unwrap_or("const");

    let signature = format!(
        "export {kind} {name}{}",
        type_annotation.map(|t| format!(": {t}")).unwrap_or_default()
    );

    let doc = extract_preceding_comment(node, source);
    let line = node.start_position().row + 1;

    Some(
        InterfaceItem::new(ItemKind::Variable, name)
            .with_visibility(Visibility::Public)
            .with_signature(signature)
            .with_source(file_path, Some(line))
            .with_doc_option(doc),
    )
}

// Helper functions

fn find_child_by_kind<'a>(node: &'a Node, kind: &str) -> Option<Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|c| c.kind() == kind)
}

fn extract_preceding_comment(node: &Node, source: &[u8]) -> Option<String> {
    // Look for preceding sibling comments
    let mut prev = node.prev_sibling();
    let mut comments = Vec::new();

    while let Some(sibling) = prev {
        if sibling.kind() == "comment" {
            if let Ok(text) = sibling.utf8_text(source) {
                let cleaned = clean_comment(text);
                if !cleaned.is_empty() {
                    comments.push(cleaned);
                }
            }
        } else if sibling.kind() != "export_statement" {
            break;
        }
        prev = sibling.prev_sibling();
    }

    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        Some(comments.join("\n"))
    }
}

fn clean_comment(text: &str) -> String {
    let text = text.trim();

    // Handle JSDoc comments: /** ... */
    if text.starts_with("/**") && text.ends_with("*/") {
        return text[3..text.len() - 2]
            .lines()
            .map(|line| {
                let line = line.trim();
                let line = line.strip_prefix('*').unwrap_or(line);
                line.trim()
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
    }

    // Handle single-line comments: // ...
    if text.starts_with("//") {
        return text[2..].trim().to_string();
    }

    // Handle block comments: /* ... */
    if text.starts_with("/*") && text.ends_with("*/") {
        return text[2..text.len() - 2].trim().to_string();
    }

    text.to_string()
}

// Extension trait for InterfaceItem builder
trait InterfaceItemExt {
    fn with_doc_option(self, doc: Option<String>) -> Self;
    fn with_generics_option(self, generics: Option<String>) -> Self;
}

impl InterfaceItemExt for InterfaceItem {
    fn with_doc_option(mut self, doc: Option<String>) -> Self {
        self.doc_comment = doc;
        self
    }

    fn with_generics_option(mut self, generics: Option<String>) -> Self {
        self.generics = generics;
        self
    }
}
