//! Rust source code parser using `syn`.

use anyhow::Result;
use syn::{
    Attribute, Fields, FnArg, GenericParam, Generics, ImplItem, Item, ItemConst, ItemEnum,
    ItemFn, ItemImpl, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, Pat, ReturnType,
    TraitItem, Visibility as SynVisibility,
};

use super::interface::{
    Example, FieldInfo, InterfaceItem, ItemKind, Language, ModuleInfo, ParseResult, Visibility,
};
use super::Parser;

/// Rust source code parser.
pub struct RustParser;

impl Parser for RustParser {
    fn parse(&self, source: &str, file_path: &str) -> Result<ParseResult> {
        let mut result = ParseResult::new(Language::Rust);

        match syn::parse_file(source) {
            Ok(file) => {
                // Extract module-level doc comment
                let module_doc = extract_outer_doc(&file.attrs);

                let mut module = ModuleInfo::new(
                    file_path
                        .rsplit('/')
                        .next()
                        .unwrap_or(file_path)
                        .trim_end_matches(".rs"),
                );
                module.doc_comment = module_doc;

                // Parse items
                for item in &file.items {
                    if let Some(parsed) = parse_item(item, file_path, &[]) {
                        module.items.push(parsed);
                        result.items.push(module.items.last().unwrap().clone());
                    }
                }

                result.module = Some(module);
            }
            Err(e) => {
                result.errors.push(format!("Parse error: {e}"));
            }
        }

        Ok(result)
    }

    fn language(&self) -> Language {
        Language::Rust
    }

    fn can_parse(&self, file_path: &str) -> bool {
        file_path.ends_with(".rs")
    }
}

fn parse_item(item: &Item, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    match item {
        Item::Fn(f) => parse_fn(f, file_path, module_path),
        Item::Struct(s) => parse_struct(s, file_path, module_path),
        Item::Enum(e) => parse_enum(e, file_path, module_path),
        Item::Trait(t) => parse_trait(t, file_path, module_path),
        Item::Impl(i) => parse_impl(i, file_path, module_path),
        Item::Type(t) => parse_type_alias(t, file_path, module_path),
        Item::Const(c) => parse_const(c, file_path, module_path),
        Item::Static(s) => parse_static(s, file_path, module_path),
        Item::Mod(m) => parse_module(m, file_path, module_path),
        _ => None,
    }
}

fn parse_fn(f: &ItemFn, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&f.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = f.sig.ident.to_string();
    let signature = format_fn_signature(f);
    let doc = extract_doc(&f.attrs);
    let examples = extract_examples(&f.attrs);
    let generics = format_generics(&f.sig.generics);

    Some(
        InterfaceItem::new(ItemKind::Function, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(1))
            .with_doc_opt(doc)
            .with_examples(examples)
            .with_generics(generics)
            .with_module_path(module_path),
    )
}

fn parse_struct(s: &ItemStruct, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&s.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = s.ident.to_string();
    let doc = extract_doc(&s.attrs);
    let examples = extract_examples(&s.attrs);
    let generics = format_generics(&s.generics);
    let fields = extract_fields(&s.fields);

    let signature = format!(
        "pub struct {}{}",
        name,
        generics.as_deref().unwrap_or("")
    );

    let mut item = InterfaceItem::new(ItemKind::Struct, name)
        .with_visibility(visibility)
        .with_signature(signature)
        .with_source(file_path, Some(1))
        .with_doc_opt(doc)
        .with_examples(examples)
        .with_generics(generics)
        .with_module_path(module_path);

    item.fields = fields;
    Some(item)
}

fn parse_enum(e: &ItemEnum, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&e.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = e.ident.to_string();
    let doc = extract_doc(&e.attrs);
    let examples = extract_examples(&e.attrs);
    let generics = format_generics(&e.generics);

    let variants: Vec<String> = e
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect();

    let signature = format!(
        "pub enum {}{} {{ {} }}",
        name,
        generics.as_deref().unwrap_or(""),
        variants.join(", ")
    );

    Some(
        InterfaceItem::new(ItemKind::Enum, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(1))
            .with_doc_opt(doc)
            .with_examples(examples)
            .with_generics(generics)
            .with_module_path(module_path),
    )
}

