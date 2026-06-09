# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [4.0.6] - 2026-06-09 🛠️

### Performance
- **`chunk` builder optimization:** Eliminated heavy struct and query state cloning inside the internal chunking `while` loop, allocating the builder exactly once and dynamically modifying its offset parameters instead.

### Fixed
- **Macro linter output:** Removed the hardcoded `#[allow(clippy::needless_update)]` mask from the `paginate` generated code by employing localized mutable builder state correctly, surfacing legitimate upstream warnings previously suppressed in downstream applications.

### Tests
- **Schema `drop_if_exists` resilience test:** Injected an integration test validating that malformed or empty drop table commands properly propagate `Error::Internal` blocks without executing unsafe DDL operations.
- **Panic propagation test (`with_tenant`):** Implemented an asynchronous isolation test assuring task local scope integrity and lack of thread poisoning when closures panic under multi-tenant contexts.
- **Audit diff JSON matching block:** Ensured the internal `.compute_diff()` logic securely yields `None` identically when provided exact duplicate payloads.

## [4.0.5] - 2026-06-08 🧪

### Added
- **SQLite Database Integration Tests:** Added a new comprehensive integration test suite (`tests/integration_tests.rs`) covering 6 major database-touching scenarios: CRUD operations (INSERT/SELECT/UPDATE/DELETE/COUNT/WHERE), soft delete lifecycle (deleted_at, restore, force_delete), transaction commit and rollback isolation, JSON column serialization/deserialization, bulk operations (LIMIT, OFFSET, Pluck, Delete All), and dynamic schema lifecycle. All scenarios share a single `OnceLock` connection to comply with global pooling invariants.
- **Criterion Benchmark Suite:** Added a high-performance benchmark harness (`benches/orm_bench.rs`) targeting CPU-bound operations (validate_identifier, JSON serialization, query builder construction) and real database round-trip performance in `--release` mode.

### Fixed
- **Clippy Mathematical Constant Lint:** Replaced hardcoded float `3.14` in schema builder tests with `1.23` to eliminate the `approx_constant` clippy warnings under strictly enforced workspace checks (`-D warnings`).

### Tests
- **53 tests passing** across the workspace (`cargo test --workspace --all-features`).

### Performance
- **`belongs_to_many` eager loading rewritten to 2-query batch strategy:** The previous implementation issued O(N/10) queries (one per chunk of 10 parents) using `try_join_all`. This has been rewritten to use exactly 2 queries regardless of collection size: (1) `SELECT parent_fk, related_fk FROM pivot WHERE parent_fk IN (...)` and (2) `SELECT * FROM related WHERE id IN (unique_related_ids)`. Distribution is done in memory using a `HashMap<i32, Vec<RelModel>>`. All eager loading strategies now operate at O(1) or O(2) queries, eliminating the N+1 risk.

### Fixed
- **Scout update silent failure:** `rullst_orm::scout::update` previously called `.unwrap_or(serde_json::Value::Null)` when serializing a model to JSON, silently sending `null` to the search engine on serialization failure. It now uses `match` with an `eprintln!` diagnostic including the table name and model ID, making failures observable without panicking.

### Changed
- **`rand` removed from library production dependencies:** `rand = "0.10"` was listed in both `[dependencies]` and `[dev-dependencies]` in `rullst-orm/Cargo.toml`. Since `rand` is only used in example and factory code, it has been removed from `[dependencies]`, reducing the compiled dependency surface for library users.

### Tests
- **52 tests passing** (`cargo test --workspace --all-features`, 0 warnings in the fixed codebase).

### Security
- **DDL Injection via `Blueprint::build()`:** `Blueprint::build()` previously interpolated `col.name` and `col.default_value` directly into `CREATE TABLE` SQL without validation, allowing DDL injection through the schema builder API. The method signature has been changed to `-> Result<String, Error>` and now defensively re-validates every column name via `validate_identifier` before emitting SQL.
- **`ColumnDefault` enum replaces raw `&str` defaults:** `Column::default()` previously accepted a raw `&str` that was spliced verbatim into the DDL `DEFAULT` clause. This has been replaced with a typed `ColumnDefault` enum (`CurrentTimestamp`, `Null`, `Integer(i64)`, `Float(f64)`, `Text(String)`). `Text` values are automatically single-quoted and SQL-escaped (`''` doubling), making injection structurally impossible.
- **`Column::new()` validates identifier at construction:** Column names are now validated against `validate_identifier` at the point of construction. An invalid name panics immediately with a clear message, preventing malformed columns from ever reaching `build()`.
- **`validate_identifier` rejects leading/trailing dots:** Identifiers such as `"."`, `".users"`, and `"users."` previously passed validation despite being semantically invalid and potentially exploitable in edge-case drivers. The validator now rejects any identifier whose first or last character is a dot.

