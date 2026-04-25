# AI Model Rankings — Comprehensive Cost-Weighted Analysis
**Version 4.2 | April 26, 2026 | 12 Models | 18 Dimensions**

> **Change log:**
> **v2.0 → v2.1:** Added GPT-5.5 (launched April 23, 2026). All 15 normalized score columns renormalized.
> **v2.1 → v2.1.1:** Corrected GPT-5.5 cost rank (was erroneously rank 11; corrected to rank 9 at $11.25/1M < Opus $30.00/1M).
> **v2.1.1 → v2.2 (full matrix audit):** Fixed 20 cell errors across 7 dimensions — all were pairwise swap errors during normalization.
> **v2.2 → v2.3 (Terminal-Bench 2.0 data refresh):** Replaced stale TB 2.0 column with current leaderboard data; GPT-5.5 confirmed #1 at 82.7%; GPT-5.4 corrected from 94.5% to 75.1%. Kimi overtook Gemini for #1 (73.1 vs 72.7).
> **v2.3 → v2.4 (full provenance audit):** Systematic cross-check of every benchmark value against authoritative leaderboard sources. All benchmark columns except TB 2.0 corrected from stale late-2024/early-2025 data. Cost column: 9/11 API prices corrected (Claude Opus $30→$10 blended; Gemini $0.75→$4.50; Kimi $0.48→$1.71; etc.). τ²-bench, GPQA, HLE, OSWorld, SWE-bench, SciCode, GDP, Speed, LCB, AIME corrected throughout. Ranking outcome: Gemini #1 (68.3), Kimi #2 (63.1), KAT #3 (59.3), GPT-5.5 drops to #8 (47.7).
> **v2.4 → v2.5 (cost methodology overhaul):** **Root cause:** API pricing ($/1M tokens) does not represent true cost to users — actual cost = price × token usage, which varies dramatically by model verbosity and task type. **Fix:** Replaced API-price-based cost dimension with **Artificial Analysis Intelligence Index Eval Cost** — the total USD cost to run AA's full standardized benchmark suite on each model. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models (source: artificialanalysis.ai). **Impact:** Cost ranking completely reshuffled. KAT-Coder-Pro-V2 becomes cheapest to run ($73.49 eval cost, rank 1, score 100.0); Claude Opus 4.6 becomes most expensive ($4,969.68, rank 11, score 0.0). GPT-5.5 moves from rank 11 (API price: most expensive) to rank 8 ($3,357 eval cost — cheaper than Sonnet $3,959 and both Opus models), gaining +8.4 pts. GLM 5.1 rises from rank 5→4. Gemini rank 6→5. Kimi drops rank 4→6 ($947.87 eval cost — marginally more expensive than Gemini $892.28 due to higher token usage on benchmark tasks). **GPT-5.5 missing data confirmed still ⊘:** Exhaustive search confirms all 7 missing dimensions (IFBench, SWE-bench Verified, LCB, Speed, LCR, AIME, SciCode) remain unconfirmed as of April 24, 2026 — model launched 24 hours ago; leaderboards not yet updated. **Ranking outcome:** Gemini #1 (71.1), KAT #2 (60.7), GLM #3 (58.9), Kimi #4 (57.5), MiniMax #5 (57.5), GPT-5.5 rises to #6 (56.1). Quality-only unchanged from v2.4: Gemini #1 (74.1), Opus 4.7 #2 (67.8), GPT-5.5 #3 (65.0).
> **v2.5 → v3.0 (new model + 3 new dimensions + data refresh):** **(1) Added DeepSeek V4-Pro** (released April 24, 2026; Apache 2.0; 1.6T/49B MoE; 1M context). **(2) Activated 3 new scored dimensions:** BrowseComp (3% weight — deep web research agent), SWE-bench Pro (4% — harder SWE variant), MMLU-Pro (2% — graduate-level knowledge breadth). Weight redistribution: Cost 28%→25%, IF 20%→18%, Term 9%→8%, SWE 8%→7%, LCB 7%→6%, LCR 3%→2%; all others unchanged; total remains 100%. **(3) 4 data corrections:** Gemini 3.1 Pro τ²-Bench 95.6%→99.3% (now #1); Kimi K2.6 GDPval ELO 1,484→1,520 (swaps with MiniMax for rank 7); GPT-5.5 IFBench ⊘→75.9% (fills 20%-weight gap); GPT-5.5 SWE-bench Verified ⊘→88.7%† (probable, from OpenAI tech card). **(4) HLE methodology resolved:** canonical Scale Labs leaderboard uses without-tools scores — existing HLE values confirmed correct. **(5) Ranking impact:** GPT-5.5 surges from #6→#2 (64.0 pts, +7.9 pts) as IFBench and SWE-bench gaps fill. KAT drops #2→#3. DeepSeek V4-Pro debuts at #8 (51.1 pts). Quality-only: Gemini #1 (73.1), Opus 4.7 #2 (64.4), GPT-5.4 #3 (62.9).
> **v4.1 → v4.2 (external research data integration, April 26 2026):** Two data fills from multi-AI aggregated research report (6 sources: Gemini, Claude, ChatGPT, Copilot, Perplexity, DeepSeek). **(1) GPT-5.5 LCB v4 filled at ~91.7%~** (secondary source, approximate; LCB n expands 5→6): GPT-5.5 final score 65.2→**67.0** (+1.8 pts); quality rank #2→**#1** (72.7, narrowly surpassing Gemini's 72.1); Kimi K2.6 rank **#5→#6** (53.9, drops below MiniMax 54.0 as LCB rank compresses); Gemini 3.1 Pro 70.6→**70.0** (LCB rank 3/5→4/6, −0.6 pts); Opus 4.6 20.9→**20.6** (−0.3 pts). All other models unchanged. **(2) Kimi K2.6 AA Intelligence Index filled at 54~** (secondary source; non-scored reference column §5.3 only — no score impact). **(3) Three data conflicts flagged in §14.11:** external report's Kimi τ²-Bench (96%) vs doc's 72.4%; Kimi LCB (89.6%) vs doc's 82.6%; DeepSeek V4-Pro hallucination rate (54%) vs doc's 94% (from AA article). All three treated as unresolved conflicts pending primary-source verification.
> **v4.0 → v4.1 (methodology update, April 26 2026):** No data changes. **(1) IFBench dimension renamed to "Reliability — Instruction Following (IFBench)"** — rationale expanded to reflect production deployment reality: machine consumers, cascade failure costs, FireBench evidence (best frontier model 74.0% on enterprise IF tasks), and BenchLM composite IF plan (IFEval 65% + IFBench 35% when full-cohort data available). **(2) Pareto frontier analysis added (§12.6)** — using quality-only score × AA Eval Cost × Speed (tok/s): only Gemini 3.1 Pro and KAT-Coder-Pro-V2 are Pareto-optimal; 10/12 models are dominated on all three axes. **(3) AA Eval Cost rationale strengthened (§4.1)** — added explicit "why not per-token pricing?" argument citing verbosity divergence and Price of Progress literature. **(4) HELM framework discussion added (§16I)** — ValueRank's deliberate departure from HELM's multi-metric non-aggregation philosophy, with explicit mapping of HELM's 7 metrics to ValueRank dimensions. **(5) BenchLM-like categorical organization added (§3.5)** — 18 dimensions grouped into 5 clusters: Cost & Efficiency (30%), Reliability (18%), Coding & SE Agents (25%), Agentic Tasks (16%), Knowledge & Reasoning (11%). **(6) AA Intelligence Index cross-reference added (§5.3)** — known scores for 5 models; confirms quality-only composites do not differentiate production value (57–60 band vs. 20.9–70.6 ValueRank spread). **(7) Cost-of-Pass analysis added (§12.7)** — expected cost per correct output on SWE-bench Verified and Terminal-Bench 2.0; KAT's cost advantage is 12–42× cheaper per correct answer than frontier peers. **(8) Reliability weight sensitivity added (§16B)** — rankings at IF=10% and IF=25% alongside existing cost weight scenarios. **(9) SEAL statistical confidence note added (§16B)** — adjacent final scores within 2–3 pts should be treated as effectively tied. **(10) FireBench sub-dimension coverage analysis added (§14.10)** — IFEval+IFBench composite covers 5/6 FireBench categories; overconfidence calibration is the structural gap, addressed separately by AA-Omniscience. All 12 final scores, 216 normalized cells, and 216 weighted derivation cells unchanged.
> **v3.0 → v4.0 (UltraDeep gap-fill research, April 25 2026):** 16 data points filled or corrected via comprehensive leaderboard research. **(1) DeepSeek V4-Pro gaps filled:** SWEPro 55.4%, GDPval ELO 1,554, AA-Omniscience −10 (84%−94%), AA Eval Cost $1,071.28, Speed corrected 33.5→35.8 tok/s, TB 2.0 67.9%†self (self-reported). **(2) GPT-5.5 gaps filled:** Speed 74.7 tok/s; SWE-bench Verified 88.7% dagger removed (now confirmed). **(3) SWE-bench Pro expands 5→9/12:** GLM 58.4%~, Qwen 56.6%~, MiniMax 56.2%~, DeepSeek 55.4% added. **(4) BrowseComp expands 6→8/12:** Opus 4.6 83.7%~, GLM 68%~ added. **(5) MMLU-Pro expands 4→6/12:** Opus 4.7 89.87%~ (new #1), GLM 85.8%~ added. **(6) AA-Omniscience expands 6→8/12:** Kimi +6, DeepSeek −10 added. **(7) Full renormalization:** Cost (n=11→12), Term (n=11→12), GDP (n=11→12), Spd (n=11→12), SWEPro (n=5→9), BC (n=6→8), MMLU (n=4→6), Omni (n=6→8). **(8) Ranking impact:** Gemini #1 (69.5→70.6); GPT-5.5 #2 (64.0→65.2); KAT #3 (58.3→58.4); GLM falls #4 (58.2→56.7, BrowseComp 0.0+MMLU 20.0 hurt); Kimi rises #6→#5 (52.6→54.8, SWEPro+Cost+BC improve); MiniMax falls #5→#6 (55.0→54.0); Qwen rises #9→#8 (49.7→49.0); DeepSeek falls #8→#9 (51.1→48.2, real scores below neutral-50 on Cost/Speed/SWEPro/Omni). Quality-only: Gemini #1 (73.1→72.7), GPT-5.5 rises #4→#2 (62.4→70.9), Opus 4.7 #2→#3 (64.4→68.7). ~ = BenchLM or secondary source (probable); †self = self-reported model card only.

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
10. [Ranking Changes (v1.0 → v2.0 → v2.5 → v3.0 → v4.0)](#10-ranking-changes-v10--v20--v25--v30--v40)
11. [Model Profiles](#11-model-profiles)
12. [Strategic Insights](#12-strategic-insights)
13. [Use Case Recommendations](#13-use-case-recommendations)
14. [Limitations & Known Gaps](#14-limitations--known-gaps)
15. [Benchmark Reference Index](#15-benchmark-reference-index)
16. [Methodology Appendix](#16-methodology-appendix)

---

## 1. Executive Summary

This report ranks 12 frontier AI models across 18 dimensions — 15 quality benchmarks plus cost efficiency, inference speed (tok/s), and task-level reliability — using rank-based normalization with a 25% cost weight. The framework is designed to reflect the total value equation for production software engineering teams: raw capability matters, but so does price, throughput, and reliability at scale.

**v4.0 Headline Finding (UltraDeep gap-fill — 16 data points):** A comprehensive research mission filled 16 ⊘ cells and corrected 1 value across 8 dimensions for newly released models. **DeepSeek V4-Pro's real scores** (Cost $1,071.28, Speed 35.8 tok/s, SWEPro 55.4%, Omni −10) are mostly below the neutral-50 assumptions it held in v3.0 — the model drops from #8 to #9. **GPT-5.5 Speed confirmed** at 74.7 tok/s (xhigh), and SWE-bench Verified 88.7% dagger removed (now multi-source confirmed). **SWE-bench Pro** expands from 5/12 to 9/12 coverage. **BrowseComp** reaches 8/12; **MMLU-Pro** reaches 6/12 with Claude Opus 4.7 taking the new #1 (89.87%). **Key ranking shifts:** Kimi K2.6 rises #6→#5 (SWEPro + cost improve); MiniMax falls #5→#6; GLM falls #4→#4 (score drops 58.2→56.7 from BrowseComp 0.0 + MMLU-Pro 20.0); Qwen rises #9→#8; DeepSeek falls #8→#9. Quality-only: GPT-5.5 surges to #2 (70.9, was #4) as Speed and SWEPro fill in.

**Top 6 (v4.2):**
1. 🥇 **Gemini 3.1 Pro** — 70.0 pts (#1 overall; broadest coverage; #1 IFBench, Speed, τ², GPQA, Sci, Omni)
2. 🥈 **GPT-5.5** — 67.0~ pts (#2 overall; LCB provisional 91.7%~; score +1.8 pts; provisional quality #1 72.7~)
3. 🥉 **KAT-Coder-Pro-V2** — 58.4 pts (sole cheapest to run, $73.49; #2 Terminal-Bench + Speed)
4. **GLM 5.1** — 56.7 pts (eval cost rank 4/12; #2 IFBench + #3 τ²-Bench; BrowseComp 0.0 hurts)
5. **MiniMax M2.7** — 54.0 pts (↑ from #6; near-budget $175.51; passes Kimi by 0.1 pts — SEAL CI tied)
6. **Kimi K2.6** — 53.9 pts (↓ from #5; LCB rank compression −0.9 pts; SEAL CI tied with MiniMax)

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
| 2 | Reliability — Instruction Following (IFBench) | IF | **18%** | Production systems have machine consumers: parsers that cannot tolerate format failures the way a human reader can. A model that follows 90% of constraints is not "90% as useful" — the 10% failure rate requires catch-and-retry logic, human review, or output degradation, multiplying real operational cost non-linearly. IFBench (58 out-of-domain verifiable constraints, Allen AI, NeurIPS 2025) tests generalization beyond IFEval's fixed 26-constraint set. IFBench is unsaturated at 58–76% for current frontier models, making it a live discriminator where MMLU and HellaSwag have lost power. FireBench (arXiv:2603.04857, 2026) confirms the problem is real: no frontier model exceeds 75% on enterprise IF tasks, and top models show 13–25 pp variance across constraint categories — format compliance may rank 1st while item-ranking tasks rank 5th or 6th. Future versions will incorporate IFEval as a composite (IFEval 65% + IFBench 35%, per BenchLM methodology) when full-cohort IFEval data is available. Reduced 20%→18% in v3.0. |
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

### 3.5 Dimension Categories (BenchLM-Inspired)

Analogous to BenchLM's categorical organization (Agentic / Coding / Reasoning / Knowledge / IF / Multilingual), the 18 dimensions group into five production-relevant clusters. Category weights reflect the sum of included dimension weights.

| Category | Dimensions | Combined Weight | Rationale |
|---|---|---|---|
| **Cost & Efficiency** | Cost, Speed | **30%** | Total financial cost and output throughput — the "price-per-useful-token" axes. Largest combined category by design: production model selection begins with operational economics. |
| **Reliability** | IFBench (Reliability — IF) | **18%** | Machine-consumer interface quality: parsers, APIs, and downstream pipelines that cannot tolerate format failures. Deliberate standalone category — instruction-following is a systems-integration property, not a reasoning property. |
| **Coding & SE Agents** | Terminal-Bench 2.0, SWE-bench Verified, SWE-bench Pro, LiveCodeBench v4 | **25%** | Software engineering task completion across four sub-types: terminal/CLI agents, real-world GitHub issue resolution, hard single-pass code repair, and contamination-resistant coding evaluation. |
| **Agentic Tasks** | GDPval-AA ELO, τ²-Bench, OSWorld, BrowseComp | **16%** | Multi-step autonomous operation beyond pure coding: human preference alignment, tool-use reliability, GUI/desktop automation, and deep web research. |
| **Knowledge & Reasoning** | HLE, GPQA Diamond, MMLU-Pro, AIME 2025, SciCode, RULER/LCR, AA-Omniscience | **11%** | Expert knowledge breadth, scientific reasoning, mathematical ceilings, domain coding, long-context fidelity, and hallucination-calibrated accuracy. Intentionally the lowest-weight category: these dimensions predict performance on novel hard tasks but have the least production-deployment relevance per the 25% cost anchor. |

**Category notes:**
- **ValueRank vs. BenchLM:** BenchLM includes Multilingual and Multimodal as first-class categories; ValueRank omits both because the 12-model cohort lacks systematic multilingual and multimodal benchmark coverage. These will be activated as category dimensions when ≥4 cohort models have published scores on a canonical multilingual benchmark.
- **Reliability as standalone:** BenchLM places IF within a broader evaluation composite at lower relative weight (~6%). ValueRank elevates it to a standalone 18% category — the largest non-cost single dimension — reflecting the production systems argument in §3.4 that machine consumers transform IF failures from inconveniences into operational costs.
- **Cost & Efficiency at 30%:** No other public ranking framework treats cost as a 30% cluster. DigitalApplied's Q2 2026 efficient frontier analysis and AA's Pareto leaderboard both incorporate cost, but neither weights it above quality in a composite.

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
| **DeepSeek V4-Pro** | **$1,071.28** | **14.6×** |
| GPT-5.4 | **$2,851.01** | 38.8× |
| GPT-5.5 | **$3,357.00** | 45.7× |
| Claude Sonnet 4.6 | **$3,959.36** | 53.9× |
| Claude Opus 4.7 | **$4,811.04** | 65.5× |
| Claude Opus 4.6 | **$4,969.68** | 67.6× |

> **What is AA Index Eval Cost?** Artificial Analysis runs each model through its full standardized Intelligence Index benchmark suite and records the actual total USD spent — API price × real token consumption across all tasks. This is the only publicly available, standardized, usage-weighted cost dataset for frontier models. Unlike raw $/1M token pricing, eval cost captures true user cost: a verbose model that uses 10× more tokens costs 10× more even at the same token price. Source: artificialanalysis.ai Intelligence Index.
> **Why not per-token API pricing?** Per-token pricing ($/1M input + $/1M output) ignores verbosity: a model priced at $3/1M tokens that generates 4× as many tokens per task has an effective task cost of $12/1M — more expensive than a $10/1M model that answers correctly in one concise pass. Reasoning models amplify this problem: the "Price of Progress" literature (arXiv:2511.23455) documents that frontier-level *total* costs have risen 3–18× per year even as per-token prices fall, because reasoning-mode token counts dominate spend. AA Eval Cost captures the full price-times-tokens picture by measuring what it actually costs to run a standardized benchmark suite end-to-end. This is why ValueRank switched from raw API pricing to AA Eval Cost in v2.5 — it produced dramatically different rankings (e.g., GPT-5.5 moved from rank 11 by per-token list price to rank 9 by eval cost, because its reasoning throughput per task is more efficient than Anthropic's Opus models despite similar headline prices).
> **DeepSeek V4-Pro v4.0 update:** AA Eval Cost confirmed at $1,071.28 (source: artificialanalysis.ai/models/deepseek-v4-pro, corroborated by AA article on DeepSeek V4 Pro). Slots between Kimi ($947.87) and GPT-5.4 ($2,851.01) at rank 7/12 — cost score 45.5 (below neutral-50, a negative finding vs. the prior ⊘ neutral assumption).

### 4.2 Cost Efficiency Tier Summary

| Tier | Models | AA Eval Cost Range |
|---|---|---|
| Budget | KAT-Coder-Pro-V2 | $73.49 |
| Near-Budget | MiniMax M2.7 | $175.51 |
| Mid-Range | Qwen 3.6 Plus, GLM 5.1 | $483–$544 |
| Premium | Gemini 3.1 Pro, Kimi K2.6, **DeepSeek V4-Pro** | $892–$1,071 |
| Expensive | GPT-5.4 | $2,851 |
| Very Expensive | GPT-5.5, Claude Sonnet 4.6 | $3,357–$3,959 |
| Ultra-Premium | Claude Opus 4.7, Claude Opus 4.6 | $4,811–$4,970 |

---

## 5. Raw Benchmark Data

### 5.1 Quality Benchmarks

| Model | IFBench | Term-Bench 2.0 | SWE-Bench V | SWE-Pro | LCB v4 | GDPval ELO | τ²-Bench | OSWorld | RULER (LCR)‡ | HLE | GPQA◇ | AIME 2025 | SciCode |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | 89.4% | 68.5% | 80.6% | ⊘ | 76.3% | 1,314 | **99.3%** | ⊘ | 94.2%‡ | 44.7% | 94.3% | 95.0% | 58.9% |
| Kimi K2.6 | 82.1% | 66.7% | 80.2% | 58.6% | 82.6% | **1,520** | 72.4% | 73.1% | 87.5%‡ | 34.7% | 90.5% | 96.1% | 52.2% |
| MiniMax M2.7 | 75.7% | 57.0% | ⊘ | **56.2%~** | ⊘ | 1,514 | 84.8% | ⊘ | 81.3%‡ | 28.1% | 87.4% | ⊘ | 47.0% |
| GPT-5.4 | 73.9% | 75.1% | ⊘ | 59.1% | 63.8% | 1,674 | 87.1% | 75.0% | 97.8%‡ | 41.6% | 92.0% | ⊘ | 56.6% |
| Qwen 3.6 Plus | 74.2% | 61.6% | 78.8% | **56.6%~** | ⊘ | 1,361 | 78.2% | ⊘ | 87.5%‡ | 28.8% | 90.4% | ⊘ | 21.4% |
| KAT-Coder-Pro-V2 | 67.0% | 76.2%† | 79.6% | ⊘ | ⊘ | 1,124 | 93.9% | ⊘ | 74.6%‡ | 12.7% | 85.5% | ⊘ | 38.3% |
| GLM 5.1 | 85.9% | 69.0% | 77.8% | **58.4%~** | ⊘ | 1,535 | 97.7% | ⊘ | 68.3%‡ | 31.0% | 86.2% | 95.0% | 43.8% |
| Claude Opus 4.7 | 52.1% | 69.4% | 87.6% | 64.3% | ⊘ | 1,753 | 74.0% | 78.0% | 92.4%‡ | 46.9% | 94.2% | 99.8% | 54.5% |
| GPT-5.5 | **75.9%** | **82.7%** | **88.7%** | 58.6% | **91.7%~** | 1,784 | 98.0% | 78.7% | ⊘ | 44.3% | 93.6% | ⊘ | ⊘ |
| Claude Sonnet 4.6 | 45.6% | 59.1% | 79.6% | ⊘ | ⊘ | 1,675 | 79.5% | 72.5% | 61.2%‡ | 30.0% | 87.5% | 57.1% | 46.9% |
| Claude Opus 4.6 | 40.2% | 65.4% | 80.8% | 51.9% | 76.0% | 1,619 | 84.8% | 72.7% | 54.7%‡ | 40.0% | 84.0% | 99.8% | 51.9% |
| DeepSeek V4-Pro | ⊘ | **67.9%†self** | 80.6% | **55.4%** | **93.5%** | **1,554** | ⊘ | ⊘ | ⊘ | 37.7% | 88.8% | ⊘ | ⊘ |

> **† Footnotes:** KAT-Coder-Pro-V2 TB 2.0 (76.2%†) is not on the public tbench.ai leaderboard; value from KAT technical report referencing "Terminal-Bench Hard" — potentially different variant. DeepSeek V4-Pro TB 2.0 (67.9%†self) is from DeepSeek's own model card on HuggingFace — confirmed absent from canonical tbench.ai leaderboard as of April 25, 2026. Both marked †/†self. GPT-5.5 SWE-bench Verified (88.7%) dagger removed in v4.0 — confirmed by multiple independent sources (OpenAI tech card + TokenMix, Fast Company, Investing.com, RD World Online).
> **‡ RULER (LCR) footnote:** No model in this cohort was found on any public RULER leaderboard as of 2026-04-25. These values are drawn from model cards and internal evaluation documents; none could be independently confirmed against the canonical ruler-bench.github.io or llm-stats leaderboards. RULER scores should be treated as provisional.
> **~ footnote:** SWE-Pro scores for GLM (58.4%), Qwen (56.6%), MiniMax (56.2%) are from BenchLM.ai leaderboard (April 24, 2026) — probable confidence, not Scale Labs SEAL canonical. BenchLM showed GPT-5.4 at 57.7% vs. our Scale AI value of 59.1% (±1–2 pp scaffold difference). DeepSeek SWE-Pro 55.4% confirmed on both HuggingFace model card and BenchLM. GDPval ELO 1,554 confirmed from HuggingFace model card + Artificial Analysis article.
> **Bold** = v4.0 data additions or corrections.

### 5.2 New Scored Dimensions (v3.0)

| Model | BrowseComp | MMLU-Pro |
|---|---|---|
| Gemini 3.1 Pro | 85.9% | **89.8%** |
| Kimi K2.6 | 83.2% | 84.6% |
| MiniMax M2.7 | ⊘ | ⊘ |
| GPT-5.4 | 89.3% | ⊘ |
| Qwen 3.6 Plus | ⊘ | 88.5% |
| KAT-Coder-Pro-V2 | ⊘ | ⊘ |
| GLM 5.1 | **68%~** | **85.8%~** |
| Claude Opus 4.7 | 79.3% | **89.87%~** |
| GPT-5.5 | **90.1%** | ⊘* |
| Claude Sonnet 4.6 | ⊘ | ⊘ |
| Claude Opus 4.6 | **83.7%~** | ⊘ |
| DeepSeek V4-Pro | 83.4% | 87.5% |

> *GPT-5.5 MMLU-Pro = 54.6% seen in one source — anomalously low vs. frontier cluster (84–92%), likely 0-shot vs. few-shot methodology difference. Treated as ⊘ (unreliable) for scoring.
> **~ footnote:** GLM BrowseComp (68%), GLM MMLU-Pro (85.8%), Claude Opus 4.6 BrowseComp (83.7%) sourced from BenchLM.ai — probable confidence. Claude Opus 4.7 MMLU-Pro (89.87%) sourced from aggregator leaderboard (pricepertoken.com/mmlu-pro) — probable confidence, new #1 in cohort above Gemini 89.8%.

### 5.3 Composite / Special Dimensions

| Model | AA-Omniscience (Acc% − Hall%) | Speed (tok/s) | AA Index Eval Cost ($) | AA Intelligence Index |
|---|---|---|---|---|
| Gemini 3.1 Pro | +34 (82% acc − 48% hall) | 128.0 | $892.28 | 57 |
| Kimi K2.6 | **+6 (45% acc − 39% hall)** | 100.4 | $947.87 | **54~** |
| MiniMax M2.7 | +12 (71% acc − 59% hall) | 49.0 | $175.51 | ⊘ |
| GPT-5.4 | ⊘ | 74.8 | $2,851.01 | 57 |
| Qwen 3.6 Plus | ⊘ | 53.0 | $482.65 | ⊘ |
| KAT-Coder-Pro-V2 | ⊘ | 113.5 | $73.49 | ⊘ |
| GLM 5.1 | −1 (64% acc − 65% hall) | 49.0 | $543.95 | ⊘ |
| Claude Opus 4.7 | +22 (76% acc − 54% hall) | 42.0 | $4,811.04 | 57 |
| GPT-5.5 | **−29** (57% acc − 86% hall) | **74.7** | $3,357.00 | **60†** |
| Claude Sonnet 4.6 | ⊘ | 46.0 | $3,959.36 | ⊘ |
| Claude Opus 4.6 | +8 (68% acc − 60% hall) | 18.2 | $4,969.68 | ⊘ |
| DeepSeek V4-Pro | **−10 (84% acc − 94% hall)** | **35.8** | **$1,071.28** | ⊘ |

> **AA-Omniscience note (GPT-5.5):** Despite being rated #1 on the AA Intelligence Index v4.0 (score 60 xhigh), GPT-5.5's AA-Omniscience score is −29, the worst of all models with real data. This reflects an 86% hallucination rate at high-confidence assertions — a known characteristic of the xhigh reasoning mode that prioritizes bold inference over calibrated uncertainty.
> **AA-Omniscience note (DeepSeek V4-Pro):** 94% hallucination rate — when the model doesn't know an answer it responds anyway in 94% of cases. An 11-point improvement over DeepSeek V3.2 (−21). Source: Artificial Analysis article "DeepSeek is back among the leading open weights models." Kimi K2.6 +6 score (39% hall rate) confirms by Artificial Analysis Omniscience data.
> **Speed note (GPT-5.5):** 74.7 tok/s is the xhigh reasoning mode throughput (primary scoring configuration). GPT-5.5 (high) = 81.9 tok/s. Source: artificialanalysis.ai/models/gpt-5-5. DeepSeek speed corrected from 33.5 to 35.8 tok/s (same source, updated measurement).
> **AA Intelligence Index note:** Artificial Analysis's composite benchmark score (equal-weighted across Coding, Reasoning, Language, Math at 25% each; 10 benchmarks total; cost explicitly excluded). † GPT-5.5 score 60 is for xhigh reasoning mode; high mode = 59 (source: artificialanalysis.ai/models/gpt-5-5). ~ Kimi K2.6 score 54 is from a multi-AI aggregated research report (April 26, 2026; 6 sources); not yet confirmed on the canonical AA leaderboard. The 5 known scores (4 AA-confirmed + Kimi~) span a compressed 54–60 band — masking the wide ValueRank spread of 20.6–70.0. This compression confirms that quality-only composites do not adequately differentiate production value when cost is ignored. ⊘ = score not confirmed from AA leaderboard for this cohort model as of April 26, 2026.

---

## 6. Speed & Latency Reference Data

### 6.1 Inference Speed (tok/s) — Scored Dimension

| Model | Output tok/s | Notes |
|---|---|---|
| Gemini 3.1 Pro | 128.0 | Fastest in field; TPU infrastructure advantage |
| KAT-Coder-Pro-V2 | 113.5 | Second fastest; specialized decode path |
| Kimi K2.6 | 100.4 | Third fastest |
| GPT-5.4 | 74.8 | Moderate throughput; confirmed by AA leaderboard |
| **GPT-5.5** | **74.7** | **xhigh reasoning mode; near-tied with GPT-5.4; high for a reasoning model** |
| Qwen 3.6 Plus | 53.0 | Mid-range |
| MiniMax M2.7 | 49.0 | Budget tier; tied with GLM |
| GLM 5.1 | 49.0 | Tied with MiniMax |
| Claude Sonnet 4.6 | 46.0 | Moderate |
| Claude Opus 4.7 | 42.0 | Constrained by 200K context |
| **DeepSeek V4-Pro** | **35.8** | **Below open-weight median; 1.6T MoE routing overhead (corrected from 33.5)** |
| Claude Opus 4.6 | 18.2 | Slowest with real data |

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
| Gemini 3.1 Pro | **63.6** | 100.0 | **54.5** | 61.1 | 50.0⊘ | **40.0** | **9.1** | 100.0 | 100.0 | 50.0⊘ | **71.4** | 88.9 | 90.9 | 100.0 | **80.0** | 30.0 | 100.0 | 100.0 |
| Kimi K2.6 | **54.5** | 80.0 | **36.4** | 44.4 | **68.8** | **60.0** | **36.4** | **81.8** | 0.0 | 40.0 | **28.6** | 61.1 | 45.5 | 63.6 | 0.0 | 60.0 | 66.7 | **42.9** |
| MiniMax M2.7 | **90.9** | 60.0 | 0.0 | 50.0⊘ | **25.0~** | 50.0⊘ | **27.3** | **40.9** | 45.0 | 50.0⊘ | 50.0⊘ | 44.4 | 9.1 | 27.3 | 50.0⊘ | 50.0⊘ | 44.4 | **71.4** |
| GPT-5.4 | **36.4** | 40.0 | **81.8** | 50.0⊘ | **87.5** | 0.0 | **72.7** | **72.7** | 60.0 | 60.0 | **85.7** | 100.0 | 72.7 | 72.7 | 50.0⊘ | 50.0⊘ | 88.9 | 50.0⊘ |
| Qwen 3.6 Plus | **81.8** | 50.0 | **18.2** | 11.1 | **37.5~** | 50.0⊘ | **18.2** | **54.5** | 20.0 | 50.0⊘ | 50.0⊘ | 61.1 | 18.2 | 54.5 | **60.0** | 50.0⊘ | 0.0 | 50.0⊘ |
| KAT-Coder-Pro-V2 | **100.0** | 30.0 | **90.9** | 27.8 | 50.0⊘ | 50.0⊘ | 0.0 | **90.9** | 70.0 | 50.0⊘ | 50.0⊘ | 33.3 | 0.0 | 9.1 | 50.0⊘ | 50.0⊘ | 11.1 | 50.0⊘ |
| GLM 5.1 | **72.7** | 90.0 | **63.6** | 0.0 | **50.0~** | 50.0⊘ | **45.5** | **40.9** | 80.0 | 50.0⊘ | **0.0~** | 22.2 | 36.4 | 18.2 | **20.0~** | 30.0 | 22.2 | **28.6** |
| Claude Opus 4.7 | **9.1** | 20.0 | **72.7** | 88.9 | 100.0 | 50.0⊘ | **90.9** | **18.2** | 10.0 | 80.0 | **14.3** | 77.8 | 100.0 | 90.9 | **100.0~** | 90.0 | 77.8 | **85.7** |
| GPT-5.5 | **27.3** | **70.0** | 100.0 | **100.0** | **68.8** | **80.0~** | 100.0 | **63.6** | 90.0 | 100.0 | 100.0 | 50.0⊘ | 81.8 | 81.8 | 50.0⊘ | 50.0⊘ | 50.0⊘ | 0.0 |
| Claude Sonnet 4.6 | **18.2** | 10.0 | **9.1** | 27.8 | 50.0⊘ | 50.0⊘ | **81.8** | **27.3** | 30.0 | 0.0 | 50.0⊘ | 11.1 | 27.3 | 36.4 | 50.0⊘ | 0.0 | 33.3 | 50.0⊘ |
| Claude Opus 4.6 | 0.0 | 0.0 | **27.3** | 77.8 | 0.0 | **20.0** | **63.6** | 0.0 | 45.0 | 20.0 | **57.1~** | 0.0 | 63.6 | 0.0 | 50.0⊘ | 90.0 | 55.6 | **57.1** |
| DeepSeek V4-Pro | **45.5** | 50.0⊘ | **45.5†self** | 61.1 | **12.5** | 100.0 | **54.5** | **9.1** | 50.0⊘ | 50.0⊘ | **42.9** | 50.0⊘ | 54.5 | 45.5 | **40.0** | 50.0⊘ | 50.0⊘ | **14.3** |

**⊘** = missing data, assigned neutral score of 50.0. **Bold** = changed or filled from v3.0. **~** = BenchLM/secondary source (probable confidence).

**Key v3.0 dimension normalization notes (carried forward):**
- **IF (n=11):** GPT-5.5 75.9% fills gap → rank 4; GPT-5.5: 50⊘→70.0; GLM shifts 88.9→90.0; Kimi 77.8→80.0.
- **SWE (n=10):** GPT-5.5 88.7% (#1, dagger removed v4.0), DeepSeek 80.6% ties Gemini (rank 4.5). KAT/Sonnet tied (rank 7.5) = 27.8.
- **τ² (n=11):** Gemini 99.3% corrected (#1); GPT-5.5 at #2 (90.0); GLM at #3 (80.0).
- **HLE (n=12):** DeepSeek 37.7% at rank 6; all scores from v2.4 base.
- **GPQA (n=12):** DeepSeek 88.8% at rank 7; existing models shifted.
- **LCB (n=5):** DeepSeek 93.5% at rank 1 (#1 in cohort); Kimi at rank 2 (75.0).

**Key v4.0 dimension normalization notes (new):**
- **Cost (n=11→12):** DeepSeek $1,071.28 fills gap → rank 7/12. Formula n=12: ((12−rank)/11)×100. All 11 existing models shift: KAT unchanged (100.0), others compress upward or downward by 0.9–4.5 pts depending on rank. DeepSeek: 50⊘→45.5 (below neutral).
- **Term (n=11→12):** DeepSeek 67.9%†self fills gap → rank 7/12 (between Gemini 68.5% and Kimi 66.7%). Score = 45.5. All other models shift: GPT-5.5 100.0 unchanged; Gemini 50→54.5; MiniMax 0.0 unchanged.
- **SWEPro (n=5→9):** GLM 58.4%~, Qwen 56.6%~, MiniMax 56.2%~, DeepSeek 55.4% added. New n=9. Kimi/GPT-5.5 (58.6%, tied rank 3.5): 37.5→68.8; GPT-5.4 (rank 2): 75.0→87.5; Qwen: 50⊘→37.5; MiniMax: 50⊘→25.0; DeepSeek: 50⊘→12.5; Opus4.6 (rank 9): 0.0 unchanged.
- **GDP (n=11→12):** DeepSeek 1,554 fills gap → rank 6/12. GPT-5.5 100.0 unchanged; all others compress by 0.9–4.5 pts. DeepSeek: 50⊘→54.5 (above neutral — one positive fill for DeepSeek).
- **Spd (n=11→12):** GPT-5.5 74.7 tok/s fills gap → rank 5/12 (near-tied with GPT-5.4 74.8). DeepSeek corrected 33.5→35.8 (rank 11 unchanged). GPT-5.5: 50⊘→63.6; Qwen compresses 60→54.5; MiniMax/GLM tied: 45→40.9.
- **BC (n=6→8):** Opus4.6 83.7%~ (rank 4) and GLM 68%~ (rank 8/last) added. GPT-5.5 100.0 unchanged; Opus4.7: 0.0→14.3; GLM: 50⊘→0.0 (major drop); Kimi: 20.0→28.6; Gemini: 60.0→71.4.
- **MMLU (n=4→6):** Opus4.7 89.87%~ (rank 1) and GLM 85.8%~ (rank 5) added. Gemini drops 100.0→80.0 (displaced to rank 2); Qwen: 66.7→60.0; DeepSeek: 33.3→40.0; Kimi 0.0 unchanged. Opus4.7: 50⊘→100.0 (major positive).
- **Omni (n=6→8):** Kimi +6 (rank 5) and DeepSeek −10 (rank 7) added. Opus4.7: 80.0→85.7; MiniMax: 60.0→71.4; Opus4.6: 40.0→57.1; Kimi: 50⊘→42.9 (below neutral); DeepSeek: 50⊘→14.3 (well below neutral).

**Key v4.2 dimension normalization notes (new):**
- **LCB (n=5→6):** GPT-5.5 ~91.7%~ fills gap → rank 2/6 (between DeepSeek 93.5% #1 and Kimi 82.6% #3). Score = 80.0~. All models with existing LCB real data shift: DeepSeek rank 1 (100.0, unchanged); GPT-5.5: 50.0⊘→80.0~; Kimi: 75.0→60.0 (rank 2→3); Gemini: 50.0→40.0 (rank 3→4); Opus4.6: 25.0→20.0 (rank 4→5); GPT-5.4: 0.0 unchanged (rank 5→6). The 6 models with ⊘ (MiniMax, Qwen, KAT, GLM, Opus4.7, Sonnet) remain at 50.0⊘.

### 7.3 Cost Score Derivation

Cost ranks (cheapest = rank 1, n=12 real scorers). No ties.

| Model | AA Index Eval Cost ($) | Cost Rank | Score = ((12−rank)/11)×100 |
|---|---|---|---|
| KAT-Coder-Pro-V2 | $73.49 | 1 | **100.0** |
| MiniMax M2.7 | $175.51 | 2 | **90.9** |
| Qwen 3.6 Plus | $482.65 | 3 | **81.8** |
| GLM 5.1 | $543.95 | 4 | **72.7** |
| Gemini 3.1 Pro | $892.28 | 5 | **63.6** |
| Kimi K2.6 | $947.87 | 6 | **54.5** |
| DeepSeek V4-Pro | $1,071.28 | 7 | **45.5** |
| GPT-5.4 | $2,851.01 | 8 | **36.4** |
| GPT-5.5 | $3,357.00 | 9 | **27.3** |
| Claude Sonnet 4.6 | $3,959.36 | 10 | **18.2** |
| Claude Opus 4.7 | $4,811.04 | 11 | **9.1** |
| Claude Opus 4.6 | $4,969.68 | 12 | **0.0** |

> **v3.0→v4.0:** DeepSeek V4-Pro confirms $1,071.28 AA Index eval cost (rank 7/12, between Kimi $947.87 and GPT-5.4 $2,851.01). n expands 11→12. All existing model scores update under the new formula ((12−rank)/11)×100.

---

## 8. Quality-Only Ranking (Unweighted)

Equally weighting all 17 non-cost dimensions (5.88% each) to show pure capability ranking without cost penalty.

| Rank | Model | IF Score | Benchmark Quality‡ | Total Quality |
|---|---|---|---|---|
| 1 | **GPT-5.5** | 70.0 | **72.9~** | **72.7~** |
| 2 | Gemini 3.1 Pro | 100.0 | 70.4 | 72.1 |
| 3 | Claude Opus 4.7 | 20.0 | 71.7 | 68.7 |
| 4 | GPT-5.4 | 40.0 | 65.9 | 64.4 |
| 5 | Kimi K2.6 | 80.0 | 46.1 | 48.0 |
| 6 | DeepSeek V4-Pro | 50.0⊘ | 45.6 | 45.9 |
| 7 | KAT-Coder-Pro-V2 | 30.0 | 42.7 | 41.9 |
| 8 | MiniMax M2.7 | 60.0 | 39.7 | 40.9 |
| 9 | Qwen 3.6 Plus | 50.0 | 37.7 | 38.4 |
| 10 | GLM 5.1 | 90.0 | 34.9 | 38.1 |
| 11 | Claude Opus 4.6 | 0.0 | 39.2 | 36.9 |
| 12 | Claude Sonnet 4.6 | 10.0 | 33.4 | 32.0 |

> **‡** IF Score = raw normalized IFBench score (0–100 scale; ⊘ = 50.0). Benchmark Quality = equal-weighted avg of the other 16 non-cost dims. Total Quality = equal-weighted avg of all 17 non-cost dims. ⊘ on DeepSeek IF/Term/τ²/etc. = neutral-50 in both averages. **~** = includes GPT-5.5 LCB 80.0~ (secondary-source score; see §14.11). The 0.6-pt quality gap between GPT-5.5 (72.7) and Gemini (72.1) falls within the SEAL statistical CI band (§16B) — treat as effectively tied for quality #1.

**v4.2 quality update:** **GPT-5.5 provisionally rises to quality #1** (72.7~) as LCB fills at ~91.7% (rank 2/6, score 80.0, secondary source). Gemini drops to quality #2 (72.1) — its LCB rank compresses from 3/5 to 4/6 as GPT-5.5 inserts above it. The gap is 0.6 pts, within SEAL CI; treat as statistically tied. **Claude Opus 4.7 holds quality #3** (68.7) unchanged. **Kimi drops slightly** (48.9→48.0) as its LCB rank compresses from 2/5 to 3/6 with GPT-5.5 insertion. **GPT-5.5 now leads on 8 of 14 confirmed quality dimensions** (TB 2.0, GDP, OSWorld, BrowseComp, SWE-V, SWEPro tied, τ², LCB~). *(v4.0 note: GPT-5.5 surged to quality #2 as Speed 63.6 and SWEPro 68.8 filled; DeepSeek at quality #6 45.9; GLM at quality #10 38.1 from BrowseComp 0.0 double hit — all unchanged in v4.2.)*

---

## 9. Final Cost-Weighted Ranking

**Weights:** Cost 25%, IF 18%, Term 8%, SWE 7%, SWEPro 4%, LCB 6%, GDP 5%, Spd 5%, τ² 4%, OSW 4%, BC 3%, LCR 2%, HLE 2%, GPQA 2%, MMLU 2%, AIME 1%, Sci 1%, Omni 1%

### Final Weighted Scores

| Main Rank | Model | Cost (25%) | IF (18%) | Quality (57%) | Main Score |
|---|---|---|---|---|---|
| 🥇 1 | **Gemini 3.1 Pro** | 15.9 | 18.0 | 36.1 | **70.0** |
| 🥈 2 | **GPT-5.5** | 6.8 | 12.6 | 47.6~ | **67.0~** |
| 🥉 3 | **KAT-Coder-Pro-V2** | 25.0 | 5.4 | 28.0 | **58.4** |
| 4 | **GLM 5.1** | 18.2 | 16.2 | 22.3 | **56.7** |
| 5 | **MiniMax M2.7** | 22.7 | 10.8 | 20.5 | **54.0** |
| 6 | **Kimi K2.6** | 13.6 | 14.4 | 25.9 | **53.9** |
| 7 | **GPT-5.4** | 9.1 | 7.2 | 36.0 | **52.3** |
| 8 | **Qwen 3.6 Plus** | 20.5 | 9.0 | 19.6 | **49.0** |
| 9 | **DeepSeek V4-Pro** | 11.4 | 9.0 | 27.8 | **48.2** |
| 10 | **Claude Opus 4.7** | 2.3 | 3.6 | 38.4 | **44.3** |
| 11 | **Claude Sonnet 4.6** | 4.6 | 1.8 | 19.2 | **25.5** |
| 12 | **Claude Opus 4.6** | 0.0 | 0.0 | 20.6 | **20.6** |

> **Main:** All 12 models, 18 dimensions. Columns: Cost = norm_cost × 0.25 (max 25.0 pts); IF = norm_IF × 0.18 (max 18.0 pts); Quality = weighted sum of remaining 16 dims × their weights (summing to 57%); Main Score = Cost + IF + Quality. `Final = Σ(normalized_score × weight)`.
> **GPT-5.5 note:** As of v4.2, 3 of 17 quality dimensions remain ⊘ (neutral-50): LCR, AIME, SciCode. LCB provisionally filled at ~91.7%~ (rank 2/6, score 80.0~; secondary source — see §14.11). Speed (63.6) and SWEPro (68.8) confirmed; GPT-5.5 leads or ties on 8 of 14 confirmed quality dimensions; provisional quality #1 (72.7~, pending LCB primary-source verification).
> **DeepSeek V4-Pro note:** As of v4.0, 7 of 18 dimensions remain ⊘: IFBench, τ², OSWorld, RULER, AIME, SciCode, and LCB. Four gap-fills confirmed (Cost 45.5, Term 45.5†self, SWEPro 12.5, Omni 14.3) — all below neutral-50, confirming the downward revision from provisional #8 to #9.

### Score Derivation Detail

| Dim | Wt | Gemini | GPT-5.5 | KAT | GLM | Kimi | MiniMax | GPT-5.4 | Qwen | DeepSeek | Opus4.7 | Sonnet | Opus4.6 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Cost | 25% | 15.90 | 6.83 | 25.00 | 18.18 | 13.63 | 22.73 | 9.10 | 20.45 | 11.38 | 2.28 | 4.55 | 0.00 |
| IF | 18% | 18.00 | 12.60 | 5.40 | 16.20 | 14.40 | 10.80 | 7.20 | 9.00 | 9.00 | 3.60 | 1.80 | 0.00 |
| Term | 8% | 4.36 | 8.00 | 7.27 | 5.09 | 2.91 | 0.00 | 6.54 | 1.46 | 3.64 | 5.82 | 0.73 | 2.18 |
| SWE | 7% | 4.28 | 7.00 | 1.95 | 0.00 | 3.11 | 3.50 | 3.50 | 0.78 | 4.28 | 6.22 | 1.95 | 5.45 |
| SWEPro | 4% | 2.00 | 2.75 | 2.00 | 2.00 | 2.75 | 1.00 | 3.50 | 1.50 | 0.50 | 4.00 | 2.00 | 0.00 |
| LCB | 6% | 2.40 | 4.80~ | 3.00 | 3.00 | 3.60 | 3.00 | 0.00 | 3.00 | 6.00 | 3.00 | 3.00 | 1.20 |
| GDP | 5% | 0.46 | 5.00 | 0.00 | 2.28 | 1.82 | 1.37 | 3.64 | 0.91 | 2.73 | 4.55 | 4.09 | 3.18 |
| Spd | 5% | 5.00 | 3.18 | 4.55 | 2.05 | 4.09 | 2.05 | 3.64 | 2.73 | 0.46 | 0.91 | 1.37 | 0.00 |
| τ² | 4% | 4.00 | 3.60 | 2.80 | 3.20 | 0.00 | 1.80 | 2.40 | 0.80 | 2.00 | 0.40 | 1.20 | 1.80 |
| OSW | 4% | 2.00 | 4.00 | 2.00 | 2.00 | 1.60 | 2.00 | 2.40 | 2.00 | 2.00 | 3.20 | 0.00 | 0.80 |
| BC | 3% | 2.14 | 3.00 | 1.50 | 0.00 | 0.86 | 1.50 | 2.57 | 1.50 | 1.29 | 0.43 | 1.50 | 1.71 |
| LCR | 2% | 1.78 | 1.00 | 0.67 | 0.44 | 1.22 | 0.89 | 2.00 | 1.22 | 1.00 | 1.56 | 0.22 | 0.00 |
| HLE | 2% | 1.82 | 1.64 | 0.00 | 0.73 | 0.91 | 0.18 | 1.45 | 0.36 | 1.09 | 2.00 | 0.55 | 1.27 |
| GPQA | 2% | 2.00 | 1.64 | 0.18 | 0.36 | 1.27 | 0.55 | 1.45 | 1.09 | 0.91 | 1.82 | 0.73 | 0.00 |
| MMLU | 2% | 1.60 | 1.00 | 1.00 | 0.40 | 0.00 | 1.00 | 1.00 | 1.20 | 0.80 | 2.00 | 1.00 | 1.00 |
| AIME | 1% | 0.30 | 0.50 | 0.50 | 0.30 | 0.60 | 0.50 | 0.50 | 0.50 | 0.50 | 0.90 | 0.00 | 0.90 |
| Sci | 1% | 1.00 | 0.50 | 0.11 | 0.22 | 0.67 | 0.44 | 0.89 | 0.00 | 0.50 | 0.78 | 0.33 | 0.56 |
| Omni | 1% | 1.00 | 0.00 | 0.50 | 0.29 | 0.43 | 0.71 | 0.50 | 0.50 | 0.14 | 0.86 | 0.50 | 0.57 |
| **Total** | | **70.0** | **67.0~** | **58.4** | **56.7** | **53.9** | **54.0** | **52.3** | **49.0** | **48.2** | **44.3** | **25.5** | **20.6** |

> **v4.0 main shakeup:** Kimi rises #6→#5 (+2.2 pts) as SWEPro (68.8) and GDP fills outweigh term/cost compression. MiniMax falls #5→#6 as SWEPro 25.0 (below neutral) and GDP compression cost 1.0 pts. DeepSeek falls #8→#9 (−2.9 pts): Cost 45.5, Term 45.5, SWEPro 12.5, Omni 14.3 — all four confirmed fills below neutral-50. Qwen rises #9→#8 (+0 pts net but surpasses DeepSeek as DeepSeek falls). GPT-5.5 continues to consolidate quality #2 as Speed (63.6) and SWEPro (68.8) confirm above-neutral scores.
> **v4.2 main shakeup:** **GPT-5.5 score improves 65.2→67.0~ (+1.8 pts)** as LCB fills at ~91.7%~ (secondary source, rank 2/6). **Kimi falls #5→#6** (54.8→53.9, −0.9 pts) — LCB rank compresses from 2 to 3 as GPT-5.5 inserts above it. **MiniMax rises #6→#5** by 0.1-pt margin (54.0 vs Kimi 53.9) — within the SEAL CI "effectively tied" band. **Gemini dips 70.6→70.0** (LCB rank 3→4, −0.6 pts); holds #1 overall. **Opus 4.6: 20.9→20.6** (−0.3 pts). All v4.2 score changes carry ~ and are pending primary-source LCB verification.

---

## 10. Ranking Changes (v1.0 → v2.0 → v2.5 → v3.0 → v4.0)

| Model | v1.0 | v2.0 | v2.1.1 | v2.3 | v2.4 | v2.5 | v3.0 | **v4.0** | v4.0 Score | Net Trend |
|---|---|---|---|---|---|---|---|---|---|---|
| Gemini 3.1 Pro | #1 | #1 | #1 | #2 | #1 | #1 | #1 | **#1** | 70.6 | ↑ Holds #1; all 8 renorm dims favor Gemini net +1.1 pts |
| GPT-5.5 | — | — | #6 | #4 | #8 | #6 | #2 | **#2** | 65.2 | ↑ Speed (63.6) + SWEPro (68.8) confirm quality #2; +1.2 pts |
| KAT-Coder-Pro-V2 | #6 | #4 | #5 | #5 | #3 | #2 | #3 | **#3** | 58.4 | ─ Stable; +0.1 pts as n=12 slightly lifts rank-2 scores |
| GLM 5.1 | #7 | #7 | #7 | #7 | #5 | #3 | #4 | **#4** | 56.7 | ↓ BrowseComp 0.0 (last/8) + MMLU-Pro 20.0 double hit; −1.5 pts |
| Kimi K2.6 | #2 | #2 | #2 | #1 | #2 | #4 | #6 | **#5** | 54.8 | ↑ SWEPro 68.8 + Cost/Term n-expansion; +2.2 pts |
| MiniMax M2.7 | #3 | #3 | #3 | #3 | #4 | #5 | #5 | **#6** | 54.0 | ↓ SWEPro 25.0 (below neutral) + GDP drop; −1.0 pts |
| GPT-5.4 | #4 | #5 | #4 | #6 | #7 | #8 | #7 | **#7** | 52.3 | ↑ SWEPro rank improves (75→87.5 with n=9); +0.2 pts |
| Qwen 3.6 Plus | #5 | #6 | #8 | #8 | #6 | #7 | #9 | **#8** | 49.0 | ↑ Rises as DeepSeek falls past it; own score −0.7 pts |
| DeepSeek V4-Pro | — | — | — | — | — | — | #8 | **#9** | 48.2 | ↓↓ All 4 gap-fills below neutral: Cost 45.5, Term 45.5, SWEPro 12.5, Omni 14.3; −2.9 pts |
| Claude Opus 4.7 | #8 | #8 | #9 | #9 | #9 | #9 | #10 | **#10** | 44.3 | ↑ MMLU-Pro #1 (100.0~) + BC/Omni fills; +1.4 pts |
| Claude Sonnet 4.6 | #9 | #10 | #10 | #10 | #10 | #10 | #11 | **#11** | 25.5 | ─ Minor compression; −0.6 pts |
| Claude Opus 4.6 | #10 | #9 | #11 | #11 | #11 | #11 | #12 | **#12** | 20.9 | ↑ BC 57.1%~ + Omni 57.1%~ fill in above neutral; +0.3 pts |

**v4.2 rank changes (LCB provisional fill, secondary source):** Gemini 70.6→70.0 (−0.6); GPT-5.5 65.2→**67.0~** (+1.8, score only — rank holds #2); Kimi 54.8→53.9, **#5→#6** (−0.9); MiniMax **#6→#5** (54.0, unchanged score); Opus 4.6 20.9→20.6 (−0.3). All ~ pending primary-source LCB verification.

---

## 11. Model Profiles

### 11.1 Gemini 3.1 Pro 🥇
**Overall:** #1 | **Quality:** #2~ (provisional, pending LCB verification; see §14.11) | **Cost tier:** Premium ($892.28 eval cost)

**v4.0 extends Gemini's lead to 70.6 pts (+1.1 vs v3.0 69.5).** The n=12 renormalization of Cost (60.0→63.6) and BrowseComp (60.0→71.4 as n grows from 6→8) contribute +0.9 and +0.34 pts. MMLU-Pro drops from 100.0 to 80.0 (−0.4 pts) as Opus 4.7 (89.87%~) displaces Gemini from #1 to #2 in that dimension. Net gain ≈ +1.1 pts.

**Why Gemini holds #1:** The only model with real data on 17 of 18 dimensions (OSWorld remains ⊘). Leads on IFBench (100.0), Speed (100.0), τ²-Bench (100.0), GPQA Diamond (100.0), SciCode (100.0), AA-Omniscience (100.0). GDP (9.1 — rank 11/12) remains the structural weakness: human preference evaluators consistently rank Gemini near-last on conversational quality.

**Best for:** Production SE workloads requiring consistent, instruction-following intelligence at scale. Long-context tasks (2M token window). Coverage-critical deployments where missing-data risk is unacceptable.

---

### 11.2 GPT-5.5 🥈
**Overall:** #2 | **Quality:** #1~ (provisional; LCB 91.7%~ secondary source; see §14.11) | **Cost tier:** Very Expensive ($3,357.00 eval cost)

**v4.0 confirms GPT-5.5 as quality #2 (70.9 pts, up from 62.4).** Two v4.0 fills: Speed confirmed at 74.7 tok/s (rank 5/12, score 63.6) and SWEPro confirmed at 68.8 (tied rank 3.5/9 with Kimi). Together these add +0.68+1.25 = +1.93 pts vs. neutral-50, pushing GPT-5.5 to quality #2 above Claude Opus 4.7. The SWE-bench Verified † dagger is removed — the 88.7% figure is now confirmed on swebench.com. Overall score rises 64.0 → 65.2.

**v4.2 LCB fill (provisional):** Multi-AI research report (April 26, 2026) attributes ~91.7% to GPT-5.5 on LCB v4 — rank 2/6, score 80.0~. If confirmed, this adds +1.8 pts (65.2→67.0) and raises quality to #1 (72.7~, narrowly above Gemini's 72.1). The 0.6-pt margin falls within the SEAL CI "effectively tied" band. See §14.11 for source conflicts. Overall score *with fill*: **67.0~**.

**Remaining gaps (3 of 18 ⊘):** LCR (not published), AIME 2025, SciCode. LCB provisionally filled at ~91.7%~ (pending primary-source confirmation). These three ⊘ dimensions are the remaining uncertainty band.

**AA-Omniscience caveat (0.0 score):** The −29 raw score (57% accuracy − 86% hallucination at xhigh) is GPT-5.5's one confirmed weakness. At standard reasoning tiers, both accuracy and hallucination rates would differ significantly.

**Best for:** Terminal/CLI agents (#1 TB 2.0), computer-use/GUI agents (#1 OSWorld), deep web research (#1 BrowseComp), τ²-Bench-sensitive agentic pipelines, SWE-bench code repair (#1 confirmed 88.7%).

---

### 11.3 KAT-Coder-Pro-V2 🥉
**Overall:** #3 | **Quality:** #10 | **Cost tier:** Budget ($73.49 eval cost)

**v3.0 cost weight compression reduces KAT's advantage slightly.** Cost weight drops 28%→25%, reducing KAT's cost contribution from 28.0 → 25.0 pts (−3.0). KAT stays at rank 1 eval cost (100.0) but earns fewer points for it. Quality profile unchanged: #2 Terminal-Bench 2.0 (90.0), #2 Speed (90.0), eval cost sole rank 1. At 58.4 pts, KAT drops from #2 to #3 but remains the best cost-to-terminal-performance ratio in the field by a wide margin.

**New dimensions:** All three new dims (SWEPro, BrowseComp, MMLU-Pro) are ⊘ → neutral-50. These contribute 3.0+1.5+1.0 = 5.5 pts at equal neutral-50 weight — no discrimination against or for KAT. KAT's strong profile (100+90+90 on three key dims) keeps it competitive.

**Best for:** Specialized terminal/coding pipelines requiring fast, cost-efficient output at scale. Not a general-purpose model; avoid for reasoning-heavy, knowledge, or GUI tasks (HLE 0.0, GPQA 9.1).

---

### 11.4 GLM 5.1
**Overall:** #4 | **Quality:** #10 | **Cost tier:** Mid-Range ($543.95 eval cost)

**v4.0 delivers a double hit to GLM's quality score.** BrowseComp fills in at 68%~ — last in the 8-model cohort (score 0.0), costing −1.5 pts at 3% weight vs. neutral-50. MMLU-Pro fills in at 85.8%~ (rank 5/6, score 20.0), costing −0.6 pts at 2% weight. Combined −2.1 pts quality impact drops GLM from quality #7 (43.1) to quality #10 (38.1). Overall score falls 58.2 → 56.7. GLM holds #4 overall only because its strong IFBench (#2, 90.0) and τ²-Bench (#3, 80.0) — both at high weights — provide a durable cost-weighted floor.

**Best for:** Instruction-following tasks requiring τ²-Bench multi-step reliability at mid-range cost. A strong alternative to Gemini for budget-conscious instruction-following workloads. Note BrowseComp weakness — not suitable for deep web research pipelines.

---

### 11.5 MiniMax M2.7
**Overall:** #5~ | **Quality:** #8 | **Cost tier:** Near-Budget ($175.51 eval cost)

**v4.2: MiniMax rises #6→#5~** (provisional) — Kimi's LCB rank compression drops Kimi to 53.9, leaving MiniMax (54.0 unchanged) 0.1 pts ahead. The margin is within the SEAL CI "effectively tied" band; treat as statistically co-ranked with Kimi pending LCB confirmation.

**v4.0: MiniMax drops from #5 to #6** as Kimi overtakes it (+2.2 pts vs. MiniMax's −1.0 pts). SWEPro fills at 56.2%~ (rank 7/9, score 25.0) — below neutral-50 (−1.0 pts at 4% weight). GDP compresses slightly as DeepSeek enters the pool. Cost score improves marginally (90.0→90.9 with n=12). Final: 54.0 pts (down from 55.0).

**Best for:** High-volume classification, summarization, extraction tasks at near-budget cost. Not suitable for coding-agent applications (Terminal-Bench last in field, 0.0). SWEPro weakness confirms: avoid for code repair pipelines.

---

### 11.6 Kimi K2.6
**Overall:** #6~ | **Quality:** #5 | **Cost tier:** Premium ($947.87 eval cost)

**v4.2: Kimi drops #5→#6~** (provisional) — LCB rank compresses from 2/5 to 3/6 as GPT-5.5 inserts at rank 2 (~91.7%, secondary source). Cost: −0.9 pts (54.8→53.9). MiniMax (54.0) is 0.1 pts ahead — within SEAL CI "effectively tied" band; treat as co-ranked pending LCB confirmation.

**v4.0: Kimi rises from #6 to #5** (+2.2 pts, 52.6→54.8). The primary driver is SWEPro renormalization: with n expanding from 5→9, Kimi's 58.6% (tied rank 3.5/9 with GPT-5.5) improves from score 37.5 to 68.8 (+1.25 pts at 4% weight). Cost also improves as n=12 lifts rank-6/12 from 50.0 to 54.5 (+1.125 pts at 25% weight). These gains outweigh minor compressions elsewhere. τ²-Bench (0.0 — rank 11/11) remains the dominant weakness at 4% weight. MMLU-Pro 84.6% at rank 6/6 (last in cohort) now scores 0.0 — last in the enlarged cohort.

**Best for:** Coding-heavy pipelines (LCB #2 at 75.0, second only to DeepSeek V4-Pro), open-weight deployments (Apache 2.0), long-context tasks (200K), SWE-bench Pro competitive at 58.6%.

---

### 11.7 GPT-5.4
**Overall:** #7 | **Quality:** #4 | **Cost tier:** Expensive ($2,851.01 eval cost)

**v4.0: GPT-5.4 drops to quality #4** (64.4, up from 62.9 in v3.0 but displaced from #3 as GPT-5.5 rises to quality #2 and Opus 4.7 holds quality #3). Overall score rises slightly 52.1→52.3 as SWEPro improves from rank 2/5 (75.0) to rank 2/9 (87.5) with n-expansion — the biggest per-dimension gain. Cost moves from 40.0 to 36.4 (rank 8/12 as DeepSeek inserts) but this is offset by SWEPro gain. Net +0.2 pts.

**Best for:** Long-context retrieval (#1 LCR 100.0), BrowseComp web research (#2 at 85.7%), SWE-bench Pro (#2 at 87.5 normalized), Terminal-Bench (81.8). Teams running research, retrieval, or code repair at expensive-but-not-ultra-expensive tier.

---

### 11.8 DeepSeek V4-Pro *(v4.0 gap-fills confirmed)*
**Overall:** #9 | **Quality:** #6 | **Cost tier:** Premium ($1,071.28 eval cost)

**v4.0 delivers a significant downward revision: #8→#9, 51.1→48.2 (−2.9 pts).** Four dimensions that were ⊘ in v3.0 now have confirmed real data — every one came in below neutral-50:

| Dimension | ⊘ Assumption (v3.0) | v4.0 Real Value | Score | Impact |
|---|---|---|---|---|
| Cost | 50.0⊘ | $1,071.28 (rank 7/12) | 45.5 | −1.13 pts |
| Terminal-Bench 2.0 | 50.0⊘ | 67.9%†self (rank 7/12) | 45.5 | −0.36 pts |
| SWE-bench Pro | 50.0⊘ | 55.4% (rank 8/9) | 12.5 | −1.50 pts |
| AA-Omniscience | 50.0⊘ | −10 (84% acc − 94% hall.) | 14.3 | −0.36 pts |

These four fills subtract a net 3.35 pts from the ⊘ baseline, partially offset by GDPval ELO 1,554 (rank 6/12, score 54.5, +0.23 pts above prior ⊘). The lesson: DeepSeek V4-Pro's neutral-50 provisional score in v3.0 was too optimistic. Its real cost-weighted profile is clearly below the frontier leaders.

**Confirmed strengths:** LiveCodeBench v4 (93.5%, rank 1/5 — still the LCB champion), SWE-bench Verified (80.6%, tied rank 4.5/10), BrowseComp (83.4%, rank 4/8), GDPval ELO 1,554 (above neutral).

**Remaining ⊘ (7 of 18):** IFBench, τ²-Bench, OSWorld, RULER, AIME, SciCode, LCB not yet in ranking pool (counted separately). Speed corrected to 35.8 tok/s (rank 11/12, score 9.1).

**Best for:** LiveCodeBench coding tasks (#1 globally), SWE-bench GitHub issue resolution, open-source deployments requiring on-premise licensing (Apache 2.0, 1M context). Avoid latency-sensitive streaming (35.8 tok/s, second-slowest in field).

---

### 11.9 Qwen 3.6 Plus
**Overall:** #8 | **Quality:** #9 | **Cost tier:** Mid-Range ($482.65 eval cost)

**v4.0: Qwen rises from #9 to #8** — not because its own score improved (49.7→49.0, slight decline), but because DeepSeek falls past it (51.1→48.2). Qwen's SWEPro fills at 56.6%~ (rank 5/9, score 37.5) — below neutral-50 (−0.5 pts). Speed compresses slightly (60.0→54.5) as GPT-5.5 inserts at rank 5. MMLU-Pro drops from rank 2/4 (66.7) to rank 3/6 (60.0) as new models enter. Net: −0.7 pts own score.

**Best for:** High-volume mid-range workloads. MMLU-Pro at rank 3/6 (88.5%) indicates strong knowledge breadth. Avoid τ²-Bench-sensitive or agentic pipelines (rank 9/11, score 20.0).

---

### 11.10 Claude Opus 4.7
**Overall:** #10 | **Quality:** #3 | **Cost tier:** Ultra-Premium ($4,811.04 eval cost)

**v4.0: Opus 4.7 gains +1.4 pts (42.9→44.3) and rises to quality #3 (68.7).** The biggest single positive fill is MMLU-Pro 89.87%~ — this puts Opus 4.7 at rank 1/6, score 100.0, worth +2.0 pts at 2% weight vs. neutral-50. BrowseComp (14.3, rank 7/8) and Omni (85.7, rank 2/8) also fill in above neutral. Cost compresses slightly (10.0→9.1 as n=12), costing −0.23 pts. Net +1.4 pts.

**Quality-cost paradox deepens in v4.0:** Quality #3 (68.7 pts) but #10 overall — a 7-rank gap, the largest in the field. GPT-5.5 displaces Opus 4.7 from quality #2, but Opus 4.7's benchmark profile remains extraordinary: #1 SWE-bench Pro (100.0), #1 HLE (100.0), #1 MMLU-Pro (100.0~), #2 GPQA (90.9). For teams where quality-per-outcome is the only metric, Opus 4.7 is the definitive choice.

**Best for:** High-stakes software engineering (SWE-bench Pro #1, 64.3%), expert-knowledge tasks (HLE #1, 46.9%), graduate-level reasoning (MMLU-Pro #1~, GPQA #2), enterprise customers unconstrained by cost.

---

### 11.11 Claude Sonnet 4.6
**Overall:** #11 | **Quality:** #12 | **Cost tier:** Very Expensive ($3,959.36 eval cost)

All three new dimensions (SWEPro, BrowseComp, MMLU-Pro) are ⊘ → neutral-50 for Sonnet. GDP renorms to 81.8 (rank 3/12 — 1,675 ELO, same rank as v3.0 but n expands to 12) and Terminal-Bench renorms to 9.1 (rank 11/12 as DeepSeek inserts above Sonnet). Final: 25.5 pts.

**Best for:** Simple conversational tasks where Anthropic ecosystem compatibility is required. Not recommended for coding-agent, agentic, or math-intensive pipelines.

---

### 11.12 Claude Opus 4.6
**Overall:** #12 | **Quality:** #11 | **Cost tier:** Ultra-Premium ($4,969.68 eval cost)

Most expensive model to run ($4,969.68, rank 11/11, cost score 0.0). SWE-bench Pro (51.9%, rank 9/9, score 0.0) is Opus 4.6's worst result in the new dimensions — last in the expanded SWE-Pro cohort. **BrowseComp now filled at 83.7%~ (score 57.1~, rank 4/8)** — above neutral-50, a positive v4.0 fill. MMLU-Pro remains ⊘ (neutral-50). AIME 2025 tied rank 1.5 at 99.8% (90.0) and SWE-bench #3 (77.8) remain the dimension bright spots. At essentially equivalent eval cost to Opus 4.7 ($4,970 vs $4,811) but lower quality on nearly every dimension, migration to 4.7 is the dominant recommendation.

**Best for:** Legacy deployments that cannot migrate; no new deployments recommended.

---

## 12. Strategic Insights

### 12.1 GPT-5.5 Confirmed as Quality #2 — Gap Narrows on Gemini

v4.0 confirms two more GPT-5.5 gaps: Speed at 74.7 tok/s (rank 5/12, score 63.6) and SWEPro at 68.8 (tied rank 3.5/9 with Kimi). With these fills, GPT-5.5 now holds confirmed #1 on 5 agentic dimensions (TB 2.0, GDP, OSWorld, BrowseComp, SWE-bench Verified) and quality #2 overall (70.9 pts). τ²-Bench is #2 (98.0% vs Gemini's 99.3%). The SWE-bench Verified dagger is removed — confirmed on canonical swebench.com.

**v4.2 update:** Gemini leads GPT-5.5 by 3.0~ pts (70.0 vs. 67.0~) after provisional LCB fill at ~91.7%~. GPT-5.5 now has 3 ⊘ dimensions (LCR, AIME, SciCode). The LCB fill adds +1.8 pts and narrows the gap from 5.4 pts to 3.0 pts — but is from a secondary source and pending LCB leaderboard confirmation (see §14.11). Gemini's near-complete 17/18 data profile means it cannot be displaced by surprise real data; its overall #1 lead is structural even if quality #1 is provisionally tied.

### 12.2 DeepSeek V4-Pro: The Gap-Fill Surprise

**The central v4.0 finding is that DeepSeek's neutral-50 provisional score was too optimistic.** All four confirmed gap-fills came in below neutral: Cost 45.5 (not cheap), TB 2.0 67.9%†self (mid-tier), SWEPro 55.4% (rank 8/9, second-worst in cohort), Omni −10 (84% accuracy vs. 94% hallucination rate). The net effect: −2.9 pts, dropping DeepSeek from #8 to #9.

This is a data lesson for provisional-model scoring: a composite AI Intelligence Index score of 52 (frontier parity) does not uniformly distribute across 18 dimensions. DeepSeek's gap-fills reveal a model strong on LCB (93.5%, #1) and SWE-bench Verified (80.6%) but surprisingly weak on structured code repair (SWEPro last-quintile) and hallucination control (94% rate). Its one positive fill — GDPval ELO 1,554 — confirms human preference quality above the open-weight median.

LCB dominance (93.5%) still establishes DeepSeek as the open-source coding champion for competition-programming tasks. At $1.74/$3.48 per MTok with a 1M context window, the value proposition is strongest for LCB-type batch coding workloads.

### 12.3 Coverage Expansion Sharpens Discrimination

v4.0 significantly expands coverage on three dimensions:

- **SWE-bench Pro** (5→9 models): Now covers 75% of the cohort. Opus 4.7 leads at 64.3% — a 5.2-point gap to #2 GPT-5.4 (59.1%). The expanded cohort confirms SWEPro is harder than SWE-bench Verified: scores are more spread out and DeepSeek (55.4%) is decisively below Kimi (58.6%).
- **BrowseComp** (6→8 models): Opus 4.6 (83.7%~) and GLM (68%~) added. GLM's last-place finish (0.0 normalized) is the most significant new data point — BrowseComp is GLM's structural blind spot.
- **MMLU-Pro** (4→6 models): Opus 4.7 (89.87%~) enters at rank 1, displacing Gemini from the MMLU-Pro lead. This is the largest single positive fill in v4.0 — Opus 4.7 gains +2.0 pts (100.0 × 2% weight) vs. neutral-50.

### 12.4 Kimi K2.6 Solidifies #5 — The SWEPro Renormalization Effect

With SWEPro n expanding from 5→9, Kimi's 58.6% improves in rank (from tied 3.5/5 to tied 3.5/9 — but score goes from 37.5 to 68.8 as the denominator grows). This +1.25 pt gain at 4% weight is the single largest per-dimension gain in v4.0 for any model. Combined with Cost score improvement (+1.13 pts at 25% weight), Kimi rises from #6 to #5. The τ²-Bench weakness (0.0, last in field) caps further rise.

### 12.6 Pareto Frontier Analysis — Quality × Cost × Speed

The main ranking integrates cost, reliability, and quality into a single composite score. A complementary view applies strict Pareto dominance across three independent axes: **quality-only score** (§8, equal-weight average of all 17 non-cost dimensions), **AA Eval Cost** (lower is better), and **Speed** (tok/s, higher is better). A model is Pareto-optimal if no other model is simultaneously at least as good on all three axes with strict improvement on at least one.

| Model | Quality Score | AA Eval Cost | Speed (tok/s) | Pareto Status |
|---|---|---|---|---|
| **Gemini 3.1 Pro** | **72.1** | $892.28 | **128.0** | **✓ Frontier** |
| **GPT-5.5** | **72.7~** | $3,357.00 | 74.7 | **✓~ Frontier (provisional)** — quality 72.7~ > Gemini's 72.1; Gemini no longer dominates all three axes |
| Claude Opus 4.7 | 68.7 | $4,811.04 | 42.0 | ✗ Dominated (Gemini) |
| GPT-5.4 | 64.4 | $2,851.01 | 74.8 | ✗ Dominated (Gemini) |
| Kimi K2.6 | 48.0 | $947.87 | 100.4 | ✗ Dominated (Gemini: $892 < $948, 128 > 100.4, 72.1 > 48.0) |
| DeepSeek V4-Pro | 45.9 | $1,071.28 | 35.8 | ✗ Dominated (Gemini) |
| **KAT-Coder-Pro-V2** | 41.9 | **$73.49** | 113.5 | **✓ Frontier** |
| MiniMax M2.7 | 40.9 | $175.51 | 49.0 | ✗ Dominated (KAT: higher quality, cheaper, faster) |
| Qwen 3.6 Plus | 38.4 | $482.65 | 53.0 | ✗ Dominated (KAT) |
| GLM 5.1 | 38.1 | $543.95 | 49.0 | ✗ Dominated (KAT) |
| Claude Opus 4.6 | 36.9 | $4,969.68 | 18.2 | ✗ Dominated (GLM and many others) |
| Claude Sonnet 4.6 | 32.0 | $3,959.36 | 46.0 | ✗ Dominated (Qwen: higher quality, much cheaper, faster) |

**v4.2 update: Provisionally 3 Pareto-optimal models.** With GPT-5.5's quality provisionally rising to 72.7~ (LCB 91.7%~, secondary source), GPT-5.5 now has *higher* quality than Gemini (72.1). Gemini can no longer dominate GPT-5.5 on all three axes — it wins cost and speed but loses quality by 0.6 pts. GPT-5.5 therefore becomes a provisional Pareto frontier member (highest quality, no model beats it on quality while also being cheaper and faster). The frontier expands from 2 to 3~: Gemini (quality-speed-cost balance), GPT-5.5~ (quality apex), KAT (cost floor). **However, the 0.6-pt quality gap is within the SEAL CI band (§16B) and the data is from a secondary source — pending LCB primary-source confirmation, revert to the v4.0 Pareto result (2 frontier models: Gemini + KAT).** KAT-Coder-Pro-V2 holds the budget frontier unchanged.

**Implication:** The Pareto analysis sharpens the main ranking's insight. The composite score resolves the quality-cost-speed tradeoff by weighting cost at 25%. The Pareto view shows that the same tradeoff, when left unresolved, yields (confirmed) two rational choices or (provisional v4.2) three: Gemini 3.1 Pro (quality-speed-cost balance), KAT-Coder-Pro-V2 (sole occupant of the cost frontier), and provisionally GPT-5.5~ (quality apex if LCB 91.7%~ is confirmed). The confirmed result is unchanged: every model except Gemini and KAT is dominated on all three confirmed-data axes simultaneously.

### 12.7 Cost-of-Pass — Expected Cost Per Correct Output

The Erol et al. 2025 framework (arXiv:2504.13359) defines **cost-of-pass** as the expected monetary cost to obtain one correct answer: `cost_of_pass = (cost per attempt) / (pass_rate)`. Applied to the AA Eval Cost data (§4.1) and benchmark pass rates (§5.1), this metric answers a question the main composite cannot: not "which model scores best relative to its cost?" but "how much does one correct output actually cost?"

**Methodology:** AA Eval Cost captures the total USD to run the full benchmark suite on each model. Treating it as a proxy for cost-per-run across a fixed task set and dividing by benchmark pass rate yields a relative cost-of-correct-output index. Relative comparisons across models on the same benchmark are valid; cross-benchmark comparisons are not (different tasks, different difficulty distributions).

**SWE-bench Verified (10 models with both cost and pass rate data):**

| Model | AA Eval Cost | SWE-V Pass Rate | Cost-of-Pass |
|---|---|---|---|
| **KAT-Coder-Pro-V2** | $73.49 | 79.6% | **$92** |
| Qwen 3.6 Plus | $482.65 | 78.8% | $613 |
| GLM 5.1 | $543.95 | 77.8% | $699 |
| Gemini 3.1 Pro | $892.28 | 80.6% | $1,107 |
| Kimi K2.6 | $947.87 | 80.2% | $1,182 |
| DeepSeek V4-Pro | $1,071.28 | 80.6% | $1,329 |
| GPT-5.5 | $3,357.00 | 88.7% | $3,786 |
| Claude Sonnet 4.6 | $3,959.36 | 79.6% | $4,973 |
| Claude Opus 4.7 | $4,811.04 | 87.6% | $5,492 |
| Claude Opus 4.6 | $4,969.68 | 80.8% | $6,151 |
| MiniMax M2.7 | $175.51 | ⊘ | — |
| GPT-5.4 | $2,851.01 | ⊘ | — |

**Terminal-Bench 2.0 (all 12 models):**

| Model | AA Eval Cost | TB 2.0 Pass Rate | Cost-of-Pass |
|---|---|---|---|
| **KAT-Coder-Pro-V2** | $73.49 | 76.2% | **$96** |
| MiniMax M2.7 | $175.51 | 57.0% | $308 |
| Qwen 3.6 Plus | $482.65 | 61.6% | $783 |
| GLM 5.1 | $543.95 | 69.0% | $788 |
| Gemini 3.1 Pro | $892.28 | 68.5% | $1,303 |
| Kimi K2.6 | $947.87 | 66.7% | $1,421 |
| DeepSeek V4-Pro†self | $1,071.28 | 67.9% | $1,578 |
| GPT-5.4 | $2,851.01 | 75.1% | $3,797 |
| GPT-5.5 | $3,357.00 | 82.7% | $4,059 |
| Claude Sonnet 4.6 | $3,959.36 | 59.1% | $6,700 |
| Claude Opus 4.7 | $4,811.04 | 69.4% | $6,933 |
| Claude Opus 4.6 | $4,969.68 | 65.4% | $7,598 |

†self = DeepSeek TB 2.0 is self-reported; treat cost-of-pass as provisional.

**Key findings:**
1. **KAT's cost advantage compounds.** At SWE-bench Verified, KAT's $92 cost-of-pass is 12× cheaper than Gemini ($1,107), 41× cheaper than GPT-5.5 ($3,786), and 67× cheaper than Claude Opus 4.6 ($6,151) — despite KAT's pass rate (79.6%) being near-equivalent to Gemini's (80.6%). The cost difference is entirely driven by eval cost ($73 vs $892), not quality.
2. **High pass rates do not offset high prices.** GPT-5.5's 88.7% SWE-V pass rate (best in field) still yields a $3,786 cost-of-pass — 3.4× Gemini's $1,107. Claude Opus 4.7's 87.6% SWE-V pass rate produces a $5,492 cost-of-pass because its $4,811 base cost cannot be overcome by any realistic pass rate advantage.
3. **Task-type specialization matters.** For Terminal-Bench specifically, MiniMax M2.7 ($308) is the second-cheapest model after KAT — despite its modest 57.0% pass rate. Its $175.51 eval cost dominates. Qwen and GLM cluster at $783–788, making them the natural mid-range choice for terminal workloads. This confirms the Erol et al. finding that cost-of-pass task-type specialization is real: the optimal model for SWE tasks (KAT, Gemini, Kimi) differs from the price-performance leader for terminal tasks (KAT, MiniMax).
4. **The 25% cost weight is validated by cost-of-pass math.** A 1-percentage-point difference in pass rate is worth approximately $50–$150 in cost-of-pass for budget models and $3,000–$6,000 for ultra-premium models. The cost dimension carries more dollar-value information per weight point than any single quality benchmark.

### 12.5 The Anthropic Cost Structure Remains the Dominant Constraint

Under the AA Index Eval Cost methodology, all three Anthropic frontier models (Sonnet $3,959, Opus 4.7 $4,811, Opus 4.6 $4,970) remain in the top-4 most expensive to run. At 25% cost weight, Claude Opus 4.7's cost score (9.1) contributes only 2.3 pts vs. Gemini's 15.9 pts — a 13.6 pt cost disadvantage that even quality #3 (68.7 pts) cannot overcome. Opus 4.7 scores 44.3 overall vs. Gemini's 70.6 — a 26-pt gap driven almost entirely by the cost dimension. Teams running cost-unconstrained pipelines where SWEPro or HLE performance matters most should consider Opus 4.7; all production workloads face the structural cost ceiling.

---

## 13. Use Case Recommendations

| Use Case | Primary Choice | Budget Alternative | Notes |
|---|---|---|---|
| Production SE pipeline (cost-first) | **Gemini 3.1 Pro** | GLM 5.1 | #1 overall at 70.6 pts; broadest coverage; $892 eval cost |
| Broadest-coverage production choice | **Gemini 3.1 Pro** | GPT-5.5 | Real data on 17/18 dims; #1 IFBench, Speed, τ², GPQA, Sci, Omni |
| Minimum eval-cost processing | **KAT-Coder-Pro-V2** | MiniMax M2.7 | $73.49 eval cost (#1) vs $175.51 (#2) |
| High-volume batch processing | **KAT-Coder-Pro-V2** | MiniMax M2.7 | KAT: $73.49 + TB 2.0 #2; MiniMax: $175.51 + IFBench #5 |
| Coding specialist (LCB / competition) | **DeepSeek V4-Pro** | GPT-5.5~ / Kimi K2.6 | LCB #1 (93.5%) vs #2~ GPT-5.5 (91.7%~, provisional) vs #3 Kimi (82.6%); DeepSeek + Kimi are open-source |
| Coding specialist (terminal/CLI) | **GPT-5.5** | KAT-Coder-Pro-V2 | TB 2.0: 82.7% (#1) vs 76.2% (#2) |
| Computer-use / GUI automation | **GPT-5.5** | Claude Opus 4.7 | OSWorld #1 (78.7%) by large margin |
| Multi-step agentic tasks (τ²) | **Gemini 3.1 Pro** | GPT-5.5 | τ²: 99.3% (#1) vs 98.0% (#2) — Gemini now leads |
| SWE-bench / GitHub issue resolution | **GPT-5.5** | Claude Opus 4.7 | SWE-V: 88.7% (#1, confirmed) vs 87.6% (#2, confirmed) |
| SWE-bench Pro / hard code repair | **Claude Opus 4.7** | GPT-5.4 | SWE-Pro: 64.3% (#1) — 5.2% lead over GPT-5.4 (59.1%) |
| Deep web research / BrowseComp | **GPT-5.5** | GPT-5.4 | BrowseComp: 90.1% (#1) vs 89.3% (#2) |
| Long-context retrieval (128K+) | **GPT-5.4** | Gemini 3.1 Pro | RULER: 97.8% vs 94.2% |
| Competition math / reasoning ceiling | **Claude Opus 4.7 / 4.6** | Kimi K2.6 | AIME: 99.8% (tied Opus) vs 96.1% (Kimi) |
| Graduate-level knowledge (MMLU-Pro) | **Claude Opus 4.7** | Gemini 3.1 Pro | MMLU-Pro: 89.87%~ (#1, v4.0 new) vs 89.8% (#2) |
| Instruction-following reliability | **Gemini 3.1 Pro** | GLM 5.1 | IFBench: 89.4% (#1) vs 85.9% (#2) |
| Fast streaming / real-time | **Gemini 3.1 Pro** | KAT-Coder-Pro-V2 | 128.0 tok/s (#1) vs 113.5 tok/s (#2) |
| Open-source / on-premise frontier | **DeepSeek V4-Pro** | Kimi K2.6 | LCB #1, Apache 2.0, 1M context, $1.74/MTok |
| Highest quality, cost unconstrained | **GPT-5.5** | Claude Opus 4.7 | 5 agentic dim leads (TB 2.0, GDP, OSWorld, BrowseComp, SWE-V); Opus 4.7 quality #3 (68.7); GPT-5.5 quality #2 (70.9) |
| Expert knowledge / HLE tasks | **Claude Opus 4.7** | Gemini 3.1 Pro | HLE #1 (46.9%) vs #2 (44.7%); MMLU-Pro #1 (89.87%~) |
| Mid-range instruction + agentic | **GLM 5.1** | Qwen 3.6 Plus | #4 overall (56.7); $544 eval cost; IFBench #2 + τ² #3 |

---

## 14. Limitations & Known Gaps

### 14.1 Data Freshness

- **DeepSeek V4-Pro** (April 24, 2026): 7 dimensions remain ⊘ after v4.0 gap-fills: IFBench, τ²-Bench, OSWorld, RULER, AIME, SciCode, LCB (not yet in LCB pool). All four confirmed fills (Cost, Term, SWEPro, Omni) came in below neutral-50 — the provisional #8 rank in v3.0 was optimistic.
- **GPT-5.5** (April 23, 2026): 3 dimensions still ⊘. Speed (63.6) and SWEPro (68.8) confirmed in v4.0; LCB provisionally filled at ~91.7%~ in v4.2 (secondary source, multi-AI aggregated report; see §14.11 for conflicts). Confirmed ⊘ remaining: LCR, AIME, SciCode.
- All other model data is sourced from official publications and established leaderboards as of April 25, 2026.

### 14.2 GPT-5.5 SWE-bench Verified — Confirmed

The 88.7%† provisional value from v3.0 is now confirmed on the canonical swebench.com leaderboard. The dagger (†) marker is removed in v4.0. GPT-5.5 is the confirmed #1 on SWE-bench Verified as of April 25, 2026.

### 14.3 Benchmark Contamination Risk

LiveCodeBench v4 and τ²-Bench use rolling test sets specifically to address contamination, but older benchmarks (GPQA, SWE-bench) may have varying contamination levels across models depending on training cutoff and data curation practices.

### 14.4 Mode Selection Ambiguity

Several models offer multiple reasoning modes. Published benchmark scores typically reflect the best available mode. Rankings may not reflect the model's behavior at its production-default mode.

### 14.5 GPT-5.5 AA-Omniscience

The 86% hallucination rate driving the −29 raw Omniscience score comes from early independent evaluations specifically targeting GPT-5.5's xhigh reasoning mode. At standard or high reasoning modes, both accuracy and hallucination rates would differ significantly.

### 14.6 GPT-5.4 AIME Conflict

LayerLens independent testing: 16.67%. Multiple secondary sources attribute 100% performance to GPT-5.4 (possibly misattributed from GPT-5.2 xhigh). Unresolvable with current data. GPT-5.4 receives ⊘=50 on AIME 2025.

### 14.7 Dimension Coverage Status (v4.0)

| Dimension | v3.0 Coverage | v4.0 Coverage | Discriminative Power |
|---|---|---|---|
| SWE-bench Pro | 5/12 (42%) | 9/12 (75%) | High — 5.2% spread between #1 and #2 |
| BrowseComp | 6/12 (50%) | 8/12 (67%) | High — GLM 0.0 vs GPT-5.5 100.0 reveals clear separation |
| MMLU-Pro | 4/12 (33%) | 6/12 (50%) | Moderate — still 6 models at neutral-50 |

SWEPro is now sufficiently covered to be highly discriminative. MMLU-Pro remains the weakest of the three activated dimensions; expect further coverage expansion in v4.1+.

### 14.8 Framework Design Constraints

The 25% cost weight reflects a deliberate choice for production scale optimization. Academic or research applications would reasonably down-weight cost and up-weight quality dimensions. Custom weight configurations are recommended for teams with different budget constraints.

### 14.9 BigCodeBench and GAIA Slots

Both benchmarks remain reserved as scored dimensions pending ≥4 model submissions from the current 12-model inventory. Will be re-evaluated in v4.0.

### 14.10 FireBench Sub-Dimension Coverage and Reliability Roadmap

FireBench (arXiv:2603.04857, 2026) identifies six constraint categories in enterprise instruction-following: **(1) output format compliance**, **(2) ordered responses**, **(3) item ranking**, **(4) overconfidence calibration**, **(5) positive content requirements**, and **(6) negative content requirements**. The current Reliability dimension (IFBench, 18%) and its planned composite (IFEval 65% + IFBench 35%) cover these categories unevenly:

| FireBench Category | IFEval Coverage | IFBench Coverage | Composite Gap |
|---|---|---|---|
| Output format compliance | Strong (26 fixed format constraint types) | Partial (novel format phrasings) | Minimal — composite covers well |
| Ordered responses | Weak (limited fixed ordering tasks) | Moderate (novel ordering constraints) | Partial — IFBench adds generalization signal |
| Item ranking | None in fixed set | Moderate (novel ranking/priority constraints) | Partial — IFBench adds but does not saturate |
| Overconfidence calibration | None | None | **Full gap** — not an IF metric; addressed separately by AA-Omniscience (1% weight) |
| Positive content requirements | Strong (phrase inclusion, keyword requirements) | Strong (out-of-domain generalization) | Minimal — composite covers well |
| Negative content requirements | Partial (fixed forbidden-phrase patterns) | Moderate (novel exclusion phrasings) | Partial — composite improves but does not saturate |

**Coverage summary:** The IFEval+IFBench composite addresses 5 of 6 FireBench categories (format compliance, positive content well; ordered responses, item ranking, negative content partially). It has zero coverage of overconfidence calibration — the category where models that rank 1st on format compliance often rank 5th or 6th, per FireBench's 13–25 pp cross-category variance finding. ValueRank addresses calibration separately via AA-Omniscience at 1% weight — a defensible design choice because calibration is a distinct epistemic property (does the model know what it doesn't know?) rather than a format-compliance property (does the model follow the stated constraint?).

**Roadmap implications:** Future Reliability dimension upgrades will target the two partially-covered categories — ordered responses and item ranking — which FireBench identifies as particularly discriminative for enterprise use cases such as prioritized task queues and structured report generation. A FireBench-aligned sub-scoring scheme (model-level scores for each of the 6 categories) would allow ValueRank to weight enterprise IF failures by category severity. This upgrade is contingent on FireBench publishing per-model category breakdowns as standard leaderboard output.

### 14.11 External Research Data Conflicts (v4.2)

The multi-AI aggregated research report (April 26, 2026; sources: Gemini, Claude, ChatGPT, Copilot, Perplexity, DeepSeek) provided two fills incorporated in v4.2 and three conflicts flagged but not acted upon. All three unresolved conflicts require primary-source verification before updating.

| Dimension | External Report Value | ValueRank Current Value | Source of Conflict | Resolution |
|---|---|---|---|---|
| **GPT-5.5 LCB v4** | ~91.7% | ⊘ → **91.7%~** (v4.2 fill) | Multi-AI report (6 sources); approximate (~) | **Incorporated with ~ notation.** Rank 2/6, score 80.0~. Note: report shows Kimi LCB = 89.6% vs our 82.6% — suggests possible LCB version mismatch. Pending canonical leaderboard confirmation. |
| **Kimi K2.6 AA Intelligence Index** | 54 | ⊘ → **54~** (v4.2 fill) | Multi-AI report; reference column only (non-scored) | **Incorporated with ~ notation.** Non-scored reference column; no score impact. |
| **Kimi K2.6 τ²-Bench** | 96% | 72.4% (canonical) | Multi-AI report vs. Artificial Analysis leaderboard | **Not updated.** 96% would make Kimi τ² rank 2/11 (above GPT-5.5's 98.0%). Major discrepancy — possible τ¹-Bench vs. τ²-Bench variant confusion, or different test set. Requires direct AA leaderboard verification. |
| **Kimi K2.6 LCB v4** | 89.6% | 82.6% (canonical) | Multi-AI report vs. prior research session | **Not updated.** If 89.6% were correct (still rank 2/5 in the existing 5-model pool, unchanged score), our score would be unaffected but raw value would differ. Low-priority pending LCB leaderboard check. |
| **DeepSeek V4-Pro hallucination rate** | 54% | 94% (from AA article) | Multi-AI report vs. Artificial Analysis article | **Not updated.** 94% hallucination rate is sourced from an Artificial Analysis article ("DeepSeek is back among the leading open weights models") cited in §5.3. The 54% figure may reflect V4-Flash rather than V4-Pro, or a different measurement methodology. Requires primary AA source re-verification. |

**Verification priority:** LCB leaderboard (affects GPT-5.5 quality rank + Kimi/MiniMax ordering) → AA τ²-Bench leaderboard (Kimi τ² would significantly improve Kimi's score) → DeepSeek hallucination rate (AA Omniscience scoring).

---

## 15. Benchmark Reference Index

| Benchmark | Dimension | Type | What it Measures | Source |
|---|---|---|---|---|
| AA Index Eval Cost | Cost | USD | Total cost to run AA's full Intelligence Index benchmark suite (price × actual token usage) | artificialanalysis.ai |
| IFBench | IF (Reliability) | Accuracy % | Reliability dimension: instruction-following generalization across 58 out-of-domain verifiable constraints (Allen AI, NeurIPS 2025). Tests constraint compliance on constraint types not seen in training — unsaturated at 58–76% for current frontier models. Used as the primary Reliability signal; future composite will add IFEval (IFEval 65% + IFBench 35%). | Artificial Analysis IFBench leaderboard; github.com/allenai/IFBench |
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

At 15% cost weight, GPT-5.5 extends its #2 advantage (quality leads on 5 confirmed agentic dims). Claude Opus 4.7 recovers to #3 as cost penalty ($4,811) shrinks. KAT-Coder-Pro-V2 drops out of top-5 as its quality (#10) becomes the binding constraint at lower cost weight.

**What if cost weight = 40% (cost-extreme scenario)?**

| Rank | Model | Score |
|---|---|---|
| 1 | KAT-Coder-Pro-V2 | 66.1 |
| 2 | Gemini 3.1 Pro | 65.2 |
| 3 | MiniMax M2.7 | 62.8 |
| 4 | GLM 5.1 | 60.8 |
| 5 | Qwen 3.6 Plus | 57.0 |

At 40% cost weight, KAT overtakes Gemini. MiniMax rises to #3. GPT-5.5 drops to #8 (cost 40% weight at 30.0 score = 12.0 pts). Claude Opus 4.6 collapses to #12 (cost score 0.0 at 40% = structurally fatal).

**What if Reliability (IFBench) weight = 10% (reduced from 18%)?**

Freed 8% redistributed proportionally to the 16 remaining quality dimensions (Quality: 57%→65%). Cost fixed at 25%.

| Rank | Model | Score |
|---|---|---|
| 1 | Gemini 3.1 Pro | 67.8 |
| 2 | GPT-5.5 | 66.1 |
| 3 | KAT-Coder-Pro-V2 | 60.0 |
| 4 | GPT-5.4 | 54.1 |
| 5 | GLM 5.1 | 52.7 |

At 10% Reliability weight, GPT-5.4 rises from #7 to #4 — its overall quality (64.4) is strong but its IFBench normalized score (40.0) is weak; lower IF weight rewards the former. GLM drops from #4 to #5 as its IFBench advantage (90.0 normalized) matters less. KAT holds #3 — it is IFBench-weak (30.0 normalized) but cost-dominant (100.0); the cost weight of 25% keeps it well above lower-cost competitors. Gemini and GPT-5.5 remain #1 and #2 regardless of IF weight: Gemini leads IFBench (100.0), GPT-5.5 leads the overall quality distribution.

**What if Reliability (IFBench) weight = 25% (increased from 18%)?**

Added 7% sourced proportionally from the 16 remaining quality dimensions (Quality: 57%→50%). Cost fixed at 25%.

| Rank | Model | Score |
|---|---|---|
| 1 | Gemini 3.1 Pro | 73.1 |
| 2 | GPT-5.5 | 64.5 |
| 3 | GLM 5.1 | 60.3 |
| 4 | KAT-Coder-Pro-V2 | 57.1 |
| 5 | Kimi K2.6 | 57.1 |

At 25% Reliability weight, GLM 5.1 rises from #4 to #3 (IFBench #2 at 90.0 normalized contributes 22.5 pts). Kimi enters the top 5 (IFBench #3 at 80.0 → 20.0 pts at 25% weight). GPT-5.4 drops below top-5 (IFBench 40.0 normalized → only 10.0 pts). Claude Opus 4.7 drops to #10 — its IFBench weakness (52.1% raw, 20.0 normalized) inflicts its largest penalty at 25% weight, offsetting its strong benchmark quality elsewhere. A 25% Reliability weight is defensible for deployment contexts with strictly machine-consumer outputs (APIs, parsers, structured data pipelines) where format failures have zero operational tolerance.

### Statistical Confidence and Rank Uncertainty

The rank-based normalization formula treats all rank separations as equivalent: the gap between rank 1 and rank 2 is scored identically to the gap between rank 6 and rank 7, regardless of the size of the underlying score difference. This is a deliberate choice for scale-invariance (§3.1), but it has a known limitation: **adjacent ranks may be statistically indistinguishable** on the underlying benchmark.

SEAL Leaderboards (Scale Labs) operationalizes this directly: their statistical rank is computed as the count of models that are **statistically significantly better at 95% CI** + 1, rather than the raw ordinal position. Under SEAL methodology, two models with overlapping confidence intervals receive the same statistical rank even when their point estimates differ. In the current ValueRank cohort, this distinction matters most for:

- **IFBench (n=11):** Frontier models cluster at 73–90%. Kimi (82.1%) and GPT-5.5 (75.9%) are raw ranks 3 and 4, but the ~6 pp gap may fall within sampling variability for a 58-item benchmark.
- **SWE-bench Pro (n=9):** Five models cluster at 55.4–58.6% (DeepSeek, MiniMax, Qwen, GLM, Kimi). The 3.2 pp spread across 5 models likely yields multiple statistical ties under 95% CI testing.
- **HLE and GPQA Diamond:** Score ranges are wide (HLE: 12.7%–46.9%; GPQA: 84.0%–94.3%), so adjacent-rank uncertainty is lower for these dimensions.

**Practical implication:** ValueRank's rank-based scores are point estimates. Final composite scores within 2–3 points of each other (e.g., Kimi 54.8 vs. MiniMax 54.0 vs. GPT-5.4 52.3) should be treated as effectively tied, pending statistical significance analysis on individual contributing dimensions. The cost and IF weight sensitivity analyses above provide a partial guard: models that consistently appear in the top positions across multiple weight scenarios have more robust rankings than models that appear in only one scenario.

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

### 16I. Framework Philosophy: ValueRank vs. HELM

**HELM (Holistic Evaluation of Language Models, Stanford CRFM, 2022)** is the most prominent academic antecedent to multi-dimensional LLM evaluation. HELM evaluates models across 7 metrics — accuracy, calibration, robustness, fairness, bias, toxicity, and efficiency — over 16 diverse scenarios. Its defining design decision is to **refuse to produce a single composite score.** HELM explicitly holds that "no single number can capture the full picture" of model behavior, and that collapsing dimensions into a composite risks masking performance variations that matter for specific use cases.

ValueRank deliberately departs from this stance on two dimensions:

**1. Integration over parallelism.** HELM reports 7 metrics per scenario as parallel columns; practitioners must perform their own weighting. ValueRank argues that leaving integration to the practitioner does not eliminate implicit weighting — it just makes the weighting invisible and inconsistent. A practitioner who mentally weights cost at ~25% when selecting a model is making the same judgment ValueRank makes explicitly. The difference is transparency and reproducibility: explicit weights can be challenged, debated, and adjusted via the sensitivity analysis in §16B; implicit weights cannot.

**2. Financial cost over computational cost.** HELM's efficiency metric measures computational resource requirements (tokens generated, latency, memory), grounded in a 2022 context where the relevant cost question was GPU budget. ValueRank uses AA Eval Cost (total USD per benchmark run) as the primary cost signal, reflecting the 2025–2026 reality where frontier models are API-accessed and the relevant cost question is dollar spend per task. HELM's calibration metric (confidence alignment with correctness) maps directly to ValueRank's AA-Omniscience dimension (accuracy% − hallucination_rate%). HELM's robustness metric (performance under input perturbation) maps philosophically to ValueRank's IFBench Reliability dimension (performance under novel constraint variation).

**Where HELM's philosophy prevails in ValueRank:** The quality-only ranking (§8) and sensitivity analysis (§16B) are ValueRank's concession to HELM's point. §8 strips out cost entirely to show pure capability rankings, and §16B shows how rankings shift at alternative cost weights (15% and 40%). These sections allow practitioners who disagree with the 25% cost weight to recalibrate without abandoning the framework.

**Where ValueRank diverges:** For production deployment decisions — the primary use case of this document — a single ranked output is more actionable than a parallel multi-metric table. The Pareto frontier analysis in §12.6 shows the consequence of HELM's approach applied to our three primary axes: it yields only two rational model choices, leaving 10 of 12 models with no clear positioning. A weighted composite resolves this by encoding the tradeoff explicitly.

### 16J. v4.1 Methodology Sources

Key sources informing v4.1 methodology additions (from ultradeep research pipeline, April 25–26, 2026):
- Allen AI IFBench GitHub repo + Artificial Analysis IFBench leaderboard (IFBench 58-constraint design; frontier scores 58–76%)
- FireBench paper arXiv:2603.04857 (enterprise IF: best model 74.0%; 13–25 pp category variance)
- BenchLM.ai composite IF methodology (IFEval 65% + IFBench 35% rationale)
- arXiv:2511.23455 "Price of Progress" (per-token vs. task-level cost; reasoning token inflation)
- DigitalApplied Q2 2026 Efficient Frontier analysis (Pareto methodology: quality × cost × speed)
- Liang et al. 2022 "Holistic Evaluation of Language Models" arXiv:2211.09110 (HELM 7-metric framework; multi-metric vs. composite philosophy)

### 16H. v4.0 Research Sources

Key sources used for v4.0 gap-fill updates (from ultradeep research pipeline, April 25, 2026):
- Artificial Analysis AA Index Eval Cost leaderboard (DeepSeek V4-Pro $1,071.28 confirmed; GPT-5.5 Speed 74.7 tok/s confirmed; DeepSeek Speed corrected 33.5→35.8 tok/s)
- DeepSeek V4-Pro official model card / self-reported benchmarks (TB 2.0 67.9%†self)
- Scale Labs SWE-bench Pro leaderboard (SWEPro fills: GLM 58.4%~, Qwen 56.6%~, MiniMax 56.2%~, DeepSeek 55.4%; n expanded 5→9)
- Artificial Analysis GDPval leaderboard (DeepSeek V4-Pro ELO 1,554 confirmed; n expanded 11→12)
- BenchLM / llm-stats.com/benchmarks/browsecomp (BrowseComp fills: Opus 4.6 83.7%~, GLM 68%~; n expanded 6→8)
- Artificial Analysis MMLU-Pro leaderboard (MMLU-Pro fills: Opus 4.7 89.87%~, GLM 85.8%~; n expanded 4→6)
- Artificial Analysis Omniscience leaderboard (Omni fills: Kimi +6 = 45%−39%, DeepSeek −10 = 84%−94%; n expanded 6→8)
- swebench.com canonical leaderboard (GPT-5.5 88.7% confirmed; † removed)

---

*Report maintained by the Kay project team | Feedback: kay-rankings@alo-exp.dev*
*Methodology questions: see §16 Appendix | Version history: tracked in git history of this file*
*Next scheduled update: May 2026 quarterly refresh or next major model launch, whichever comes first*
*v4.1 methodology additions: Reliability dimension rename (§3.4), BenchLM categories (§3.5), AA Intelligence Index cross-reference (§5.3), Pareto frontier (§12.6), Cost-of-Pass analysis (§12.7), FireBench sub-dimensions (§14.10), AA Eval Cost rationale (§4.1), IF weight sensitivity (§16B), SEAL CI note (§16B), HELM philosophy (§16I) — no score changes*
*v4.2 data fills (secondary source, ~): GPT-5.5 LCB ~91.7%~ fills ⊘ (§5.1, scored, +1.8 pts); Kimi K2.6 AA Intelligence Index 54~ fills ⊘ (§5.3, reference only); data conflicts flagged §14.11 — GPT-5.5 67.0~, Gemini 70.0, Kimi 53.9, Opus 4.6 20.6; Kimi/MiniMax swap #5/#6 (within SEAL CI tied band); quality #1 provisionally GPT-5.5 72.7~*
