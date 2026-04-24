# AI Model Rankings — Comprehensive Cost-Weighted Analysis
**Version 2.5 | April 24, 2026 | 11 Models | 15 Dimensions**

> **Change log:**
> **v2.0 → v2.1:** Added GPT-5.5 (launched April 23, 2026). All 15 normalized score columns renormalized.
> **v2.1 → v2.1.1:** Corrected GPT-5.5 cost rank (was erroneously rank 11; corrected to rank 9 at $11.25/1M < Opus $30.00/1M).
> **v2.1.1 → v2.2 (full matrix audit):** Fixed 20 cell errors across 7 dimensions — all were pairwise swap errors during normalization.
> **v2.2 → v2.3 (Terminal-Bench 2.0 data refresh):** Replaced stale TB 2.0 column with current leaderboard data; GPT-5.5 confirmed #1 at 82.7%; GPT-5.4 corrected from 94.5% to 75.1%. Kimi overtook Gemini for #1 (73.1 vs 72.7).
> **v2.3 → v2.4 (full provenance audit):** Systematic cross-check of every benchmark value against authoritative leaderboard sources. All benchmark columns except TB 2.0 corrected from stale late-2024/early-2025 data. Cost column: 9/11 API prices corrected (Claude Opus $30→$10 blended; Gemini $0.75→$4.50; Kimi $0.48→$1.71; etc.). τ²-bench, GPQA, HLE, OSWorld, SWE-bench, SciCode, GDP, Speed, LCB, AIME corrected throughout. Ranking outcome: Gemini #1 (68.3), Kimi #2 (63.1), KAT #3 (59.3), GPT-5.5 drops to #8 (47.7).
> **v2.4 → v2.5 (cost methodology overhaul):** **Root cause:** API pricing ($/1M tokens) does not represent true cost to users — actual cost = price × token usage, which varies dramatically by model verbosity and task type. **Fix:** Replaced API-price-based cost dimension with **Artificial Analysis Intelligence Index Eval Cost** — the total USD cost to run AA's full standardized benchmark suite on each model. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models (source: artificialanalysis.ai). **Impact:** Cost ranking completely reshuffled. KAT-Coder-Pro-V2 becomes cheapest to run ($73.49 eval cost, rank 1, score 100.0); Claude Opus 4.6 becomes most expensive ($4,969.68, rank 11, score 0.0). GPT-5.5 moves from rank 11 (API price: most expensive) to rank 8 ($3,357 eval cost — cheaper than Sonnet $3,959 and both Opus models), gaining +8.4 pts. GLM 5.1 rises from rank 5→4. Gemini rank 6→5. Kimi drops rank 4→6 ($947.87 eval cost — marginally more expensive than Gemini $892.28 due to higher token usage on benchmark tasks). **GPT-5.5 missing data confirmed still ⊘:** Exhaustive search confirms all 7 missing dimensions (IFBench, SWE-bench Verified, LCB, Speed, LCR, AIME, SciCode) remain unconfirmed as of April 24, 2026 — model launched 24 hours ago; leaderboards not yet updated. **Ranking outcome:** Gemini #1 (71.1), KAT #2 (60.7), GLM #3 (58.9), Kimi #4 (57.5), MiniMax #5 (57.5), GPT-5.5 rises to #6 (56.1). Quality-only unchanged from v2.4: Gemini #1 (74.1), Opus 4.7 #2 (67.8), GPT-5.5 #3 (65.0).

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Model Inventory](#2-model-inventory)
3. [Methodology](#3-methodology)
4. [Cost Analysis](#4-cost-analysis)
5. [Raw Benchmark Data](#5-raw-benchmark-data)
6. [Speed & Latency Reference Data](#6-speed--latency-reference-data)
7. [Normalized Rank Scores](#7-normalized-rank-scores)
8. [Quality-Only Ranking (Unweighted)](#8-quality-only-ranking-unweighted)
9. [Final Cost-Weighted Ranking](#9-final-cost-weighted-ranking)
10. [Ranking Changes (v1.0 → v2.0 → v2.1 → v2.5)](#10-ranking-changes-v10--v20--v21--v25)
11. [Model Profiles](#11-model-profiles)
12. [Strategic Insights](#12-strategic-insights)
13. [Use Case Recommendations](#13-use-case-recommendations)
14. [Limitations & Known Gaps](#14-limitations--known-gaps)
15. [Benchmark Reference Index](#15-benchmark-reference-index)
16. [Methodology Appendix](#16-methodology-appendix)

---

## 1. Executive Summary

This report ranks 11 frontier AI models across 15 dimensions — 12 quality benchmarks plus cost efficiency, inference speed (tok/s), and task-level reliability — using rank-based normalization with a 28% cost weight. The framework is designed to reflect the total value equation for production software engineering teams: raw capability matters, but so does price, throughput, and reliability at scale.

**v2.5 Headline Finding (cost methodology overhaul):** API pricing ($/1M tokens) is a misleading cost signal — actual cost = price × token usage, and verbosity varies dramatically by model. v2.5 replaces the blended API price metric with **Artificial Analysis Intelligence Index Eval Cost**: the total USD Artificial Analysis spent running each model through its full standardized benchmark suite. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models. **The most consequential finding:** Anthropic's extended-thinking models (Opus 4.7 at $4,811; Opus 4.6 at $4,970) are the most expensive to run — not GPT-5.5 ($3,357). GPT-5.5's concise xhigh reasoning outputs are cheaper than Sonnet ($3,959) on actual tasks despite higher token pricing. **KAT-Coder-Pro-V2 becomes the sole cheapest model** ($73.49 eval cost, rank 1, score 100.0), rising from #3 to #2 overall. **GLM 5.1 jumps to #3** as eval cost rank improves from 5→4. **Kimi K2.6 drops from #2 to #4** — benchmark task verbosity ($947.87) exceeds Gemini's ($892.28) despite lower token price.

**Top 6 (v2.5):**
1. 🥇 **Gemini 3.1 Pro** — 71.1 pts (#1 overall, #1 quality; eval cost rank 5/11 = score 60.0; broadest coverage)
2. 🥈 **KAT-Coder-Pro-V2** — 60.7 pts (sole cheapest to run, $73.49; #2 TB 2.0 + #2 Speed)
3. 🥉 **GLM 5.1** — 58.9 pts (eval cost rank 4/11; #2 IFBench + #2 τ²-Bench)
4. **Kimi K2.6** — 57.5 pts (#1 LCB; drops as eval cost reveals token verbosity)
5. **MiniMax M2.7** — 57.5 pts (near-budget $175.51; ties Kimi to 0.1 pts)
6. **GPT-5.5** — 56.1 pts (#1 TB 2.0, τ², OSWorld, GDP; escapes cost trap — eval cost cheaper than Anthropic premium tier)

---

## 2. Model Inventory

| # | Model | Provider | Release | Context | Primary Tier |
|---|---|---|---|---|---|
| 1 | Gemini 3.1 Pro | Google DeepMind | 2026-02 | 2M tokens | Frontier |
| 2 | Kimi K2.6 | Moonshot AI | 2026-01 | 200K tokens | Frontier |
| 3 | MiniMax M2.7 | MiniMax | 2026-02 | 1M tokens | Frontier |
| 4 | GPT-5.4 | OpenAI | 2026-03 | 256K tokens | Frontier |
| 5 | Qwen 3.6 Plus | Alibaba | 2026-02 | 128K tokens | Frontier |
| 6 | KAT-Coder-Pro-V2 | KAT AI | 2026-01 | 64K tokens | Specialist |
| 7 | GLM 5.1 | Zhipu AI | 2026-01 | 128K tokens | Frontier |
| 8 | Claude Opus 4.7 | Anthropic | 2026-03 | 200K tokens | Frontier |
| 9 | GPT-5.5 | OpenAI | 2026-04-23 | 512K tokens | Frontier |
| 10 | Claude Sonnet 4.6 | Anthropic | 2026-02 | 200K tokens | Frontier |
| 11 | Claude Opus 4.6 | Anthropic | 2025-11 | 200K tokens | Frontier |

---

## 3. Methodology

### 3.1 Scoring Formula

All dimensions use rank-based normalization:

```
Normalized Score = ((n − rank) / (n − 1)) × 100
```

Where `n` is the number of models with real data for that dimension. This formula is scale-invariant — percentages, ELO ratings, tokens/second, and composite indices all normalize identically. Rank 1 (best) always scores 100; rank n (worst) always scores 0.

**Missing data:** Models without published results for a dimension receive a neutral score of **50** (midpoint). This neither rewards nor penalizes absent data. Missing values are marked **⊘** throughout.

### 3.2 Final Score Formula

```
Final Score = Σ (Normalized Score × Dimension Weight)
```

Weights are fixed; see §3.4 for rationale.

### 3.3 TTFT Exclusion

Time-to-first-token (TTFT) was evaluated for inclusion in v2.0 but excluded from scoring in both v2.0 and v2.1. Root cause: reasoning-mode models (GPT-5.5 xhigh, Gemini 3.1 Pro think mode) exhibit TTFT values of 100,000–200,000 ms because the metric captures full chain-of-thought latency. Non-reasoning models like Claude Sonnet 4.6 return first tokens in ~1,400 ms. These two populations measure fundamentally different things. Including both in a single scored dimension would penalize reasoning models for being reasoning models. TTFT raw data is preserved in §6.2 for reference.

### 3.4 Dimension Weights

| # | Dimension | Abbrev | Weight | Rationale |
|---|---|---|---|---|
| 1 | Cost Efficiency (AA Index Eval Cost) | Cost | **28%** | Dominant production selection factor; total USD to run AA's full benchmark suite (price × actual benchmark token usage) |
| 2 | Instruction Following (IFBench) | IF | **20%** | Critical for agentic reliability; high-weight due to direct task-completion impact |
| 3 | Terminal-Bench 2.0 | Term | **9%** | Primary SE harness benchmark; direct proxy for coding-agent performance |
| 4 | SWE-bench Verified | SWE | **8%** | Real GitHub issue resolution; industry standard for SE agents |
| 5 | LiveCodeBench v4 | LCB | **7%** | Contamination-resistant coding; rolling test set |
| 6 | GDPval-AA ELO | GDP | **5%** | Aggregated human preference across 141 models |
| 7 | Speed (tok/s) | Spd | **5%** | Output throughput; practical for streaming and bulk workloads |
| 8 | τ²-Bench | τ² | **4%** | Multi-step tool-use reliability; phone/computer agent tasks |
| 9 | OSWorld | OSW | **4%** | Computer use / GUI agent benchmark |
| 10 | Long-Context Retrieval (RULER) | LCR | **3%** | Needle-in-haystack at 128K+; long-context faithfulness |
| 11 | Humanity's Last Exam (HLE) | HLE | **2%** | Frontier expert knowledge breadth |
| 12 | GPQA Diamond | GPQA | **2%** | Graduate-level science reasoning |
| 13 | AIME 2025 | AIME | **1%** | Competition math; reasoning ceiling |
| 14 | SciCode | Sci | **1%** | Scientific coding (domain-expert tasks) |
| 15 | AA-Omniscience | Omni | **1%** | Hallucination-adjusted accuracy (accuracy% − hallucination_rate%) |
| | **Total** | | **100%** | |

**Framework slots (collected, not yet scored):** BigCodeBench, GAIA Level 3. As of April 24, 2026, no 2026-era frontier models have submitted to these leaderboards. Will be activated in v3.0 when ≥ 4 models have real scores.

---

## 4. Cost Analysis

### 4.1 Artificial Analysis Index Eval Cost (Total USD to Run AA Benchmark Suite)

| Model | AA Index Eval Cost | vs. Cheapest |
|---|---|---|
| KAT-Coder-Pro-V2 | **$73.49** | 1.0× (baseline) |
| MiniMax M2.7 | **$175.51** | 2.4× |
| Qwen 3.6 Plus | **$482.65** | 6.6× |
| GLM 5.1 | **$543.95** | 7.4× |
| Gemini 3.1 Pro | **$892.28** | 12.1× |
| Kimi K2.6 | **$947.87** | 12.9× |
| GPT-5.4 | **$2,851.01** | 38.8× |
| GPT-5.5 | **$3,357.00** | 45.7× |
| Claude Sonnet 4.6 | **$3,959.36** | 53.9× |
| Claude Opus 4.7 | **$4,811.04** | 65.5× |
| Claude Opus 4.6 | **$4,969.68** | 67.6× |

> **What is AA Index Eval Cost?** Artificial Analysis runs each model through its full standardized Intelligence Index benchmark suite and records the actual total USD spent — API price × real token consumption across all tasks. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models. Unlike raw $/1M token pricing, eval cost captures true user cost: a verbose model that uses 10× more tokens costs 10× more even at the same token price. Source: artificialanalysis.ai Intelligence Index.

### 4.2 Cost Efficiency Tier Summary

| Tier | Models | AA Eval Cost Range |
|---|---|---|
| Budget | KAT-Coder-Pro-V2 | $73.49 |
| Near-Budget | MiniMax M2.7 | $175.51 |
| Mid-Range | Qwen 3.6 Plus, GLM 5.1 | $483–$544 |
| Premium | Gemini 3.1 Pro, Kimi K2.6 | $892–$948 |
| Expensive | GPT-5.4 | $2,851 |
| Very Expensive | GPT-5.5, Claude Sonnet 4.6 | $3,357–$3,959 |
| Ultra-Premium | Claude Opus 4.7, Claude Opus 4.6 | $4,811–$4,970 |

---

## 5. Raw Benchmark Data

### 5.1 Quality Benchmarks

| Model | IFBench | Term-Bench 2.0 | SWE-Bench V | LCB v4 | GDPval ELO | τ²-Bench | OSWorld | RULER (LCR)‡ | HLE | GPQA◇ | AIME 2025 | SciCode |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | 89.4% | 68.5% | 80.6% | 76.3% | 1,314 | 95.6% | ⊘ | 94.2%‡ | 44.7% | 94.3% | 95.0% | 58.9% |
| Kimi K2.6 | 82.1% | 66.7% | 80.2% | 82.6% | 1,484 | 72.4% | 73.1% | 87.5%‡ | 34.7% | 90.5% | 96.1% | 52.2% |
| MiniMax M2.7 | 75.7% | 57.0% | ⊘ | ⊘ | 1,514 | 84.8% | ⊘ | 81.3%‡ | 28.1% | 87.4% | ⊘ | 47.0% |
| GPT-5.4 | 73.9% | 75.1% | ⊘ | 63.8% | 1,674 | 87.1% | 75.0% | 97.8%‡ | 41.6% | 92.0% | ⊘ | 56.6% |
| Qwen 3.6 Plus | 74.2% | 61.6% | 78.8% | ⊘ | 1,361 | 78.2% | ⊘ | 87.5%‡ | 28.8% | 90.4% | ⊘ | 21.4% |
| KAT-Coder-Pro-V2 | 67.0% | 76.2%† | 79.6% | ⊘ | 1,124 | 93.9% | ⊘ | 74.6%‡ | 12.7% | 85.5% | ⊘ | 38.3% |
| GLM 5.1 | 85.9% | 69.0% | 77.8% | ⊘ | 1,535 | 97.7% | ⊘ | 68.3%‡ | 31.0% | 86.2% | 95.0% | 43.8% |
| Claude Opus 4.7 | 52.1% | 69.4% | 87.6% | ⊘ | 1,753 | 74.0% | 78.0% | 92.4%‡ | 46.9% | 94.2% | 99.8% | 54.5% |
| GPT-5.5 | ⊘ | **82.7%** | ⊘ | ⊘ | 1,784 | 98.0% | 78.7% | ⊘ | 44.3% | 93.6% | ⊘ | ⊘ |
| Claude Sonnet 4.6 | 45.6% | 59.1% | 79.6% | ⊘ | 1,675 | 79.5% | 72.5% | 61.2%‡ | 30.0% | 87.5% | 57.1% | 46.9% |
| Claude Opus 4.6 | 40.2% | 65.4% | 80.8% | 76.0% | 1,619 | 84.8% | 72.7% | 54.7%‡ | 40.0% | 84.0% | 99.8% | 51.9% |

> **† TB 2.0 footnote:** KAT-Coder-Pro-V2 (76.2%) is not listed on the TB 2.0 public leaderboard; value is from the KAT-Coder-V2 technical report referencing "Terminal-Bench Hard" — a potentially different benchmark variant. Score is unverifiable against the canonical tbench.ai leaderboard and remains dagger-flagged. MiniMax M2.7 corrected from v2.3's 28.4% to **57.0%** (confirmed via MiniMax's official announcement and multiple aggregators — the 28.4% was from an older MiniMax M2 evaluation).
> **‡ RULER (LCR) footnote:** No model in this cohort was found on any public RULER leaderboard as of 2026-04-24. These values are drawn from model cards and internal evaluation documents; none could be independently confirmed against the canonical ruler-bench.github.io or llm-stats leaderboards. RULER scores should be treated as provisional; they are retained to avoid wholesale ⊘ on a 3%-weight dimension, but the rank order among them is unverified.

### 5.2 Composite / Special Dimensions

| Model | AA-Omniscience (Acc% − Hall%) | Speed (tok/s) | AA Index Eval Cost ($) |
|---|---|---|---|
| Gemini 3.1 Pro | +34 (82% acc − 48% hall) | 128.0 | $892.28 |
| Kimi K2.6 | ⊘ | 100.4 | $947.87 |
| MiniMax M2.7 | +12 (71% acc − 59% hall) | 49.0 | $175.51 |
| GPT-5.4 | ⊘ | 74.8 | $2,851.01 |
| Qwen 3.6 Plus | ⊘ | 53.0 | $482.65 |
| KAT-Coder-Pro-V2 | ⊘ | 113.5 | $73.49 |
| GLM 5.1 | −1 (64% acc − 65% hall) | 49.0 | $543.95 |
| Claude Opus 4.7 | +22 (76% acc − 54% hall) | 42.0 | $4,811.04 |
| GPT-5.5 | **−29** (57% acc − 86% hall) | ⊘ | $3,357.00 |
| Claude Sonnet 4.6 | ⊘ | 46.0 | $3,959.36 |
| Claude Opus 4.6 | +8 (68% acc − 60% hall) | 18.2 | $4,969.68 |

> **AA-Omniscience note (GPT-5.5):** Despite being rated #1 on the AA Intelligence Index v4.0 (score 60 xhigh), GPT-5.5's AA-Omniscience score is −29, the worst of all models with real data. This reflects an 86% hallucination rate at high-confidence assertions — a known characteristic of the xhigh reasoning mode that prioritizes bold inference over calibrated uncertainty. At lower reasoning tiers, accuracy and hallucination rates would both shift.

---

## 6. Speed & Latency Reference Data

### 6.1 Inference Speed (tok/s) — Scored Dimension

| Model | Output tok/s | Notes |
|---|---|---|
| Gemini 3.1 Pro | 128.0 | Fastest in field (v2.4 correction from 88.3); TPU infrastructure advantage |
| KAT-Coder-Pro-V2 | 113.5 | Second fastest (v2.4 correction from 95.7); specialized decode path |
| Kimi K2.6 | 100.4 | Third fastest (v2.4 correction from 112.4; was #1 in v2.3) |
| GPT-5.4 | 74.8 | Moderate throughput; confirmed by AA leaderboard |
| Qwen 3.6 Plus | 53.0 | Mid-range (v2.4 correction from 63.2) |
| MiniMax M2.7 | 49.0 | Budget tier; tied with GLM |
| GLM 5.1 | 49.0 | Tied with MiniMax (v2.4 correction from 34.5) |
| Claude Sonnet 4.6 | 46.0 | Moderate (v2.4 correction from 51.4) |
| Claude Opus 4.7 | 42.0 | Constrained by 200K context (v2.4 correction from 28.1) |
| Claude Opus 4.6 | 18.2 | Slowest with real data; not found on current AA leaderboard |
| GPT-5.5 | ⊘ | Not published; reasoning mode complicates direct comparison |

### 6.2 Time-to-First-Token (TTFT) — Reference Only, Not Scored

| Model | TTFT (ms) | Mode | Notes |
|---|---|---|---|
| Claude Sonnet 4.6 | ~1,420 | Non-reasoning | Fast initial response |
| Kimi K2.6 | ~2,100 | Standard | Consistent |
| MiniMax M2.7 | ~2,400 | Standard | Budget model; acceptable |
| Gemini 3.1 Pro | ~3,800 | Think mode | Includes partial reasoning warmup |
| GPT-5.4 | ~4,200 | Standard | Moderate |
| Qwen 3.6 Plus | ~5,100 | Thinking mode | |
| GLM 5.1 | ~6,300 | Standard | Slower architecture |
| Claude Opus 4.7 | ~8,900 | Extended thinking | |
| Claude Opus 4.6 | ~9,200 | Extended thinking | |
| GPT-5.5 | ~172,720 | xhigh reasoning | Full CoT included; not comparable to above |
| KAT-Coder-Pro-V2 | ⊘ | n/a | Not published |

> **Why TTFT is excluded from scoring:** At xhigh, GPT-5.5's TTFT encompasses the entire chain-of-thought computation. Comparing 172,720 ms (GPT-5.5 xhigh) to 1,420 ms (Claude Sonnet) would penalize reasoning models for their reasoning — a category error. Speed (tok/s) is the only throughput dimension scored, as it measures a comparable post-generation metric across all modes.

---

## 7. Normalized Rank Scores

### 7.1 Methodology Recap

For each dimension, models with real data are ranked 1–n (best to worst). Score = `((n − rank) / (n − 1)) × 100`. Missing data = **50⊘**. All columns are fully renormalized for the 11-model v2.1 field.

### 7.2 Complete Normalized Score Matrix

| Model | Cost | IF | Term | SWE | LCB | GDP | Spd | τ² | OSW | LCR | HLE | GPQA | AIME | Sci | Omni |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | **60.0** | 100.0 | 50.0 | 71.4 | 66.7 | 10.0 | 100.0 | 80.0 | **50⊘** | 88.9 | 90.0 | 100.0 | 30.0 | 100.0 | 100.0 |
| Kimi K2.6 | **50.0** | 77.8 | 40.0 | 57.1 | 100.0 | 30.0 | 77.8 | 0.0 | 40.0 | 61.1 | 50.0 | 60.0 | 60.0 | 66.7 | **50⊘** |
| MiniMax M2.7 | **90.0** | 66.7 | 0.0 | **50⊘** | **50⊘** | 40.0 | 38.9 | 45.0 | **50⊘** | 44.4 | 10.0 | 30.0 | **50⊘** | 44.4 | 60.0 |
| GPT-5.4 | **40.0** | 44.4 | 80.0 | **50⊘** | 0.0 | 70.0 | 66.7 | 60.0 | 60.0 | 100.0 | 70.0 | 70.0 | **50⊘** | 88.9 | **50⊘** |
| Qwen 3.6 Plus | **80.0** | 55.6 | 20.0 | 14.3 | **50⊘** | 20.0 | 55.6 | 20.0 | **50⊘** | 61.1 | 20.0 | 50.0 | **50⊘** | 0.0 | **50⊘** |
| KAT-Coder-Pro-V2 | **100.0** | 33.3 | 90.0 | 35.7 | **50⊘** | 0.0 | 88.9 | 70.0 | **50⊘** | 33.3 | 0.0 | 10.0 | **50⊘** | 11.1 | **50⊘** |
| GLM 5.1 | **70.0** | 88.9 | 60.0 | 0.0 | **50⊘** | 50.0 | 38.9 | 90.0 | **50⊘** | 22.2 | 40.0 | 20.0 | 30.0 | 22.2 | 20.0 |
| Claude Opus 4.7 | **10.0** | 22.2 | 70.0 | 100.0 | **50⊘** | 90.0 | 11.1 | 10.0 | 80.0 | 77.8 | 100.0 | 90.0 | 90.0 | 77.8 | 80.0 |
| GPT-5.5 | **30.0** | **50⊘** | 100.0 | **50⊘** | **50⊘** | 100.0 | **50⊘** | 100.0 | 100.0 | **50⊘** | 80.0 | 80.0 | **50⊘** | **50⊘** | 0.0 |
| Claude Sonnet 4.6 | **20.0** | 11.1 | 10.0 | 35.7 | **50⊘** | 80.0 | 22.2 | 30.0 | 0.0 | 11.1 | 30.0 | 40.0 | 0.0 | 33.3 | **50⊘** |
| Claude Opus 4.6 | **0.0** | 0.0 | 30.0 | 85.7 | 33.3 | 60.0 | 0.0 | 45.0 | 20.0 | 0.0 | 60.0 | 0.0 | 90.0 | 55.6 | 40.0 |

**⊘** = missing data, assigned neutral score of 50.0
**v2.5 cost overhaul:** Cost column fully replaced with AA Index Eval Cost rankings (cheapest eval run = rank 1). All 14 quality columns unchanged from v2.4. Key cost shifts: KAT-Coder-Pro-V2 sole rank 1 (100.0, $73.49); MiniMax drops to rank 2 (90.0); Gemini improves to rank 5 (60.0); Kimi drops to rank 6 (50.0); GPT-5.5 rises to rank 8 (30.0) as it costs less than Sonnet and both Opus models on actual benchmark tasks; Opus 4.6 becomes most expensive (rank 11, 0.0).

### 7.3 Cost Score Derivation

Cost ranks (cheapest = rank 1, n=11). No ties in this ranking — all 11 AA Index Eval Cost values are distinct.

| Model | AA Index Eval Cost ($) | Cost Rank | Score = ((11−rank)/10)×100 |
|---|---|---|---|
| KAT-Coder-Pro-V2 | $73.49 | 1 | **100.0** |
| MiniMax M2.7 | $175.51 | 2 | **90.0** |
| Qwen 3.6 Plus | $482.65 | 3 | **80.0** |
| GLM 5.1 | $543.95 | 4 | **70.0** |
| Gemini 3.1 Pro | $892.28 | 5 | **60.0** |
| Kimi K2.6 | $947.87 | 6 | **50.0** |
| GPT-5.4 | $2,851.01 | 7 | **40.0** |
| GPT-5.5 | $3,357.00 | 8 | **30.0** |
| Claude Sonnet 4.6 | $3,959.36 | 9 | **20.0** |
| Claude Opus 4.7 | $4,811.04 | 10 | **10.0** |
| Claude Opus 4.6 | $4,969.68 | 11 | **0.0** |

> **v2.5 cost methodology:** Replaced API pricing ($/1M tokens) with **Artificial Analysis Intelligence Index Eval Cost** — total USD cost to run AA's full standardized benchmark suite on each model (API price × actual token consumption across all tasks). This is the only publicly available, standardized, usage-weighted cost dataset for frontier models. Source: artificialanalysis.ai. **Key insight:** eval cost reveals true verbosity effects — KAT ($73.49) is 67.6× cheaper than Opus 4.6 ($4,969.68), vs. the v2.4 API-price ratio of only 18.9×. GPT-5.5 ($3,357) runs cheaper than Sonnet ($3,959) and both Opus models on the same tasks, despite higher token pricing, because GPT-5.5's reasoning mode produces more concise outputs. Claude Opus 4.6 ($4,969.68) displaces GPT-5.5 as most expensive (rank 11, score 0.0).

---

## 8. Quality-Only Ranking (Unweighted)

Equally weighting all 14 non-cost dimensions (7.14% each) to show pure capability ranking. IFBench is factored out as a separate column given its outsized weight (20%) in the main ranking.

| Rank | Model | IF Score | Benchmark Quality‡ | Total Quality |
|---|---|---|---|---|
| 1 | Gemini 3.1 Pro | 100.0 | 72.1 | 74.1 |
| 2 | Claude Opus 4.7 | 22.2 | 71.3 | 67.8 |
| 3 | GPT-5.5 | 50.0⊘ | 66.2 | 65.0 |
| 4 | GPT-5.4 | 44.4 | 62.7 | 61.4 |
| 5 | Kimi K2.6 | 77.8 | 53.3 | 55.0 |
| 6 | GLM 5.1 | 88.9 | 38.0 | 41.6 |
| 7 | MiniMax M2.7 | 66.7 | 39.4 | 41.4 |
| 8 | KAT-Coder-Pro-V2 | 33.3 | 41.5 | 40.9 |
| 9 | Claude Opus 4.6 | 0.0 | 40.0 | 37.1 |
| 10 | Qwen 3.6 Plus | 55.6 | 35.5 | 36.9 |
| 11 | Claude Sonnet 4.6 | 11.1 | 30.2 | 28.8 |

> **‡** IF Score = raw normalized IFBench score (0–100 scale). Benchmark Quality = equal-weighted avg of the other 13 non-cost dims (Term, SWE, LCB, GDP, Speed, τ², OSW, LCR, HLE, GPQA, AIME, Sci, Omni). Total Quality = equal-weighted avg of all 14 non-cost dims (unchanged from prior versions; rank is by Total Quality). ⊘ on GPT-5.5 IF = neutral-50.

**v2.4 quality shakeup:** Gemini 3.1 Pro extends its quality lead to 74.1 (+1.7 vs v2.3 72.4) driven by corrected GPQA Diamond (90→100.0), Speed (#1 at 128.0 tok/s replacing Kimi), and τ²-Bench (60→80.0). **Claude Opus 4.7 surges from #3 to #2** (67.8, +2.1) as HLE corrects to 100.0 (was 70.0), GPQA corrects to 90.0 (was 70.0), and τ²-Bench corrects upward. **GPT-5.5 falls from #2 to #3** (65.0, −2.9): HLE corrects downward 100→80.0 and GPQA 100→80.0 — both were overstated in v2.3. GPT-5.4 holds #4 (61.4, −4.1): LCB drops 50.0→0.0 as Qwen leaves the LCB cohort and GPT-5.4 becomes last of the 4 confirmed scorers. Kimi K2.6 drops to #5 (55.0, −8.4): τ²-Bench corrects 70.0→0.0 (was reflecting early-2025 data). Budget tier models (GLM, MiniMax, KAT) cluster tightly 40.9–41.6 after τ² and GPQA corrections balance out. Qwen drops to #10 on quality (τ²-Bench 85.0→20.0 from the τ¹→τ² correction was the largest single quality penalty).

---

## 9. Final Cost-Weighted Ranking

**Weights:** Cost 28%, IF 20%, Term 9%, SWE 8%, LCB 7%, GDP 5%, Spd 5%, τ² 4%, OSW 4%, LCR 3%, HLE 2%, GPQA 2%, AIME 1%, Sci 1%, Omni 1%

### Corrected Normalized Score Matrix (v2.5 Final)

| Model | Cost | IF | Term | SWE | LCB | GDP | Spd | τ² | OSW | LCR | HLE | GPQA | AIME | Sci | Omni |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | **60.0** | 100.0 | 50.0 | 71.4 | 66.7 | 10.0 | 100.0 | 80.0 | 50.0⊘ | 88.9 | 90.0 | 100.0 | 30.0 | 100.0 | 100.0 |
| Kimi K2.6 | **50.0** | 77.8 | 40.0 | 57.1 | 100.0 | 30.0 | 77.8 | 0.0 | 40.0 | 61.1 | 50.0 | 60.0 | 60.0 | 66.7 | 50.0⊘ |
| MiniMax M2.7 | **90.0** | 66.7 | 0.0 | 50.0⊘ | 50.0⊘ | 40.0 | 38.9 | 45.0 | 50.0⊘ | 44.4 | 10.0 | 30.0 | 50.0⊘ | 44.4 | 60.0 |
| GPT-5.4 | **40.0** | 44.4 | 80.0 | 50.0⊘ | 0.0 | 70.0 | 66.7 | 60.0 | 60.0 | 100.0 | 70.0 | 70.0 | 50.0⊘ | 88.9 | 50.0⊘ |
| Qwen 3.6 Plus | **80.0** | 55.6 | 20.0 | 14.3 | 50.0⊘ | 20.0 | 55.6 | 20.0 | 50.0⊘ | 61.1 | 20.0 | 50.0 | 50.0⊘ | 0.0 | 50.0⊘ |
| KAT-Coder-Pro-V2 | **100.0** | 33.3 | 90.0 | 35.7 | 50.0⊘ | 0.0 | 88.9 | 70.0 | 50.0⊘ | 33.3 | 0.0 | 10.0 | 50.0⊘ | 11.1 | 50.0⊘ |
| GLM 5.1 | **70.0** | 88.9 | 60.0 | 0.0 | 50.0⊘ | 50.0 | 38.9 | 90.0 | 50.0⊘ | 22.2 | 40.0 | 20.0 | 30.0 | 22.2 | 20.0 |
| Claude Opus 4.7 | **10.0** | 22.2 | 70.0 | 100.0 | 50.0⊘ | 90.0 | 11.1 | 10.0 | 80.0 | 77.8 | 100.0 | 90.0 | 90.0 | 77.8 | 80.0 |
| GPT-5.5 | **30.0** | 50.0⊘ | 100.0 | 50.0⊘ | 50.0⊘ | 100.0 | 50.0⊘ | 100.0 | 100.0 | 50.0⊘ | 80.0 | 80.0 | 50.0⊘ | 50.0⊘ | 0.0 |
| Claude Sonnet 4.6 | **20.0** | 11.1 | 10.0 | 35.7 | 50.0⊘ | 80.0 | 22.2 | 30.0 | 0.0 | 11.1 | 30.0 | 40.0 | 0.0 | 33.3 | 50.0⊘ |
| Claude Opus 4.6 | **0.0** | 0.0 | 30.0 | 85.7 | 33.3 | 60.0 | 0.0 | 45.0 | 20.0 | 0.0 | 60.0 | 0.0 | 90.0 | 55.6 | 40.0 |

### Final Weighted Scores

| Main Rank | Model | Cost (28%) | IF (20%) | Quality (52%) | Main Score | CD Rank† | CD Score† |
|---|---|---|---|---|---|---|---|
| 🥇 1 | **Gemini 3.1 Pro** | 16.8 | 20.0 | 34.3 | **71.1** | 🥇 1 | **72.3** |
| 🥈 2 | **KAT-Coder-Pro-V2** | 28.0 | 6.7 | 26.0 | **60.7** | 🥉 3 | **66.5** |
| 🥉 3 | **GLM 5.1** | 19.6 | 17.8 | 21.5 | **58.9** | 🥈 2 | **67.6** |
| 4 | **Kimi K2.6** | 14.0 | 15.6 | 27.9 | **57.5** | 7 | **53.8** |
| 5 | **MiniMax M2.7** | 25.2 | 13.3 | 19.0 | **57.5** | 4 | **59.6** |
| 6 | **GPT-5.5** | 8.4 | 10.0⊘ | 37.7 | **56.1** | excl. | — |
| 7 | **Qwen 3.6 Plus** | 22.4 | 11.1 | 17.3 | **50.8** | 6 | **54.5** |
| 8 | **GPT-5.4** | 11.2 | 8.9 | 30.5 | **50.6** | 5 | **54.6** |
| 9 | **Claude Opus 4.7** | 2.8 | 4.4 | 35.1 | **42.3** | 8 | **34.7** |
| 10 | **Claude Sonnet 4.6** | 5.6 | 2.2 | 16.2 | **24.0** | 9 | **23.5** |
| 11 | **Claude Opus 4.6** | 0.0 | 0.0 | 20.5 | **20.5** | 10 | **12.9** |

> **Main:** All 11 models, 15 dimensions. Columns show weighted point contributions: Cost = norm_cost × 0.28 (max 28.0 pts); IF = norm_IF × 0.20 (max 20.0 pts); Quality = weighted sum of remaining 13 dims × their weights, summing to 52% (max 52.0 pts); Main Score = Cost + IF + Quality. ⊘ on GPT-5.5 IF = neutral-50 input (10.0 pts). `Final = Σ(normalized_score × weight)`.
> **†CD (Complete-Data):** GPT-5.5 excluded (7 of 14 quality dims ⊘ at time of publication). 5 dimensions with any ⊘ removed: SWE-bench (MiniMax/GPT-5.4 ⊘), LCB (6 models ⊘), OSWorld (5 models ⊘), AIME (4 models ⊘), Omni (5 models ⊘). 10 remaining dimensions renormalized to 100% (cost rises 28%→35.4%). All ranks recomputed among n=10. Note: Kimi(#4) and MiniMax(#5) are separated by 0.059 pts in the main ranking.

> **v2.5 main shakeup:** Gemini leads at 71.1 (+2.8 vs v2.4). KAT surges to #2 — sole cheapest to run ($73.49, score 100.0). GLM rises to #3 on improved eval cost rank 4/11. Kimi drops to #4 — benchmark verbosity ($947.87 > Gemini $892.28) despite lower token price. GPT-5.5 rises to #6 (+8.4 pts) — eval cost $3,357 is below Sonnet and both Opus models. Opus 4.6 most expensive to run ($4,970, score 0.0).

> **Key CD divergences:** (1) **GLM #2** — τ² rises to #1 once GPT-5.5 is removed; excluded dims (SWE/LCB/OSW/AIME/Omni) were all weak for GLM. (2) **Kimi #7** — τ² 0.0 at 5.1% weight with no neutral-50 floor; LCB (its #1 strength at 100.0) excluded. (3) **GPT-5.4 #5** — its two ⊘ dims (SWE, AIME) are exactly the excluded ones; strong across all 10 remaining. (4) **Cost amplification** (35.4%) crushes Anthropic tier — Opus 4.7 drops to 34.7 despite #2 quality.

### Complete-Data Variant — Methodology

**Excluded dimensions** (at least one of the 10 models has ⊘):

| Dimension | Excluded Because |
|---|---|
| SWE-bench Verified | MiniMax M2.7, GPT-5.4 ⊘ |
| LiveCodeBench v4 | MiniMax, Qwen, KAT, GLM, Opus 4.7, Sonnet ⊘ |
| OSWorld | Gemini, MiniMax, Qwen, KAT, GLM ⊘ |
| AIME 2025 | MiniMax, GPT-5.4, Qwen, KAT ⊘ |
| AA-Omniscience | Kimi, GPT-5.4, Qwen, KAT, Sonnet ⊘ |

**Renormalized weights** (10 included dims originally sum to 79%; scaled to 100%):

| Dim | Original | Renormalized | | Dim | Original | Renormalized |
|---|---|---|---|---|---|---|
| Cost | 28% | **35.4%** | | τ²-Bench | 4% | **5.1%** |
| IFBench | 20% | **25.3%** | | LCR | 3% | **3.8%** |
| Terminal | 9% | **11.4%** | | HLE | 2% | **2.5%** |
| GDP | 5% | **6.3%** | | GPQA | 2% | **2.5%** |
| Speed | 5% | **6.3%** | | SciCode | 1% | **1.3%** |

**Complete-Data score matrix** (all ranks recomputed among n=10; formula `((10−rank)/9)×100`):

| Model | Cost | IF | Term | GDP | Spd | τ² | LCR | HLE | GPQA | Sci |
|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | 55.6 | 100.0 | 55.6 | 11.1 | 100.0 | 88.9 | 88.9 | 88.9 | 100.0 | 100.0 |
| KAT-Coder-Pro-V2 | 100.0 | 33.3 | 100.0 | 0.0 | 88.9 | 77.8 | 33.3 | 0.0 | 11.1 | 11.1 |
| GLM 5.1 | 66.7 | 88.9 | 66.7 | 55.6 | 38.9 | 100.0 | 22.2 | 44.4 | 22.2 | 22.2 |
| MiniMax M2.7 | 88.9 | 66.7 | 0.0 | 44.4 | 38.9 | 50.0 | 44.4 | 11.1 | 33.3 | 44.4 |
| GPT-5.4 | 33.3 | 44.4 | 88.9 | 77.8 | 66.7 | 66.7 | 100.0 | 77.8 | 77.8 | 88.9 |
| Qwen 3.6 Plus | 77.8 | 55.6 | 22.2 | 22.2 | 55.6 | 22.2 | 61.1 | 22.2 | 55.6 | 0.0 |
| Kimi K2.6 | 44.4 | 77.8 | 44.4 | 33.3 | 77.8 | 0.0 | 61.1 | 55.6 | 66.7 | 66.7 |
| Claude Opus 4.7 | 11.1 | 22.2 | 77.8 | 100.0 | 11.1 | 11.1 | 77.8 | 100.0 | 88.9 | 77.8 |
| Claude Sonnet 4.6 | 22.2 | 11.1 | 11.1 | 88.9 | 22.2 | 33.3 | 11.1 | 33.3 | 44.4 | 33.3 |
| Claude Opus 4.6 | 0.0 | 0.0 | 33.3 | 66.7 | 0.0 | 50.0 | 0.0 | 66.7 | 0.0 | 55.6 |

> Ties: Speed rank 5.5 (MiniMax/GLM both 49.0 tok/s) = 50.0; τ² rank 5.5 (MiniMax/Opus 4.6 both 84.8%) = 50.0; LCR rank 4.5 (Kimi/Qwen both 87.5%‡) = 61.1.

---

## 10. Ranking Changes (v1.0 → v2.0 → v2.1 → v2.3 → v2.4 → v2.5)

| Model | v1.0 | v2.0 | v2.1.1 | v2.2 | v2.3 | v2.4 | **v2.5** | Net Trend |
|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | #1 | #1 | #1 | #1 | #2 | #1 | **#1** | ↑ Holds #1; eval cost rank 5/11 better than API rank 6/11 |
| Kimi K2.6 | #2 | #2 | #2 | #2 | #1 | #2 | **#4** | ↓ Eval cost reveals verbosity; drops −5.6 pts |
| KAT-Coder-Pro-V2 | #6 | #4 | #5 | #5 | #5 | #3 | **#2** | ↑↑ Sole cheapest to run ($73.49); Term #2 holds |
| MiniMax M2.7 | #3 | #3 | #3 | #3 | #3 | #4 | **#5** | ─ Near-budget tier; slight drop as KAT separates |
| GLM 5.1 | #7 | #7 | #7 | #7 | #7 | #5 | **#3** | ↑↑ Eval cost rank 4/11 (score 70.0); τ² #2 + IF #2 |
| Qwen 3.6 Plus | #5 | #6 | #8 | #8 | #8 | #6 | **#7** | ─ Eval cost rank 3/11 unchanged; displaced by GPT-5.5 |
| GPT-5.4 | #4 | #5 | #4 | #4 | #6 | #7 | **#8** | ↓ Eval cost rank 7/11; quality quality holds but others rise |
| GPT-5.5 | — | — | #6 | #6 | #4 | #8 | **#6** | ↑↑ Eval cost $3,357 < Sonnet/Opus; rises +8.4 pts |
| Claude Opus 4.7 | #8 | #8 | #9 | #9 | #9 | #9 | **#9** | ─ Most expensive in field ($4,811 eval cost); quality #2 |
| Claude Sonnet 4.6 | #9 | #10 | #10 | #10 | #10 | #10 | **#10** | ─ |
| Claude Opus 4.6 | #10 | #9 | #11 | #11 | #11 | #11 | **#11** | ─ Most expensive to run ($4,970); cost 0.0 |

**Notable v2.4 → v2.5 shifts (cost methodology overhaul: API pricing → AA Index Eval Cost):**
- **Gemini 3.1 Pro holds #1** (71.1 pts, +2.8 vs v2.4) — eval cost rank improves 6→5 (score 50.0→60.0, +2.8 pts). Quality unchanged. Extends lead further; no other model closes the gap.
- **KAT-Coder-Pro-V2 surges #3 → #2** (60.7 pts, +1.4) — sole rank 1 on eval cost (score 95.0→100.0, +1.4 pts). Terminal-Bench #2 (76.2%) unchanged. The cheapest model to actually run on real benchmarks.
- **GLM 5.1 jumps #5 → #3** (58.9 pts, +2.8) — eval cost rank improves 5→4 (score 60.0→70.0, +2.8 pts). IFBench #2 and τ²-Bench #2 give it solid quality backing.
- **Kimi K2.6 drops #2 → #4** (57.5 pts, −5.6) — eval cost rank drops 4→6 (score 70.0→50.0, −5.6 pts). Kimi's benchmark tasks use more tokens than Gemini despite similar token pricing; $947.87 vs $892.28.
- **MiniMax M2.7 drops #4 → #5** (57.5 pts, −1.4) — eval cost rank drops 1.5→2 (score 95.0→90.0, −1.4 pts). KAT separates as sole budget champion.
- **GPT-5.5 rises #8 → #6** (56.1 pts, +8.4) — the defining v2.5 correction: GPT-5.5's xhigh reasoning mode uses fewer tokens on benchmark tasks than Sonnet and both Opus models. Eval cost $3,357 vs Sonnet $3,959 — rank 8/11 (score 30.0 vs prior 0.0). Net: +8.4 pts, biggest single-version gainer.
- **Claude Opus 4.6 cost deteriorates #11 → #11** (20.5 pts, −4.2) — eval cost $4,969.68 makes it definitively most expensive to run (score 15.0→0.0). Displaces GPT-5.5 in that role.

---

## 11. Model Profiles

### 11.1 Kimi K2.6
**Overall:** #4 | **Quality:** #5 | **Cost tier:** Premium ($947.87 eval cost)

**v2.5 reveals Kimi's hidden token verbosity.** On AA's benchmark tasks, Kimi costs $947.87 to run the full suite — rank 6/11, cost score 50.0 (was 70.0, −5.6 weighted pts). Despite lower token pricing ($1.71/1M blended), Kimi uses more tokens on benchmark tasks than Gemini ($892.28 at $4.50/1M) — verbosity on actual tasks matters more than raw token price. This drops Kimi from #2 to #4 overall (57.5 pts vs prior 63.1). Dimension strengths remain: #1 LiveCodeBench v4 (100.0), SWE-bench #4 (57.1), IFBench #3 (77.8), Speed #3 (77.8 — 100.4 tok/s). τ²-Bench (0.0) and OSWorld (neutral-50) are unchanged weaknesses. Final score 57.5 — ties MiniMax (57.5) to one decimal, but Kimi edges it 57.513 vs 57.454. Free-tier availability remains a differentiator.

**Best for:** Coding-heavy pipelines, LCB-style competitive programming, deployments where τ²-Bench reliability is not a primary concern.

---

### 11.2 Gemini 3.1 Pro 🥇
**Overall:** #1 | **Quality:** #1 | **Cost tier:** Premium ($892.28 eval cost)

**v2.5 extends Gemini's lead.** At $892.28 eval cost (rank 5/11), Gemini's cost score improves from 50.0 → 60.0 (+2.8 pts), pushing the final score to 71.1 (+2.8 vs v2.4 68.3). Gemini's benchmark tasks are concise relative to its token price: $892.28 at $4.50/1M blended compares favourably to Kimi's $947.87 at $1.71/1M — Gemini spends more per token but generates fewer tokens, netting a cheaper run. The only model with real benchmark data on all 14 quality dimensions — zero neutral-50 placeholders on any quality dimension. Leads on IFBench (100.0), SciCode (100.0), Speed (100.0), GPQA Diamond (100.0, sole #1). Primary weakness unchanged: GDPval-AA ELO rank 10/11 (score 10.0) — human preference evaluators consistently rank Gemini below peers on conversational quality.

**Best for:** Production SE workloads requiring consistent, instruction-following intelligence at scale. Research pipelines. Long-context tasks (2M token window). Coverage-critical deployments where missing-data risk is unacceptable.

---

### 11.3 MiniMax M2.7
**Overall:** #5 | **Quality:** #7 | **Cost tier:** Near-Budget ($175.51 eval cost)

v2.5 places MiniMax at eval cost rank 2/11 ($175.51 — score 90.0, was 95.0 as tied rank 1.5, −1.4 pts). KAT-Coder-Pro-V2 ($73.49) separates as the sole budget champion; MiniMax moves to near-budget tier. Still the second-cheapest model to run on AA's full benchmark suite: $175.51 is 5.1× cheaper than Gemini ($892.28) and 19× cheaper than GPT-5.5 ($3,357). Quality profile unchanged: TB 2.0 last in field (57.0%, rank 11/11, score 0.0); IFBench #5 (66.7); GDPval-AA ELO rank 4 (40.0); τ²-Bench #5 (45.0). Net: −1.4 pts overall, drops from #4 → #5.

**Best for:** High-volume classification, summarization, extraction tasks where quality requirements are moderate. Not suitable for coding-agent or terminal-agent applications (TB 2.0 last in field).

---

### 11.4 KAT-Coder-Pro-V2 🥈
**Overall:** #2 | **Quality:** #8 | **Cost tier:** Budget ($73.49 eval cost)

**The biggest v2.5 riser.** KAT's eval cost of $73.49 makes it the sole cheapest model to run on AA's full benchmark suite — rank 1/11, score 100.0 (was tied rank 1.5 at 95.0 with MiniMax; +1.4 pts). Combined with its #2 Terminal-Bench standing and #2 Speed, KAT rises from #3 to #2 overall (60.7 pts). At $73.49 eval cost, KAT is 2.4× cheaper than MiniMax ($175.51), 12.1× cheaper than Gemini ($892.28), and 45.7× cheaper than GPT-5.5 ($3,357) on standardized benchmark tasks. Dimension profile: #2 Terminal-Bench 2.0 (76.2%, †-flagged — from KAT technical report, not on public tbench.ai leaderboard), #2 Speed (88.9 — 113.5 tok/s), #1 eval cost. Missing data on LCB, AIME, OSWorld, and Omni (all neutral-50) remain the quality assessment ceiling.

**Best for:** Specialized terminal/coding pipelines requiring fast, cost-efficient output at scale. Not a general-purpose model; avoid for reasoning-heavy or knowledge tasks (HLE 0.0, GPQA 10.0).

---

### 11.5 GPT-5.4
**Overall:** #8 | **Quality:** #4 | **Cost tier:** Expensive ($2,851.01 eval cost)

GPT-5.4's eval cost of $2,851.01 places it rank 7/11 (cost score 40.0 — identical to its v2.4 API-price rank 7). Quality profile unchanged from v2.4: #1 LCR (100.0 — 97.8% RULER), #3 Terminal-Bench (80.0 — 75.1%), #4 quality overall (61.4). Overall rank slips from #7 → #8 as GPT-5.5 rises above it with a better eval cost (rank 8 vs GPT-5.4's rank 7, but GPT-5.5's quality contribution of 47.7 vs GPT-5.4's 39.4 edges it). At $2,851 eval cost, GPT-5.4 is firmly in the expensive tier — 32× more expensive than KAT ($73.49) but 3.2× more cost-efficient than GPT-5.5 ($3,357) per benchmark suite run.

**Best for:** Long-context retrieval (#1 LCR 100.0), Terminal-Bench-sensitive tasks, GPQA Diamond (70.0), HLE (70.0) applications where mid-premium pricing is acceptable.

---

### 11.6 Qwen 3.6 Plus
**Overall:** #7 | **Quality:** #10 | **Cost tier:** Mid-Range ($482.65 eval cost)

Qwen's eval cost of $482.65 places it rank 3/11 (cost score 80.0 — unchanged from v2.4 API-price rank 3; cost contribution identical at 22.4 pts). Overall rank slips from #6 → #7 as GPT-5.5 rises above it in v2.5. Quality profile unchanged: τ²-Bench 20.0, LCB ⊘, AIME ⊘ — #10 quality overall (36.9). At $483 eval cost, Qwen sits in the mid-range tier: 6.6× cheaper than Gemini ($892) but 2.7× more expensive than MiniMax ($176) on actual benchmark tasks. The API-price "near-budget" label from v2.4 is misleading — on actual usage, Qwen is solidly mid-range.

**Best for:** High-volume workloads requiring moderate quality at mid-range running cost. Avoid τ²-Bench and AIME-sensitive applications.

---

### 11.7 GLM 5.1 🥉
**Overall:** #3 | **Quality:** #6 | **Cost tier:** Mid-Range ($543.95 eval cost)

**v2.5's biggest quality-cost surprise.** GLM's eval cost ($543.95) places it rank 4/11 — cost score improves from 60.0 → 70.0 (+2.8 pts), lifting GLM from #5 to #3 overall (58.9 pts). GLM delivers strong mid-tier value: #2 IFBench (88.9), #2 τ²-Bench (90.0), and solid eval cost at $544 — comparable to Qwen's $483 despite meaningfully higher quality across every quality dimension. At rank 4 eval cost, GLM is the best-value mid-range choice for instruction-following and agentic reliability. GLM's GDPval-AA ELO (50.0 — rank 6/11) indicates mid-field human preference performance. The AA-Omniscience of 20.0 (−1 raw: 64% accuracy − 65% hallucination) is nearly neutral.

**Best for:** Instruction-following tasks requiring τ²-Bench multi-step reliability at mid-range cost. Terminal-agent workloads where Gemini/KAT are priced too high.

---

### 11.8 Claude Opus 4.7
**Overall:** #9 | **Quality:** #2 | **Cost tier:** Ultra-Premium ($4,811.04 eval cost)

**v2.5 reveals Anthropic's true cost burden.** At $4,811.04 eval cost (rank 10/11, cost score 10.0 vs prior 15.0, −1.4 pts), Claude Opus 4.7 is the second most expensive model to run on AA's benchmark suite — only Opus 4.6 ($4,969.68) costs more. Anthropic's extended-thinking mode consumes far more tokens on benchmark tasks than other frontier models at comparable token pricing. Final score drops from 43.7 → 42.3. Quality profile unchanged: #2 quality overall (67.8), #1 SWE-bench Verified (100.0), #1 HLE (100.0), #2 GPQA (90.0). The quality-cost gap now spans 9 rank positions — the largest quality-vs-overall-rank gap in the field. For teams prioritising capability over cost, Opus 4.7 remains the top choice for software engineering and expert-knowledge tasks.

**Best for:** High-stakes software engineering (SWE #1), expert-knowledge tasks (HLE #1), enterprise customers where quality-per-outcome matters more than cost-per-token.

---

### 11.9 GPT-5.5
**Overall:** #6 | **Quality:** #3 | **Cost tier:** Very Expensive ($3,357.00 eval cost)

**The defining v2.5 story: cost ceiling breaks.** In v2.4, GPT-5.5 ranked #8 (47.7 pts) with cost score 0.0 — definitively the most expensive model on API pricing ($11.25/1M). The v2.5 methodology switch to AA eval cost changes this picture: GPT-5.5's xhigh reasoning mode generates more concise benchmark outputs than Anthropic's extended-thinking models. Actual eval cost $3,357.00 places it rank 8/11 (score 30.0), cheaper than Sonnet ($3,959.36) and both Opus models ($4,811–$4,970). Net: +8.4 pts overall, rises #8 → #6 (56.1 pts). This is the single largest gain of any model in any version.

**Key insight:** On API pricing, GPT-5.5 at $11.25/1M looked 19× more expensive than MiniMax at $0.53/1M. On actual benchmark task cost, it is only 19× more expensive ($3,357 vs $175). The ratio holds — but GPT-5.5 is genuinely cheaper to run than the Anthropic premium tier.

**Dimension leadership (5 unchanged):**
- #1 on **Terminal-Bench 2.0** (82.7% — confirmed #1 in the field)
- #1 on GDPval-AA ELO (1,784 — highest human preference ELO)
- #1 on τ²-Bench (98.0% — dominant multi-step agent reliability)
- #1 on OSWorld (78.7% — strongest computer-use agent)
- HLE: #3 (80.0); GPQA: #3 (80.0)

**Why GPT-5.5 ranks #3 on quality:** 7 of 14 quality dimensions remain ⊘ (neutral-50): IFBench, SWE-bench, LCB, Speed, LCR, AIME, SciCode. Quality score 65.0 vs Gemini 74.1 and Opus 4.7 67.8. As missing data fills in, quality rank may rise to #1–2.

**Why GPT-5.5 ranks #6 not higher:** Cost score 30.0 at 28% weight contributes only 8.4 pts — Gemini earns 16.8 pts from cost, KAT earns 28.0 pts. GPT-5.5 must compensate purely on quality. AA-Omniscience = 0.0 (−29 raw; 86% hallucination rate at xhigh mode).

**Best for:** Terminal/CLI agents (#1 TB 2.0), computer-use/GUI agents (#1 OSWorld), τ²-Bench-sensitive agentic pipelines, research applications where quality is unconstrained by cost.

---

### 11.10 Claude Sonnet 4.6
**Overall:** #10 | **Quality:** #11 | **Cost tier:** Very Expensive ($3,959.36 eval cost)

v2.5 places Sonnet at eval cost rank 9/11 ($3,959.36 — cost score 20.0, was 30.0, −2.8 pts). Despite a lower API token price ($6.00/1M blended) than GPT-5.5 ($11.25/1M), Sonnet actually costs more to run on AA's benchmark tasks ($3,959 vs $3,357) due to higher token verbosity on standardized tasks. Final score drops from 26.8 → 24.0. Quality profile unchanged: #11 quality overall (28.8), #9 IFBench, #9 LCR, OSWorld 0.0 (last among scorers), AIME 0.0. The benchmark reveals that Sonnet's verbose outputs make it more expensive than GPT-5.5 in real-world benchmark usage despite cheaper nominal pricing.

**Best for:** Simple Q&A or conversational tasks where cost sensitivity is paramount among Anthropic models. Not recommended for coding-agent, terminal-agent, or math-intensive pipelines.

---

### 11.11 Claude Opus 4.6
**Overall:** #11 | **Quality:** #9 | **Cost tier:** Ultra-Premium ($4,969.68 eval cost)

**v2.5's most expensive model to run.** At $4,969.68 eval cost (rank 11/11, cost score 0.0 vs prior 15.0, −4.2 pts), Claude Opus 4.6 displaces GPT-5.5 as the most expensive model to run on AA's benchmark suite. Anthropic's extended-thinking models consume the most tokens on standardised tasks: Opus 4.6 ($4,970) > Opus 4.7 ($4,811) > Sonnet ($3,959). Final score drops from 24.7 → 20.5. Quality profile unchanged: #9 overall quality (37.1), IFBench 0.0 (40.2% — last in cohort), Speed 0.0 (18.2 tok/s — slowest with real data), LCR 0.0, GPQA 0.0. Dimension bright spots: AIME 90.0 (tied #1 with Opus 4.7), SWE-bench 85.7 (#2). At essentially equivalent eval cost to Opus 4.7 ($4,970 vs $4,811) but lower quality on nearly every dimension, migration to 4.7 is the dominant recommendation.

**Best for:** Legacy deployments that cannot migrate; no new deployments recommended given Opus 4.7 superiority at essentially equivalent eval cost.

---

## 12. Strategic Insights

### 12.1 GPT-5.5 Escapes the Cost Trap (v2.5 update)

The v2.5 methodology switch reveals that API pricing was a misleading cost signal for GPT-5.5. In v2.4, GPT-5.5 ranked #8 (47.7 pts) with cost score 0.0 — it was the most expensive model at $11.25/1M API token price. Under AA eval cost, GPT-5.5 ($3,357 to run the full benchmark suite) is actually cheaper than Sonnet ($3,959), Opus 4.7 ($4,811), and Opus 4.6 ($4,970). GPT-5.5's xhigh reasoning mode generates more concise outputs on standardized benchmark tasks than Anthropic's extended-thinking models — lower token count offsets the higher token price. Result: cost rank 8/11 (score 30.0), +8.4 pts, rises to #6 overall (56.1 pts).

**Implication:** GPT-5.5 is not the budget choice (KAT at $73.49 is 46× cheaper), but it is no longer penalised for verbosity it doesn't exhibit. Teams running cost-sensitive production workloads should still prefer Gemini/KAT/GLM; teams running agentic pipelines get GPT-5.5's dimension leadership (#1 TB 2.0, τ², OSWorld, GDP) at a cost that is competitive with the Anthropic premium tier.

### 12.2 The True Cost Story: Verbosity Matters More Than Token Price

v2.5's most important insight: **nominal token price is a poor predictor of actual running cost.** Three striking examples:
- **Kimi K2.6** has a lower token price ($1.71/1M blended) than **Gemini 3.1 Pro** ($4.50/1M), yet Kimi costs *more* to run on AA's benchmarks ($947.87 vs $892.28) — Kimi generates more tokens per task.
- **Claude Sonnet 4.6** has a lower token price ($6.00/1M) than **GPT-5.5** ($11.25/1M), yet Sonnet costs *more* ($3,959 vs $3,357) — Anthropic's models are verbose.
- **Claude Opus 4.6** and **Opus 4.7** have the *same* token price ($10.00/1M blended) yet different eval costs ($4,970 vs $4,811) — even within the same provider, task verbosity varies.

The API "Premium vs Ultra-Premium" framing of v2.4 is replaced by a clearer picture: Anthropic's extended-thinking models are the most expensive to run, regardless of token price.

### 12.3 KAT-Coder-Pro-V2 Claims the Budget Crown

KAT-Coder-Pro-V2 ($73.49 eval cost) is now the sole cheapest model to run on AA's full benchmark suite — 2.4× cheaper than MiniMax ($175.51). In v2.4, both shared the $0.53/1M API price (tied rank 1.5, score 95.0). On actual benchmark tasks, KAT's concise output style ($73.49) dramatically separates it from MiniMax ($175.51). This earns KAT a perfect cost score (100.0) and pushes it from #3 to #2 overall. For terminal/coding workloads at minimum running cost, KAT is the unambiguous choice. MiniMax remains the near-budget option for instruction-following tasks where TB 2.0 performance is not required.

### 12.4 τ²-Bench Correction Reshapes the Mid-Tier

The single largest quality correction in v2.4 was the τ²-bench systemic error — 9/11 models had τ¹-bench values masquerading as τ²-bench. The correction reshuffles mid-tier quality rankings dramatically: Kimi drops from τ² rank 3 → last (70.0→0.0), Qwen drops from tied-top to rank 9 (85.0→20.0). GLM and GPT-5.5 are confirmed #2 and #1 on actual τ²-bench. For teams selecting models based on agentic task reliability, the ranking order is fundamentally different under corrected data.

### 12.5 Speed Leadership Shifts to Gemini

Gemini 3.1 Pro is now confirmed as the fastest model at 128.0 tok/s (Speed score 100.0), displacing Kimi (corrected to 100.4 tok/s, score 77.8 — rank 3). KAT is confirmed #2 at 113.5 tok/s. For streaming-intensive or real-time applications, Gemini's combination of #1 speed, #1 IFBench, and #1 quality makes it the dominant choice regardless of cost tier.

### 12.6 The Missing-Data Problem Persists

GPT-5.5 still has 7 of 14 quality dimensions missing (⊘ neutral-50) as of April 24, 2026. At 7.14% per dimension, each confirmed score can move quality ±~3.6 pts. As data fills in over 30–60 days, GPT-5.5's quality rank could reach #1. Recommend "provisional" annotation for any model with >4 missing quality dimensions.

---

## 13. Use Case Recommendations

| Use Case | Primary Choice | Budget Alternative | Notes |
|---|---|---|---|
| Production SE pipeline (cost-first) | **Gemini 3.1 Pro** | GLM 5.1 | #1 overall at 71.1 pts; best cost-quality balance; $892 eval cost |
| Broadest-coverage production choice | **Gemini 3.1 Pro** | Kimi K2.6 | Real data on all 14 quality dims; #1 IFBench, Speed, GPQA, Sci |
| Minimum eval-cost processing | **KAT-Coder-Pro-V2** | MiniMax M2.7 | $73.49 eval cost (#1) vs $175.51 (#2); KAT is sole budget champion |
| High-volume batch processing | **KAT-Coder-Pro-V2** | MiniMax M2.7 | KAT: $73.49 eval cost + TB 2.0 #2; MiniMax: $175.51 + IFBench #5 |
| Coding specialist (LCB) | **Kimi K2.6** | Gemini 3.1 Pro | LCB #1 (100.0); free tier available |
| Coding specialist (terminal/CLI) | **GPT-5.5** | KAT-Coder-Pro-V2 | TB 2.0: 82.7% (#1) vs 76.2% (#2) |
| Computer-use / GUI automation | **GPT-5.5** | Claude Opus 4.7 | OSWorld #1 (78.7%) by large margin |
| Multi-step agentic tasks (τ²) | **GPT-5.5** | GLM 5.1 | τ²-Bench: 98.0% (#1) vs 97.7% (#2) |
| SWE-bench / GitHub issue resolution | **Claude Opus 4.7** | Gemini 3.1 Pro | SWE #1 (87.6%); Gemini #3 (80.6%) |
| Long-context retrieval (128K+) | **GPT-5.4** | Gemini 3.1 Pro | RULER: 97.8% vs 94.2% |
| Competition math / reasoning ceiling | **Claude Opus 4.7 / 4.6** | Kimi K2.6 | AIME: 99.8% (tied Opus) vs 96.1% (Kimi) |
| Instruction-following reliability | **Gemini 3.1 Pro** | GLM 5.1 | IFBench: 89.4% (#1) vs 85.9% (#2) |
| Fast streaming / real-time | **Gemini 3.1 Pro** | KAT-Coder-Pro-V2 | 128.0 tok/s (#1) vs 113.5 tok/s (#2) |
| Highest quality, cost unconstrained | **GPT-5.5** | Claude Opus 4.7 | Quality #3 (65.0) but leads 5 agentic dims; Opus 4.7 #2 quality (67.8) |
| Expert knowledge / HLE tasks | **Claude Opus 4.7** | Gemini 3.1 Pro | HLE #1 (46.9%) vs #2 (44.7%) |
| Mid-range instruction + agentic | **GLM 5.1** | Qwen 3.6 Plus | #3 overall (58.9); $544 eval cost; IFBench #2 + τ² #2 |

---

## 14. Limitations & Known Gaps

### 14.1 Data Freshness

- GPT-5.5 launched April 23, 2026 — most benchmark data is from initial release papers and preliminary third-party evaluations. Eight dimensions currently missing; expect rapid data availability in next 30–60 days.
- All other model data is sourced from official publications and established leaderboards as of April 24, 2026.

### 14.2 Benchmark Contamination Risk

LiveCodeBench v4 and τ²-Bench use rolling test sets specifically to address contamination, but older benchmarks (GPQA, SWE-bench) may have varying contamination levels across models depending on training cutoff and data curation practices.

### 14.3 Mode Selection Ambiguity

Several models offer multiple reasoning modes (xhigh/high/standard for GPT-5.5; extended thinking for Claude; think mode for Gemini). Published benchmark scores typically reflect the best available mode. Production deployments often use lower-cost modes. Rankings may not reflect the model's behavior at its production-default mode.

### 14.4 GPT-5.5 AA-Omniscience

The 86% hallucination rate driving the −29 raw Omniscience score comes from early independent evaluations specifically targeting GPT-5.5's xhigh reasoning mode. At standard or high reasoning modes, both accuracy and hallucination rates would differ. This is noted but not adjusted — rankings use published figures.

### 14.5 GPT-5.4 AIME Conflict

LayerLens independent testing: 16.67%. Multiple secondary sources attribute 100% performance to GPT-5.4 (possibly misattributed from GPT-5.2 xhigh). Unresolvable with current data. GPT-5.4 receives ⊘=50 on AIME 2025.

### 14.6 Framework Design Constraints

The 28% cost weight reflects a deliberate choice for production scale optimization. Academic or research applications would reasonably down-weight cost and up-weight quality dimensions. Custom weight configurations are recommended for teams with different budget constraints.

### 14.7 BigCodeBench and GAIA Slots

Both benchmarks are reserved as scored dimensions pending ≥4 model submissions. These will be activated in v3.0.

---

## 15. Benchmark Reference Index

| Benchmark | Dimension | Type | What it Measures | Source |
|---|---|---|---|---|
| AA Index Eval Cost | Cost | USD | Total cost to run AA's full Intelligence Index benchmark suite (price × actual token usage) | artificialanalysis.ai |
| IFBench | IF | Accuracy % | Instruction-following compliance across task types | alo-exp/IFBench leaderboard |
| Terminal-Bench 2.0 | Term | Accuracy % | Terminal/CLI agentic task completion | alo-exp/terminal-bench |
| SWE-bench Verified | SWE | Accuracy % | Real GitHub issue resolution (verified subset) | swebench.com |
| LiveCodeBench v4 | LCB | Accuracy % | Rolling contamination-resistant coding eval | livecodebench.github.io |
| GDPval-AA ELO | GDP | ELO rating | Human preference pairwise comparisons, 141 models | alo-exp/GDPval-AA |
| Speed (tok/s) | Spd | Tokens/second | Output generation throughput | Artificial Analysis; provider specs |
| τ²-Bench | τ² | Accuracy % | Multi-step phone/computer agent task success | tau2-bench leaderboard |
| OSWorld | OSW | Accuracy % | Desktop GUI agent task success | os-world.github.io |
| RULER (LCR) | LCR | Accuracy % | Long-context retrieval fidelity (128K–1M) | ruler-bench.github.io |
| Humanity's Last Exam | HLE | Accuracy % | Expert-level knowledge breadth (3000 questions) | scale.ai/hle |
| GPQA Diamond | GPQA | Accuracy % | Graduate-level science reasoning (diamond subset) | arXiv:2311.12022 |
| AIME 2025 | AIME | Accuracy % | American Invitational Mathematics Exam 2025 | aops.com / arXiv reports |
| SciCode | Sci | Accuracy % | Scientific coding tasks (domain-expert problems) | scicode-bench.github.io |
| AA-Omniscience | Omni | Score | accuracy% − hallucination_rate% | alo-exp/AA-Omniscience v2 |
| TTFT | Ref only | Milliseconds | Time-to-first-token (not scored; see §3.3) | Artificial Analysis |
| BigCodeBench | (reserved) | Accuracy % | Function-level coding with complex instructions | bigcodebench.github.io |
| GAIA Level 3 | (reserved) | Accuracy % | General AI assistant real-world tasks, hardest tier | huggingface.co/GAIA |

---

## 16. Methodology Appendix

### 16A. Normalization Examples

**Example 1: IFBench (n=10 models with real data, 1 model ⊘)**

Raw scores: Gemini 89.4%, GLM 85.9%, Kimi 82.1%, MiniMax 76.5%, Qwen 70.8%, GPT-5.4 64.2%, KAT 58.3%, Opus 4.7 52.1%, Sonnet 45.6%, Opus 4.6 40.2%
GPT-5.5: ⊘ → 50.0

Ranks (1=best): Gemini=1, GLM=2, Kimi=3, MiniMax=4, Qwen=5, GPT-5.4=6, KAT=7, Opus 4.7=8, Sonnet=9, Opus 4.6=10
Formula `((n-rank)/(n-1)) × 100` where n=10:
- Gemini: (9/9)×100 = **100.0**
- GLM: (8/9)×100 = **88.9**
- Kimi: (7/9)×100 = **77.8**
- Opus 4.6: (0/9)×100 = **0.0**

**Example 2: Cost (n=11, all models ranked, no ties — v2.5 AA Index Eval Cost)**

KAT-Coder-Pro-V2: eval cost $73.49, rank 1. Formula `((11-1)/(11-1)) × 100 = (10/10) × 100 = **100.0**`
Claude Opus 4.6: eval cost $4,969.68, rank 11. Formula `((11-11)/(11-1)) × 100 = (0/10) × 100 = **0.0**`
Gemini 3.1 Pro: eval cost $892.28, rank 5. Formula `((11-5)/(11-1)) × 100 = (6/10) × 100 = **60.0**`

**Example 3: GPT-5.5 AA-Omniscience**

GPT-5.5 raw score: 57% accuracy − 86% hallucination = −29
Other models with real scores: Gemini(+34), Claude Opus 4.7(+22), Claude Opus 4.6(+8), MiniMax(+12), GLM(−1)
n=6. Rank order: Gemini(1), Opus 4.7(2), MiniMax(3), Opus 4.6(4), GLM(5), GPT-5.5(6)
GPT-5.5: ((6-6)/(6-1))×100 = **0.0**

### 16B. Sensitivity Analysis

**What if cost weight = 15% (quality-forward scenario)?**

| Rank | Model | Score |
|---|---|---|
| 1 | Gemini 3.1 Pro | 73.1 |
| 2 | GPT-5.5 | 60.8 |
| 3 | Kimi K2.6 | 58.8 |
| 4 | GLM 5.1 | 56.9 |
| 5 | GPT-5.4 | 52.5 |

At 15% cost weight, GPT-5.5 rises to #2 (quality #3 plus partial cost credit) and Kimi recovers to #3. KAT-Coder-Pro-V2 drops out of top-5 as its cost advantage shrinks and quality (#8) is the binding constraint. Claude Opus 4.7 (#2 quality) reaches #6 at 48.1.

**What if cost weight = 40% (cost-extreme scenario)?**

| Rank | Model | Score |
|---|---|---|
| 1 | Gemini 3.1 Pro | 69.3 |
| 2 | KAT-Coder-Pro-V2 | 67.3 |
| 3 | MiniMax M2.7 | 62.9 |
| 4 | GLM 5.1 | 60.8 |
| 5 | Kimi K2.6 | 56.3 |

At 40% cost weight, Gemini holds #1 (eval cost rank 5 contributes 24.0 pts) but KAT surges to #2 (cost score 100.0 contributes 40.0 pts). MiniMax rises to #3. GPT-5.5 ranks #7 (51.8) — cost score 30.0 at 40% weight is a modest 12.0 pts contribution; it no longer drops out of contention as it did in v2.4's API-price framework. Claude Opus 4.6 falls to #11 (17.1) — cost=0 at 40% is a structurally fatal penalty.

### 16C. Adding a New Model (Maintenance Protocol)

When a new model is released:

1. **Collect all 15 dimension values** — use official publications, then established third-party evaluators (Artificial Analysis, LMSYS, alo-exp leaderboards). Mark any missing as ⊘.
2. **Recompute all ranks** — every dimension with real data for the new model requires full reranking of ALL models in that dimension. Do not preserve old ranks.
3. **Apply normalization formula** using the new `n` (count of models with real data per dimension).
4. **Recalculate all final scores** — renormalization changes every model's score, not just the new arrival's.
5. **Update Ranking Changes table** — document Δ vs previous version for every model.
6. **Tag version** — increment minor version (e.g., v2.1 → v2.2) for model additions; increment major version for dimension additions or weight changes.
7. **Flag provisional** — mark models with >4 missing dimensions as "provisional ranking" in the Executive Summary.

### 16D. Dimension Activation Protocol

Reserved dimensions (BigCodeBench, GAIA Level 3) activate when:
- ≥4 models in the current inventory have published scores
- Scores come from the primary leaderboard (not secondary reports)
- At least one model in the top-3 of the current ranking has a score

When activated: increment to next major version, rerun full normalization including new dimension weight (determined at activation time), update all profiles and strategic insights.

### 16E. Benchmark Retirement Protocol

A dimension may be retired if:
- The primary leaderboard ceases maintenance for >12 months
- Strong evidence of widespread test-set leakage across ≥5 models emerges
- A successor benchmark explicitly replaces it

Retirement requires: remove from scoring, mark as "retired" in §15, document rationale in changelog, do NOT retroactively recalculate historical versions.

### 16F. Scheduled Review Cadence

| Trigger | Action |
|---|---|
| New frontier model launch | Add to inventory within 7 days; publish next minor version within 14 days |
| Major benchmark update | Evaluate for dimension replacement; publish impact analysis |
| Quarterly | Full data refresh on all 15 dimensions for all models |
| Semi-annual | Weight review — validate 28% cost weight against production survey data |

---

*Report maintained by the Kay project team | Feedback: kay-rankings@alo-exp.dev*
*Methodology questions: see §16 Appendix | Version history: tracked in git history of this file*
*Next scheduled update: May 2026 quarterly refresh or next major model launch, whichever comes first*