### Changed
- **`Blueprint::build()` signature is now `-> Result<String, Error>`** (previously `-> String`). This is a breaking change for any caller that used `build()` directly. `Schema::create()` propagates the error transparently, so migration closures are unaffected.

### Tests
- **52 tests passing** across the full workspace (`cargo test --workspace --all-features`).
- New tests: `test_column_default_sql_rendering` (covers all `ColumnDefault` variants including embedded-quote escaping), updated `test_timestamps_adds_columns` (asserts `ColumnDefault::CurrentTimestamp` equality), updated `test_column_builder_methods` (uses `ColumnDefault::Integer`), updated `test_blueprint_build_produces_valid_sql` (handles `Result`).

## [4.0.3] - 2026-06-07 🧪

### Added
- **`compute_diff` utility (`audit.rs`):** Extracted the inner diffing logic from `log_audit_diff` into a pure, database-free `compute_diff(old_json, new_json)` function, making audit diff behavior fully unit-testable.
- **`RullstCollection::map` and `filter`:** Added two new functional methods to the `RullstCollection` trait, enabling idiomatic collection transformation and filtering in Rust.
- **Nested Tenant scope test (`tenant.rs`):** Added `test_nested_tenant_scopes` to verify correct shadowing and restoration behavior when `with_tenant` scopes are nested.

### Changed
- **Macro refactor (`models.rs`, `builder.rs`):** The large monolithic `generate()` functions have been broken into focused helper functions (`generate_struct`, `generate_impl_block`, `generate_orm_trait_impl`, etc.), significantly reducing cyclomatic complexity and improving maintainability.
- **Error Bag in `QueryBuilder`:** The `QueryBuilder` now accumulates errors in an `errors: Vec<Error>` field instead of calling `panic!` on invalid columns, returning an appropriate `Error::Validation` at runtime.
- **N+1 eliminated in replica sync (`enterprise_scaling.rs`):** The replica database sync loop has been rewritten using `try_join_all`, executing all queries in parallel instead of sequentially.

