# AI Model Rankings — Comprehensive Cost-Weighted Analysis
**Version 3.0 | April 25, 2026 | 12 Models | 18 Dimensions**

> **Change log:**
> **v2.0 → v2.1:** Added GPT-5.5 (launched April 23, 2026). All 15 normalized score columns renormalized.
> **v2.1 → v2.1.1:** Corrected GPT-5.5 cost rank (was erroneously rank 11; corrected to rank 9 at $11.25/1M < Opus $30.00/1M).
> **v2.1.1 → v2.2 (full matrix audit):** Fixed 20 cell errors across 7 dimensions — all were pairwise swap errors during normalization.
> **v2.2 → v2.3 (Terminal-Bench 2.0 data refresh):** Replaced stale TB 2.0 column with current leaderboard data; GPT-5.5 confirmed #1 at 82.7%; GPT-5.4 corrected from 94.5% to 75.1%. Kimi overtook Gemini for #1 (73.1 vs 72.7).
> **v2.3 → v2.4 (full provenance audit):** Systematic cross-check of every benchmark value against authoritative leaderboard sources. All benchmark columns except TB 2.0 corrected from stale late-2024/early-2025 data. Cost column: 9/11 API prices corrected (Claude Opus $30→$10 blended; Gemini $0.75→$4.50; Kimi $0.48→$1.71; etc.). τ²-bench, GPQA, HLE, OSWorld, SWE-bench, SciCode, GDP, Speed, LCB, AIME corrected throughout. Ranking outcome: Gemini #1 (68.3), Kimi #2 (63.1), KAT #3 (59.3), GPT-5.5 drops to #8 (47.7).
> **v2.4 → v2.5 (cost methodology overhaul):** **Root cause:** API pricing ($/1M tokens) does not represent true cost to users — actual cost = price × token usage, which varies dramatically by model verbosity and task type. **Fix:** Replaced API-price-based cost dimension with **Artificial Analysis Intelligence Index Eval Cost** — the total USD cost to run AA's full standardized benchmark suite on each model. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models (source: artificialanalysis.ai). **Impact:** Cost ranking completely reshuffled. KAT-Coder-Pro-V2 becomes cheapest to run ($73.49 eval cost, rank 1, score 100.0); Claude Opus 4.6 becomes most expensive ($4,969.68, rank 11, score 0.0). GPT-5.5 moves from rank 11 (API price: most expensive) to rank 8 ($3,357 eval cost — cheaper than Sonnet $3,959 and both Opus models), gaining +8.4 pts. GLM 5.1 rises from rank 5→4. Gemini rank 6→5. Kimi drops rank 4→6 ($947.87 eval cost — marginally more expensive than Gemini $892.28 due to higher token usage on benchmark tasks). **GPT-5.5 missing data confirmed still ⊘:** Exhaustive search confirms all 7 missing dimensions (IFBench, SWE-bench Verified, LCB, Speed, LCR, AIME, SciCode) remain unconfirmed as of April 24, 2026 — model launched 24 hours ago; leaderboards not yet updated. **Ranking outcome:** Gemini #1 (71.1), KAT #2 (60.7), GLM #3 (58.9), Kimi #4 (57.5), MiniMax #5 (57.5), GPT-5.5 rises to #6 (56.1). Quality-only unchanged from v2.4: Gemini #1 (74.1), Opus 4.7 #2 (67.8), GPT-5.5 #3 (65.0).
> **v2.5 → v3.0 (new model + 3 new dimensions + data refresh):** **(1) Added DeepSeek V4-Pro** (released April 24, 2026; Apache 2.0; 1.6T/49B MoE; 1M context). **(2) Activated 3 new scored dimensions:** BrowseComp (3% weight — deep web research agent), SWE-bench Pro (4% — harder SWE variant), MMLU-Pro (2% — graduate-level knowledge breadth). Weight redistribution: Cost 28%→25%, IF 20%→18%, Term 9%→8%, SWE 8%→7%, LCB 7%→6%, LCR 3%→2%; all others unchanged; total remains 100%. **(3) 4 data corrections:** Gemini 3.1 Pro τ²-Bench 95.6%→99.3% (now #1); Kimi K2.6 GDPval ELO 1,484→1,520 (swaps with MiniMax for rank 7); GPT-5.5 IFBench ⊘→75.9% (fills 20%-weight gap); GPT-5.5 SWE-bench Verified ⊘→88.7%† (probable, from OpenAI tech card). **(4) HLE methodology resolved:** canonical Scale Labs leaderboard uses without-tools scores — existing HLE values confirmed correct. **(5) Ranking impact:** GPT-5.5 surges from #6→#2 (64.0 pts, +7.9 pts) as IFBench and SWE-bench gaps fill. KAT drops #2→#3. DeepSeek V4-Pro debuts at #8 (51.1 pts). Quality-only: Gemini #1 (73.1), Opus 4.7 #2 (64.4), GPT-5.4 #3 (62.9).

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
10. [Ranking Changes (v1.0 → v2.0 → v2.5 → v3.0)](#10-ranking-changes-v10--v20--v25--v30)
11. [Model Profiles](#11-model-profiles)
12. [Strategic Insights](#12-strategic-insights)
13. [Use Case Recommendations](#13-use-case-recommendations)
14. [Limitations & Known Gaps](#14-limitations--known-gaps)
15. [Benchmark Reference Index](#15-benchmark-reference-index)
16. [Methodology Appendix](#16-methodology-appendix)

---

## 1. Executive Summary

This report ranks 12 frontier AI models across 18 dimensions — 15 quality benchmarks plus cost efficiency, inference speed (tok/s), and task-level reliability — using rank-based normalization with a 25% cost weight. The framework is designed to reflect the total value equation for production software engineering teams: raw capability matters, but so does price, throughput, and reliability at scale.

**v3.0 Headline Finding (GPT-5.5 data fills + 3 new dimensions):** Two gaps in GPT-5.5's profile filled in — IFBench (75.9%) at 18% weight and SWE-bench Verified (88.7%†) at 7% weight — catapulting GPT-5.5 from #6 to #2 overall. The data confirms what was suspected: GPT-5.5 is a top-2 model when fairly measured. **New model DeepSeek V4-Pro** debuts at #8 (51.1 pts) with strong LCB (93.5%, #1 in cohort) and competitive SWE-bench (80.6%), but many AA Index component scores remain unpublished, leaving 8+ dimensions at neutral-50. **Three new scored dimensions** — BrowseComp (web research depth), SWE-bench Pro (harder code repair), and MMLU-Pro (graduate knowledge) — redistribute 9% of weight from existing dims and add granularity at the frontier level.

**Top 6 (v3.0):**
1. 🥇 **Gemini 3.1 Pro** — 69.5 pts (#1 overall; τ²-Bench corrected to #1 at 99.3%; #1 MMLU-Pro 89.8%; broadest data coverage)
2. 🥈 **GPT-5.5** — 64.0 pts (#2 overall; IFBench and SWE-bench gaps filled; leads 5 agentic dims + BrowseComp; surges from #6)
3. 🥉 **KAT-Coder-Pro-V2** — 58.3 pts (sole cheapest to run, $73.49; #2 Terminal-Bench + Speed)
4. **GLM 5.1** — 58.2 pts (eval cost rank 4/11; #2 IFBench + #3 τ²-Bench; separates from KAT by 0.1 pts)
5. **MiniMax M2.7** — 55.0 pts (near-budget $175.51; cost score 90.0 + IFBench 60.0)
6. **Kimi K2.6** — 52.6 pts (#1 LCB; GDPval updated 1,520; τ²-Bench weakness remains at 0.0)

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
| 12 | DeepSeek V4-Pro | DeepSeek | 2026-04-24 | 1M tokens | Frontier (OSS) |

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
| 1 | Cost Efficiency (AA Index Eval Cost) | Cost | **25%** | Dominant production selection factor; total USD to run AA's full benchmark suite (price × actual benchmark token usage). Reduced 28%→25% in v3.0 to accommodate 3 new dimensions. |
| 2 | Instruction Following (IFBench) | IF | **18%** | Critical for agentic reliability; high-weight due to direct task-completion impact. Reduced 20%→18% in v3.0. |
| 3 | Terminal-Bench 2.0 | Term | **8%** | Primary SE harness benchmark; direct proxy for coding-agent performance. Reduced 9%→8%. |
| 4 | SWE-bench Verified | SWE | **7%** | Real GitHub issue resolution; industry standard for SE agents. Reduced 8%→7%. |
| 5 | SWE-bench Pro *(new v3.0)* | SWEPro | **4%** | Harder single-pass SWE variant; complementary to Verified score. |
| 6 | LiveCodeBench v4 | LCB | **6%** | Contamination-resistant coding; rolling test set. Reduced 7%→6%. |
| 7 | GDPval-AA ELO | GDP | **5%** | Aggregated human preference across 141 models. Unchanged. |
| 8 | Speed (tok/s) | Spd | **5%** | Output throughput; practical for streaming and bulk workloads. Unchanged. |
| 9 | τ²-Bench | τ² | **4%** | Multi-step tool-use reliability; phone/computer agent tasks. Unchanged. |
| 10 | OSWorld | OSW | **4%** | Computer use / GUI agent benchmark. Unchanged. |
| 11 | BrowseComp *(new v3.0)* | BC | **3%** | Deep web research agent; 1,266 hard-to-find information retrieval tasks. |
| 12 | Long-Context Retrieval (RULER) | LCR | **2%** | Needle-in-haystack at 128K+; long-context faithfulness. Reduced 3%→2%. |
| 13 | Humanity's Last Exam (HLE) | HLE | **2%** | Frontier expert knowledge breadth (without-tools, canonical Scale Labs leaderboard). Unchanged. |
| 14 | GPQA Diamond | GPQA | **2%** | Graduate-level science reasoning. Unchanged. |
| 15 | MMLU-Pro *(new v3.0)* | MMLU | **2%** | Graduate-level knowledge breadth; 10-choice enhanced MMLU format. |
| 16 | AIME 2025 | AIME | **1%** | Competition math; reasoning ceiling. Unchanged. |
| 17 | SciCode | Sci | **1%** | Scientific coding (domain-expert tasks). Unchanged. |
| 18 | AA-Omniscience | Omni | **1%** | Hallucination-adjusted accuracy (accuracy% − hallucination_rate%). Unchanged. |
| | **Total** | | **100%** | |

**Weight redistribution v2.5→v3.0:** Added 9% total for 3 new dimensions (SWEPro=4%, BC=3%, MMLU=2%). Offset by reducing: Cost −3%, IF −2%, Term −1%, SWE −1%, LCB −1%, LCR −1%. All other dimensions unchanged. Rationale: proportional reductions from higher-weight dimensions; LCR reduced to 2% given its provisional data footnote (no canonical leaderboard confirmations).

**Framework slots (collected, not yet scored):** BigCodeBench, GAIA Level 3. As of April 25, 2026, no 2026-era frontier models have submitted to these leaderboards in sufficient numbers for activation. Will be re-evaluated in v4.0.

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
| DeepSeek V4-Pro | **⊘** | est. $1,200–$1,800 (unverified) |

> **What is AA Index Eval Cost?** Artificial Analysis runs each model through its full standardized Intelligence Index benchmark suite and records the actual total USD spent — API price × real token consumption across all tasks. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models. Unlike raw $/1M token pricing, eval cost captures true user cost: a verbose model that uses 10× more tokens costs 10× more even at the same token price. Source: artificialanalysis.ai Intelligence Index.
> **DeepSeek V4-Pro:** AA Eval Cost not yet published. Estimated $1,200–$1,800 based on $1.74/$3.48 pricing × AA benchmark token volume norms for MoE models at similar scale. DeepSeek receives ⊘ (neutral-50) for cost scoring pending AA publication.

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
| Unpublished | DeepSeek V4-Pro | est. $1,200–$1,800 |

---

## 5. Raw Benchmark Data

### 5.1 Quality Benchmarks

| Model | IFBench | Term-Bench 2.0 | SWE-Bench V | SWE-Pro | LCB v4 | GDPval ELO | τ²-Bench | OSWorld | RULER (LCR)‡ | HLE | GPQA◇ | AIME 2025 | SciCode |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | 89.4% | 68.5% | 80.6% | ⊘ | 76.3% | 1,314 | **99.3%** | ⊘ | 94.2%‡ | 44.7% | 94.3% | 95.0% | 58.9% |
| Kimi K2.6 | 82.1% | 66.7% | 80.2% | 58.6% | 82.6% | **1,520** | 72.4% | 73.1% | 87.5%‡ | 34.7% | 90.5% | 96.1% | 52.2% |
| MiniMax M2.7 | 75.7% | 57.0% | ⊘ | ⊘ | ⊘ | 1,514 | 84.8% | ⊘ | 81.3%‡ | 28.1% | 87.4% | ⊘ | 47.0% |
| GPT-5.4 | 73.9% | 75.1% | ⊘ | 59.1% | 63.8% | 1,674 | 87.1% | 75.0% | 97.8%‡ | 41.6% | 92.0% | ⊘ | 56.6% |
| Qwen 3.6 Plus | 74.2% | 61.6% | 78.8% | ⊘ | ⊘ | 1,361 | 78.2% | ⊘ | 87.5%‡ | 28.8% | 90.4% | ⊘ | 21.4% |
| KAT-Coder-Pro-V2 | 67.0% | 76.2%† | 79.6% | ⊘ | ⊘ | 1,124 | 93.9% | ⊘ | 74.6%‡ | 12.7% | 85.5% | ⊘ | 38.3% |
| GLM 5.1 | 85.9% | 69.0% | 77.8% | ⊘ | ⊘ | 1,535 | 97.7% | ⊘ | 68.3%‡ | 31.0% | 86.2% | 95.0% | 43.8% |
| Claude Opus 4.7 | 52.1% | 69.4% | 87.6% | 64.3% | ⊘ | 1,753 | 74.0% | 78.0% | 92.4%‡ | 46.9% | 94.2% | 99.8% | 54.5% |
| GPT-5.5 | **75.9%** | **82.7%** | **88.7%†** | 58.6% | ⊘ | 1,784 | 98.0% | 78.7% | ⊘ | 44.3% | 93.6% | ⊘ | ⊘ |
| Claude Sonnet 4.6 | 45.6% | 59.1% | 79.6% | ⊘ | ⊘ | 1,675 | 79.5% | 72.5% | 61.2%‡ | 30.0% | 87.5% | 57.1% | 46.9% |
| Claude Opus 4.6 | 40.2% | 65.4% | 80.8% | 51.9% | 76.0% | 1,619 | 84.8% | 72.7% | 54.7%‡ | 40.0% | 84.0% | 99.8% | 51.9% |
| DeepSeek V4-Pro | ⊘ | ⊘ | 80.6% | ⊘ | **93.5%** | ⊘ | ⊘ | ⊘ | ⊘ | 37.7% | 88.8% | ⊘ | ⊘ |

> **† Footnotes:** KAT-Coder-Pro-V2 TB 2.0 (76.2%) is not on the public tbench.ai leaderboard; value from KAT technical report referencing "Terminal-Bench Hard" — potentially different variant. GPT-5.5 SWE-bench Verified (88.7%) is from OpenAI's launch tech card (April 23); canonical swebench.com leaderboard showed Claude Opus 4.7 (87.6%) as leader as of April 24 — GPT-5.5 formal submission pending. Both marked †.
> **‡ RULER (LCR) footnote:** No model in this cohort was found on any public RULER leaderboard as of 2026-04-25. These values are drawn from model cards and internal evaluation documents; none could be independently confirmed against the canonical ruler-bench.github.io or llm-stats leaderboards. RULER scores should be treated as provisional.
> **Bold** = v3.0 data additions or corrections.

### 5.2 New Scored Dimensions (v3.0)

| Model | BrowseComp | MMLU-Pro |
|---|---|---|
| Gemini 3.1 Pro | 85.9% | **89.8%** |
| Kimi K2.6 | 83.2% | 84.6% |
| MiniMax M2.7 | ⊘ | ⊘ |
| GPT-5.4 | 89.3% | ⊘ |
| Qwen 3.6 Plus | ⊘ | 88.5% |
| KAT-Coder-Pro-V2 | ⊘ | ⊘ |
| GLM 5.1 | ⊘ | ⊘ |
| Claude Opus 4.7 | 79.3% | ⊘ |
| GPT-5.5 | **90.1%** | ⊘* |
| Claude Sonnet 4.6 | ⊘ | ⊘ |
| Claude Opus 4.6 | ⊘ | ⊘ |
| DeepSeek V4-Pro | 83.4% | 87.5% |

> *GPT-5.5 MMLU-Pro = 54.6% seen in one source — anomalously low vs. frontier cluster (84–92%), likely 0-shot vs. few-shot methodology difference. Treated as ⊘ (unreliable) for scoring.

### 5.3 Composite / Special Dimensions

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
| DeepSeek V4-Pro | ⊘ | **33.5** | ⊘ |

> **AA-Omniscience note (GPT-5.5):** Despite being rated #1 on the AA Intelligence Index v4.0 (score 60 xhigh), GPT-5.5's AA-Omniscience score is −29, the worst of all models with real data. This reflects an 86% hallucination rate at high-confidence assertions — a known characteristic of the xhigh reasoning mode that prioritizes bold inference over calibrated uncertainty.

---

## 6. Speed & Latency Reference Data

### 6.1 Inference Speed (tok/s) — Scored Dimension

| Model | Output tok/s | Notes |
|---|---|---|
| Gemini 3.1 Pro | 128.0 | Fastest in field; TPU infrastructure advantage |
| KAT-Coder-Pro-V2 | 113.5 | Second fastest; specialized decode path |
| Kimi K2.6 | 100.4 | Third fastest |
| GPT-5.4 | 74.8 | Moderate throughput; confirmed by AA leaderboard |
| Qwen 3.6 Plus | 53.0 | Mid-range |
| MiniMax M2.7 | 49.0 | Budget tier; tied with GLM |
| GLM 5.1 | 49.0 | Tied with MiniMax |
| Claude Sonnet 4.6 | 46.0 | Moderate |
| Claude Opus 4.7 | 42.0 | Constrained by 200K context |
| DeepSeek V4-Pro | 33.5 | Below open-weight median (55.1 t/s); 1.6T MoE overhead |
| Claude Opus 4.6 | 18.2 | Slowest with real data |
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
| DeepSeek V4-Pro | ~6,800 | Reasoning mode | Estimated; MoE warmup overhead |
| GPT-5.5 | ~172,720 | xhigh reasoning | Full CoT included; not comparable to above |
| KAT-Coder-Pro-V2 | ⊘ | n/a | Not published |

> **Why TTFT is excluded from scoring:** At xhigh, GPT-5.5's TTFT encompasses the entire chain-of-thought computation. Comparing 172,720 ms (GPT-5.5 xhigh) to 1,420 ms (Claude Sonnet) would penalize reasoning models for their reasoning — a category error. Speed (tok/s) is the only throughput dimension scored, as it measures a comparable post-generation metric across all modes.

---

## 7. Normalized Rank Scores

### 7.1 Methodology Recap

For each dimension, models with real data are ranked 1–n (best to worst). Score = `((n − rank) / (n − 1)) × 100`. Missing data = **50⊘**. All columns are fully renormalized for the 12-model v3.0 field. Ties receive the average of their tied ranks.

### 7.2 Complete Normalized Score Matrix (18 dimensions)

| Model | Cost | IF | Term | SWE | SWEPro | LCB | GDP | Spd | τ² | OSW | BC | LCR | HLE | GPQA | MMLU | AIME | Sci | Omni |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | **60.0** | 100.0 | 50.0 | 61.1 | 50.0⊘ | 50.0 | 10.0 | 100.0 | **100.0** | 50.0⊘ | 60.0 | 88.9 | 90.9 | 100.0 | 100.0 | 30.0 | 100.0 | 100.0 |
| Kimi K2.6 | **50.0** | 80.0 | 40.0 | 44.4 | 37.5 | 75.0 | **40.0** | 80.0 | 0.0 | 40.0 | 20.0 | 61.1 | 45.5 | 63.6 | 0.0 | 60.0 | 66.7 | 50.0⊘ |
| MiniMax M2.7 | **90.0** | 60.0 | 0.0 | 50.0⊘ | 50.0⊘ | 50.0⊘ | **30.0** | 45.0 | 45.0 | 50.0⊘ | 50.0⊘ | 44.4 | 9.1 | 27.3 | 50.0⊘ | 50.0⊘ | 44.4 | 60.0 |
| GPT-5.4 | **40.0** | 40.0 | 80.0 | 50.0⊘ | 75.0 | 0.0 | 70.0 | 70.0 | 60.0 | 60.0 | 80.0 | 100.0 | 72.7 | 72.7 | 50.0⊘ | 50.0⊘ | 88.9 | 50.0⊘ |
| Qwen 3.6 Plus | **80.0** | 50.0 | 20.0 | 11.1 | 50.0⊘ | 50.0⊘ | 20.0 | 60.0 | 20.0 | 50.0⊘ | 50.0⊘ | 61.1 | 18.2 | 54.5 | 66.7 | 50.0⊘ | 0.0 | 50.0⊘ |
| KAT-Coder-Pro-V2 | **100.0** | 30.0 | 90.0 | 27.8 | 50.0⊘ | 50.0⊘ | 0.0 | 90.0 | 70.0 | 50.0⊘ | 50.0⊘ | 33.3 | 0.0 | 9.1 | 50.0⊘ | 50.0⊘ | 11.1 | 50.0⊘ |
| GLM 5.1 | **70.0** | 90.0 | 60.0 | 0.0 | 50.0⊘ | 50.0⊘ | 50.0 | 45.0 | 80.0 | 50.0⊘ | 50.0⊘ | 22.2 | 36.4 | 18.2 | 50.0⊘ | 30.0 | 22.2 | 20.0 |
| Claude Opus 4.7 | **10.0** | 20.0 | 70.0 | 88.9 | 100.0 | 50.0⊘ | 90.0 | 20.0 | 10.0 | 80.0 | 0.0 | 77.8 | 100.0 | 90.9 | 50.0⊘ | 90.0 | 77.8 | 80.0 |
| GPT-5.5 | **30.0** | **70.0** | 100.0 | **100.0†** | 37.5 | 50.0⊘ | 100.0 | 50.0⊘ | 90.0 | 100.0 | 100.0 | 50.0⊘ | 81.8 | 81.8 | 50.0⊘ | 50.0⊘ | 50.0⊘ | 0.0 |
| Claude Sonnet 4.6 | **20.0** | 10.0 | 10.0 | 27.8 | 50.0⊘ | 50.0⊘ | 80.0 | 30.0 | 30.0 | 0.0 | 50.0⊘ | 11.1 | 27.3 | 36.4 | 50.0⊘ | 0.0 | 33.3 | 50.0⊘ |
| Claude Opus 4.6 | **0.0** | 0.0 | 30.0 | 77.8 | 0.0 | 25.0 | 60.0 | 0.0 | 45.0 | 20.0 | 50.0⊘ | 0.0 | 63.6 | 0.0 | 50.0⊘ | 90.0 | 55.6 | 40.0 |
| DeepSeek V4-Pro | **50.0⊘** | 50.0⊘ | 50.0⊘ | 61.1 | 50.0⊘ | 100.0 | 50.0⊘ | 10.0 | 50.0⊘ | 50.0⊘ | 40.0 | 50.0⊘ | 54.5 | 45.5 | 33.3 | 50.0⊘ | 50.0⊘ | 50.0⊘ |

**⊘** = missing data, assigned neutral score of 50.0. **Bold** = changed or filled from v2.5.

**Key v3.0 dimension normalization notes:**
- **IF (n=11):** GPT-5.5 75.9% fills gap → rank 4; all existing models re-ranked. GPT-5.5: 50⊘→70.0; GLM shifts 88.9→90.0; Kimi 77.8→80.0.
- **SWE (n=10):** GPT-5.5 88.7%† (#1), DeepSeek 80.6% ties Gemini (rank 4.5). KAT/Sonnet tied (rank 7.5) = 27.8.
- **τ² (n=11):** Gemini 99.3% corrected (#1); GPT-5.5 drops to #2 (90.0); GLM drops to #3 (80.0).
- **GDP (n=11):** Kimi 1,520 moves rank 8→7 (30→40.0); MiniMax moves rank 7→8 (40→30.0).
- **HLE (n=12):** DeepSeek 37.7% enters at rank 6; all scores compress (n=11→n=12).
- **GPQA (n=12):** DeepSeek 88.8% enters at rank 7; existing models shift down one notch.
- **LCB (n=5):** DeepSeek 93.5% enters at rank 1 (#1 in cohort); Kimi drops to rank 2 (75.0).
- **Spd (n=11):** DeepSeek 33.5 tok/s enters at rank 10; Opus 4.6 last (0.0).

### 7.3 Cost Score Derivation

Cost ranks (cheapest = rank 1, n=11 real scorers; DeepSeek ⊘). No ties.

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
| DeepSeek V4-Pro | ⊘ | — | **50.0⊘** |

> **v2.5→v3.0:** Cost column values unchanged for the 11 existing models. DeepSeek V4-Pro receives neutral-50 pending AA publication of its benchmark evaluation cost.

---

## 8. Quality-Only Ranking (Unweighted)

Equally weighting all 17 non-cost dimensions (5.88% each) to show pure capability ranking without cost penalty.

| Rank | Model | IF Score | Benchmark Quality‡ | Total Quality |
|---|---|---|---|---|
| 1 | Gemini 3.1 Pro | 100.0 | 69.7 | 73.1 |
| 2 | Claude Opus 4.7 | 20.0 | 68.3 | 64.4 |
| 3 | GPT-5.4 | 40.0 | 66.6 | 62.9 |
| 4 | GPT-5.5 | 70.0 | 61.3 | 62.4 |
| 5 | DeepSeek V4-Pro | 50.0⊘ | 49.8 | 49.7 |
| 6 | Kimi K2.6 | 80.0 | 44.0 | 47.3 |
| 7 | GLM 5.1 | 90.0 | 40.5 | 43.1 |
| 8 | Qwen 3.6 Plus | 50.0 | 42.0 | 43.0 |
| 9 | MiniMax M2.7 | 60.0 | 39.5 | 42.1 |
| 10 | KAT-Coder-Pro-V2 | 30.0 | 43.0 | 41.8 |
| 11 | Claude Opus 4.6 | 0.0 | 37.9 | 35.7 |
| 12 | Claude Sonnet 4.6 | 10.0 | 33.6 | 32.1 |

> **‡** IF Score = raw normalized IFBench score (0–100 scale; ⊘ = 50.0). Benchmark Quality = equal-weighted avg of the other 16 non-cost dims. Total Quality = equal-weighted avg of all 17 non-cost dims. ⊘ on DeepSeek IF/Term/τ²/etc. = neutral-50 in both averages.

**v3.0 quality shakeup:** Gemini extends its quality lead (73.1) driven by τ²-Bench correction to rank 1 (100.0 from 80.0), adding BrowseComp (60.0) and leading MMLU-Pro (100.0). **Claude Opus 4.7 holds #2** (64.4) with the strongest SWE-Pro profile (100.0, rank 1). **GPT-5.4 overtakes GPT-5.5 for #3** on quality (62.9 vs 62.4) — GPT-5.4 has real data on BrowseComp (80.0) and SWE-Pro (75.0) while GPT-5.5 has ⊘ on Speed, LCR, AIME, SciCode, MMLU-Pro. **GPT-5.5 quality recovers to #4** as IFBench (70.0 from neutral-50) and SWE (100.0) fill in. **DeepSeek debuts at #5** on quality (49.7) — most dimensions ⊘ averaging to 50.0, anchored by strong LCB (100.0).

---

## 9. Final Cost-Weighted Ranking

**Weights:** Cost 25%, IF 18%, Term 8%, SWE 7%, SWEPro 4%, LCB 6%, GDP 5%, Spd 5%, τ² 4%, OSW 4%, BC 3%, LCR 2%, HLE 2%, GPQA 2%, MMLU 2%, AIME 1%, Sci 1%, Omni 1%

### Final Weighted Scores

| Main Rank | Model | Cost (25%) | IF (18%) | Quality (57%) | Main Score |
|---|---|---|---|---|---|
| 🥇 1 | **Gemini 3.1 Pro** | 15.0 | 18.0 | 36.5 | **69.5** |
| 🥈 2 | **GPT-5.5** | 7.5 | 12.6 | 43.9 | **64.0** |
| 🥉 3 | **KAT-Coder-Pro-V2** | 25.0 | 5.4 | 27.9 | **58.3** |
| 4 | **GLM 5.1** | 17.5 | 16.2 | 24.5 | **58.2** |
| 5 | **MiniMax M2.7** | 22.5 | 10.8 | 21.7 | **55.0** |
| 6 | **Kimi K2.6** | 12.5 | 14.4 | 25.7 | **52.6** |
| 7 | **GPT-5.4** | 10.0 | 7.2 | 34.9 | **52.1** |
| 8 | **DeepSeek V4-Pro** | 12.5⊘ | 9.0⊘ | 29.6 | **51.1** |
| 9 | **Qwen 3.6 Plus** | 20.0 | 9.0 | 20.7 | **49.7** |
| 10 | **Claude Opus 4.7** | 2.5 | 3.6 | 36.8 | **42.9** |
| 11 | **Claude Sonnet 4.6** | 5.0 | 1.8 | 19.3 | **26.1** |
| 12 | **Claude Opus 4.6** | 0.0 | 0.0 | 20.6 | **20.6** |

> **Main:** All 12 models, 18 dimensions. Columns: Cost = norm_cost × 0.25 (max 25.0 pts); IF = norm_IF × 0.18 (max 18.0 pts); Quality = weighted sum of remaining 16 dims × their weights (summing to 57%); Main Score = Cost + IF + Quality. `Final = Σ(normalized_score × weight)`.
> **GPT-5.5 note:** As of v3.0, 6 of 17 quality dimensions remain ⊘ (neutral-50): Speed, LCR, AIME, SciCode, MMLU-Pro, and LCB. As these fill in, GPT-5.5's score could rise further — it leads 6 of the 11 quality dimensions with real data.
> **DeepSeek V4-Pro note:** 10 of 18 dimensions are ⊘ (neutral-50): IFBench, TB 2.0, SWEPro, GDPval, τ², OSW, LCR, AIME, SciCode, Omni, and Cost. Score is provisional; expect significant movement as AA Index components become individually published.

### Score Derivation Detail

| Dim | Wt | Gemini | GPT-5.5 | KAT | GLM | MiniMax | Kimi | GPT-5.4 | DeepSeek | Qwen | Opus4.7 | Sonnet | Opus4.6 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Cost | 25% | 15.0 | 7.5 | 25.0 | 17.5 | 22.5 | 12.5 | 10.0 | 12.5 | 20.0 | 2.5 | 5.0 | 0.0 |
| IF | 18% | 18.0 | 12.6 | 5.4 | 16.2 | 10.8 | 14.4 | 7.2 | 9.0 | 9.0 | 3.6 | 1.8 | 0.0 |
| Term | 8% | 4.0 | 8.0 | 7.2 | 4.8 | 0.0 | 3.2 | 6.4 | 4.0 | 1.6 | 5.6 | 0.8 | 2.4 |
| SWE | 7% | 4.3 | 7.0 | 1.9 | 0.0 | 3.5 | 3.1 | 3.5 | 4.3 | 0.8 | 6.2 | 1.9 | 5.4 |
| SWEPro | 4% | 2.0 | 1.5 | 2.0 | 2.0 | 2.0 | 1.5 | 3.0 | 2.0 | 2.0 | 4.0 | 2.0 | 0.0 |
| LCB | 6% | 3.0 | 3.0 | 3.0 | 3.0 | 3.0 | 4.5 | 0.0 | 6.0 | 3.0 | 3.0 | 3.0 | 1.5 |
| GDP | 5% | 0.5 | 5.0 | 0.0 | 2.5 | 1.5 | 2.0 | 3.5 | 2.5 | 1.0 | 4.5 | 4.0 | 3.0 |
| Spd | 5% | 5.0 | 2.5 | 4.5 | 2.25 | 2.25 | 4.0 | 3.5 | 0.5 | 3.0 | 1.0 | 1.5 | 0.0 |
| τ² | 4% | 4.0 | 3.6 | 2.8 | 3.2 | 1.8 | 0.0 | 2.4 | 2.0 | 0.8 | 0.4 | 1.2 | 1.8 |
| OSW | 4% | 2.0 | 4.0 | 2.0 | 2.0 | 2.0 | 1.6 | 2.4 | 2.0 | 2.0 | 3.2 | 0.0 | 0.8 |
| BC | 3% | 1.8 | 3.0 | 1.5 | 1.5 | 1.5 | 0.6 | 2.4 | 1.2 | 1.5 | 0.0 | 1.5 | 1.5 |
| LCR | 2% | 1.78 | 1.0 | 0.67 | 0.44 | 0.89 | 1.22 | 2.0 | 1.0 | 1.22 | 1.56 | 0.22 | 0.0 |
| HLE | 2% | 1.82 | 1.64 | 0.0 | 0.73 | 0.18 | 0.91 | 1.45 | 1.09 | 0.36 | 2.0 | 0.55 | 1.27 |
| GPQA | 2% | 2.0 | 1.64 | 0.18 | 0.36 | 0.55 | 1.27 | 1.45 | 0.91 | 1.09 | 1.82 | 0.73 | 0.0 |
| MMLU | 2% | 2.0 | 1.0 | 1.0 | 1.0 | 1.0 | 0.0 | 1.0 | 0.67 | 1.33 | 1.0 | 1.0 | 1.0 |
| AIME | 1% | 0.30 | 0.5 | 0.5 | 0.30 | 0.5 | 0.60 | 0.5 | 0.5 | 0.5 | 0.90 | 0.0 | 0.90 |
| Sci | 1% | 1.0 | 0.5 | 0.11 | 0.22 | 0.44 | 0.67 | 0.89 | 0.5 | 0.0 | 0.78 | 0.33 | 0.56 |
| Omni | 1% | 1.0 | 0.0 | 0.5 | 0.20 | 0.60 | 0.5 | 0.5 | 0.5 | 0.5 | 0.80 | 0.5 | 0.40 |
| **Total** | | **69.5** | **64.0** | **58.3** | **58.2** | **55.0** | **52.6** | **52.1** | **51.1** | **49.7** | **42.9** | **26.1** | **20.6** |

> **v3.0 main shakeup:** GPT-5.5 surges from #6 → #2 (+7.9 pts) as IFBench (50⊘→70.0) and SWE-bench (50⊘→100.0†) fill in, adding 12.6+7.0=19.6 pts at 18%+7% weight vs. 10.0+3.5 = 13.5 pts from neutral-50. Net gain ≈ +6 pts from IF alone. KAT falls #2→#3 as weight shifts from Cost 28%→25% reduce the budget champion's cost contribution (28→25 pts max). GLM falls #3→#4 (same weight shift). Kimi drops to #6 as its τ²-Bench (0.0) weight advantage compressed. DeepSeek debuts at #8 with provisional score dominated by neutral-50 values.

---

## 10. Ranking Changes (v1.0 → v2.0 → v2.5 → v3.0)

| Model | v1.0 | v2.0 | v2.1.1 | v2.3 | v2.4 | v2.5 | **v3.0** | v3.0 Score | Net Trend |
|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | #1 | #1 | #1 | #2 | #1 | **#1** | **#1** | 69.5 | ↑ Holds #1; τ² now rank 1 (99.3%); MMLU-Pro #1 |
| GPT-5.5 | — | — | #6 | #4 | #8 | **#6** | **#2** | 64.0 | ↑↑ IFBench (⊘→75.9%) + SWE (⊘→88.7%†) fill gaps |
| KAT-Coder-Pro-V2 | #6 | #4 | #5 | #5 | #3 | **#2** | **#3** | 58.3 | ↓ Cost weight 28%→25% reduces budget champion bonus |
| GLM 5.1 | #7 | #7 | #7 | #7 | #5 | **#3** | **#4** | 58.2 | ↓ Same weight shift; holds quality position |
| MiniMax M2.7 | #3 | #3 | #3 | #3 | #4 | **#5** | **#5** | 55.0 | ─ Stable; near-budget tier |
| Kimi K2.6 | #2 | #2 | #2 | #1 | #2 | **#4** | **#6** | 52.6 | ↓ GDP swap with MiniMax; τ²=0.0 weight stable |
| GPT-5.4 | #4 | #5 | #4 | #6 | #7 | **#8** | **#7** | 52.1 | ↑ BrowseComp 80.0 + SWEPro 75.0 (new dims) help |
| DeepSeek V4-Pro | — | — | — | — | — | — | **#8** | 51.1 | NEW; provisional (#10+ dims at ⊘) |
| Qwen 3.6 Plus | #5 | #6 | #8 | #8 | #6 | **#7** | **#9** | 49.7 | ↓ MMLU-Pro helps (66.7) but other gaps hurt |
| Claude Opus 4.7 | #8 | #8 | #9 | #9 | #9 | **#9** | **#10** | 42.9 | ─ Quality #2 but BrowseComp 0.0 (last/6) hurts |
| Claude Sonnet 4.6 | #9 | #10 | #10 | #10 | #10 | **#10** | **#11** | 26.1 | ─ |
| Claude Opus 4.6 | #10 | #9 | #11 | #11 | #11 | **#11** | **#12** | 20.6 | ─ Most expensive to run |

---

## 11. Model Profiles

### 11.1 Gemini 3.1 Pro 🥇
**Overall:** #1 | **Quality:** #1 | **Cost tier:** Premium ($892.28 eval cost)

**v3.0 extends Gemini's lead.** Three v3.0 changes work in Gemini's favor simultaneously: (1) τ²-Bench corrected to 99.3% → rank 1 (100.0 from 80.0), adding +0.8 pts at 4% weight. (2) New MMLU-Pro dimension where Gemini leads (89.8%, rank 1/4, score 100.0), worth +2.0 pts at 2% weight. (3) BrowseComp at 85.9% (rank 3/6, score 60.0), worth +1.8 pts at 3% weight. Net quality gain ≈ +4.6 pts vs. v2.5, offset by cost weight reduction 28%→25% (−1.7 pts). Final score: 69.5.

**Why Gemini holds #1:** The only model with real data on 17 of 18 dimensions (only OSWorld ⊘ — confirmed no published score). Leads on IFBench (100.0), Speed (100.0), τ²-Bench (100.0), GPQA Diamond (100.0), SciCode (100.0), MMLU-Pro (100.0), AA-Omniscience (100.0). GDP (10.0 — rank 10/11) remains the structural weakness: human preference evaluators consistently rank Gemini below peers on conversational quality.

**Best for:** Production SE workloads requiring consistent, instruction-following intelligence at scale. Long-context tasks (2M token window). Coverage-critical deployments where missing-data risk is unacceptable.

---

### 11.2 GPT-5.5 🥈
**Overall:** #2 | **Quality:** #4 | **Cost tier:** Very Expensive ($3,357.00 eval cost)

**The v3.0 story is GPT-5.5's emergence.** Two gaps fill simultaneously: IFBench (⊘→75.9%, rank 4/11, score 70.0) at 18% weight and SWE-bench Verified (⊘→88.7%†, rank 1/10, score 100.0) at 7% weight. Together these add ≈ 9.1 pts vs. neutral-50 placeholders. Combined with being #1 on 5 dimensions (Terminal-Bench 82.7%, GDP 1,784, τ² 98.0%, OSWorld 78.7%, BrowseComp 90.1%), GPT-5.5 at 64.0 pts is a clear #2 with only 6 dimensions still ⊘ (Speed, LCB, LCR, AIME, SciCode, MMLU-Pro). As those fill in, GPT-5.5 could reach #1.

**Remaining gaps (6 of 18 ⊘):** Speed (not published for xhigh mode), LCB (not submitted), LCR (not published), AIME 2025 (OpenAI hasn't published), SciCode (not submitted), MMLU-Pro (anomalous 54.6% treated as ⊘). Each confirmed gap at neutral-50 represents a ±several-point swing when real data arrives.

**AA-Omniscience caveat (0.0 score):** The −29 raw score (57% accuracy − 86% hallucination at xhigh) is GPT-5.5's one confirmed weakness. At lower reasoning tiers, this would differ significantly.

**Best for:** Terminal/CLI agents (#1 TB 2.0), computer-use/GUI agents (#1 OSWorld), deep web research (#1 BrowseComp), τ²-Bench-sensitive agentic pipelines, SWE-bench-sensitive code repair (probable #1).

---

### 11.3 KAT-Coder-Pro-V2 🥉
**Overall:** #3 | **Quality:** #10 | **Cost tier:** Budget ($73.49 eval cost)

**v3.0 cost weight compression reduces KAT's advantage slightly.** Cost weight drops 28%→25%, reducing KAT's cost contribution from 28.0 → 25.0 pts (−3.0). KAT stays at rank 1 eval cost (100.0) but earns fewer points for it. Quality profile unchanged: #2 Terminal-Bench 2.0 (90.0), #2 Speed (90.0), eval cost sole rank 1. At 58.3 pts, KAT drops from #2 to #3 but remains the best cost-to-terminal-performance ratio in the field by a wide margin.

**New dimensions:** All three new dims (SWEPro, BrowseComp, MMLU-Pro) are ⊘ → neutral-50. These contribute 3.0+1.5+1.0 = 5.5 pts at equal neutral-50 weight — no discrimination against or for KAT. KAT's strong profile (100+90+90 on three key dims) keeps it competitive.

**Best for:** Specialized terminal/coding pipelines requiring fast, cost-efficient output at scale. Not a general-purpose model; avoid for reasoning-heavy, knowledge, or GUI tasks (HLE 0.0, GPQA 9.1).

---

### 11.4 GLM 5.1
**Overall:** #4 | **Quality:** #7 | **Cost tier:** Mid-Range ($543.95 eval cost)

GLM separates from KAT by only 0.1 pts (58.2 vs 58.3). GLM's profile: #2 IFBench (90.0), #3 τ²-Bench (80.0), eval cost rank 4 (70.0). New dimensions all ⊘ → neutral-50 (neither helps nor hurts). The τ²-Bench correction that made Gemini rank 1 moved GLM from rank 2 to rank 3 (from 90.0 to 80.0, −0.4 pts at 4% weight).

**Best for:** Instruction-following tasks requiring τ²-Bench multi-step reliability at mid-range cost. A strong alternative to Gemini for budget-conscious instruction-following workloads.

---

### 11.5 MiniMax M2.7
**Overall:** #5 | **Quality:** #9 | **Cost tier:** Near-Budget ($175.51 eval cost)

MiniMax holds near-budget status (eval cost rank 2, score 90.0). New dimensions: SWEPro ⊘, BrowseComp ⊘, MMLU-Pro ⊘ — all neutral-50. One v3.0 change affects MiniMax: the GDPval ranking swap with Kimi. Kimi's GDPval ELO updated to 1,520 moves it to rank 7; MiniMax at 1,514 drops to rank 8 (score 40.0→30.0, −0.5 pts at 5% weight). Final: 55.0 pts.

**Best for:** High-volume classification, summarization, extraction tasks at near-budget cost. Not suitable for coding-agent applications (Terminal-Bench last in field, 0.0).

---

### 11.6 Kimi K2.6
**Overall:** #6 | **Quality:** #6 | **Cost tier:** Premium ($947.87 eval cost)

Kimi's v3.0 changes: GDPval ELO updated 1,484→1,520 → GDP score improves 30.0→40.0 (+0.5 pts at 5% weight). SWE-bench Pro activated at 58.6% (tied rank 3.5/5 with GPT-5.5) → score 37.5 vs. prior ⊘ neutral-50 (−0.5 pts vs. prior neutral). BrowseComp 83.2% (rank 5/6) → score 20.0 (below neutral-50, −0.9 pts vs. prior ⊘). MMLU-Pro 84.6% (rank 4/4) → score 0.0 (well below neutral-50, −1.0 pts vs. prior ⊘). Net v3.0 quality impact: roughly neutral to slightly negative from the 3 new dimensions despite having real data for all of them. τ²-Bench (0.0 — rank 11/11) remains the dominant weakness at 4% weight.

**LCB update:** DeepSeek V4-Pro (93.5%) enters at LCB rank 1, pushing Kimi to rank 2 (75.0 from 100.0, −0.84 pts at 6% weight). Kimi is no longer the LCB champion.

**Best for:** Coding-heavy pipelines (LCB #2 at 75.0), open-weight deployments (Apache 2.0), long-context tasks (200K), SWE-bench Pro is competitive (58.6%).

---

### 11.7 GPT-5.4
**Overall:** #7 | **Quality:** #3 | **Cost tier:** Expensive ($2,851.01 eval cost)

GPT-5.4 benefits from two new dimensions: BrowseComp 89.3% (rank 2/6, score 80.0) and SWE-bench Pro 59.1% (rank 2/5, score 75.0). These add 2.4+3.0 = 5.4 pts above prior neutral-50 (at 3%+4% weight). LCR holds #1 (100.0). This earns GPT-5.4 quality rank #3 (62.9). But eval cost ($2,851, rank 7/11, score 40.0) at 25% weight caps the total at 52.1 pts — rank #7 overall.

**Best for:** Long-context retrieval (#1 LCR 100.0), BrowseComp web research (#2), SWE-bench Pro (2nd best), Terminal-Bench (80.0), GPQA (72.7). Teams running research, retrieval, or code repair at expensive-but-not-ultra-expensive tier.

---

### 11.8 DeepSeek V4-Pro *(new)*
**Overall:** #8 | **Quality:** #5 | **Cost tier:** Unpublished (est. $1,200–$1,800)

**Profile:** DeepSeek V4-Pro (released April 24, 2026) is Apache 2.0 open-source, 1.6T total / 49B active parameters (MoE), 1M context window. AA Intelligence Index: 52 (#2 open-weight behind Kimi K2.6 at 54). Pricing: $1.74/$3.48 per MTok in/out — below Kimi's comparable tier.

**Confirmed strengths:** LiveCodeBench v4 (93.5%, rank 1/5 in cohort — new LCB champion), SWE-bench Verified (80.6%, tied rank 4.5/10), BrowseComp (83.4%, rank 4/6). Speed (33.5 tok/s) is below the open-weight median — MoE routing overhead at 1.6T total parameters.

**Confirmed weaknesses:** Slow throughput (33.5 tok/s, rank 10/11, score 10.0), HLE (37.7%, rank 6/12, score 54.5 — below Claude 40.0%, GPT-5.4 41.6%, Gemini 44.7%). GPQA Diamond (88.8%) at probable-confidence suggests mid-tier science reasoning.

**⊘ coverage:** 10 of 18 dimensions are ⊘: IFBench, Terminal-Bench 2.0 Pro (only Flash 79.0% found), GDPval-AA ELO, τ²-Bench, OSWorld, RULER, AIME 2025 (87.5% seen but attributed to "DeepSeek" — may be R1), SciCode, AA-Omniscience, and Cost. All receive neutral-50. The AA Intelligence Index composite (52) confirms overall frontier parity but component breakdowns are not individually published.

**Expected trajectory:** As leaderboards process the April 24 release, expect IFBench, τ², and Cost to fill in within 2–4 weeks. Based on the composite AA score and architecture, realistic IFBench estimate: 70–75%; τ² estimate: 85–92%; Cost estimate: likely rank 7 ($1,400 est., between Kimi $948 and GPT-5.4 $2,851).

**Best for:** LiveCodeBench coding tasks (#1), SWE-bench GitHub issue resolution, BrowseComp research tasks, open-source deployments requiring on-premise licensing. Avoid latency-sensitive streaming applications (33.5 tok/s).

---

### 11.9 Qwen 3.6 Plus
**Overall:** #9 | **Quality:** #8 | **Cost tier:** Mid-Range ($482.65 eval cost)

Qwen benefits from MMLU-Pro where it ranks #2 (88.5%, score 66.7) — a meaningful data point at 2% weight (+0.33 pts vs. neutral-50). BrowseComp ⊘ (neutral-50). SWEPro ⊘ (neutral-50). LCR unchanged at 61.1 (tied rank 4.5 with Kimi). τ²-Bench (20.0, rank 9/11) remains a weakness. Final: 49.7 pts.

**Best for:** High-volume mid-range workloads. MMLU-Pro strength suggests knowledge-breadth tasks. Avoid τ²-Bench-sensitive or agentic pipelines.

---

### 11.10 Claude Opus 4.7
**Overall:** #10 | **Quality:** #2 | **Cost tier:** Ultra-Premium ($4,811.04 eval cost)

**The v3.0 story for Opus 4.7 is BrowseComp (0.0 — rank 6/6, last).** Opus 4.7 at 79.3% is last in the 6-model BrowseComp cohort, worth 0.0 at 3% weight vs. neutral-50 in v2.5 (−1.5 pts). SWE-bench Pro offset this: Opus 4.7 leads with 64.3% (rank 1/5, score 100.0) → +2.0 pts at 4% weight vs. neutral-50. Net from new dims: roughly neutral. Eval cost contribution drops (28%→25% weight) by −0.7 pts. Final: 42.9 pts.

**Quality-cost paradox:** #2 quality overall (64.4 pts) but #10 in the main ranking. The 10-rank gap between quality and overall position remains the largest in the field. Cost score 10.0 at 25% weight contributes only 2.5 pts — Gemini earns 15.0 pts from cost. For teams prioritizing capability over cost, Opus 4.7 is the go-to for SWE-bench (#1, 88.9), SWE-bench Pro (#1, 100.0), HLE (#1, 100.0), GPQA (#2, 90.9).

**Best for:** High-stakes software engineering, SWE-bench Pro (#1 at 64.3%), expert-knowledge tasks (HLE #1), enterprise customers where quality-per-outcome matters more than cost.

---

### 11.11 Claude Sonnet 4.6
**Overall:** #11 | **Quality:** #12 | **Cost tier:** Very Expensive ($3,959.36 eval cost)

All three new dimensions (SWEPro, BrowseComp, MMLU-Pro) are ⊘ → neutral-50 for Sonnet. No changes to existing data. GDP at 80.0 (rank 3/11 — 1,675 ELO) and Terminal-Bench (10.0, rank 10/11) continue to bookend Sonnet's profile. Final: 26.1 pts.

**Best for:** Simple conversational tasks where Anthropic ecosystem compatibility is required. Not recommended for coding-agent, agentic, or math-intensive pipelines.

---

### 11.12 Claude Opus 4.6
**Overall:** #12 | **Quality:** #11 | **Cost tier:** Ultra-Premium ($4,969.68 eval cost)

Most expensive model to run ($4,969.68, rank 11/11, cost score 0.0). SWE-bench Pro (51.9%, rank 5/5, score 0.0) is Opus 4.6's worst result in the new dimensions — last in the SWE-Pro cohort. BrowseComp and MMLU-Pro remain ⊘ (neutral-50). AIME 2025 tied rank 1.5 at 99.8% (90.0) and SWE-bench #3 (77.8) remain the dimension bright spots. At essentially equivalent eval cost to Opus 4.7 ($4,970 vs $4,811) but lower quality on nearly every dimension, migration to 4.7 is the dominant recommendation.

**Best for:** Legacy deployments that cannot migrate; no new deployments recommended.

---

## 12. Strategic Insights

### 12.1 GPT-5.5 Is Now a Clear #2 — But Provisional

v3.0 fills two critical GPT-5.5 gaps: IFBench (75.9%, rank 4/11) and SWE-bench Verified (88.7%†, rank 1/10). These two dimensions — at 18% and 7% weight — account for +6.1 pts of GPT-5.5's total improvement. The model now holds confirmed #1 on 5 agentic dimensions (TB 2.0, GDP, τ², OSWorld, BrowseComp) and probable #1 on SWE-bench Verified. Six dimensions remain ⊘. When Speed, LCB, AIME, SciCode, LCR, and MMLU-Pro fill in, GPT-5.5 has a realistic path to #1 if its quality profile holds.

**The remaining gap:** Gemini leads GPT-5.5 by 5.5 pts. Gemini has zero ⊘ quality dimensions — it cannot be displaced by "surprise" real data. GPT-5.5 must earn its remaining quality scores; each neutral-50 dim is a risk in both directions.

### 12.2 DeepSeek V4-Pro: Open-Source Frontier Contender

DeepSeek V4-Pro's debut at #8 (51.1 pts) is provisional but meaningful. The model's LCB leadership (93.5%, above Kimi's 82.6%) establishes it as the current open-source coding champion at the competition-programming level. Its Codeforces rating (3,206) and Putnam-2025 (120/120, first open-source perfect score) confirm exceptional mathematical reasoning not yet captured in the scored dimensions.

The speed penalty (33.5 tok/s, rank 10/11) is significant for streaming applications — nearly 3× slower than Gemini (128.0 tok/s). For batch processing and offline coding tasks, the speed disadvantage matters less. At $1.74/$3.48 pricing with a 1M context window, DeepSeek V4-Pro offers frontier-class coding capability at mid-range token cost, making it a strong open-source alternative to Kimi K2.6 for LCB-type tasks.

### 12.3 The New Dimension Landscape

Three new dimensions reveal new capability differentiators at the frontier:

- **BrowseComp** separates GPT-5.5 (90.1%, #1) and GPT-5.4 (89.3%, #2) as superior deep-web research agents, with Claude Opus 4.7 (79.3%) last in the 6-model cohort — a meaningful finding for enterprise research automation decisions.
- **SWE-bench Pro** reveals Claude Opus 4.7 as the definitive code repair leader (64.3%, 5.2% gap to #2 GPT-5.4 at 59.1%). This harder benchmark better differentiates frontier models than the Verified variant where the top 10 cluster within 11 percentage points.
- **MMLU-Pro** confirms Gemini's knowledge breadth leadership (89.8%) and places Qwen at #2 (88.5%) — giving Qwen its first clear dimension strength in this cohort.

### 12.4 Kimi K2.6 Is No Longer the LCB Champion

DeepSeek V4-Pro's 93.5% on LiveCodeBench v4 displaces Kimi K2.6 (82.6%) as LCB leader. The 10.9-point gap is substantial. Combined with Kimi's τ²-Bench weakness (0.0, last in field), this raises questions about Kimi's "coding specialist" positioning. Kimi remains competitive on SWE-bench (80.2%) and SWE-bench Pro (58.6%), but for pure coding evaluation benchmarks, DeepSeek V4-Pro is the new open-source standard.

### 12.5 The Anthropic Cost Structure Remains the Dominant Constraint

Under the AA Index Eval Cost methodology, all three Anthropic frontier models (Sonnet $3,959, Opus 4.7 $4,811, Opus 4.6 $4,970) are in the top-4 most expensive to run. At 25% cost weight, Claude Opus 4.7's cost score (10.0) contributes only 2.5 pts vs. Gemini's 15.0 pts — a 12.5 pt cost disadvantage that quality (#2 at 64.4 pts) cannot overcome at this weighting. Teams running cost-unconstrained research pipelines where SWE-Pro or HLE performance matters most should consider Opus 4.7; all production workloads face a structural cost ceiling.

---

## 13. Use Case Recommendations

| Use Case | Primary Choice | Budget Alternative | Notes |
|---|---|---|---|
| Production SE pipeline (cost-first) | **Gemini 3.1 Pro** | GLM 5.1 | #1 overall at 69.5 pts; broadest coverage; $892 eval cost |
| Broadest-coverage production choice | **Gemini 3.1 Pro** | GPT-5.5 | Real data on 17/18 dims; #1 IFBench, Speed, τ², GPQA, Sci, MMLU |
| Minimum eval-cost processing | **KAT-Coder-Pro-V2** | MiniMax M2.7 | $73.49 eval cost (#1) vs $175.51 (#2) |
| High-volume batch processing | **KAT-Coder-Pro-V2** | MiniMax M2.7 | KAT: $73.49 + TB 2.0 #2; MiniMax: $175.51 + IFBench #5 |
| Coding specialist (LCB / competition) | **DeepSeek V4-Pro** | Kimi K2.6 | LCB #1 (93.5%) vs LCB #2 (82.6%); both open-source |
| Coding specialist (terminal/CLI) | **GPT-5.5** | KAT-Coder-Pro-V2 | TB 2.0: 82.7% (#1) vs 76.2% (#2) |
| Computer-use / GUI automation | **GPT-5.5** | Claude Opus 4.7 | OSWorld #1 (78.7%) by large margin |
| Multi-step agentic tasks (τ²) | **Gemini 3.1 Pro** | GPT-5.5 | τ²: 99.3% (#1) vs 98.0% (#2) — Gemini now leads |
| SWE-bench / GitHub issue resolution | **GPT-5.5†** | Claude Opus 4.7 | SWE-V: 88.7%† (#1, probable) vs 87.6% (#2, confirmed) |
| SWE-bench Pro / hard code repair | **Claude Opus 4.7** | GPT-5.4 | SWE-Pro: 64.3% (#1) — 5.2% lead over GPT-5.4 (59.1%) |
| Deep web research / BrowseComp | **GPT-5.5** | GPT-5.4 | BrowseComp: 90.1% (#1) vs 89.3% (#2) |
| Long-context retrieval (128K+) | **GPT-5.4** | Gemini 3.1 Pro | RULER: 97.8% vs 94.2% |
| Competition math / reasoning ceiling | **Claude Opus 4.7 / 4.6** | Kimi K2.6 | AIME: 99.8% (tied Opus) vs 96.1% (Kimi) |
| Graduate-level knowledge (MMLU-Pro) | **Gemini 3.1 Pro** | Qwen 3.6 Plus | MMLU-Pro: 89.8% (#1) vs 88.5% (#2) |
| Instruction-following reliability | **Gemini 3.1 Pro** | GLM 5.1 | IFBench: 89.4% (#1) vs 85.9% (#2) |
| Fast streaming / real-time | **Gemini 3.1 Pro** | KAT-Coder-Pro-V2 | 128.0 tok/s (#1) vs 113.5 tok/s (#2) |
| Open-source / on-premise frontier | **DeepSeek V4-Pro** | Kimi K2.6 | LCB #1, Apache 2.0, 1M context, $1.74/MTok |
| Highest quality, cost unconstrained | **GPT-5.5** | Claude Opus 4.7 | 5 agentic dim leads; Opus 4.7 #2 quality (64.4) |
| Expert knowledge / HLE tasks | **Claude Opus 4.7** | Gemini 3.1 Pro | HLE #1 (46.9%) vs #2 (44.7%) |
| Mid-range instruction + agentic | **GLM 5.1** | Qwen 3.6 Plus | #4 overall (58.2); $544 eval cost; IFBench #2 + τ² #3 |

---

## 14. Limitations & Known Gaps

### 14.1 Data Freshness

- **DeepSeek V4-Pro** (April 24, 2026): provisional entry. 10+ dimensions ⊘. Expect rapid data availability in next 2–4 weeks as AA Index processes and leaderboards update.
- **GPT-5.5** (April 23, 2026): 6 dimensions still ⊘. IFBench and SWE-bench Verified filled in v3.0. Speed, LCB, LCR, AIME, SciCode, MMLU-Pro pending.
- All other model data is sourced from official publications and established leaderboards as of April 25, 2026.

### 14.2 GPT-5.5 SWE-bench Verified Uncertainty

The 88.7%† value is from OpenAI's launch tech card (April 23). The canonical swebench.com leaderboard showed Claude Opus 4.7 (87.6%) as leader as of April 24 — GPT-5.5 may not have been formally submitted yet. If confirmed, GPT-5.5 becomes #1 on SWE-bench Verified. If actual swebench.com leaderboard result differs materially, the SWE column will be corrected in v3.1.

### 14.3 Benchmark Contamination Risk

LiveCodeBench v4 and τ²-Bench use rolling test sets specifically to address contamination, but older benchmarks (GPQA, SWE-bench) may have varying contamination levels across models depending on training cutoff and data curation practices.

### 14.4 Mode Selection Ambiguity

Several models offer multiple reasoning modes. Published benchmark scores typically reflect the best available mode. Rankings may not reflect the model's behavior at its production-default mode.

### 14.5 GPT-5.5 AA-Omniscience

The 86% hallucination rate driving the −29 raw Omniscience score comes from early independent evaluations specifically targeting GPT-5.5's xhigh reasoning mode. At standard or high reasoning modes, both accuracy and hallucination rates would differ significantly.

### 14.6 GPT-5.4 AIME Conflict

LayerLens independent testing: 16.67%. Multiple secondary sources attribute 100% performance to GPT-5.4 (possibly misattributed from GPT-5.2 xhigh). Unresolvable with current data. GPT-5.4 receives ⊘=50 on AIME 2025.

### 14.7 New Dimension Coverage Thresholds

BrowseComp (6/12 models), SWE-bench Pro (5/12), MMLU-Pro (4/12) were activated at v3.0 at minimum threshold. Models receiving ⊘ on these dimensions (especially MMLU-Pro, where 8/12 are ⊘) receive neutral-50, reducing the dimension's discriminative power. As coverage improves in v3.1+, these dimensions will gain more precision.

### 14.8 Framework Design Constraints

The 25% cost weight reflects a deliberate choice for production scale optimization. Academic or research applications would reasonably down-weight cost and up-weight quality dimensions. Custom weight configurations are recommended for teams with different budget constraints.

### 14.9 BigCodeBench and GAIA Slots

Both benchmarks remain reserved as scored dimensions pending ≥4 model submissions from the current 12-model inventory. Will be re-evaluated in v4.0.

---

## 15. Benchmark Reference Index

| Benchmark | Dimension | Type | What it Measures | Source |
|---|---|---|---|---|
| AA Index Eval Cost | Cost | USD | Total cost to run AA's full Intelligence Index benchmark suite (price × actual token usage) | artificialanalysis.ai |
| IFBench | IF | Accuracy % | Instruction-following compliance across task types | Artificial Analysis IFBench leaderboard |
| Terminal-Bench 2.0 | Term | Accuracy % | Terminal/CLI agentic task completion | tbench.ai |
| SWE-bench Verified | SWE | Accuracy % | Real GitHub issue resolution (verified subset) | swebench.com |
| SWE-bench Pro | SWEPro | Accuracy % | Harder single-pass GitHub issue resolution; stricter criteria | Scale Labs leaderboard |
| LiveCodeBench v4 | LCB | Accuracy % | Rolling contamination-resistant coding eval | livecodebench.github.io |
| GDPval-AA ELO | GDP | ELO rating | Human preference pairwise comparisons, 141 models | Artificial Analysis GDPval |
| Speed (tok/s) | Spd | Tokens/second | Output generation throughput | Artificial Analysis; provider specs |
| τ²-Bench | τ² | Accuracy % | Multi-step phone/computer agent task success | Artificial Analysis τ²-Bench leaderboard |
| OSWorld | OSW | Accuracy % | Desktop GUI agent task success | os-world.github.io |
| BrowseComp | BC | Accuracy % | Persistent web navigation; 1,266 hard-to-find information retrieval | BenchLM / llm-stats.com/benchmarks/browsecomp |
| RULER (LCR) | LCR | Accuracy % | Long-context retrieval fidelity (128K–1M) | ruler-bench.github.io (provisional data) |
| Humanity's Last Exam | HLE | Accuracy % | Expert-level knowledge breadth; 3,000 questions; WITHOUT-TOOLS canonical | Scale Labs / agi.safe.ai |
| GPQA Diamond | GPQA | Accuracy % | Graduate-level science reasoning (diamond subset) | arXiv:2311.12022 |
| MMLU-Pro | MMLU | Accuracy % | Enhanced MMLU with 10-choice reasoning-focused questions | Artificial Analysis / Kaggle MMLU-Pro leaderboard |
| AIME 2025 | AIME | Accuracy % | American Invitational Mathematics Exam 2025 | aops.com / arXiv reports |
| SciCode | Sci | Accuracy % | Scientific coding tasks (domain-expert problems) | scicode-bench.github.io |
| AA-Omniscience | Omni | Score | accuracy% − hallucination_rate% | Artificial Analysis Omniscience v2 |
| TTFT | Ref only | Milliseconds | Time-to-first-token (not scored; see §3.3) | Artificial Analysis |
| BigCodeBench | (reserved) | Accuracy % | Function-level coding with complex instructions | bigcodebench.github.io |
| GAIA Level 3 | (reserved) | Accuracy % | General AI assistant real-world tasks, hardest tier | huggingface.co/GAIA |

---

## 16. Methodology Appendix

### 16A. Normalization Examples

**Example 1: IFBench v3.0 (n=11 models with real data, 1 model ⊘)**

Raw scores: Gemini 89.4%, GLM 85.9%, Kimi 82.1%, GPT-5.5 75.9%, MiniMax 75.7%, Qwen 74.2%, GPT-5.4 73.9%, KAT 67.0%, Opus 4.7 52.1%, Sonnet 45.6%, Opus 4.6 40.2%
DeepSeek: ⊘ → 50.0

Ranks (1=best): Gemini=1, GLM=2, Kimi=3, GPT-5.5=4, MiniMax=5, Qwen=6, GPT-5.4=7, KAT=8, Opus 4.7=9, Sonnet=10, Opus 4.6=11
Formula `((n-rank)/(n-1)) × 100` where n=11:
- Gemini: (10/10)×100 = **100.0**
- GLM: (9/10)×100 = **90.0**
- GPT-5.5: (7/10)×100 = **70.0** ← filled from ⊘ in v2.5
- Opus 4.6: (0/10)×100 = **0.0**

**Example 2: Cost v3.0 (n=11 real scorers, DeepSeek ⊘)**

Same as v2.5. KAT-Coder-Pro-V2: rank 1 → (10/10)×100 = **100.0**. Gemini: rank 5 → (6/10)×100 = **60.0**. Opus 4.6: rank 11 → (0/10)×100 = **0.0**. DeepSeek: ⊘ → **50.0⊘**.

**Example 3: SWE-bench Verified v3.0 (n=10 scorers, 2 ties)**

GPT-5.5 (88.7%) rank 1, Opus 4.7 (87.6%) rank 2, Opus 4.6 (80.8%) rank 3, Gemini+DeepSeek (80.6%) tie rank 4.5, Kimi (80.2%) rank 6, KAT+Sonnet (79.6%) tie rank 7.5, Qwen (78.8%) rank 9, GLM (77.8%) rank 10. MiniMax+GPT-5.4 ⊘.
Ties: `(n-avg_rank)/(n-1)×100`: Gemini/DeepSeek = (10-4.5)/9×100 = **61.1**; KAT/Sonnet = (10-7.5)/9×100 = **27.8**.

### 16B. Sensitivity Analysis

**What if cost weight = 15% (quality-forward scenario)?**

| Rank | Model | Score |
|---|---|---|
| 1 | Gemini 3.1 Pro | 72.8 |
| 2 | GPT-5.5 | 67.7 |
| 3 | Claude Opus 4.7 | 61.8 |
| 4 | GPT-5.4 | 56.5 |
| 5 | GLM 5.1 | 55.0 |

At 15% cost weight, GPT-5.5 extends its #2 advantage (quality leads on 6+ agentic dims). Claude Opus 4.7 recovers to #3 as cost penalty ($4,811) shrinks. KAT-Coder-Pro-V2 drops out of top-5 as its quality (#10) becomes the binding constraint at lower cost weight.

**What if cost weight = 40% (cost-extreme scenario)?**

| Rank | Model | Score |
|---|---|---|
| 1 | KAT-Coder-Pro-V2 | 66.1 |
| 2 | Gemini 3.1 Pro | 65.2 |
| 3 | MiniMax M2.7 | 62.8 |
| 4 | GLM 5.1 | 60.8 |
| 5 | Qwen 3.6 Plus | 57.0 |

At 40% cost weight, KAT overtakes Gemini. MiniMax rises to #3. GPT-5.5 drops to #8 (cost 40% weight at 30.0 score = 12.0 pts). Claude Opus 4.6 collapses to #12 (cost score 0.0 at 40% = structurally fatal).

### 16C. Adding a New Model (Maintenance Protocol)

When a new model is released:

1. **Collect all 18 dimension values** — use official publications, then established third-party evaluators (Artificial Analysis, LMSYS, alo-exp leaderboards). Mark any missing as ⊘.
2. **Recompute all ranks** — every dimension with real data for the new model requires full reranking of ALL models in that dimension. Do not preserve old ranks.
3. **Apply normalization formula** using the new `n` (count of models with real data per dimension).
4. **Recalculate all final scores** — renormalization changes every model's score, not just the new arrival's.
5. **Update Ranking Changes table** — document Δ vs previous version for every model.
6. **Tag version** — increment minor version (e.g., v3.0 → v3.1) for model additions; increment major version for dimension additions or weight changes.
7. **Flag provisional** — mark models with >6 missing dimensions as "provisional ranking" in the Executive Summary.

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
| Quarterly | Full data refresh on all 18 dimensions for all models |
| Semi-annual | Weight review — validate 25% cost weight against production survey data |

### 16G. v3.0 Research Sources

Key sources used for v3.0 data updates (from deep-research pipeline, April 25, 2026):
- Artificial Analysis Kimi K2.6 article (GDPval 1,520 confirmed)
- Digital Applied / AA τ²-Bench leaderboard (Gemini 99.3% confirmed)
- OpenAI GPT-5.5 technical card, Harvey research preview, designforonline.com (IFBench 75.9% confirmed)
- VentureBeat, BuildFastWithAI (DeepSeek V4-Pro SWE-bench 80.6% confirmed)
- BuildFastWithAI, OfficeChai (DeepSeek V4-Pro LCB 93.5% confirmed)
- Artificial Analysis (DeepSeek V4-Pro Speed 33.5 tok/s confirmed)
- Scale Labs leaderboard, agi.safe.ai (HLE methodology resolved: without-tools canonical)
- BenchLM, Vellum, multiple sources (BrowseComp scores for 6 models)
- Scale Labs SWE-bench Pro leaderboard, marc0.dev (SWE-Pro scores for 5 models)
- Artificial Analysis MMLU-Pro, Kaggle MMLU-Pro leaderboard (MMLU-Pro scores for 4 models)

---

*Report maintained by the Kay project team | Feedback: kay-rankings@alo-exp.dev*
*Methodology questions: see §16 Appendix | Version history: tracked in git history of this file*
*Next scheduled update: May 2026 quarterly refresh or next major model launch, whichever comes first*
