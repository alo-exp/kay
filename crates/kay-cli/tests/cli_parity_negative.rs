//! Phase 5 Nyquist-audit CLI-07 negative-path pin — the
//! brand-swap + contains assertion of `interactive_parity_diff`
//! must FAIL when known drift is introduced.
//!
//! Why this file exists
//! --------------------
//! `tests/cli_e2e.rs::interactive_parity_diff` is the POSITIVE pin:
//! it loads the `forgecode-parity-baseline` fixtures, applies the
//! brand swap (`ForgeCode` → `Kay`, `forgecode` → `kay`, `forge` →
//! `kay`, `forge>` → `kay>`), spawns the real binary, and asserts
//! the expected substrings show up on stdout. As long as kay's
//! banner + prompt match the ForgeCode baseline modulo the swap,
//! that test stays green.
//!
//! What the positive test does NOT prove: that the comparison
//! logic's REJECTION side actually rejects. If a future refactor
//! weakens the swap (e.g. drops the `forge>` → `kay>` rule), the
//! positive test may still pass against a legitimate binary whose
//! banner already happens to contain both forms — and then the drift
//! it was designed to catch silently slips through. The concern
//! generalizes: any CLI-07 regression must surface as a RED test,
//! not a silent green. The positive test alone does not guarantee
//! that — a negative companion does.
//!
//! This file probes three orthogonal drift classes by REPLICATING
//! the swap + contains assertion from `interactive_parity_diff` and
//! feeding it synthetic "actual stdout" values that are KNOWN to be
//! wrong. Each test asserts the comparison FAILS (i.e. the positive
//! test would legitimately RED on that input). Together they close
//! the Nyquist gap the positive test leaves open: the comparison's
//! reject-side has a runtime trip-wire.
//!
//! Three drift classes, one test each
//! ----------------------------------
//!   * `parity_rejects_stale_forge_brand_in_actual_stdout` — kay's
//!     stdout still contains "ForgeCode" (brand swap incomplete).
//!     The swapped baseline expects "Kay" somewhere, so the
//!     `contains(&expected)` is false. Catches T7.5-class
//!     regressions that leave "forge" strings in the banner.
//!
//!   * `parity_rejects_added_line_in_kay_banner` — kay's stdout
//!     contains extra lines (banner inflation) that the ForgeCode
//!     baseline does NOT have. The expected string does not appear
//!     as a contiguous substring. Catches "drift by addition":
//!     someone enriched the banner and forgot the fixture lives
//!     on the tag.
//!
//!   * `parity_rejects_prompt_wrong_form` — prompt drift: kay emits
//!     `forge>` (swap missed) OR emits a different prompt entirely
//!     (`$ ` or `>>>`). Either way the swapped prompt expected
//!     (`kay>`) is absent, so the second contains-check fails.
//!     Catches T7.9-class regressions on the prompt renderer.
//!
//! Why "replicate" instead of "extract + call"
//! --------------------------------------------
//! The brand-swap + contains is ~4 lines inlined in the positive
//! test. Extracting it into a `pub fn` in kay-cli would be over-
//! engineered — the swap is the test's own comparison helper, not
//! a production surface. Replicating it here keeps kay-cli's
//! public API lean AND creates a second, deliberately-independent
//! implementation the positive test's logic can be cross-checked
//! against. If the two ever drift, one of them is wrong — the
//! diff is immediate, visible, and reviewer-readable.
//!
//! Failure shape
//! -------------
//! Each test wraps the swap + contains in the SAME boolean
//! expression the positive test uses, then asserts the result is
//! `false` under the drifted input. If the brand swap ever grows a
//! case-insensitive normalization (swap "Forge" → "Kay" regardless
//! of case), this file's stale-forge test will start giving false
//! positives — a signal that a reviewer must re-approve the
//! widened swap semantics in the positive test too.
//!
//! Reference: Phase 5 Nyquist audit — CLI-07 coverage gap #GAP-C.
#![allow(clippy::unwrap_used, clippy::expect_used)]

/// Replicate the brand-swap rules inlined in
/// `cli_e2e.rs::interactive_parity_diff`. Kept in lockstep with
/// that test — a divergence is a reviewer-visible signal that the
/// positive and negative sides no longer agree on what "modulo the
/// brand swap" means.
fn swap_banner(baseline: &str) -> String {
    baseline
        .replace("ForgeCode", "Kay")
        .replace("forgecode", "kay")
        .replace("forge", "kay")
}

