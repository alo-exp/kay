//! Help System for Kay CLI
//!
//! Provides contextual help for commands and topics.

/// Print general help
pub fn print_general_help() {
    println!("Kay — Terminal Coding Agent");
    println!();
    println!("USAGE:");
    println!("  kay [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  run       Run a headless agent turn");
    println!("  eval      Run evaluation harnesses");
    println!("  tools     Introspect the built-in tool registry");
    println!("  session   Manage sessions");
    println!("  rewind    Rewind to pre-edit snapshot");
    println!("  build     Build the workspace");
    println!("  check     Type-check the workspace");
    println!("  fmt       Format code");
    println!("  clippy    Run linter");
    println!("  test      Run tests");
    println!("  review    Run code review");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help     Print help");
    println!("  -V, --version  Print version");
    println!();
    println!("EXAMPLES:");
    println!("  kay run --live --prompt \"What is 2+2?\"");
    println!("  kay build -p kay-core");
    println!("  kay session list");
}

/// Print run command help
pub fn print_run_help() {
    println!("kay run — Run a headless agent turn");
    println!();
    println!("USAGE: kay run [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --live                  Use live API (requires MINIMAX_API_KEY)");
    println!("  --offline               Use offline mode (test signals)");
    println!("  --prompt <TEXT>          Prompt to execute");
    println!("  --model <MODEL>         Model to use (default: MiniMax-M2.1)");
    println!("  --max-turns <N>         Maximum turns (default: unlimited)");
    println!("  --persona <PATH>         Custom persona file");
    println!();
    println!("EXAMPLES:");
    println!("  kay run --live --prompt \"Hello world\"");
    println!("  kay run --offline --prompt \"TEST:done\"");
    println!("  kay run --live --model \"MiniMax-M2.1\" --prompt \"Hi\"");
}

/// Print session help
pub fn print_session_help() {
    println!("kay session — Session management");
    println!();
    println!("USAGE: kay session <ACTION>");
    println!();
    println!("ACTIONS:");
    println!("  list    List all sessions");
    println!("  load    Load a session by ID");
    println!("  delete  Delete a session");
    println!();
    println!("EXAMPLES:");
    println!("  kay session list");
    println!("  kay session load <session-id>");
    println!("  kay session delete <session-id> --force");
}

/// Print build help
pub fn print_build_help() {
    println!("kay build — Build the workspace or a crate");
    println!();
    println!("USAGE: kay build [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -p, --crate-name <NAME>  Build specific crate");
    println!("  -r, --release           Build in release mode");
    println!();
    println!("EXAMPLES:");
    println!("  kay build               # Build entire workspace");
    println!("  kay build -p kay-cli    # Build specific crate");
    println!("  kay build --release     # Release build");
}

/// Print error help for common issues
/// Print error help for common issues
pub fn print_error_help(error: &str) {
    println!("Error: {}", error);
    println!();
    
    if error.contains("API key") || error.contains("MINIMAX_API_KEY") {
        println!("To use the live API, set your MiniMax API key:");
        println!("  export MINIMAX_API_KEY=\"your-key-here\"");
        println!();
    }
    
    if error.contains("session") {
        println!("Session issues? Try:");
        println!("  kay session list");
        println!("  kay help session");
    }
}

/// Dispatch help based on topic
/// topic is a simple string representation to avoid circular deps
pub fn dispatch_help(topic: Option<&str>) {
    match topic {
        Some("run") => print_run_help(),
        Some("session") => print_session_help(),
        Some("build") | Some("check") | Some("fmt") | Some("clippy") => print_build_help(),
        _ => print_general_help(),
    }
}