fn parse_trait(t: &ItemTrait, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&t.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = t.ident.to_string();
    let doc = extract_doc(&t.attrs);
    let examples = extract_examples(&t.attrs);
    let generics = format_generics(&t.generics);

    let signature = format!(
        "pub trait {}{}",
        name,
        generics.as_deref().unwrap_or("")
    );

    let mut item = InterfaceItem::new(ItemKind::Trait, name)
        .with_visibility(visibility)
        .with_signature(signature)
        .with_source(file_path, Some(1))
        .with_doc_opt(doc)
        .with_examples(examples)
        .with_generics(generics)
        .with_module_path(module_path);

    // Parse trait items (methods, associated types)
    for trait_item in &t.items {
        match trait_item {
            TraitItem::Fn(method) => {
                let method_sig = format_trait_method_signature(method);
                let method_doc = extract_doc(&method.attrs);
                let method_item = InterfaceItem::new(ItemKind::Function, method.sig.ident.to_string())
                    .with_visibility(Visibility::Public)
                    .with_signature(method_sig)
                    .with_doc_opt(method_doc);
                item.methods.push(method_item);
            }
            TraitItem::Type(assoc_type) => {
                item.associated_types.push(assoc_type.ident.to_string());
            }
            _ => {}
        }
    }

    Some(item)
}

