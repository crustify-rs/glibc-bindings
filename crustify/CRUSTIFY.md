
Fill in as many answers as you want before starting Crustify. It will ask only
for campaign decisions that remain unresolved.

# Mandatory questions

## Campaign

1. **Which repository and revision should this campaign use?** (`id: source`)
   - Answer: `https://sourceware.org/git/glibc.git`, `glibc-2.39` (`ef321e23c2`): the
     `c/` submodule. The release the parent campaign, `crustify-rs/ffmpeg-bindings`, links:
     the system's glibc, package `libc6-dev 2.39-0ubuntu8.9`, whose distribution patches
     `crustify/artifacts/dependencies.json` of the parent records. Side campaign of
     `crustify-rs/ffmpeg-bindings` and of `crustify-rs/libgit2-bindings`, which links the
     same glibc 2.39.

2. **Should this campaign port the C implementation to Rust, or create safe
   Rust wrappers?** (`id: campaign`)
   - Answer: `wrap`

3. **What should this campaign target: a named subset of modules, or a named
   subset of functions and types, or the whole target repo? You can define them
   now or we can brainstorm them during the live session. You can also let the
   orchestrator decide.** (`id: scope`)
   - Answer: what the parent campaigns consume from libc, the entities their dependency
     records count: for `crustify-rs/ffmpeg-bindings`, `struct tm` (`libavutil/bprint.c`,
     `libavutil/parseutils.c`) and `FILE` (`libavformat/dump.c`); for
     `crustify-rs/libgit2-bindings`, 871 functions, 95 structs and 2 globals, counted per
     module over its 168 modules that use libc; growing as the parents' waves need more

4. **Which model and billing should do the translation work?**
   (`id: translate-agent`)
   - Answer: `openrouter/anthropic/claude-opus-5.5, api`
   - Format: `<provider/model, api | subscription>`, e.g.
     `openrouter/anthropic/claude-opus-5, api`. The provider is the first
     segment and selects both the billing rate table and the backend CLI.
     Supported providers:
     - `anthropic/<model>` — Anthropic direct, driven by Claude Code;
     - `openai/<model>` — OpenAI direct, driven by Codex;
     - `openrouter/<vendor>/<model>` — OpenRouter, which resells every vendor
       behind one key, so the vendor segment picks the CLI: `anthropic/*` is
       driven by Claude Code and everything else by Codex. OpenRouter
       supports `api` billing only.

5. **Do you want agentic review after translated work lands? If so, which
   model and billing should perform each review?** (`id: review-agent`)
   - Answer: `openrouter/anthropic/claude-opus-5.5, api`, after each wave

6. **Should I run fully autonomously end to end?** (`id: autonomy`)
   - Answer: `no`

# Optional questions

Unanswered optional questions use their defaults.

## Campaign execution

7. **Which model and billing should plan the campaign, and how many planner runs
   should it make? Each run emits what the plan lacks and corrects what the runs
   before it committed.** (`id: plan-agent`)
   - Answer: the translation model and billing, 2 runs.
   - By default: the translation model and billing, 2 runs.

8. **Should the campaign use the default batching and parallelism settings, or
   customize them?** (`id: workload`)
   - Answer: `max-types: 2, max-syms: 25, max-loc: 500`, at least 20 fields per type
     batch; parallelism up to 16 agents.

9. **What batch caps should review agents use? We recommend the same caps as
   translation by default.** (`id: review-workload`)
   - Answer: `as many as the reviewed wave requires`

10. **Which objectives should review agents review for? By default, every one the
    reviewer offers.** (`id: review-objectives`)
    - Answer: `all`
    - Without `fix`, reviewers report and reproduce what they find but patch nothing.

11. **Which sanitizers should the campaign build and test under? By default, every one
    the toolchain supports.** (`id: sanitizers`)
    - Answer: `asan-ubsan, tsan, bsan`, instrumenting the Rust; the system's glibc they
      link is not instrumented. MSan is not built.
    - Miri needs no build and is always on.

12. **How should the target be configured: which of its build features should be
    enabled or disabled?** (`id: target-config`)
    - Answer: none: libc is the system's, which the parent and every other library on
      the host link, so glibc is not built; `c/` is its source, for reading its behaviour,
      and the raw bindings are generated from the host's headers.
    - By default: the target's default configuration, with deprecated features disabled.

## Autonomy (if question 6 is answered `no`)

Answer these approval-gate questions only if question 6 is answered `no`.

13. **Should I wait for your approval before starting the setup phase?** (`id: gates.setup`)
    - Answer: `no`
14. **Should I wait for your approval before starting the translation phase?**
    (`id: gates.translation`)
    - Answer: `yes`
15. **Should I wait for your approval between modules?** (`id: gates.module`)
    - Answer: `no`
16. **Should I wait for your approval before starting review passes?** (`id: gates.review`)
    - Answer: `no`

# Benchmark recording questions

17. **Where and in what format should the dashboard be recorded?** (`id: dashboard`)
    - Answer: `<repo-checkout>/crustify/artifacts/dashboard.md`, `standard` template

# Additional instructions

Unless otherwise stated, preserve the format of the dashboard exactly.