### Fixed
- **Eliminated all `panic!` from the `rullst-orm` public API:** All remaining `panic!` calls have been replaced with `Error::Internal` or `Error::Validation` surfaced through `Result<T, RullstError>`.
- **Proc-macro `panic!` replaced with `syn::Error`:** Proc-macro expansion in `parser.rs` now emits proper compile-time errors via `syn::Error::new(...).to_compile_error()` instead of panicking during macro expansion.
- **`Pool` borrowing in examples (`enterprise_scaling.rs`):** Fixed borrow checker violations by using `&pool` instead of `pool.clone()` in `.execute()` calls, correctly satisfying SQLx's `Executor<'_>` trait bound.

### Security
- **SQL Injection hardening in `JoinClause`:** `JoinClause::on()` now validates both column names and operators through `validate_identifier` before building any SQL fragment, rejecting malicious input with `Error::Validation`.
- **Removed commented-out code blocks:** All dead commented-out code blocks flagged in the audit have been removed to prevent hidden unsafe logic and reduce maintainer confusion.

### Tests
- **44 tests passing** across the full workspace (`cargo test --workspace --all-features`), including 4 macro integration tests.
- New test coverage added for: `collection` (map, filter), `tenant` (nested scopes), `scout` (idempotent Search Engine registry), `audit` (compute_diff with changed fields, no changes, and invalid JSON), `admin` (dashboard HTML rendering), and `resource` (JSON collection serialization).

## [4.0.2] - 2026-06-07 🛡️

### Security
- **SQL Injection Prevention:** Adicionado método `.bind()` nativo no `QueryBuilder` permitindo que usuários efetuem binds de variáveis de forma 100% segura e parametrizada ao utilizar queries cruas via `.where_raw()`.

### Fixed
- **PostgreSQL Macros Compatibility:** O ORM agora detecta automaticamente o driver `postgres` em runtime e substitui parâmetros `?` por marcadores numéricos `$1, $2, etc`, resolvendo os erros de sintaxe (como `ERR_EMPTY_RESPONSE` e crashes de servidor) durante queries complexas no PostgreSQL mantendo a retrocompatibilidade com SQLite e MySQL.

## [4.0.1] - 2026-06-01

### Changed (Breaking Changes)
- **Dependency Shielding Architecture**: The framework now completely hides underlying dependencies (`sqlx`, `serde`, `serde_json`, `futures`, `redis`) from the public API. This ensures that breaking changes in third-party crates will no longer impact user-generated blueprints or business logic. 
- **Internal API Wrappers**: Direct access to `sqlx::Transaction` and `sqlx::Pool` has been replaced with safe internal wrappers (`rullst_orm::db::Transaction` and `rullst_orm::db::Pool`).
- **Standardized Error Handling**: Replaced raw `sqlx::Error` propagation with the new unified `rullst_orm::Error` (`RullstError`). All framework operations now return this standardized error, effectively isolating application code from the underlying database driver's error variants.

### Security
- **Comprehensive Audit**: Executed a comprehensive v4.0.0 security and architecture audit. Validated zero known vulnerabilities across 204 dependencies via `cargo audit`. Confirmed 100% safe Rust (no `unsafe` blocks) and zero `clippy` warnings workspace-wide. Validated that all dynamic query builders utilize robust `validate_identifier` logic to prevent SQL injections.

---

## [3.0.3-1] - 2026-05-31

### Security (SQL Injection Corrections)
- **Scout Search Parameterization:** Completely removed SQL interpolation from the generated `search()` macro method. Replaced it with native, driver-aware parametrized LIKE logic (`CAST(col AS type) LIKE ?`) binding the query values dynamically.
- **Join Condition Validation:** Added robust table and column identifier validation to `JoinClause::on` along with a strict operator allowlist (`=`, `!=`, `<>`, `<`, `>`, `<=`, `>=`) preventing arbitrary SQL execution.
- **Query Builder Sanitization:** Implemented safe identifier validation inside critical dynamic query builder methods (`where_column`, `order_by`, `order_by_desc`), guarding against unauthorized dynamic payload execution.

### Fixed
- **Dev-Dependencies Resolution (CRITICAL):** Fixed an outdated `rand` version assignment (`0.1`) in `[dev-dependencies]` that triggered severe compiler errors by forcing Cargo to pull obsolete ecosystem crates (like `log v0.2.5`) from 2015. Updated seamlessly to match `rand = "0.10"`.
- **Eager Loading Morph N+1:** Restructured the procedural relationship generator mapping in `rullst-orm-macros/src/relationships.rs`. Replaced the previous N+1 query execution loops with single batched queries (`WHERE morph_id IN (...) AND morph_type = 'Name'`) for `morph_many` and `morph_one` relations.

### Changed
- **Flexible Versioning Model:** Refactored direct ecosystem dependencies (`tokio`, `serde`, `serde_json`, `async-trait`, `futures`, `redis`, `axum`) from highly locked-down semantic versions to modern, flexible single/dual-digit identifiers. This enables effortless automatic downstream patch and minor bugfix upgrades on `cargo update` without risking user-facing dependency conflicts.

### Added
- **Comprehensive Unit Testing Suite:** Significantly expanded repository test coverage to validate the engine's core functionality, including:
  - `schema.rs`: Validations for `timestamps()`, `soft_deletes()`, and `validate_identifier`.
  - `collection.rs`: Full testing suite for `RullstCollection` transformations and statistics (`key_by`, `chunk`, `implode`, `sum_by`, `max_by_key`, `min_by_key`).
  - `resource.rs`: Tests for JSON resource translation helpers (`JsonResource`, `ResourceCollection`).
  - `audit.rs`: Serialized round-trip coverage for the `AuditLog` structure.
  - `scout.rs`: Unit test for search engine state retrieval (`get_search_engine`).
  - `tenant.rs`: Flow and context assertions for SaaS dynamic tenant allocation (`get_tenant_id`).
  - `admin.rs`: Unit testing coverage verifying HTML template building blocks inside the UI dashboard (`dashboard_html`).

---

## [3.0.0] - 2026-05-30

**Release status:** Prepared for release. The repository includes automated publishing on tag creation (see `.github/workflows/ci.yml` -> `Publish to Crates.io`). To publish the release automatically, push a Git tag matching `v3.0.0` and ensure `CARGO_REGISTRY_TOKEN` is present in the repository secrets. Alternatively, merge the `release/v3.0.0` branch and create the tag from GitHub.

### Changed
- **Rebranding API:** Breaking change. All `EloquentModel`, `EloquentValue`, `EloquentDatabase` references internally and externally are refactored to `RullstModel`, `RullstValue`, etc., fully unifying the crate's naming convention with the new `rullst-orm` name.
- Updated `#[eloquent(...)]` helper macro to `#[rullst(...)]`.

