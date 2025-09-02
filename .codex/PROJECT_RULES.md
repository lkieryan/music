# Project Rules (Codex Memory)

This file captures the project’s working rules so I consistently follow them while coding. Source of truth lives under `.qoder/rules/*`.

Sources
- .qoder/rules/app.md
- .qoder/rules/base.md
- .qoder/rules/git-rules.md

Core Principles
- App: Tauri 2.7 + Rust backend, React 19 + TypeScript 5.8, Vite 7.
- State: Jotai (atoms), Zustand (complex state), Immer.
- UI: Radix UI (themes + primitives), Headless UI, TailwindCSS 3.4, CSS variables, tailwind-variants/CVA.
- Motion: Prefer `motion` with the `m` component for animations.
- Structure: components/{ui,layout,modules,common}, atoms, lib, hooks, providers, services, types, styles.
- Rust: Multiple crates for database, scanner, providers, themes, settings; TS bindings via ts-rs.

Implementation Guidelines
- Reuse-first: search for existing components/hooks/utils before adding new ones; extend or add variants when possible.
- Consistency: follow existing naming, design tokens, CSS variables, file organization, and component API patterns.
- Type safety: strict TS, generated Rust bindings; keep props/variants typed.
- Styling: utility-first Tailwind with layered CSS; keep theme variables and gradients consistent.
- Performance: lazy-load where sensible; keep bundles lean; prefer lightweight primitives.

Workflow & Process
- Phases: assess (read README/code), implement (clarify, code, optimize, test), complete (summarize, document risks, update docs).
- Communication: keep explanations clear and actionable; state assumptions; prefer simplest effective solutions.

Git Commit Rules
- Language: English only.
- Conventional commits: `type(scope): description` with optional body/footer.
- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert.
- Scopes: player, ui, drag, hooks, types, config, build, deps, etc.
- Checklist: concise subject (≤ ~50 chars), clear body when needed, references, tests pass, lint/format clean.

Notes for Tauri Runtime
- Use `tauri::async_runtime::spawn` instead of `tokio::spawn` to ensure a reactor exists within the Tauri app context.

I will adhere to these rules for all future changes and reference the `.qoder/rules` files if ambiguities arise.

