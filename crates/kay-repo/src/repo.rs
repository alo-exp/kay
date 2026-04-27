//! Repository analysis - equivalent to forge_repo/src/repo.rs

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// File type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Markdown,
    Json,
    Yaml,
    Toml,
    Config,
    Source,
    Test,
    Binary,
    Image,
    Other,
}

/// File info for repository analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub file_type: FileType,
    pub size: u64,
    pub is_test: bool,
    pub is_binary: bool,
}

/// Repository workspace analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub root: String,
    pub crates: Vec<CrateInfo>,
    pub file_count: usize,
    pub total_size: u64,
    pub language_stats: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub path: String,
    pub version: String,
}

impl Workspace {
    pub async fn analyze<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref();
        let mut file_count = 0usize;
        let mut total_size = 0u64;
        let mut language_stats = HashMap::new();

        // Walk and analyze
        let mut walkdir = tokio::fs::read_dir(root).await?;
        while let Some(entry) = walkdir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                }
            }

            if let Ok(meta) = entry.metadata().await {
                file_count += 1;
                total_size += meta.len();

                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let file_type = Self::classify_file(&path, ext);
                *language_stats.entry(Self::file_type_to_lang(&file_type)).or_insert(0) += 1;
            }
        }

        // Find crates
        let crates = Self::find_crates(root).await?;

        Ok(Self {
            root: root.to_string_lossy().to_string(),
            crates,
            file_count,
            total_size,
            language_stats,
        })
    }

    fn classify_file(path: &Path, ext: &str) -> FileType {
        let path_str = path.to_string_lossy();
        match ext {
            "rs" => {
                if path_str.contains("/tests/") || path_str.contains("#[test]") {
                    FileType::Test
                } else {
                    FileType::Rust
                }
            }
            "ts" => FileType::TypeScript,
            "js" => FileType::JavaScript,
            "py" => FileType::Python,
            "md" => FileType::Markdown,
            "json" => FileType::Json,
            "yaml" | "yml" => FileType::Yaml,
            "toml" => FileType::Toml,
            "rs" | "toml" | "json" => FileType::Config,
            _ => FileType::Other,
        }
    }

    fn file_type_to_lang(ft: &FileType) -> String {
        match ft {
            FileType::Rust => "Rust".to_string(),
            FileType::TypeScript => "TypeScript".to_string(),
            FileType::JavaScript => "JavaScript".to_string(),
            FileType::Python => "Python".to_string(),
            FileType::Markdown => "Markdown".to_string(),
            FileType::Json => "JSON".to_string(),
            FileType::Yaml => "YAML".to_string(),
            FileType::Toml => "TOML".to_string(),
            FileType::Config => "Config".to_string(),
            FileType::Source => "Source".to_string(),
            FileType::Test => "Test".to_string(),
            FileType::Binary => "Binary".to_string(),
            FileType::Image => "Image".to_string(),
            FileType::Other => "Other".to_string(),
        }
    }

    async fn find_crates<P: AsRef<Path>>(root: P) -> Result<Vec<CrateInfo>> {
        let mut crates = vec![];
        let mut walkdir = tokio::fs::read_dir(root.as_ref()).await?;
        while let Some(entry) = walkdir.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let cargo_toml = path.join("Cargo.toml");
                if cargo_toml.is_file() {
                    crates.push(CrateInfo {
                        name: path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string(),
                        path: path.to_string_lossy().to_string(),
                        version: "0.1.0".to_string(),
                    });
                }
            }
        }
        Ok(crates)
    }
}
