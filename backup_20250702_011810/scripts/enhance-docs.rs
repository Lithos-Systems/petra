//! Script to ensure all public APIs have examples

use std::fs;
use std::path::Path;
use syn::{Item, ItemFn, ItemStruct, ItemEnum, ItemImpl};

fn check_docs(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let ast = syn::parse_file(&content)?;
    
    let mut missing_docs = Vec::new();
    let mut missing_examples = Vec::new();
    
    for item in ast.items {
        match item {
            Item::Fn(func) if is_public(&func.vis) => {
                if !has_doc_comment(&func.attrs) {
                    missing_docs.push(format!("Function: {}", func.sig.ident));
                } else if !has_example(&func.attrs) {
                    missing_examples.push(format!("Function: {}", func.sig.ident));
                }
            }
            Item::Struct(s) if is_public(&s.vis) => {
                if !has_doc_comment(&s.attrs) {
                    missing_docs.push(format!("Struct: {}", s.ident));
                }
            }
            Item::Enum(e) if is_public(&e.vis) => {
                if !has_doc_comment(&e.attrs) {
                    missing_docs.push(format!("Enum: {}", e.ident));
                }
            }
            _ => {}
        }
    }
    
    if !missing_docs.is_empty() {
        println!("Missing documentation in {}:", path.display());
        for item in &missing_docs {
            println!("  - {}", item);
        }
    }
    
    if !missing_examples.is_empty() {
        println!("Missing examples in {}:", path.display());
        for item in &missing_examples {
            println!("  - {}", item);
        }
    }
    
    Ok(())
}

fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

fn has_doc_comment(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("doc"))
}

fn has_example(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path.is_ident("doc") {
            if let Ok(meta) = attr.parse_meta() {
                if let syn::Meta::NameValue(nv) = meta {
                    if let syn::Lit::Str(s) = nv.lit {
                        return s.value().contains("# Example");
                    }
                }
            }
        }
        false
    })
}
