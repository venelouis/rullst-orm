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

## 🔮 Phase 5: Version 3.0.0 Architecture (Completed)

With the release of `v3.0.0`, we successfully rebranded from Eloquent to Rullst and solidified our architectural direction. We made a conscious design decision to **abandon the "Zero-Copy" (`std::borrow::Cow`) architecture** that was previously planned for the query builder.

**Why abandon Zero-Copy?** 
Rullst ORM is built on the philosophy of extreme developer productivity (Laravel-like ease of use). Introducing lifetimes (`<'a>`) into the public API would force developers to fight the Rust borrow checker during standard database operations, entirely defeating the purpose of the library. We prioritize ergonomics, and the negligible overhead of heap `String` allocation is a tradeoff we gladly accept for a clean, lifetime-free API.

Instead, we achieved **Compile-Time Safety** without lifetimes through our Strict Feature Flags:

### 🛡️ Strict SQL Typing (Delivered via Feature Flags)
We introduced the `strict-postgres`, `strict-mysql`, and `strict-sqlite` feature flags. 
- When enabled, the ORM bypasses the dynamic `sqlx::AnyPool` and natively binds to the specific database driver, enabling strict compile-time verification without polluting the user's code with lifetimes.
- The default behavior remains dynamically typed, ensuring maximum flexibility for rapid prototyping.

## 🌍 Phase 6: The Ultimate Ecosystem (SaaS & Open Source Mastery)

Our goal is to provide tools that normally cost thousands of dollars, completely free and open-source, ensuring `rullst-orm` stands unrivaled in the Rust ecosystem.

- [x] **Native Multi-Tenancy**: Built-in support for SaaS applications. Automatic isolation of tenant data via magic `tenant_id` columns or separate database schemas.
- [x] **Audit Trails (Revision History)**: A `#[orm(auditable)]` macro that automatically tracks "who changed what" in a separate history table for compliance and rollbacks.
- [x] **Built-in Full-Text Search (Scout)**: `.search("query")` method that automatically syncs your models with Meilisearch, Algolia, or Elasticsearch upon saving.
- [x] **Rullst ORM Admin Panel**: A drop-in function that generates a beautiful web dashboard to manage your data without writing frontend code.
- [x] **API Resources & Transformers**: A declarative way to transform Rullst Models and eager-loaded relationships into clean JSON API responses, handling hidden fields, date formatting, and nested relations effortlessly.

## 🧠 Phase 7: The Future (AI, Quantum & Infrastructure)

Pushing the boundaries of what an ORM can do in the modern era of computing.

- [ ] **Native Vector DB & RAG Support (`pgvector`)**: Methods like `.where_similar("embedding", vector)` to natively support AI applications and Retrieval-Augmented Generation directly in standard SQL databases.
- [ ] **AI-Powered Auto Migrations**: An opt-in tool that analyzes your Rust structs and uses a local or remote LLM to automatically generate the perfect SQL migration diffs, eliminating manual SQL typing.
- [ ] **Wasm & Edge Computing**: Running the ORM directly on Cloudflare Workers or Vercel Edge with Serverless DB drivers (PlanetScale, Neon).
- [ ] **Orm Sail (Instant Docker)**: A CLI command that instantly spins up a `docker-compose` environment with Postgres, Redis, Meilisearch, and your Rust app pre-configured. Zero infra setup.
- [ ] **Post-Quantum Cryptography**: A `#[orm(encrypt_pq)]` macro to encrypt sensitive columns (like medical records, passwords) at rest using post-quantum algorithms (e.g., CRYSTALS-Kyber) to future-proof against quantum computer attacks.
- [ ] **Distributed Graph Traversal**: Transforming standard SQL tables into Graph-like queries for deep recursive relationships (e.g., `friends.of.friends`) using advanced CTEs automatically generated by the ORM.
