//! R-1 regression — `execute_commands::should_use_pty` tokenizes on
//! `[\s;|&]` rather than picking only the first whitespace-separated
//! token.
//!
//! Before R-1 the heuristic read only the first token, so commands like
//! `cd /tmp; vim file` slipped through without a PTY because the first
//! token `cd` is not in the denylist. The fix walks every shell-level
//! token (split on `[\s;|&]` while respecting `"` / `'` quoted runs).
//!
//! These six tests lock:
//!   1. semicolon-separated commands detect downstream PTY tokens;
//!   2. pipes do the same;
//!   3. `&` (background) and `&&` (logical-and) both split correctly;
//!   4. multi-space / multi-separator runs do not swallow tokens;
//!   5. plain commands with no PTY word still bypass the PTY path;
//!   6. quoted substrings containing separators are left alone
//!      (both directions: real unquoted separators still split, and a
//!      PTY word appearing only inside a quoted string must not trip
//!      the heuristic).
//!
//! Reference: `.planning/REQUIREMENTS.md` R-1,
//! `.planning/phases/05-agent-loop/05-PLAN.md` Wave 6a.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use kay_tools::should_use_pty;

#[test]
fn pty_needed_for_semicolon_separated_commands() {
    // The first token is `cd`, which is not PTY-requiring on its own.
    // After the `;` the next pipeline is `vim file.txt`, which IS
    // PTY-requiring. Pre-R-1 this returned false; post-R-1 it must
    // return true because tokenization crosses the `;`.
    assert!(
        should_use_pty("cd /tmp; vim file.txt", false),
        "semicolon must split into a new pipeline, and `vim` on the \
         right-hand side must engage the PTY"
    );
}

#[test]
fn pty_needed_for_pipe() {
    // `git log` → `less`. Pre-R-1, the naive first-token check picked
    // `git` and missed `less`. Post-R-1, pipe-splitting surfaces `less`.
    assert!(
        should_use_pty("git log --oneline | less", false),
        "pipe separator must expose `less` as a PTY-requiring token"
    );
}

#[test]
fn pty_needed_for_ampersand_bg() {
    // Both `&` (background) and `&&` (logical-and) are members of the
    // `[\s;|&]` character class. Splitting on `&` turns
    // `make build && htop` into tokens that include `htop`, which is
    // in the denylist.
    assert!(
        should_use_pty("make build && htop", false),
        "`&&` sequencing must expose `htop` on the right-hand side"
    );
    // Pure background form: `long-running-thing & htop`.
    assert!(
        should_use_pty("long-running-thing & htop", false),
        "single `&` background operator must also split into tokens"
    );
}

#[test]
fn pty_needed_for_multi_space_separator() {
    // Runs of whitespace + `;` must collapse — we should not leave
    // empty tokens lying around that would mask real PTY words after
    // them. This case mixes tabs, spaces, and a semicolon to exercise
    // the `[\s;|&]+` collapse behavior.
    let cmd = "echo   done\t\t;   vim   file";
    assert!(
        should_use_pty(cmd, false),
        "multi-whitespace + semicolon runs must still surface `vim`"
    );
}

#[test]
fn pty_not_needed_for_simple_command() {
    // No separator, no PTY-requiring token → no PTY. Guard rail that
    // the new tokenizer does not turn into a false-positive factory.
    assert!(
        !should_use_pty("ls -la", false),
        "plain, non-PTY-requiring command must never engage the PTY path"
    );
    assert!(
        !should_use_pty("cargo build --release", false),
        "plain cargo invocation must never engage the PTY path"
    );
}

#[test]
fn pty_needed_for_quoted_substring_containing_separator() {
    // Two-direction lock on quote-awareness.
    //
    // (a) A separator that only appears INSIDE a double-quoted string
    //     must not split. The real, unquoted `|` after the closing
    //     quote is what surfaces `less` — that is the tokenizer's job.
    assert!(
        should_use_pty(r#"echo "step;one" | less"#, false),
        "semicolon inside quotes must be inert, but the real pipe \
         after the quote must still surface `less`"
    );
    // (b) A PTY-requiring word that only appears inside a quoted
    //     substring MUST NOT trigger the heuristic. Without quote-
    //     awareness a naive `[\s;|&]+` regex split would emit
    //     `"less`, `than`, `zero"` and wrongly match `less`.
    assert!(
        !should_use_pty(r#"echo "less than zero""#, false),
        "a PTY-requiring word that only appears inside double quotes \
         must not engage the PTY path"
    );
    // Mirror check with single quotes.
    assert!(
        !should_use_pty(r#"echo 'vim is a text editor'"#, false),
        "a PTY-requiring word inside single quotes must also be inert"
    );
}
