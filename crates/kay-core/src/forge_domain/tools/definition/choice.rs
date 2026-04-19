use serde::{Deserialize, Serialize};

use crate::forge_domain::ToolName;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum ToolChoice {
    #[default]
    None,
    Auto,
    Required,
    Call(ToolName),
}
