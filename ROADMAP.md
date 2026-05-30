# Rullst ORM Roadmap

Our goal is to bring the best of the **Laravel Orm** experience to the Rust ecosystem.
Here we track the key features that differentiate Orm from other query builders and our implementation status.

## ✅ Implemented
- **Active Record/Models**: Structs directly connected to the database (`#[derive(Orm)]`).
- **Fluent Query Builder**: Method chaining (`.where_eq()`, `.order_by()`, etc).
- **Asynchronous Execution**: Powered by Tokio + SQLx.
- **Basic Magic Methods**: `.where_name("...").where_email("...")`.
- **Pagination**: `.paginate()` method to return paginated results and meta-information easily.
- **Auto Timestamps**: Native control of `created_at` and `updated_at` in `save/update/insert` methods.
- **Helper Methods**: `.first_or_fail()`, `.find_or_fail()`.
- **Pluck**: Fetching a single column.
- **Eager Loading**: N+1 problem prevention using `.with("comments")`.
- **Mutators and Accessors**: Handling data transformation via lifecycle hooks.
- **Events and Observers**: Handling hooks like `before_save`, `after_fetch`, etc.
- **Local and Global Scopes**: Reusable query constraints.
- **Soft Deletes**: Logical deletion hiding the record (`deleted_at` column).
- **Relationships**: `HasOne`, `HasMany`, `BelongsTo`, `BelongsToMany`.
- **Migrations**: Fluent schema builder API for creating tables.

## 🎉 Phase 1 Completed!
All core features of Laravel Orm have been successfully ported to Rust.

## 🚀 Phase 2: Advanced Features & Rust Superpowers
- [x] **Database Transactions**: Wrapping queries in transactional blocks (`Orm::transaction`).
- [x] **Orm Collections**: Custom collection struct with high-level methods (`map`, `pluck`, `key_by`).
- [x] **Compile-Time Safety**: Using Rust's strict typing and macros to check SQL columns at compile-time.
- [x] **Polymorphic Relationships**: `morphTo`, `morphMany`, `morphOne`.
- [x] **Factories and Seeders**: Fluent API for generating fake testing data.

## 👑 Phase 3: The Rust Masterpiece
- [x] **Many-to-Many Relationships**: Implement pivot table support (`belongsToMany`).
- [x] **Pagination with Metadata**: `.paginate(15)` returning total, current page, and data.
- [x] **JSON Column Casting**: `#[orm(json)]` macro parameter to auto-deserialize `serde_json` structs.
- [x] **Constrained Eager Loading**: Passing closures to relationships like `.with_posts_constrained(|q| q...)`.
- [x] **Rust Artisan (Migrations CLI)**: Command-line tool to generate, run, and rollback database migrations.
- [x] **Observers & Lifecycle Events**: Global observer pattern to listen to model events (`creating`, `saved`, `deleted`) externally.
- [x] **Subqueries & Advanced Joins**: Allowing closures for complex SQL joins and subqueries.
- [x] **Artisan Seeding (db:seed)**: Populate tables via Artisan CLI using Seeders and Factories.
- [x] **Query Logging & Debugging**: Inspect the executed SQL directly in terminal for optimization.
- [x] **Model Serialization (Hiding Fields)**: Attribute `#[orm(hidden)]` to automatically skip sensitive columns during JSON serialization.

## 🏭 Phase 4: Enterprise Scale (v1.0.0)
- [x] **Read/Write Connections Split**: Automatic routing of `SELECT` queries to database replicas and `INSERT/UPDATE/DELETE` to the primary node.
- [x] **Query Chunking & Cursors**: Methods like `.chunk(1000, |batch| ...)` to process millions of records safely without high memory usage.
- [x] **Integrated Caching Layer**: Add `.remember(seconds)` using an optional Redis feature flag to automatically cache repetitive queries.
- [x] **Background Event Hooks**: Optional pub/sub event broadcasting when models change, allowing seamless integration with external worker queues.
- [x] **Security & Performance Static Audit**: All critical and medium-priority findings from the Jules/Antigravity architecture audit resolved in v1.1.13 (QueryBuilder binding fix, error propagation, clippy compliance).

## 🔮 Phase 5: Version 2.0.0 Roadmap (Breaking Changes)

This section outlines the planned breaking changes and architectural upgrades for the next major release (`v2.0.0`). These changes were intentionally deferred from the `v1.x` branch to preserve backward compatibility and maintain the simplistic, lifetime-free API.

### 1. ⚡ Zero-Copy Query Builder (String Optimization)

**Current State (v1.x):**
The `QueryBuilder` allocates new `String` objects on the heap using `format!` for every condition (e.g., `where_eq`, `join`, `order_by`). This was done to keep the API simple and avoid polluting the builder and `ActiveRecord` implementation with generic lifetimes (`<'a>`).

**Proposed Change (v2.0):**
Refactor the internal `wheres`, `joins`, and `selects` collections to use `std::borrow::Cow<'a, str>`.
- This will completely eliminate heap allocations for static column names and SQL fragments.
- **Breaking Change:** The `QueryBuilder` struct will require a lifetime parameter `QueryBuilder<'a>`. All functions returning or chaining the builder will need to declare this lifetime, cascading into the asynchronous `Future` bounds of `ActiveRecord` methods.
- **Implementation Strategy:** This profound transition will be implemented iteratively on the `dev` branch to ensure we can solve the complex lifetime cascades before enforcing it on end users.

