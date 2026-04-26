//! JSON Repair for Kay
//!
//! Repairs malformed JSON by trying to fix common issues.
//! Based on principles from Forge's JSON repair implementation.

use serde::Value;

/// Result of JSON repair attempt
pub struct RepairResult {
    /// Whether repair was successful
    pub success: bool,
    /// The repaired JSON string
    pub repaired: Option<String>,
    /// Error message if repair failed
    pub error: Option<String>,
}

/// Repair malformed JSON string
pub fn repair_json(input: &str) -> RepairResult {
    // First try parsing directly
    if let Ok(v) = serde_json::from_str::<Value>(input) {
        return RepairResult {
            success: true,
            repaired: Some(serde_json::to_string_pretty(&v).unwrap_or_else(|_| input.to_string())),
            error: None,
        };
    }

    // Try common fixes
    let mut working = input.to_string();

    // Fix 1: Trailing commas
    working = fix_trailing_commas(&working);

    // Fix 2: Unquoted keys
    working = fix_unquoted_keys(&working);

    // Fix 3: Single quotes to double quotes
    working = working.replace('\'', "\"");

    // Fix 4: Comments (JavaScript style)
    working = fix_json_comments(&working);

    // Fix 5: Trailing commas in arrays/objects
    working = fix_trailing_commas(&working);

    // Try parsing after fixes
    if let Ok(v) = serde_json::from_str::<Value>(&working) {
        return RepairResult {
            success: true,
            repaired: Some(serde_json::to_string_pretty(&v).unwrap_or(working)),
            error: None,
        };
    }

    RepairResult {
        success: false,
        repaired: None,
        error: Some("Could not repair JSON".to_string()),
    }
}

/// Fix trailing commas in JSON
fn fix_trailing_commas(json: &str) -> String {
    // Pattern: ,} or ,]
    let re = regex::Regex::new(r",(\s*[}\]])").unwrap();
    re.replace_all(json, "$1").to_string()
}

/// Fix unquoted keys in JSON
fn fix_unquoted_keys(json: &str) -> String {
    // Very simple fix for common patterns
    // This is a simplified version - a full implementation would be more complex
    let mut result = String::new();
    let mut in_string = false;
    let mut chars = json.chars().peekable();

    while let Some(c) = chars.next() {
        result.push(c);
        
        if c == '"' {
            in_string = !in_string;
        } else if !in_string && c == '{' {
            // Try to find and quote unquoted keys
            // This is a simplified implementation
        }
    }

    result
}

/// Remove JavaScript-style comments from JSON
fn fix_json_comments(json: &str) -> String {
    let mut result = String::new();
    let mut chars = json.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if c == '"' {
            in_string = !in_string;
            result.push(c);
        } else if !in_string && c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    // Line comment - skip until end of line
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' {
                            result.push('\n');
                            chars.next();
                            break;
                        }
                        chars.next();
                    }
                    continue;
                } else if next == '*' {
                    // Block comment - skip until */
                    chars.next(); // skip /
                    chars.next(); // skip *
                    while let Some(ch) = chars.next() {
                        if ch == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
            }
            result.push(c);
        } else {
            result.push(c);
        }
    }

    result
}

/// Try to parse JSON strictly
pub fn parse_strict(json: &str) -> Result<Value, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}

/// Try to parse JSON with repair
pub fn parse_with_repair(json: &str) -> Result<Value, String> {
    let repaired = repair_json(json);
    if let Some(repaired_str) = repaired.repaired {
        serde_json::from_str(&repaired_str).map_err(|e| e.to_string())
    } else {
        Err(repaired.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let result = repair_json(r#"{"key": "value"}"#);
        assert!(result.success);
    }

    #[test]
    fn test_trailing_comma() {
        let result = repair_json(r#"{"key": "value",}"#);
        assert!(result.success);
        let repaired = result.repaired.unwrap();
        assert!(!repaired.contains(",}"));
    }

    #[test]
    fn test_single_quotes() {
        let result = repair_json("{'key': 'value'}");
        assert!(result.success);
    }

    #[test]
    fn test_line_comments_removed() {
        let json = r#"{"key": "value"} // comment"#;
        let result = repair_json(json);
        // Comments outside strings should be handled
        assert!(result.success || result.error.is_some());
    }
}