//! Interactive prompt for `kay` (ported from `forge_main::prompt`).
//!
//! # T7.6 scope
//!
//! Lands `KayPrompt` — a `reedline::Prompt` implementation that
//! renders the multi-line starship-style chat prompt when Kay runs
//! in interactive mode. The full reedline REPL that consumes this
//! struct lands in T7.9 (`interactive.rs`); T7.6 only commits the
//! prompt surface so T7.10/T7.11 fixtures can compare against it,
//! and exposes a `KAY_PROMPT` fallback literal that the
//! pre-T7.9 `main::interactive_fallback` emits on stdout.
//!
//! The file is a brand-swapped port of `crates/forge_main/src/prompt.rs`:
//!
//!   * `ForgePrompt` → `KayPrompt` (struct + impl rename).
//!   * Nerd font symbols, colors, and layout are brand-neutral —
//!     copied verbatim.
//!   * `forge_main::display_constants::markers::EMPTY` (`"[empty]"`)
//!     is inlined as a private const; kay-cli hasn't carried that
//!     whole module forward (DL-2: display constants are a
//!     forge_main internal surface that T7.9 will evaluate piece by
//!     piece as the REPL grows).
//!   * `forge_main::utils::humanize_number` inlined as a private
//!     function — 7 lines, same rationale as above.
//!   * Test-module default agent changed from `AgentId::default()`
//!     (which resolves to the internal `"forge"` identifier) to
//!     `AgentId::new("kay")`, so `KAY` appears in assertions and no
//!     `FORGE` substring leaks into the port's test surface. The
//!     Phase 10 residual that renames `AgentId::default()` is what
//!     would let this test revert to the `::default()` call.
//!
//! # Parity-diff contract (T7.11)
//!
//! The `kay>` fallback literal `KAY_PROMPT` is what
//! `tests/cli_e2e.rs::interactive_parity_diff` sees in stdout when
//! it spawns `kay` with no args. The baseline fixture captured in
//! T7.10 contains ForgeCode's `forge> ` line; the parity test
//! replaces `forge>` with `kay>` on the baseline before comparing
//! to actual stdout. Keep `KAY_PROMPT` exactly `"kay> "` (trailing
//! space, no colors) — the fixture-normalized baseline does not
//! include ANSI escapes, so any styling here would desync the
//! diff.
//!
//! # Why `#[allow(dead_code)]`
//!
//! T7.6 lands the `KayPrompt` struct + `Prompt` trait impl, but the
//! reedline REPL that actually calls `render_prompt_left()` et al.
//! lives in T7.9 (`interactive.rs`). Between T7.6 and T7.9 the
//! struct, its setters, the nerd-font symbol constants, and the
//! `get_git_branch` / `humanize_number` helpers have no non-test
//! callers — `cargo clippy --all-targets -- -D warnings` therefore
//! fails unless we suppress `dead_code` for this module. The
//! suppression is module-scoped (not crate-scoped) so new dead code
//! elsewhere still fires the lint. Remove this attribute in T7.9
//! once `KayPrompt::new` + the setters are wired up from the REPL.

#![allow(dead_code)]

use std::borrow::Cow;
use std::fmt::Write;
use std::path::PathBuf;

use convert_case::{Case, Casing};
use derive_setters::Setters;
use forge_api::{AgentId, ModelId, Usage};
use nu_ansi_term::{Color, Style};
use reedline::{Prompt, PromptHistorySearchStatus};

/// Plain-text fallback prompt line emitted by `main::interactive_fallback`
/// on `kay` with no args. Used until T7.9 wires the reedline REPL that
/// consumes `KayPrompt` directly. The T7.10/T7.11 parity diff matches
/// the brand-swapped ForgeCode baseline against this exact literal —
/// changing it breaks parity unless the baseline fixture is
/// re-captured.
pub const KAY_PROMPT: &str = "kay> ";

