# AGENTS: Recommended agent context for rullst-orm

**Purpose:** Provide quick context and example prompts for AI agents and contributors.

**Version covered:** `rullst-orm v4.0.3`

**Reference spec:** See [docs/spec.md](docs/spec.md) — the Single Source of Truth for all macros, query builder methods, and public API signatures.

---

## Key Architecture Facts

- All public methods return `Result<T, rullst_orm::Error>` — never `sqlx::Error`.
- `sqlx`, `serde`, `serde_json`, `futures`, `redis` are **internal** dependencies, not re-exported.
- The `QueryBuilder` uses an **Error Bag** (`errors: Vec<Error>`) — invalid column names do not `panic!`, they accumulate and are returned on execution.
- Proc-macro errors use `syn::Error::new(...).to_compile_error()` — no `panic!` in macro expansion.
- Multi-tenant context is propagated via Tokio task-local storage (`with_tenant` / `get_tenant_id`).

---

## Suggested Agent Prompts

- "Summarize the macro expansion for `#[derive(Orm)]` in one paragraph."
- "List every public API method for `rullst-orm` and their return types."
- "Run `cargo clippy --workspace --all-features --all-targets -- -D warnings` and suggest minimal fixes."
- "Check if all public-facing methods in `rullst-orm/src/` return `Result<T, rullst_orm::Error>` and report any that still use `unwrap()` or `expect()` outside of tests."
- "Write a new unit test for `[module]` following the existing test patterns in the same file."
- "Is the `docs/spec.md` still accurate with respect to the current source code? List any discrepancies."

---

## CI Hints

Required checks for every PR:

```
cargo test --workspace --all-features
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo audit
```

---

## Maintainers

Add project-specific agent prompts here as the codebase evolves.
