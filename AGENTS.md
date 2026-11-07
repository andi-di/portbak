# AGENTS.md

`portbak` — portable desktop backup app for Linux x86-64 and Windows 10/11 x86-64.
The stack, dependencies, commands, and build/platform facts live in `Cargo.toml` and `README.md`; the
observable behavior contract is `SPECIFICATION.md`; open tasks live locally in the ignored `todos/` directory. Don't restate any of
them here.

## Rules

- Before making changes, create a dedicated branch from current `origin/main`, such as `feat/<topic>`, `fix/<topic>`, `docs/<topic>`, or `chore/<topic>`. Never commit or push directly to `main`.
- Submit changes through a pull request to `main`. Before opening a PR, ask the user to confirm the feature is finished. Agents must not merge them. Contributors with repository write access may merge PRs after both required CI builds pass.
- Commit after each change set. Do not batch unrelated work into one commit.
- Use conventional commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `build:`, `chore:`. Imperative subject, optional scope, body explains the why.
- Follow clean code principles: small functions, meaningful names, no duplication, no dead code, handle errors explicitly.
- Keep comments short and concise — one or two lines explaining the why or a non-obvious constraint, never a restatement of what the code already says.
- Keep this file short and durable-facts-only. Open tasks belong in the local `todos/` directory; never add task lists or history here, or anything that goes stale with the next refactor.
- Keep `README.md` current whenever a change requires it; update `SPECIFICATION.md` when observable behavior changes.
- Before finishing: run `cargo fmt`, `cargo test --locked`, and `cargo clippy --locked --all-targets -- -D warnings`.