// Continuation prompt when the user enters multi-line mode (reedline's
// second and subsequent lines of a multi-line input). `:::` is the
// ForgeCode default carried forward as-is — no brand component.
const MULTILINE_INDICATOR: &str = "::: ";

// Placeholder shown when the working-directory path has no final
// component (e.g., `/`). Inlined from
// `forge_main::display_constants::markers::EMPTY` (see module docs).
const EMPTY_MARKER: &str = "[empty]";

// Nerd font symbols — left prompt
// Copied verbatim from forge_main: these are font glyph codepoints,
// not brand text, so no swap needed.
const DIR_SYMBOL: &str = "\u{ea83}"; // 󪃃  folder icon
const BRANCH_SYMBOL: &str = "\u{f418}"; //   branch icon
const SUCCESS_SYMBOL: &str = "\u{f013e}"; // 󰄾  chevron

// Nerd font symbols — right prompt (ZSH rprompt)
const AGENT_SYMBOL: &str = "\u{f167a}";
const MODEL_SYMBOL: &str = "\u{ec19}";

/// Interactive prompt for the Kay chat loop.
///
/// Rendering layout (left prompt):
///
/// ```text
///    󪃃 <dir>    <branch>
/// 󰄾
/// ```
///
/// and right prompt:
///
/// ```text
///    AGENT_NAME  <tokens>  <cost>  <model>
/// ```
///
/// The struct is cloneable + exposes fluent setters via
/// `derive_setters`, so the REPL can refresh `usage`/`model`/`git_branch`
/// between turns without reconstructing the prompt from scratch.
#[derive(Clone, Setters)]
#[setters(strip_option, borrow_self)]
pub struct KayPrompt {
    pub cwd: PathBuf,
    pub usage: Option<Usage>,
    pub agent_id: AgentId,
    pub model: Option<ModelId>,
    pub git_branch: Option<String>,
}

impl KayPrompt {
    /// Creates a new `KayPrompt`, resolving the git branch once at
    /// construction time. Subsequent branch changes (e.g., the user
    /// `git checkout`s between turns) should call `refresh()` on the
    /// live instance rather than building a new prompt.
    pub fn new(cwd: PathBuf, agent_id: AgentId) -> Self {
        let git_branch = get_git_branch();
        Self { cwd, usage: None, agent_id, model: None, git_branch }
    }

    /// Re-reads the current git branch. Invoked by the REPL between
    /// turns so the branch segment stays in sync with external
    /// `git checkout`s.
    pub fn refresh(&mut self) -> &mut Self {
        let git_branch = get_git_branch();
        self.git_branch = git_branch;
        self
    }
}