### 2. 🛡️ Strict SQL Typing (Via Feature Flags)

**Current State (v1.x):**
The library uses `sqlx::AnyPool` and a custom generic enum (`RullstValue`) to map types dynamically at runtime. This allows the ORM to connect to PostgreSQL, MySQL, and SQLite seamlessly without changing the Rust codebase. However, it sacrifices Rust's powerful compile-time SQL verification.

**Proposed Change (v2.0):**
Introduce an optional "Strict Mode" via Cargo **Feature Flags** (e.g., `features = ["strict-postgres"]`).
- **Strategic Update:** Instead of removing `AnyPool` entirely and breaking compatibility for all current users, the `v1.x` dynamic mode will remain available.
- When the strict feature flag is enabled, the ORM will inject strongly-typed executors (`PgPool`, `MySqlPool`, `SqlitePool`) directly into the AST generation. All internal query builders and connection handlers will drop `sqlx::Any` and statically map parameters natively to the compiled target driver, eliminating runtime conversion errors and performance overhead.
- Additionally, the ORM will use `sqlx::query!` macros to validate SQL queries against the actual database schema at compile time, and the underlying pool will switch to specific pools like `PgPool` or `MySqlPool`.
- **Build Setup Note:** `sqlx::query!` compile-time validation requires SQLx metadata during builds (typically `DATABASE_URL` or SQLx offline `.sqlx` data), so strict mode introduces extra build/CI setup requirements.
- This dual-approach provides a safe migration path for existing applications while offering maximum safety for new projects.

### 3. 🧹 Automated Resource Cleanup (Subquery Scopes)

**Current State (v1.x):**
Subqueries and raw scope injections do not automatically drop their memory footprints until the parent query completes execution.

**Proposed Change (v2.0):**
Implement custom `Drop` traits or an explicit arena allocator for complex query chains to reduce the maximum memory footprint during large `EXISTS` subquery resolutions.

### 4. 🧬 Query Builder Generics & Type-Safe Bindings

**Current State (v1.x):**
The library relies on a dynamic enum (`RullstValue`) to represent and bind variables into SQL statements (e.g. `String`, `Int`, `Float`). This creates an unnecessary indirection layer and a small memory overhead allocating variables into the enum wrapper before binding them.

**Proposed Change (v2.0):**
Refactor the query builder API (e.g., `.where_eq()`, `.or_where()`) to accept generic types bound by SQLx's native `sqlx::Encode` and `sqlx::Type` traits.
- This will completely remove the need for `RullstValue` in strict mode environments.
- Bindings will be statically pushed down to the underlying database driver natively, making execution slightly faster and memory-safe.

## 🌍 Phase 6: The Ultimate Ecosystem (SaaS & Open Source Mastery)

Our goal is to provide tools that normally cost thousands of dollars, completely free and open-source, ensuring `rullst-orm` stands unrivaled in the Rust ecosystem.

- [ ] **Native Multi-Tenancy**: Built-in support for SaaS applications. Automatic isolation of tenant data via magic `tenant_id` columns or separate database schemas.
- [ ] **Audit Trails (Revision History)**: A `#[orm(auditable)]` macro that automatically tracks "who changed what" in a separate history table for compliance and rollbacks.
- [ ] **Built-in Full-Text Search (Scout)**: `.search("query")` method that automatically syncs your models with Meilisearch, Algolia, or Elasticsearch upon saving.
- [ ] **Rullst ORM Admin Panel**: A drop-in crate that reads your `#[derive(Orm)]` models and instantly generates a beautiful web dashboard to manage your data without writing frontend code.
- [ ] **Wasm & Edge Computing**: Running the ORM directly on Cloudflare Workers or Vercel Edge with Serverless DB drivers (PlanetScale, Neon).

## 🧠 Phase 7: The Future (AI, Quantum & Infrastructure)

Pushing the boundaries of what an ORM can do in the modern era of computing.

- [ ] **Native Vector DB & RAG Support (`pgvector`)**: Methods like `.where_similar("embedding", vector)` to natively support AI applications and Retrieval-Augmented Generation directly in standard SQL databases.
- [ ] **AI-Powered Auto Migrations**: An opt-in tool that analyzes your Rust structs and uses a local or remote LLM to automatically generate the perfect SQL migration diffs, eliminating manual SQL typing.
- [ ] **Orm Sail (Instant Docker)**: A CLI command that instantly spins up a `docker-compose` environment with Postgres, Redis, Meilisearch, and your Rust app pre-configured. Zero infra setup.
- [ ] **Post-Quantum Cryptography**: A `#[orm(encrypt_pq)]` macro to encrypt sensitive columns (like medical records, passwords) at rest using post-quantum algorithms (e.g., CRYSTALS-Kyber) to future-proof against quantum computer attacks.
- [ ] **Distributed Graph Traversal**: Transforming standard SQL tables into Graph-like queries for deep recursive relationships (e.g., `friends.of.friends`) using advanced CTEs automatically generated by the ORM.