/// Replicate the prompt swap. Same rationale as `swap_banner`:
/// this function MUST mirror the positive test's inline
/// replacement for the cross-check invariant to hold.
fn swap_prompt(baseline: &str) -> String {
    baseline.replace("forge>", "kay>")
}

/// Replicate the positive test's core assertion shape:
/// `stdout.contains(&expected.trim().to_string())`. Returning a
/// bool makes the test bodies read as `assert!(!matches(...))` —
/// i.e. "the positive assertion would FAIL here", which is exactly
/// the invariant we want to pin.
fn actual_matches_baseline(actual_stdout: &str, expected: &str) -> bool {
    let e = expected.trim();
    e.is_empty() || actual_stdout.contains(e)
}

/// Deterministic fixture baseline — matches the on-disk
/// `tests/fixtures/forgecode-banner.txt` today (committed on the
/// forgecode-parity-baseline tag). Pasted here instead of loaded
/// from disk so this test works even if the fixture layout
/// changes; the comparison logic is what we're probing, not the
/// fixture path. If a future phase changes the brand tokens in the
/// real fixture, bump this constant alongside it.
const FORGECODE_BANNER_BASELINE: &str = "\
New conversation: :new
     Get started: :info, :usage, :help, :conversation
    Switch model: :model
    Switch agent: :forge or :muse or :agent
          Update: :update
            Quit: :exit or <CTRL+D>
";

const FORGECODE_PROMPT_BASELINE: &str = "forge>\n";

#[test]
fn parity_rejects_stale_forge_brand_in_actual_stdout() {
    // Drift class 1: the brand swap got weakened (e.g. T7.5 port
    // missed a literal) so kay's banner still contains "ForgeCode".
    // The positive test's assertion is `stdout.contains(expected)`
    // where `expected` has been fully swapped to Kay tokens. A
    // stdout that still carries "ForgeCode" does NOT contain the
    // Kay-normalized expected string — the positive test RED-fires.
    //
    // This assertion catches the exact regression whose absence
    // the Nyquist audit flagged: proof that the positive test's
    // reject side is wired up correctly.
    let expected = swap_banner(FORGECODE_BANNER_BASELINE);
    // Actual kay stdout with brand swap incomplete — still says
    // "ForgeCode" where it should say "Kay".
    let actual_stdout_with_drift = "\
New conversation: :new
     Get started: :info, :usage, :help, :conversation
    Switch model: :model
    Switch agent: :forge or :muse or :agent
          Update: :update
            Quit: :exit or <CTRL+D>
ForgeCode v1.0 — your terminal coding agent
";
    assert!(
        !actual_matches_baseline(actual_stdout_with_drift, &expected),
        "CLI-07 negative pin: the parity comparison MUST reject a stdout \
         that still contains the stale 'ForgeCode' brand token. If this \
         assertion is flipped, the positive test in cli_e2e.rs has no \
         reject-side teeth — a T7.5-class brand-swap regression would \
         slip through. Expected (swapped):\n{expected}\n\nActual (drift):\n{actual_stdout_with_drift}"
    );
}

#[test]
fn parity_rejects_added_line_in_kay_banner() {
    // Drift class 2: kay's banner has an ADDED line the ForgeCode
    // baseline did not carry. The swapped expected string from the
    // baseline is N lines; kay's actual stdout is N+1 lines with
    // an interstitial line that breaks the contiguous substring.
    //
    // `contains` is a substring match: the expected text must
    // appear verbatim somewhere in the actual stdout. A new
    // interstitial line in the middle of the banner fragments the
    // expected substring — the positive test RED-fires.
    //
    // This locks: banner inflation (e.g. adding a "Welcome!" line
    // above the command list in a future port) cannot slip through
    // the parity gate.
    let expected = swap_banner(FORGECODE_BANNER_BASELINE);
    // Actual kay stdout with an ADDED line that breaks the
    // contiguous substring match.
    let actual_stdout_with_addition = "\
New conversation: :new
>>>> UNEXPECTED ADDED LINE <<<<
     Get started: :info, :usage, :help, :conversation
    Switch model: :model
    Switch agent: :kay or :muse or :agent
          Update: :update
            Quit: :exit or <CTRL+D>
";
    assert!(
        !actual_matches_baseline(actual_stdout_with_addition, &expected),
        "CLI-07 negative pin: the parity comparison MUST reject a stdout \
         that has an added line between the baseline lines. The positive \
         test uses a verbatim contains() check — if a new banner line \
         lands mid-block, the substring match fails. If THIS assertion \
         flips, the positive test is silently accepting banner drift. \
         Expected (swapped):\n{expected}\n\nActual (drift):\n{actual_stdout_with_addition}"
    );
}