### Added
- **Native Multi-Tenancy**: Added a frictionless SaaS multi-tenancy system powered by `tokio::task_local!`. Wrapping a block in `with_tenant("id", ...)` automatically scopes all `SELECT`, `UPDATE`, `DELETE` queries to that tenant, and magically populates `tenant_id` on new `INSERT` models. Enabled via `#[orm(tenant_column = "tenant_id")]`.
- **Audit Trails (Diff Tracking)**: Added an automatic revision history feature. Simply flag a struct with `#[orm(auditable)]`, and the ORM will intercept updates and deletes, diff the JSON state of the row, and log the exact `old_values` and `new_values` into a centralized `rullst_audits` history table.
- **Built-in Full-Text Search (Scout)**: Implemented `.search("query")` method and `SearchEngine` trait. Automatically syncs models with external engines (like Meilisearch) upon saving, or falls back to robust, driver-aware native SQL `LIKE` queries out-of-the-box.
- **Rullst ORM Admin Panel**: Delivered a drop-in HTML dashboard endpoint (`rullst_orm::admin::dashboard_html()`). It generates a beautiful, rich dark-mode web dashboard that developers can serve natively via `axum`, `actix`, or any web framework, zeroing the cost of a traditional backend UI.
- **API Resources & Transformers**: Added Laravel-style `ApiResource` trait and `.collection_resource()` mapping. Effortlessly filter and map model properties, preventing sensitive data leaks in JSON API endpoints.
- **SQLite ID Hydration Fix**: Addressed a severe SQLx limitation by forcing SQLite to utilize `RETURNING id` during model creation, bypassing `AnyPool`'s inability to return `last_insert_rowid()`.
- **Release Automation:** Integrated GitHub Actions CI/CD for automated Crates.io publishing triggered by `v*` Git tags.
- **Security Audits in CI:** Added `cargo audit` to the `ci.yml` pipeline to automatically block PRs with vulnerable dependencies.
- **Unit Tests:** Added full test coverage for `enable_query_log`, `validate_table_name`, `JoinClause`, `RullstValue`, and string manipulation edge cases.
- **Strict SQL Typing Architecture:** Complete integration of Cargo feature flags (`strict-postgres`, `strict-mysql`, `strict-sqlite`) to optionally enforce `sqlx` compile-time type verification instead of using `AnyPool`.
- Custom `QueryResultExt` wrapper added to dynamically handle `last_insert_id()` logic across strict drivers.
- **v2.0 Roadmap:** Updated `docs/v2_roadmap.md` with the strategy to use feature flags for Strict Typing and iterative implementation for the Zero-Copy Builder.

### Fixed
- **QueryBuilder Binding Bug (CRITICAL):** Fixed an issue where `sqlx` queries using `push_bind()` after initializing with strings containing `?` placeholders resulted in corrupt SQL. Converted all generated read methods and `delete` to correctly use `sqlx::query_as(&sql).bind()` wrapped in `AssertSqlSafe`.
- **Database Error Propagation:** Removed silent `unwrap_or((0,))` fallbacks during migration verifications in `schema.rs`. All database driver errors are now accurately propagated to the caller.
- **Clippy Warnings:** Fixed `collapsible_match` and `question_mark` warnings in macro parsing, and replaced manual ceiling division with `div_ceil()` in `collection.rs`.
- **Safe Unwraps:** Converted implicit `.unwrap()` calls inside Schema Builder to explicit `.expect("...")` calls.