fn parse_impl(i: &ItemImpl, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    // Only parse trait implementations with public methods
    let trait_name = i.trait_.as_ref().map(|(_, path, _)| {
        path.segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    });

    let self_type = quote::quote!(#i.self_ty).to_string().replace(' ', "");

    let name = if let Some(trait_name) = &trait_name {
        format!("{trait_name} for {self_type}")
    } else {
        self_type.clone()
    };

    let signature = format!("impl {name}");

    let mut item = InterfaceItem::new(ItemKind::TraitImpl, name)
        .with_visibility(Visibility::Public)
        .with_signature(signature)
        .with_source(file_path, Some(1))
        .with_module_path(module_path);

    // Parse impl methods
    for impl_item in &i.items {
        if let ImplItem::Fn(method) = impl_item {
            let vis = convert_visibility(&method.vis);
            if vis.is_public() || trait_name.is_some() {
                let method_sig = format_impl_method_signature(method);
                let method_doc = extract_doc(&method.attrs);
                let method_item = InterfaceItem::new(ItemKind::Function, method.sig.ident.to_string())
                    .with_visibility(vis)
                    .with_signature(method_sig)
                    .with_doc_opt(method_doc);
                item.methods.push(method_item);
            }
        }
    }

    // Only include impl blocks with public methods or trait impls
    if item.methods.is_empty() && trait_name.is_none() {
        return None;
    }

    Some(item)
}

fn parse_type_alias(t: &ItemType, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&t.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = t.ident.to_string();
    let doc = extract_doc(&t.attrs);
    let generics = format_generics(&t.generics);
    let ty = quote::quote!(#t.ty).to_string();

    let signature = format!(
        "pub type {}{} = {}",
        name,
        generics.as_deref().unwrap_or(""),
        ty
    );

    Some(
        InterfaceItem::new(ItemKind::TypeAlias, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(1))
            .with_doc_opt(doc)
            .with_generics(generics)
            .with_module_path(module_path),
    )
}

fn parse_const(c: &ItemConst, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&c.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = c.ident.to_string();
    let doc = extract_doc(&c.attrs);
    let ty = quote::quote!(#c.ty).to_string();

    let signature = format!("pub const {name}: {ty}");

    Some(
        InterfaceItem::new(ItemKind::Constant, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(1))
            .with_doc_opt(doc)
            .with_module_path(module_path),
    )
}

fn parse_static(s: &ItemStatic, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&s.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = s.ident.to_string();
    let doc = extract_doc(&s.attrs);
    let ty = quote::quote!(#s.ty).to_string();
    let mutability = if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
        "mut "
    } else {
        ""
    };

    let signature = format!("pub static {mutability}{name}: {ty}");

    Some(
        InterfaceItem::new(ItemKind::Static, name)
            .with_visibility(visibility)
            .with_signature(signature)
            .with_source(file_path, Some(1))
            .with_doc_opt(doc)
            .with_module_path(module_path),
    )
}

fn parse_module(m: &ItemMod, file_path: &str, module_path: &[String]) -> Option<InterfaceItem> {
    let visibility = convert_visibility(&m.vis);
    if !visibility.is_public() {
        return None;
    }

    let name = m.ident.to_string();
    let doc = extract_doc(&m.attrs);

    let signature = format!("pub mod {name}");

    let mut item = InterfaceItem::new(ItemKind::Module, name.clone())
        .with_visibility(visibility)
        .with_signature(signature)
        .with_source(file_path, Some(1))
        .with_doc_opt(doc)
        .with_module_path(module_path);

    // Parse inline module content
    if let Some((_, items)) = &m.content {
        let mut new_path = module_path.to_vec();
        new_path.push(name);

        for sub_item in items {
            if let Some(parsed) = parse_item(sub_item, file_path, &new_path) {
                item.methods.push(parsed);
            }
        }
    }

    Some(item)
}

// Helper functions

fn convert_visibility(vis: &SynVisibility) -> Visibility {
    match vis {
        SynVisibility::Public(_) => Visibility::Public,
        SynVisibility::Restricted(r) => {
            let path = quote::quote!(#r.path).to_string();
            if path == "crate" {
                Visibility::Crate
            } else if path == "super" {
                Visibility::Super
            } else {
                Visibility::Private
            }
        }
        SynVisibility::Inherited => Visibility::Private,
    }
}

fn extract_doc(attrs: &[Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect();

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

fn extract_outer_doc(attrs: &[Attribute]) -> Option<String> {
    let docs: Vec<String> = attrs
        .iter()
        .filter(|attr| matches!(attr.style, syn::AttrStyle::Inner(_)))
        .filter_map(|attr| {
            if attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(nv) = &attr.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect();

    if docs.is_empty() {
        None
    } else {
        Some(docs.join("\n"))
    }
}

fn extract_examples(attrs: &[Attribute]) -> Vec<Example> {
    let doc = extract_doc(attrs).unwrap_or_default();
    let mut examples = Vec::new();
    let mut in_code_block = false;
    let mut current_code = String::new();
    let mut code_lang = String::new();

    for line in doc.lines() {
        if line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                if !current_code.trim().is_empty() {
                    examples.push(Example {
                        title: None,
                        code: current_code.trim().to_string(),
                        language: if code_lang.is_empty() {
                            "rust".to_string()
                        } else {
                            code_lang.clone()
                        },
                    });
                }
                current_code.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                // Start of code block
                code_lang = line.trim().trim_start_matches("```").to_string();
                in_code_block = true;
            }
        } else if in_code_block {
            current_code.push_str(line);
            current_code.push('\n');
        }
    }

    examples
}

fn format_generics(generics: &Generics) -> Option<String> {
    if generics.params.is_empty() {
        return None;
    }

    let params: Vec<String> = generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(t) => t.ident.to_string(),
            GenericParam::Lifetime(l) => format!("'{}", l.lifetime.ident),
            GenericParam::Const(c) => format!("const {}", c.ident),
        })
        .collect();

    Some(format!("<{}>", params.join(", ")))
}

fn format_fn_signature(f: &ItemFn) -> String {
    let vis = if matches!(f.vis, SynVisibility::Public(_)) {
        "pub "
    } else {
        ""
    };
    let asyncness = if f.sig.asyncness.is_some() {
        "async "
    } else {
        ""
    };
    let name = &f.sig.ident;
    let generics = format_generics(&f.sig.generics).unwrap_or_default();

    let inputs: Vec<String> = f
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => {
                let mutability = if r.mutability.is_some() { "mut " } else { "" };
                let reference = if r.reference.is_some() { "&" } else { "" };
                format!("{reference}{mutability}self")
            }
            FnArg::Typed(t) => {
                let pat = match &*t.pat {
                    Pat::Ident(i) => i.ident.to_string(),
                    _ => "_".to_string(),
                };
                let ty = quote::quote!(#t.ty).to_string();
                format!("{pat}: {ty}")
            }
        })
        .collect();

    let output = match &f.sig.output {
        ReturnType::Default => String::new(),
        ReturnType::Type(_, ty) => format!(" -> {}", quote::quote!(#ty)),
    };

    format!("{vis}{asyncness}fn {name}{generics}({}){output}", inputs.join(", "))
}

fn format_trait_method_signature(method: &syn::TraitItemFn) -> String {
    let asyncness = if method.sig.asyncness.is_some() {
        "async "
    } else {
        ""
    };
    let name = &method.sig.ident;
    let generics = format_generics(&method.sig.generics).unwrap_or_default();

    let inputs: Vec<String> = method
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => {
                let mutability = if r.mutability.is_some() { "mut " } else { "" };
                let reference = if r.reference.is_some() { "&" } else { "" };
                format!("{reference}{mutability}self")
            }
            FnArg::Typed(t) => {
                let pat = match &*t.pat {
                    Pat::Ident(i) => i.ident.to_string(),
                    _ => "_".to_string(),
                };
                let ty = quote::quote!(#t.ty).to_string();
                format!("{pat}: {ty}")
            }
        })
        .collect();

    let output = match &method.sig.output {
        ReturnType::Default => String::new(),
        ReturnType::Type(_, ty) => format!(" -> {}", quote::quote!(#ty)),
    };

    format!("{asyncness}fn {name}{generics}({}){output}", inputs.join(", "))
}

fn format_impl_method_signature(method: &syn::ImplItemFn) -> String {
    let vis = if matches!(method.vis, SynVisibility::Public(_)) {
        "pub "
    } else {
        ""
    };
    let asyncness = if method.sig.asyncness.is_some() {
        "async "
    } else {
        ""
    };
    let name = &method.sig.ident;
    let generics = format_generics(&method.sig.generics).unwrap_or_default();

    let inputs: Vec<String> = method
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => {
                let mutability = if r.mutability.is_some() { "mut " } else { "" };
                let reference = if r.reference.is_some() { "&" } else { "" };
                format!("{reference}{mutability}self")
            }
            FnArg::Typed(t) => {
                let pat = match &*t.pat {
                    Pat::Ident(i) => i.ident.to_string(),
                    _ => "_".to_string(),
                };
                let ty = quote::quote!(#t.ty).to_string();
                format!("{pat}: {ty}")
            }
        })
        .collect();

    let output = match &method.sig.output {
        ReturnType::Default => String::new(),
        ReturnType::Type(_, ty) => format!(" -> {}", quote::quote!(#ty)),
    };

    format!("{vis}{asyncness}fn {name}{generics}({}){output}", inputs.join(", "))
}

fn extract_fields(fields: &Fields) -> Vec<FieldInfo> {
    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .filter_map(|f| {
                let vis = convert_visibility(&f.vis);
                let name = f.ident.as_ref()?.to_string();
                let ty = quote::quote!(#f.ty).to_string();
                let doc = extract_doc(&f.attrs);

                Some(FieldInfo {
                    name,
                    type_annotation: ty,
                    visibility: vis,
                    doc_comment: doc,
                })
            })
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let vis = convert_visibility(&f.vis);
                let ty = quote::quote!(#f.ty).to_string();
                let doc = extract_doc(&f.attrs);

                FieldInfo {
                    name: format!("{i}"),
                    type_annotation: ty,
                    visibility: vis,
                    doc_comment: doc,
                }
            })
            .collect(),
        Fields::Unit => vec![],
    }
}

// Extension trait for InterfaceItem builder
trait InterfaceItemExt {
    fn with_doc_opt(self, doc: Option<String>) -> Self;
    fn with_examples(self, examples: Vec<Example>) -> Self;
    fn with_generics(self, generics: Option<String>) -> Self;
    fn with_module_path(self, path: &[String]) -> Self;
}

impl InterfaceItemExt for InterfaceItem {
    fn with_doc_opt(mut self, doc: Option<String>) -> Self {
        self.doc_comment = doc;
        self
    }

    fn with_examples(mut self, examples: Vec<Example>) -> Self {
        self.examples = examples;
        self
    }

    fn with_generics(mut self, generics: Option<String>) -> Self {
        self.generics = generics;
        self
    }

    fn with_module_path(mut self, path: &[String]) -> Self {
        self.module_path = path.to_vec();
        self
    }
}
