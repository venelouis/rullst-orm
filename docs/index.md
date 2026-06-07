# The Most Productive Rust ORM

Rullst ORM is a database-agnostic Object Relational Mapper for Rust.

## Why Rullst ORM?

- **Zero Boilerplate:** Use `#[derive(Orm)]` and immediately gain access to a powerful fluent Query Builder — no `impl` blocks, no repetitive SQL.
- **Full-Text Search:** Register any search backend (Meilisearch, Typesense, Algolia, etc.) via the `SearchEngine` trait and plug it into your models with zero framework lock-in.
- **Rullst Admin Panel:** Instantly deploy a beautiful dark-mode web dashboard to manage your tables natively — no compilation, no React, just one function call.
- **Multi-Tenancy:** Effortlessly isolate tenant data using async-safe task-local context (`with_tenant`).
- **Audit Logging:** Automatic change diffing and audit trails built into the framework.
- **Collection Utilities:** Functional collection methods (`map`, `filter`, `chunk`, `implode`, `sum_by`, ...) on every `Vec<Model>`.
- **API Resources:** Transform models to JSON API responses with a clean `ApiResource` trait.

Get started instantly and build the future of Rust backend systems.
