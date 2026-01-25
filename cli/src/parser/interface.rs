//! Common interface types for parsed source code.

use serde::{Deserialize, Serialize};

/// Programming language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Unknown,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::Python => "python",
            Self::Unknown => "unknown",
        }
    }

    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            Self::Rust => &["rs"],
            Self::TypeScript => &["ts", "tsx"],
            Self::Python => &["py"],
            Self::Unknown => &[],
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Item visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Visibility {
    Public,
    #[default]
    Private,
    Crate,      // Rust: pub(crate)
    Super,      // Rust: pub(super)
    Protected,  // Python: single underscore
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public)
    }
}

/// Kind of interface item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    // Rust items
    Function,
    Struct,
    Enum,
    Trait,
    TraitImpl,
    TypeAlias,
    Constant,
    Static,
    Macro,
    Module,

    // TypeScript items
    Class,
    Interface,
    TypeDef,
    Variable,

    // Python items
    // (reuses Class, Function)
}

impl ItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
            Self::TraitImpl => "impl",
            Self::TypeAlias => "type",
            Self::Constant => "const",
            Self::Static => "static",
            Self::Macro => "macro",
            Self::Module => "module",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::TypeDef => "typedef",
            Self::Variable => "variable",
        }
    }
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A parsed interface item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceItem {
    /// Kind of item (function, struct, etc.)
    pub kind: ItemKind,

    /// Item name
    pub name: String,

    /// Full signature (e.g., `pub fn foo(x: i32) -> String`)
    pub signature: String,

    /// Documentation comment
    pub doc_comment: Option<String>,

    /// Visibility
    pub visibility: Visibility,

    /// Module path (e.g., `["crate", "module", "submodule"]`)
    pub module_path: Vec<String>,

    /// Code examples from doc comments
    pub examples: Vec<Example>,

    /// Source file path
    pub source_file: String,

    /// Line number in source file
    pub line_number: Option<usize>,

    /// Generic parameters (if any)
    pub generics: Option<String>,

    /// For traits/classes: list of methods
    pub methods: Vec<InterfaceItem>,

    /// For traits: list of associated types
    pub associated_types: Vec<String>,

    /// For structs/classes: list of fields
    pub fields: Vec<FieldInfo>,
}

impl InterfaceItem {
    pub fn new(kind: ItemKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
            signature: String::new(),
            doc_comment: None,
            visibility: Visibility::Private,
            module_path: vec![],
            examples: vec![],
            source_file: String::new(),
            line_number: None,
            generics: None,
            methods: vec![],
            associated_types: vec![],
            fields: vec![],
        }
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = signature.into();
        self
    }

    #[allow(dead_code)]
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc_comment = Some(doc.into());
        self
    }

    pub fn with_source(mut self, file: impl Into<String>, line: Option<usize>) -> Self {
        self.source_file = file.into();
        self.line_number = line;
        self
    }
}

/// Field information for structs/classes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub type_annotation: String,
    pub visibility: Visibility,
    pub doc_comment: Option<String>,
}

/// A code example from documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub title: Option<String>,
    pub code: String,
    pub language: String,
}

/// Module information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub path: Vec<String>,
    pub doc_comment: Option<String>,
    pub items: Vec<InterfaceItem>,
    pub submodules: Vec<ModuleInfo>,
}

impl ModuleInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: vec![],
            doc_comment: None,
            items: vec![],
            submodules: vec![],
        }
    }

    /// Get all items recursively (including from submodules).
    #[allow(dead_code)]
    pub fn all_items(&self) -> Vec<&InterfaceItem> {
        let mut items: Vec<&InterfaceItem> = self.items.iter().collect();
        for submodule in &self.submodules {
            items.extend(submodule.all_items());
        }
        items
    }
}

/// Result of parsing a source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub language: Language,
    pub module: Option<ModuleInfo>,
    pub items: Vec<InterfaceItem>,
    pub errors: Vec<String>,
}

impl ParseResult {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            module: None,
            items: vec![],
            errors: vec![],
        }
    }

    /// Get all public items.
    #[allow(dead_code)]
    pub fn public_items(&self) -> Vec<&InterfaceItem> {
        self.items
            .iter()
            .filter(|item| item.visibility.is_public())
            .collect()
    }

    /// Merge another parse result into this one.
    #[allow(dead_code)]
    pub fn merge(&mut self, other: ParseResult) {
        self.items.extend(other.items);
        self.errors.extend(other.errors);
    }
}