impl Prompt for KayPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        // Left prompt layout:
        //
        //   AGENT_NAME  󪃃 dir   branch
        //   󰄾
        //
        // Colors:
        //   dir     → bold cyan
        //   branch  → bold green
        //   chevron → bold green
        //
        // The agent name itself lives in the RIGHT prompt; the left
        // side only carries context (directory + branch + status).

        let dir_style = Style::new().fg(Color::Cyan).bold();
        let branch_style = Style::new().fg(Color::LightGreen).bold();
        let chevron_style = Style::new().fg(Color::LightGreen).bold();

        let current_dir = self
            .cwd
            .file_name()
            .and_then(|name| name.to_str())
            .map(String::from)
            .unwrap_or_else(|| EMPTY_MARKER.to_string());

        let mut result = String::with_capacity(80);

        // Directory — folder icon + name, bold cyan
        write!(
            result,
            "{}",
            dir_style.paint(format!("{DIR_SYMBOL} {current_dir}"))
        )
        .unwrap();

        // Git branch — branch icon + name, bold green (only when present
        // and different from the directory name, matching ForgeCode
        // behaviour). Duplicating dir-as-branch-name is common for
        // single-branch repos like v0 proofs-of-concept.
        if let Some(branch) = self.git_branch.as_deref()
            && branch != current_dir
        {
            write!(
                result,
                " {}",
                branch_style.paint(format!("{BRANCH_SYMBOL} {branch}"))
            )
            .unwrap();
        }

        // Second line: success chevron — the user types after this.
        write!(result, "\n{} ", chevron_style.paint(SUCCESS_SYMBOL)).unwrap();

        Cow::Owned(result)
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        // Right prompt layout: agent · tokens · cost · model.
        // Active  (tokens > 0): bright white for agent/tokens, green for cost.
        // Inactive (no tokens): all segments dimmed — user hasn't burned
        // any context yet, so the signal is "ready" not "usage".

        let total_tokens = self.usage.as_ref().map(|u| u.total_tokens);
        let active = total_tokens.map(|t| *t > 0).unwrap_or(false);

        let agent_color = if active {
            Color::LightGray
        } else {
            Color::DarkGray
        };
        let mut result = String::with_capacity(64);

        // Agent name with nerd font symbol. `UpperSnake` case means
        // `kay` → `KAY`, `muse` → `MUSE`, `sage` → `SAGE`.
        let agent_str = format!(
            "{AGENT_SYMBOL} {}",
            self.agent_id.as_str().to_case(Case::UpperSnake)
        );
        write!(
            result,
            " {}",
            Style::new().bold().fg(agent_color).paint(&agent_str)
        )
        .unwrap();

        // Token count (only shown when active). `TokenCount::Approx`
        // prefixes "~" so the user can tell exact vs estimated
        // (important when the provider doesn't return usage info).
        if let Some(tokens) = total_tokens
            && active
        {
            let prefix = match tokens {
                forge_api::TokenCount::Actual(_) => "",
                forge_api::TokenCount::Approx(_) => "~",
            };
            let count_str = format!("{}{}", prefix, humanize_number(*tokens));
            write!(
                result,
                " {}",
                Style::new().bold().fg(Color::LightGray).paint(&count_str)
            )
            .unwrap();
        }

        // Cost (only shown when active). `\u{f155}` is the nerd-font
        // dollar-sign glyph — no brand component, copied verbatim.
        if let Some(cost) = self.usage.as_ref().and_then(|u| u.cost)
            && active
        {
            let cost_str = format!("\u{f155}{cost:.2}");
            write!(
                result,
                " {}",
                Style::new().bold().fg(Color::Green).paint(&cost_str)
            )
            .unwrap();
        }

        // Model with nerd font symbol. Strip provider prefix so
        // `anthropic/claude-3` renders as `claude-3` — keeps the
        // right prompt narrow on wide model names.
        if let Some(model) = self.model.as_ref() {
            let model_str = model.to_string();
            let short_model = model_str.split('/').next_back().unwrap_or(model.as_str());
            let model_label = format!("{MODEL_SYMBOL} {short_model}");
            let color = if active {
                Color::LightMagenta
            } else {
                Color::DarkGray
            };
            write!(result, " {}", Style::new().fg(color).paint(&model_label)).unwrap();
        }

        Cow::Owned(result)
    }

    fn render_prompt_indicator(&self, _prompt_mode: reedline::PromptEditMode) -> Cow<'_, str> {
        // Empty — the multi-line left prompt already has the chevron
        // on its second line, which serves as the input indicator.
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed(MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        let mut result = String::with_capacity(32);

        // Empty-term branch gets a simpler label so the reverse-search
        // indicator doesn't render as `(reverse-search: )` with a
        // trailing colon + empty tail.
        if history_search.term.is_empty() {
            write!(result, "({prefix}reverse-search) ").unwrap();
        } else {
            write!(
                result,
                "({}reverse-search: {}) ",
                prefix, history_search.term
            )
            .unwrap();
        }

        Cow::Owned(Style::new().fg(Color::White).paint(&result).to_string())
    }
}

