//! Struct with generics derives ToolDescription

use forge_tool_macros::ToolDescription;

/// A struct with generic type parameters
#[derive(ToolDescription)]
/// This is a generic test struct.
struct GenericStruct<T> {
    /// The value
    value: T,
}

fn main() {
    let _ = GenericStruct::<i32> {
        value: 42,
    };
    println!("GenericStruct implements ToolDescription");
}
