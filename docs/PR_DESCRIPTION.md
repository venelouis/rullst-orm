# Pull Request

## Title

`feat(orm): simplify tenant context API to with_tenant + without_tenant()`

## Description

The tenant context API is simplified to its minimum useful shape.

End state:

- `rullst_orm::with_tenant(t, …)` — the only ambient source of
  truth, set once at the request boundary. Every read, delete and
  `entity.save()` inside the closure sees `t` as its
  `tenant_column` value.
- `QueryBuilder::without_tenant()` — the only per-query opt-out, for
  the rare cases that need to drop the auto-injected
  `WHERE <tenant_column> = ?` (super-admin reads, cross-tenant
  reports, migrations).

The end state is the original multi-tenant API. This PR is a small
cleanup that removed a few extra knobs that were added on top of it
but did not earn their keep.

---

## Why this is the right API

The original API was already correct:

- `with_tenant(t, …)` is the request-scoped ambient tenant id. The
  `?` substitution, the `entity.save()` tenant stamping, and the
  `delete_all()` tenant filter all consume it.
- `QueryBuilder::without_tenant()` exists for the rare cases where a
  single operation inside a `with_tenant(t)` scope must run
  unfiltered.

Cross-tenant reads that need a target tenant other than the active
scope are already reachable without a third knob:

- `with_tenant(t2, …)` — open a new scope for the request.
- `without_tenant()` + `where_eq("tenant_id", t2)` — for a single
  query.

---

## API at a glance

| Surface                                    | Use case                                                              |
| ------------------------------------------ | --------------------------------------------------------------------- |
| `with_tenant(t, …)`                        | Every operation in a request runs under `t`.                          |
| `QueryBuilder::without_tenant()`           | Drop the WHERE for a single operation (super-admin, cross-tenant).    |

That's the whole API. One task-local, one entry point, one
per-query opt-out. No "per-builder override" tier, no parallel
task-local, no separate "skip" flag.

---

## Changes

### 1. Tenant context API

- `rullst_orm::tenant::CURRENT_TENANT` is the only task-local.
- `rullst_orm::with_tenant(t, …)` is the only scope helper.
- `rullst_orm::get_tenant_id()` returns the active scope value (or
  `None` outside any `with_tenant` closure).
- `rullst_orm::cond_mentions_column()` and
  `rullst_orm::render_tenant_literal()` remain as the supported
  helpers, both unit-tested in `rullst-orm/src/tenant.rs`.

### 2. QueryBuilder

- The per-builder `QueryBuilder::without_tenant()` flag is the only
  per-query knob.
- `push_tenant_filter` and `delete_all_with_tx_internal` consult
  only `get_tenant_id()` and `skip_tenant`. The auto-injected
  `WHERE <col> = <literal>` clause short-circuits when the user has
  already pinned the tenant column with their own predicate
  (`where_eq`, `where_in`, `where_not_null`, …), so there is no
  duplicate `<col> = ?` and no phantom binding in the params vec.

### 3. Documentation

- The "Per-Query Tenant Overrides" section in
  `docs/3-advanced-features.md` is removed.
- A short pointer to `without_tenant()` and the SQL-literal escape
  rules sits next to the main `with_tenant` example in the
  `## 🏢 Multi-Tenancy` section.

### 4. Tests and example

- `scenario_tenant_context_switching` in
  `rullst-orm/tests/integration_tests.rs` covers 5 cases: plain
  scope, `without_tenant()`, dedup of a user-pinned tenant column,
  SQL-string-literal escape, `delete_all()` honouring the active
  scope.
- `rullst-orm/examples/tenant_context_switching.rs` shows
  `with_tenant` + `without_tenant()` against two tenants.

---

## Files changed

| File                                                       | What changed                                                                                          |
| ---------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `rullst-orm/src/tenant.rs`                                 | Public surface is `CURRENT_TENANT` task-local + `with_tenant` / `get_tenant_id` / `cond_mentions_column` / `render_tenant_literal` helpers. The doc comment on `with_tenant` is rewritten to point at the pre-existing `without_tenant()` opt-out. |
| `rullst-orm/src/lib.rs`                                    | Re-exports reduced to `cond_mentions_column`, `get_tenant_id`, `render_tenant_literal`, `with_tenant`. |
| `rullst-orm-macros/src/builder.rs`                         | The generated `*QueryBuilder` has only the `skip_tenant` flag and the `without_tenant()` method. `push_tenant_filter` and `delete_all_with_tx_internal` read only `get_tenant_id()` and `skip_tenant`. |
| `rullst-orm-macros/src/models.rs`                          | `tenant_scope_logic` reads only `get_tenant_id()` to stamp the `tenant_column` on insert. Same `try_into` warning behaviour. |
| `rullst-orm-macros/tests/macro_tests.rs`                   | Existing `tenant_column` tests still pass (no behavioural change for the basic API).                 |
| `rullst-orm/tests/integration_tests.rs`                    | `scenario_tenant_context_switching` covers the 5 cases listed in §4. |
| `rullst-orm/examples/tenant_context_switching.rs`          | Demonstrates `with_tenant` + `without_tenant()` against two tenants. |
| `docs/3-advanced-features.md`                              | The "Per-Query Tenant Overrides" section is removed. A short pointer to `without_tenant()` and the SQL-literal escape rules sits next to the existing `with_tenant` example. |
| `docs/PR_DESCRIPTION.md`                                   | This file.                                                                                            |

---

## How to verify

```bash
# 1. Unit tests for the tenant helpers (render + cond_mentions_column + with_tenant)
cargo test -p rullst-orm --lib tenant

# 2. Macro tests (existing tenant_column variants)
cargo test -p rullst-orm-macros

# 3. End-to-end integration test (real SQLite)
cargo test -p rullst-orm --test integration_tests

# 4. The runnable example
cargo run -p rullst-orm --example tenant_context_switching
```

The full test suite passes on the local machine; the
`scenario_tenant_context_switching` integration test covers:

1. A plain `with_tenant(t)` scope filters by that tenant.
2. `without_tenant()` inside an active `t1` scope sees every row.
3. A tenant id with a single quote (SQL-injection bait) is escaped
   rather than terminating the string literal.
4. An explicit `where_eq("tenant_id", …)` deduplicates the
   auto-injected tenant filter for both `to_sql()` and
   `delete_all()` (no phantom binding, column appears exactly
   once in the rendered SQL).
5. `delete_all` honours the active `with_tenant` scope the same
   way `to_sql()` does.

---

## Type of change

- [x] Refactor (no behavioural change to callers using the public API)
- [x] This change requires a documentation update
- [x] Breaking change (a small number of per-builder / per-scope
      helpers that sat on top of `with_tenant` and
      `without_tenant()` are removed; the surviving public API is
      exactly the pre-existing one)

> The public `with_tenant` and `QueryBuilder::without_tenant` keep
> the exact same behaviour they always had. Callers that used the
> removed per-builder / per-scope knobs can use the equivalent
> documented in the API table above (`with_tenant(t2, …)` for a new
> scope, or `without_tenant()` + `where_eq("tenant_id", t2)` for a
> single query).

## Checklist

- [x] My code follows the style guidelines of this project (`cargo fmt`)
- [x] I have performed a self-review of my own code
- [x] I have commented my code, particularly in hard-to-understand areas
- [x] I have made corresponding changes to the documentation
- [x] My changes generate no new warnings (`cargo check --all-targets`
      is clean on the workspace)
- [x] I have added tests that prove my fix is effective or that my
      feature works
- [x] New and existing unit tests pass locally with my changes