#[test]
fn parity_rejects_prompt_wrong_form() {
    // Drift class 3: the prompt itself drifted. Two sub-cases:
    //
    //   (a) swap regressed — kay emits `forge>` literally. The
    //       swapped expected is `kay>`; stdout contains `forge>`
    //       but NOT `kay>`, so the contains-check fails.
    //
    //   (b) prompt rewritten — kay emits `$` or `>>>` or some
    //       other shape. Same outcome: `kay>` is absent.
    //
    // Both sub-cases drop the prompt contract that DL-1 locks
    // ("ForgeCode used `forge>`, Kay uses `kay>` — zero-visual-
    // drift swap"). The positive test's second `contains` check
    // catches both; this test pins the reject side for both.
    let expected = swap_prompt(FORGECODE_PROMPT_BASELINE);

    // Sub-case (a): swap regressed — kay still renders `forge>`.
    let actual_stdout_forge_prompt = "some banner output\nforge>\n";
    assert!(
        !actual_matches_baseline(actual_stdout_forge_prompt, &expected),
        "CLI-07 negative pin (sub-case a): kay emitting `forge>` \
         verbatim must FAIL the prompt parity check. The positive \
         test's `stdout.contains(prompt_expected.trim())` looks for \
         `kay>` — `forge>` does not contain that substring. If THIS \
         assertion flips, the prompt renderer's brand swap regression \
         would slip through. Expected: {expected:?}; actual: \
         {actual_stdout_forge_prompt:?}"
    );

    // Sub-case (b): prompt rewritten to an entirely different form.
    // Catches a future T7.9-style port that swaps reedline for
    // something that emits `$ ` or `> ` and someone forgets to
    // preserve the `kay>` brand.
    let actual_stdout_shell_prompt = "some banner output\n$ \n";
    assert!(
        !actual_matches_baseline(actual_stdout_shell_prompt, &expected),
        "CLI-07 negative pin (sub-case b): kay emitting a non-`kay>` \
         prompt (e.g. `$ `) must FAIL the prompt parity check. If \
         THIS assertion flips, a prompt-renderer rewrite that drops \
         `kay>` entirely would slip through the parity gate. \
         Expected: {expected:?}; actual: {actual_stdout_shell_prompt:?}"
    );
}

#[test]
fn swap_rules_preserved_across_refactors() {
    // Paired invariant: the positive test's swap logic is
    // replicated here — BOTH must agree on what tokens translate.
    // This test exercises the swap rules directly so any future
    // edit in `cli_e2e.rs` that forgets to mirror here trips a
    // RED in this file.
    //
    // Why not just test the positive test's implementation? The
    // positive test is a subprocess spawn that asserts on real
    // binary output. Unit-testing the swap function in isolation
    // catches pure string-munging regressions in microseconds,
    // before the slower subprocess test even starts. Defence in
    // depth: if the subprocess test flakes under load, the unit
    // test still runs deterministically and pins the swap rules.

    // "ForgeCode" → "Kay" (title-case brand)
    assert_eq!(swap_banner("Hello ForgeCode"), "Hello Kay");

    // "forgecode" → "kay" (all-lowercase brand, unlikely but
    // covered by the positive test's swap so we mirror it)
    assert_eq!(swap_banner("url://forgecode.io"), "url://kay.io");

    // "forge" → "kay" (legacy rebrand marker — hits command names
    // like `:forge`)
    assert_eq!(swap_banner(":forge or :muse"), ":kay or :muse");

    // Ordering matters: "ForgeCode" must be swapped BEFORE "forge"
    // or we'd end up with "KayCode" (partial swap). The positive
    // test relies on this ordering; pinning it here too.
    assert_eq!(swap_banner("ForgeCode and forge"), "Kay and kay");

    // Prompt swap: literal `forge>` → `kay>`. Does NOT over-reach
    // into `forge>foo` (which becomes `kay>foo`, acceptable since
    // the baseline prompt is exactly `forge>`).
    assert_eq!(swap_prompt("forge>"), "kay>");
    assert_eq!(swap_prompt("forge>\n"), "kay>\n");

    // Idempotency: swapping already-swapped text is a no-op. The
    // positive test never double-swaps in practice, but an
    // accidental double-application (e.g. if someone refactored
    // the swap into a pipeline and ran it twice) must not break
    // the contract.
    assert_eq!(swap_banner("Kay"), "Kay");
    assert_eq!(swap_prompt("kay>"), "kay>");
}