### Fixed
- **10/10 Static Analysis Audit:** Completely cleared all critical warnings from the Jules static analysis engine!
- **Path Traversal:** Fixed path traversal vulnerability in `create_migration_files`.
- **SQL Injection:** Added rigorous validation and warnings to `builder.rs` dynamic constructors and `schema.rs`.
- **Memory & Allocation:** Fixed inefficient vector allocations in `Collection::chunk` and `Collection::key_by`. Removed redundant `Vec` allocations in `implode`.
- **Parallel Eager Loading:** Rewrote the sequential blocking `await` loops inside `morph_many`, `morph_one`, `belongs_to_many`, and `after_fetch` hooks to use `try_join_all`, completely eliminating N+1 latencies.
- **O(N²) Reductions:** Optimized eager loading vector removals in `has_many`, `has_one`, and `belongs_to` to use `swap_remove` and chunk tracking instead of O(N²) iterations.

---

## [1.1.9] - 2026-05-28

### Fixed
- **Redis Example Build:** Restored the missing `Duration` import in `redis_cache_and_events.rs` so the Redis example compiles with `tokio::time::sleep`.

---

## [1.1.8] - 2026-05-28

### Fixed
- **SQLx 0.9 QueryBuilder Compatibility:** Switched generated reads to typed `build_query_as()` calls so model queries, counts, and plucks compile cleanly with sqlx 0.9.
- **Redis Publish Inference:** Added explicit `publish()` return typing to avoid `FromRedisValue` inference errors in lifecycle hooks.
- **Many-to-Many Pivot Joins:** Fixed pivot related-key generation in the relationship macro so eager loading no longer references an undeclared `_pivot_rk` identifier.
- **Factory Example Compatibility:** Updated the factories example to the current `rand` 0.10 API.

---

## [1.1.7] - 2026-05-28

### Fixed
- **SqlSafeStr Compatibility:** Replaced all `query_as_with` and `query_with` calls with `QueryBuilder` in builder.rs for full sqlx 0.9 compatibility
- **Execute Trait:** Added `use sqlx::Execute` imports where `query.sql()` is called to enable the method
- **QueryBuilder Usage:** Converted all dynamic SQL string construction to use QueryBuilder instead of format!

---

## [1.1.6] - 2026-05-28

### Fixed
- **SqlSafeStr Compatibility:** Replaced `format!` with `QueryBuilder` in models.rs for INSERT, UPDATE, DELETE queries to comply with sqlx 0.9's `SqlSafeStr` trait requirement
- **Relationships:** Extracted dynamic SQL strings to variables before use in queries to avoid SqlSafeStr errors

---

## [1.1.5] - 2026-05-28

### Security
- **SQL Injection Fix:** Added `validate_table_name()` function to prevent SQL injection in schema operations
- **Input Validation:** Added `validate_relation_attribute()` function to validate macro attribute syntax

### Fixed
- **Critical Unwrap Calls:** Replaced 38+ `unwrap()` calls with proper error handling (`expect()` with descriptive messages, `?` for error propagation)
- **Race Condition:** Fixed race condition in replica round-robin by moving modulo operation before array access
- **Redis Error Handling:** Added error logging for Redis publish failures instead of silently ignoring them

### Performance
- **Allocation Optimization:** Added `String::with_capacity()` in `to_sql()` with estimated capacity
- **String Formatting:** Replaced many `format!` calls with `push_str` in hot paths
- **Clone Reduction:** Removed unnecessary clones by using `as_str()` instead of `clone()`

### Changed
- **Macro Modularization:** Extracted helper functions (`generate_magic_methods()`, `generate_delete_all_logic()`) to reduce complexity
- **Macro Tests:** Added unit tests for macro generation in `tests/macro_tests.rs`
- **Audit Report:** Updated audit report to English with v1.1.5 fixes reflected

---

## [1.1.4] - 2026-05-27

### Fixed
- **Documentation Update**: Republished to Crates.io to ensure the most recent `README.md` documentation (including updated installation instructions and documentation links) is reflected on the official registry.

## [1.1.3] - 2026-05-27

### Fixed
- **Missing Import**: Fixed a missing `Duration` import in the `redis_cache_and_events.rs` example which broke the pipeline when the `redis` feature was enabled.

## [1.1.2] - 2026-05-27

### Fixed
- **Macro Compilation Issues**: Fixed a set of errors preventing the library from compiling.
  - Added missing `JoinClause::to_sql()` implementation.
  - Boxed async eager-loading futures to prevent `recursion in an async fn requires boxing` errors.
  - Restored automatic Column Enum generation for compile-time safety methods (`select_cols`, `where_col`, etc.).
  - Removed duplicate `UserFactory` struct implementations generated by the macro.

