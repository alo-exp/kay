use crate::error::ContextError;
use crate::language::Language;
use crate::store::{Symbol, SymbolKind, SymbolStore};
use sha2::{Digest, Sha256};
use std::path::Path;

const SIG_MAX: usize = 256;

/// Truncate signature to SIG_MAX chars; append U+2026 if truncated.
pub fn truncate_sig(sig: &str) -> String {
    if sig.chars().count() > SIG_MAX {
        let truncated: String = sig.chars().take(SIG_MAX).collect();
        format!("{truncated}\u{2026}")
    } else {
        sig.to_string()
    }
}

fn sha256_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    pub files: usize,
    pub symbols: usize,
    pub skipped_files: usize,
}

pub struct TreeSitterIndexer;

impl TreeSitterIndexer {
    pub fn new() -> Self {
        Self
    }

    pub async fn index_file(
        &self,
        path: &Path,
        store: &SymbolStore,
    ) -> Result<IndexStats, ContextError> {
        let content = tokio::fs::read(path).await?;
        let content_str = String::from_utf8_lossy(&content);
        let hash = sha256_hash(&content);

        let file_path_str = path.to_string_lossy().to_string();

        let should_skip = store.check_and_set_index_state(&file_path_str, &hash)?;
        if should_skip {
            return Ok(IndexStats {
                files: 1,
                symbols: 0,
                skipped_files: 1,
            });
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let lang = Language::from_extension(ext);

        let symbols = match lang {
            Language::Unknown => {
                vec![file_boundary_symbol(&file_path_str, &content_str)]
            }
            _ => self.extract_symbols(lang, &content_str, &file_path_str)?,
        };

        let sym_count = symbols.len();
        for sym in symbols {
            store.upsert_symbol(&sym)?;
        }

        Ok(IndexStats {
            files: 1,
            symbols: sym_count,
            skipped_files: 0,
        })
    }

    fn extract_symbols(
        &self,
        lang: Language,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<Symbol>, ContextError> {
        let ts_lang: tree_sitter::Language = match lang {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::Unknown => return Ok(vec![]),
        };

        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&ts_lang)
            .map_err(|e| std::io::Error::other(format!("tree-sitter set_language: {e}")))?;

        let tree = parser
            .parse(source, None)
            .ok_or_else(|| std::io::Error::other("tree-sitter parse returned None"))?;

        let query_str = match lang {
            Language::Rust => RUST_QUERY,
            Language::TypeScript => TS_QUERY,
            Language::Python => PYTHON_QUERY,
            Language::Go => GO_QUERY,
            Language::Unknown => return Ok(vec![]),
        };

        let query = tree_sitter::Query::new(&ts_lang, query_str)
            .map_err(|e| std::io::Error::other(format!("tree-sitter query error: {e}")))?;

        let source_bytes = source.as_bytes();
        let mut cursor = tree_sitter::QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let capture_idx_name = query
            .capture_index_for_name("name")
            .ok_or_else(|| std::io::Error::other("query missing @name capture"))?;
        let capture_idx_def = query.capture_index_for_name("def");

        let mut symbols = Vec::new();
        for m in matches {
            let name_node = m
                .captures
                .iter()
                .find(|c| c.index == capture_idx_name)
                .map(|c| c.node);

            let def_node = capture_idx_def
                .and_then(|def_idx| {
                    m.captures
                        .iter()
                        .find(|c| c.index == def_idx)
                        .map(|c| c.node)
                })
                .or(name_node);

            let name = match name_node.and_then(|n| n.utf8_text(source_bytes).ok()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            if name.is_empty() {
                continue;
            }

            let start_line = def_node
                .map(|n| n.start_position().row as u32)
                .unwrap_or(0);
            let end_line = def_node
                .map(|n| n.end_position().row as u32)
                .unwrap_or(0);

            let sig = def_node
                .and_then(|n| n.utf8_text(source_bytes).ok())
                .map(|text| text.lines().next().unwrap_or("").to_string())
                .unwrap_or_default();

            let kind = def_node
                .map(|n| kind_from_node_type(n.kind()))
                .unwrap_or(SymbolKind::Function);

            symbols.push(Symbol {
                id: 0,
                name,
                kind,
                file_path: file_path.to_string(),
                start_line,
                end_line,
                sig: truncate_sig(&sig),
            });
        }

        Ok(symbols)
    }
}

impl Default for TreeSitterIndexer {
    fn default() -> Self {
        Self::new()
    }
}

fn kind_from_node_type(node_type: &str) -> SymbolKind {
    match node_type {
        "function_item" | "function_declaration" | "function_definition" => SymbolKind::Function,
        "trait_item" => SymbolKind::Trait,
        "struct_item" => SymbolKind::Struct,
        "enum_item" => SymbolKind::Enum,
        "mod_item" => SymbolKind::Module,
        "class_declaration" | "class_definition" => SymbolKind::Class,
        "method_declaration" => SymbolKind::Method,
        // impl methods: function_item nested inside impl — detected by parent context
        _ => SymbolKind::Function,
    }
}

fn file_boundary_symbol(file_path: &str, content: &str) -> Symbol {
    let first_10: String = content.lines().take(10).collect::<Vec<_>>().join("\n");
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path)
        .to_string();
    Symbol {
        id: 0,
        name: file_name,
        kind: SymbolKind::FileBoundary,
        file_path: file_path.to_string(),
        start_line: 0,
        end_line: content.lines().count() as u32,
        sig: truncate_sig(&first_10),
    }
}

// Rust tree-sitter query: @name captures the symbol name, @def captures the whole node
const RUST_QUERY: &str = r#"
(function_item name: (identifier) @name) @def
(trait_item name: (type_identifier) @name) @def
(struct_item name: (type_identifier) @name) @def
(enum_item name: (type_identifier) @name) @def
(mod_item name: (identifier) @name) @def
"#;

// TypeScript query
const TS_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @def
(class_declaration name: (type_identifier) @name) @def
"#;

// Python query
const PYTHON_QUERY: &str = r#"
(function_definition name: (identifier) @name) @def
(class_definition name: (identifier) @name) @def
"#;

// Go query
const GO_QUERY: &str = r#"
(function_declaration name: (identifier) @name) @def
(method_declaration name: (field_identifier) @name) @def
"#;
