#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    Python,
    Go,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "rs" => Self::Rust,
            "ts" | "tsx" => Self::TypeScript,
            "py" => Self::Python,
            "go" => Self::Go,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn from_extension_rust() {
        assert!(matches!(Language::from_extension("rs"), Language::Rust));
    }

    #[test]
    fn from_extension_typescript() {
        assert!(matches!(
            Language::from_extension("ts"),
            Language::TypeScript
        ));
        assert!(matches!(
            Language::from_extension("tsx"),
            Language::TypeScript
        ));
    }

    #[test]
    fn from_extension_python() {
        assert!(matches!(Language::from_extension("py"), Language::Python));
    }

    #[test]
    fn from_extension_go() {
        assert!(matches!(Language::from_extension("go"), Language::Go));
    }

    #[test]
    fn from_extension_unknown() {
        assert!(matches!(
            Language::from_extension("java"),
            Language::Unknown
        ));
        assert!(matches!(Language::from_extension(""), Language::Unknown));
        assert!(matches!(Language::from_extension("c"), Language::Unknown));
    }
}