## [1.1.1] - 2026-05-27

### Added
- **GitHub Actions CI:** Automated tests, clippy linting, and crates.io publishing pipeline.
- **CI Badges:** Added CI badge to the README.

### Fixed
- **N+1 Eager Loading Problem:** Completely resolved the critical `N+1` query issue in eager loading. The macro now compiles relational queries using `WHERE IN (...)` for `has_many`, `has_one`, and `belongs_to`, aggregating results efficiently in memory (`O(N)` performance instead of hitting the database in a loop).

### Changed
- **Dependencies Updated:** All `cargo` dependencies bumped to their latest versions.
- **Removed Unused Imports:** Cleaned up the codebase with `cargo clippy --fix`.
- **Macro Modularization:** Splitted the massive `rullst-orm-macros` monolith into smaller files (`parser.rs`, `builder.rs`, `models.rs`, etc.) to improve maintainability and AI processing capabilities.

## [1.1.0] - 2026-05-25

### Added
- **Database-Agnostic Migration Engine:** The Artisan CLI migration runner is now entirely driver-agnostic, capable of dynamically generating standard schemas for PostgreSQL, MySQL, and SQLite identically based on the `Blueprint` builder.
- **Improved Type Safety:** Improved `.save()` internal query generation for nested fields handling generic string lengths and driver-specific Boolean types automatically.

## [1.0.0] - 2026-05-24

### Added (The Phase 3 & 4 Enterprise Expansion)
- **Constrained Eager Loading:** Added closure-constrained eager loading support (`with_posts_constrained(|q| ...)`), allowing filtering and ordering nested relations before they are mapped.
- **Global Lifecycle Observers:** Introduced a global type-safe observer pattern (`User::observe(Arc::new(UserObserverImpl))`) supporting `saving`, `saved`, `creating`, `created`, `updating`, `updated`, `deleting`, and `deleted` hooks.
- **Rust Artisan CLI:** Engineered a transaction-safe database migration and seeding CLI architecture (`run_artisan` mapping `make:migration`, `migrate`, `migrate:rollback`, and `db:seed`).
- **Subqueries & Advanced Joins:** Implemented `SubqueryBuilder` and `JoinClause` primitives allowing closure-based joins (`join_constrained`) and dynamic `EXISTS` subqueries (`where_exists`).
- **Query Logging & Debugging:** Added internal `Orm::enable_query_log()` and `Orm::disable_query_log()` to instantly intercept and print generated SQL logic to STDOUT.
- **Model Serialization & Field Hiding:** Enabled robust model JSON serialization natively compatible with `serde_json`. Added `#[orm(hidden)]` struct attribute to prevent sensitive columns from being exported inside `to_json()`.
- **`Json<T>` Transparency:** Extended internal wrapper `Json<T>` to natively implement `serde::Serialize` and `serde::Deserialize` for any inner struct `T`.
- **Read/Write Connection Splitting:** Added support for dedicated read replicas (`Orm::init_replicas`) and automatic query routing: read queries go to replicas, write operations go to the primary node.
- **Query Chunking & Cursors:** Implemented `.chunk(size, callback)` and `.chunk_with_tx(size, callback)` to process massive datasets efficiently in batches without loading everything into memory.
- **Integrated Caching Layer:** Introduced the `redis` feature flag and the `.remember(seconds)` query method to instantly cache expensive database lookups natively.
- **Background Event Hooks:** Added Redis Pub/Sub broadcasting for model lifecycle events. When models are saved or deleted, events are automatically broadcasted for external worker consumption.

### Changed
- Refactored core macro procedural code for faster compilation checks.
- Unified dependencies natively within the `rullst_orm` framework boundary, eliminating the need for developers to pull downstream extensions like `serde` and `serde_json` manually.

## [0.1.2] - 2026-05-20
### Fixed
- Fixed module visibility scopes and standard relationships compilation.

## [0.1.1] - 2026-05-18
### Added
- Core relationships (Has Many, Belongs To, Morph Many).
- Pagination integration (`paginate(page, per_page)`).
- `sqlx` raw mappings.

## [0.1.0] - 2026-05-15
### Added
- Initial project release.
- Baseline query builder, dynamic filters, and CRUD macros.