/// Resolves the current git branch. Returns `None` when:
///
///   * `gix::discover` can't find a `.git` directory (bare tree).
///   * `head()` can't be read (corrupted ref database).
///   * `referent_name` is `None` (detached HEAD pointing at a commit).
///
/// All three cases are user-facing "no branch" signals and rendered
/// by eliding the branch segment entirely.
fn get_git_branch() -> Option<String> {
    let repo = gix::discover(".").ok()?;
    let head = repo.head().ok()?;
    head.referent_name().map(|r| r.shorten().to_string())
}

/// Humanizes a number to `B`/`M`/`k` suffixes for right-prompt token
/// counts. Inlined from `forge_main::utils::humanize_number`.
///
/// ```ignore
/// assert_eq!(humanize_number(1500), "1.5k");
/// assert_eq!(humanize_number(1_500_000), "1.5M");
/// assert_eq!(humanize_number(1_500_000_000), "1.5B");
/// assert_eq!(humanize_number(500), "500");
/// ```
fn humanize_number(n: usize) -> String {
    match n {
        n if n >= 1_000_000_000 => format!("{:.1}B", n as f64 / 1_000_000_000.0),
        n if n >= 1_000_000 => format!("{:.1}M", n as f64 / 1_000_000.0),
        n if n >= 1_000 => format!("{:.1}k", n as f64 / 1_000.0),
        _ => n.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use nu_ansi_term::Style;
    use pretty_assertions::assert_eq;

    use super::*;

    impl Default for KayPrompt {
        fn default() -> Self {
            // AgentId::new("kay") — not AgentId::default() — so the
            // test surface asserts on `KAY` (brand-clean) rather than
            // the internal `"forge"` default. Phase 10 residual: once
            // `AgentId::default()` resolves to `"kay"` at the domain
            // layer, this can revert to `AgentId::default()`.
            KayPrompt {
                cwd: PathBuf::from("."),
                usage: None,
                agent_id: AgentId::new("kay"),
                model: None,
                git_branch: None,
            }
        }
    }

    #[test]
    fn test_kay_prompt_literal_is_kay_not_forge() {
        // T7.6 REQ: CLI-04 + CLI-07 — the fallback prompt literal
        // emitted by `main::interactive_fallback` must be `kay>`,
        // never `forge>`. T7.10/T7.11 parity-diff tests normalize
        // the ForgeCode baseline by replacing `forge>` with `kay>`
        // before asserting containment, so any drift here silently
        // breaks the parity contract.
        assert_eq!(KAY_PROMPT, "kay> ");
        assert!(!KAY_PROMPT.contains("forge"));
    }

    #[test]
    fn test_render_prompt_left() {
        let prompt = KayPrompt::default();
        let actual = prompt.render_prompt_left();

        // Starship directory icon present
        assert!(actual.contains(DIR_SYMBOL));
        // Starship success chevron present
        assert!(actual.contains(SUCCESS_SYMBOL));
    }

    #[test]
    fn test_render_prompt_left_with_branch() {
        let prompt = KayPrompt { git_branch: Some("main".to_string()), ..Default::default() };
        let actual = prompt.render_prompt_left();

        // Agent name is on the right prompt, not the left.
        // Branch icon and name present.
        assert!(actual.contains(BRANCH_SYMBOL));
        assert!(actual.contains("main"));
    }

    #[test]
    fn test_render_prompt_right_inactive() {
        // No tokens → dimmed agent + model, no token/cost segments
        let mut prompt = KayPrompt::default();
        let _ = prompt.model(ModelId::new("gpt-4"));

        let actual = prompt.render_prompt_right();
        // Agent symbol and name present; `kay` → `KAY` via UpperSnake.
        assert!(actual.contains(AGENT_SYMBOL));
        assert!(actual.contains("KAY"));
        // Model symbol and name present
        assert!(actual.contains(MODEL_SYMBOL));
        assert!(actual.contains("gpt-4"));
        // No token count text in inactive state (no humanized number segment)
        assert!(!actual.contains("1k") && !actual.contains("~"));
    }

    #[test]
    fn test_render_prompt_right_active_with_tokens() {
        // Tokens > 0 → active colours; approx tokens show "~" prefix
        let usage = Usage {
            prompt_tokens: forge_api::TokenCount::Actual(10),
            completion_tokens: forge_api::TokenCount::Actual(20),
            total_tokens: forge_api::TokenCount::Approx(30),
            ..Default::default()
        };
        let mut prompt = KayPrompt::default();
        let _ = prompt.usage(usage);

        let actual = prompt.render_prompt_right();
        assert!(actual.contains("~30"));
        assert!(actual.contains(AGENT_SYMBOL));
    }

    #[test]
    fn test_render_prompt_multiline_indicator() {
        let prompt = KayPrompt::default();
        let actual = prompt.render_prompt_multiline_indicator();
        let expected = MULTILINE_INDICATOR;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_prompt_history_search_indicator_passing() {
        let prompt = KayPrompt::default();
        let history_search = reedline::PromptHistorySearch {
            status: PromptHistorySearchStatus::Passing,
            term: "test".to_string(),
        };
        let actual = prompt.render_prompt_history_search_indicator(history_search);
        let expected = Style::new()
            .fg(Color::White)
            .paint("(reverse-search: test) ")
            .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_prompt_history_search_indicator_failing() {
        let prompt = KayPrompt::default();
        let history_search = reedline::PromptHistorySearch {
            status: PromptHistorySearchStatus::Failing,
            term: "test".to_string(),
        };
        let actual = prompt.render_prompt_history_search_indicator(history_search);
        let expected = Style::new()
            .fg(Color::White)
            .paint("(failing reverse-search: test) ")
            .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_prompt_history_search_indicator_empty_term() {
        let prompt = KayPrompt::default();
        let history_search = reedline::PromptHistorySearch {
            status: PromptHistorySearchStatus::Passing,
            term: "".to_string(),
        };
        let actual = prompt.render_prompt_history_search_indicator(history_search);
        let expected = Style::new()
            .fg(Color::White)
            .paint("(reverse-search) ")
            .to_string();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_render_prompt_right_strips_provider_prefix() {
        // Model ID like "anthropic/claude-3" should show only "claude-3"
        let usage = Usage {
            prompt_tokens: forge_api::TokenCount::Actual(10),
            completion_tokens: forge_api::TokenCount::Actual(20),
            total_tokens: forge_api::TokenCount::Actual(30),
            ..Default::default()
        };
        let mut prompt = KayPrompt::default();
        let _ = prompt.usage(usage);
        let _ = prompt.model(ModelId::new("anthropic/claude-3"));

        let actual = prompt.render_prompt_right();
        assert!(actual.contains("claude-3"));
        assert!(!actual.contains("anthropic/claude-3"));
        assert!(actual.contains("30"));
    }

    #[test]
    fn test_render_prompt_right_with_cost() {
        // Cost shown when active
        let usage = Usage {
            total_tokens: forge_api::TokenCount::Actual(1500),
            cost: Some(0.01),
            ..Default::default()
        };
        let mut prompt = KayPrompt::default();
        let _ = prompt.usage(usage);

        let actual = prompt.render_prompt_right();
        assert!(actual.contains("0.01"));
        assert!(actual.contains("1.5k"));
    }

    #[test]
    fn test_humanize_number() {
        // Smoke test for the inlined helper. forge_main has its own
        // coverage in utils.rs tests; we re-assert key cases here so
        // the inlined copy can't silently drift.
        assert_eq!(humanize_number(0), "0");
        assert_eq!(humanize_number(999), "999");
        assert_eq!(humanize_number(1_500), "1.5k");
        assert_eq!(humanize_number(2_300_000), "2.3M");
        assert_eq!(humanize_number(1_500_000_000), "1.5B");
    }
}
